use crate::{
  AnnouncementMessage, CreateCustomReward, EventSubError, SendMessage, Subscription, Token,
  TwitchEventSubApi, Validation,
};
use curl::easy::{Easy, List};

#[cfg(feature = "logging")]
use log::{error, info};

use crate::modules::{
  consts::*,
  //  generic_message::{SendTimeoutRequest, TimeoutRequestData};
};

use twitcheventsub_structs::*;

pub struct TwitchApi;

impl TwitchApi {
  pub fn get_chatters<S: Into<String>, T: Into<String>, X: Into<String>, Z: Into<String>>(
    broadcaster_id: S,
    moderator_id: T,
    access_token: X,
    client_id: Z,
  ) -> Result<String, EventSubError> {
    let url = RequestBuilder::new()
      .add_key_value("broadcaster_id", broadcaster_id.into())
      .add_key_value("moderator_id", moderator_id)
      .build(GET_CHATTERS_URL);

    TwitchHttpRequest::new(url)
      .header_authorisation(access_token.into(), AuthType::Bearer)
      .header_client_id(client_id.into())
      .run()
  }

  pub fn get_ad_schedule<S: Into<String>, T: Into<String>, X: Into<String>>(
    broadcaster_id: X,
    access_token: S,
    client_id: T,
  ) -> Result<String, EventSubError> {
    let url = RequestBuilder::new()
      .add_key_value("broadcaster_id", broadcaster_id.into())
      .build(GET_AD_SCHEDULE_URL);

    TwitchHttpRequest::new(url)
      .header_authorisation(access_token.into(), AuthType::Bearer)
      .header_client_id(client_id.into())
      .run()
  }

  pub fn send_chat_message<S: Into<String>, T: Into<String>, V: Into<String>, X: Into<String>>(
    message: S,
    access_token: T,
    client_id: V,
    broadcaster_account_id: X,
    sender_account_id: Option<V>,
    is_reply_parent_message_id: Option<String>,
  ) -> Result<String, EventSubError> {
    let message = message.into();
    if message.len() > 500 {
      return Err(EventSubError::MessageTooLong);
    }

    let broadcaster_account_id = broadcaster_account_id.into();
    TwitchHttpRequest::new(SEND_MESSAGE_URL)
      .json_content()
      .full_auth(access_token, client_id)
      .is_post(
        serde_json::to_string(&SendMessage {
          broadcaster_id: broadcaster_account_id.to_owned(),
          sender_id: sender_account_id
            .ok_or(broadcaster_account_id)
            .map(|s| s.into())
            .unwrap(),
          message: message.into(),
          reply_parent_message_id: is_reply_parent_message_id,
        })
        .unwrap(),
      )
      .run()
  }

  pub fn send_announcement<
    S: Into<String>,
    T: Into<String>,
    V: Into<String>,
    X: Into<String>,
    Z: Into<String>,
    P: Into<String>,
  >(
    message: S,
    access_token: T,
    client_id: V,
    broadcaster_account_id: X,
    sender_account_id: Z,
    colour: Option<P>,
  ) -> Result<String, EventSubError> {
    let message = message.into();
    if message.len() > 500 {
      return Err(EventSubError::MessageTooLong);
    }

    let broadcaster_account_id = broadcaster_account_id.into();
    let sender_account_id: String = sender_account_id.into();
    let url = RequestBuilder::new()
      .add_key_value("broadcaster_id", broadcaster_account_id)
      .add_key_value("moderator_id", sender_account_id)
      .build(SEND_ANNOUNCEMENT_URL);

    TwitchHttpRequest::new(url)
      .json_content()
      .full_auth(access_token, client_id)
      .is_post(
        serde_json::to_string(&AnnouncementMessage {
          message,
          colour: colour.and_then(|c| Some(c.into())),
        })
        .unwrap(),
      )
      .run()
  }

  pub fn send_shoutout<
    T: Into<String>,
    S: Into<String>,
    V: Into<String>,
    X: Into<String>,
    Z: Into<String>,
  >(
    access_token: T,
    client_id: S,
    from_broadcaster_id: X,
    to_broadcaster_id: Z,
    moderator_id: V,
  ) -> Result<String, EventSubError> {
    let url = RequestBuilder::new()
      .add_key_value("from_broadcaster_id", from_broadcaster_id.into())
      .add_key_value("to_broadcaster_id", to_broadcaster_id.into())
      .add_key_value("moderator_id", moderator_id.into())
      .build(SEND_SHOUTOUT_URL);

    TwitchHttpRequest::new(url)
      .header_authorisation(access_token.into(), AuthType::Bearer)
      .header_client_id(client_id.into())
      .json_content()
      .is_post("")
      .run()
  }

  pub fn generate_token_from_refresh_token<S: Into<String>, T: Into<String>, V: Into<String>>(
    client_id: S,
    client_secret: T,
    refresh_token: V,
  ) -> Result<Token, EventSubError> {
    let post_data = format!(
      "grant_type=refresh_token&refresh_token={}&client_id={}&client_secret={}",
      refresh_token.into(),
      client_id.into(),
      client_secret.into()
    );

    TwitchEventSubApi::process_token_query(post_data)
  }

  pub fn get_user_token_from_authorisation_code<
    S: Into<String>,
    T: Into<String>,
    V: Into<String>,
    W: Into<String>,
  >(
    client_id: S,
    client_secret: T,
    authorisation_code: V,
    redirect_url: W,
  ) -> Result<Token, EventSubError> {
    let post_data = format!(
      "client_id={}&client_secret={}&code={}&grant_type=authorization_code&redirect_uri={}",
      client_id.into(),
      client_secret.into(),
      authorisation_code.into(),
      redirect_url.into()
    );

    TwitchEventSubApi::process_token_query(post_data)
  }

  pub fn get_authorisation_code<S: Into<String>, T: Into<String>>(
    client_id: S,
    redirect_url: T,
    scopes: &Vec<Subscription>,
    is_local: bool,
  ) -> Result<String, EventSubError> {
    let redirect_url = redirect_url.into();

    let scope = &scopes
      .iter()
      .map(|s| s.required_scope())
      .filter(|s| !s.is_empty())
      .collect::<Vec<String>>()
      .join("+");

    let get_authorisation_code_request = format!(
      "{}authorize?response_type=code&client_id={}&redirect_uri={}&scope={}&force_verify=true",
      TWITCH_AUTHORISE_URL,
      client_id.into(),
      redirect_url.to_owned(),
      scope
    );

    match TwitchEventSubApi::open_browser(get_authorisation_code_request, redirect_url, is_local) {
      Ok(http_response) => {
        if http_response.contains("error") {
          Err(EventSubError::UnhandledError(format!("{}", http_response)))
        } else {
          let auth_code = http_response.split('&').collect::<Vec<_>>()[0]
            .split('=')
            .collect::<Vec<_>>()[1];
          Ok(auth_code.to_string())
        }
      }
      e => e,
    }
  }

  pub fn generate_user_token<S: Into<String>, T: Into<String>, V: Into<String>>(
    client_id: S,
    client_secret: T,
    redirect_url: V,
    is_local: bool,
    subscriptions: &Vec<Subscription>,
  ) -> Result<Token, EventSubError> {
    let client_id = client_id.into();
    let client_secret = client_secret.into();
    let redirect_url = redirect_url.into();

    TwitchApi::get_authorisation_code(
      client_id.to_owned(),
      redirect_url.to_owned(),
      &subscriptions,
      is_local,
    )
    .and_then(|authorisation_code| {
      TwitchApi::get_user_token_from_authorisation_code(
        client_id.to_owned(),
        client_secret.to_owned(),
        authorisation_code.to_owned(),
        redirect_url.to_owned(),
      )
    })
  }

  pub fn delete_message<
    U: Into<String>,
    S: Into<String>,
    X: Into<String>,
    Z: Into<String>,
    F: Into<String>,
  >(
    broadcaster_id: X,
    moderator_id: Z,
    message_id: S,
    access_token: U,
    client_id: F,
  ) -> Result<String, EventSubError> {
    let url = RequestBuilder::new()
      .add_key_value("broadcaster_id", broadcaster_id.into())
      .add_key_value("moderator_id", moderator_id.into())
      .add_key_value("message_id", message_id.into())
      .build(TWITCH_DELETE_MESSAGE_URL);

    TwitchHttpRequest::new(url)
      .header_authorisation(access_token.into(), AuthType::Bearer)
      .header_client_id(client_id.into())
      .is_delete()
      .run()
  }

  pub fn timeout_user<
    T: Into<String>,
    S: Into<String>,
    V: Into<String>,
    X: Into<String>,
    Z: Into<String>,
    O: Into<String>,
  >(
    access_token: T,
    client_id: S,
    broadcaster_id: X,
    moderator_id: Z,
    user_id: V,
    duration_secs: u32,
    reason: O,
  ) -> Result<String, EventSubError> {
    let broadcaster_id = broadcaster_id.into();
    let url = RequestBuilder::new()
      .add_key_value("broadcaster_id", broadcaster_id.to_owned())
      .add_key_value("moderator_id", moderator_id.into())
      .build(TWITCH_BAN_URL);

    let post_data = SendTimeoutRequest {
      data: TimeoutRequestData {
        user_id: user_id.into(),
        duration: duration_secs,
        reason: reason.into(),
      },
    };

    let post_data = serde_json::to_string(&post_data).unwrap();

    TwitchHttpRequest::new(url)
      .header_authorisation(access_token.into(), AuthType::Bearer)
      .header_client_id(client_id.into())
      .json_content()
      .is_post(post_data)
      .run()
  }

  pub fn get_users<T: Into<String>, I: Into<String>, S: Into<String>, V: Into<String>>(
    access_token: T,
    id: Vec<I>,
    login: Vec<S>,
    client_id: V,
  ) -> Result<String, EventSubError> {
    let mut url = RequestBuilder::new();
    if !id.is_empty() {
      url = url.add_key_value(
        "id",
        id.into_iter()
          .map(|id| id.into())
          .collect::<Vec<String>>()
          .join("&"),
      );
    }
    if !login.is_empty() {
      url = url.add_key_value(
        "login",
        login
          .into_iter()
          .map(|login| login.into())
          .collect::<Vec<String>>()
          .join("&"),
      );
    }
    let url = url.build(GET_USERS_URL);

    TwitchHttpRequest::new(url)
      .header_authorisation(access_token.into(), AuthType::Bearer)
      .header_client_id(client_id.into())
      .run()
  }

  pub fn get_channel_emotes<T: Into<String>, S: Into<String>, X: Into<String>>(
    access_token: T,
    client_id: S,
    broadcaster_id: X,
  ) -> Result<String, EventSubError> {
    let url = RequestBuilder::new()
      .add_key_value("broadcaster_id", broadcaster_id.into())
      .build(GET_CHANNEL_EMOTES_URL);

    TwitchHttpRequest::new(url)
      .header_authorisation(access_token.into(), AuthType::Bearer)
      .header_client_id(client_id.into())
      .run()
  }

  pub fn get_global_emotes<T: Into<String>, S: Into<String>>(
    access_token: T,
    client_id: S,
  ) -> Result<String, EventSubError> {
    let url = RequestBuilder::new().build(GET_GLOBAL_EMOTES_URL);

    TwitchHttpRequest::new(url)
      .header_authorisation(access_token.into(), AuthType::Bearer)
      .header_client_id(client_id.into())
      .run()
  }

  pub fn get_emote_set<X: Into<String>, T: Into<String>, S: Into<String>>(
    emote_set_id: X,
    access_token: T,
    client_id: S,
  ) -> Result<String, EventSubError> {
    let url = RequestBuilder::new()
      .add_key_value("emote_set_id", emote_set_id.into())
      .build(GET_EMOTE_SETS_URL);

    TwitchHttpRequest::new(url)
      .header_authorisation(access_token.into(), AuthType::Bearer)
      .header_client_id(client_id.into())
      .run()
  }

  pub fn get_moderators<T: Into<String>, S: Into<String>, X: Into<String>>(
    access_token: T,
    client_id: S,
    broadcaster_id: X,
  ) -> Result<String, EventSubError> {
    let url = RequestBuilder::new()
      .add_key_value("broadcaster_id", broadcaster_id.into())
      .build(GET_MODERATORS_URL);

    TwitchHttpRequest::new(url)
      .header_authorisation(access_token.into(), AuthType::Bearer)
      .header_client_id(client_id.into())
      .run()
  }

  pub fn get_custom_rewards<T: Into<String>, S: Into<String>, X: Into<String>>(
    access_token: T,
    client_id: S,
    broadcaster_id: X,
  ) -> Result<String, EventSubError> {
    let url = RequestBuilder::new()
      .add_key_value("broadcaster_id", broadcaster_id.into())
      .build(CUSTOM_REWARDS_URL);
    TwitchHttpRequest::new(url)
      .header_authorisation(access_token.into(), AuthType::Bearer)
      .header_client_id(client_id.into())
      .run()
  }

  pub fn create_custom_reward<T: Into<String>, S: Into<String>, X: Into<String>>(
    access_token: T,
    client_id: S,
    broadcaster_id: X,
    custom_reward_data: CreateCustomReward,
  ) -> Result<String, EventSubError> {
    let url = RequestBuilder::new()
      .add_key_value("broadcaster_id", broadcaster_id.into())
      .build(CUSTOM_REWARDS_URL);
    let data = serde_json::to_string(&custom_reward_data).unwrap();
    TwitchHttpRequest::new(url)
      .header_authorisation(access_token.into(), AuthType::Bearer)
      .header_client_id(client_id.into())
      .json_content()
      .is_post(data)
      .run()
  }

  pub fn delete_custom_reward<
    T: Into<String>,
    S: Into<String>,
    X: Into<String>,
    Z: Into<String>,
  >(
    access_token: T,
    client_id: S,
    broadcaster_id: X,
    reward_id: Z,
  ) -> Result<String, EventSubError> {
    let url = RequestBuilder::new()
      .add_key_value("broadcaster_id", broadcaster_id.into())
      .add_key_value("id", reward_id.into())
      .build(CUSTOM_REWARDS_URL);

    TwitchHttpRequest::new(url)
      .header_authorisation(access_token.into(), AuthType::Bearer)
      .header_client_id(client_id.into())
      .is_delete()
      .run()
  }
}

#[derive(PartialEq, Clone, Debug)]
pub enum RequestType {
  Post(String),
  Delete,
}

#[derive(Clone, PartialEq, Debug)]
pub enum AuthType {
  Bearer,
  OAuth,
}

impl AuthType {
  pub fn to_string(&self) -> String {
    match self {
      AuthType::Bearer => "Bearer",
      AuthType::OAuth => "OAuth",
    }
    .into()
  }
}

impl RequestType {
  pub fn apply(&self, handle: &mut Easy) {
    match self {
      RequestType::Post(data) => {
        handle.post(true).unwrap();
        handle.post_fields_copy(data.as_bytes()).unwrap();
      }
      RequestType::Delete => {
        let _ = handle.custom_request("DELETE");
      }
    }
  }
}

pub struct RequestBuilder {
  data: Vec<(String, String)>,
}

impl RequestBuilder {
  fn new() -> RequestBuilder {
    RequestBuilder { data: Vec::new() }
  }

  fn add_key_value<S: Into<String>, T: Into<String>>(mut self, key: S, value: T) -> RequestBuilder {
    self.data.push((key.into(), value.into()));
    self
  }

  fn build<S: Into<String>>(self, url: S) -> String {
    let mut request = url.into();

    if !self.data.is_empty() {
      request = format!("{}?", request);
    }

    for (key, value) in self.data {
      request = format!("{}&{}={}", request, key, value);
    }

    request
  }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Header {
  Auth((AuthType, String)),
  ClientId(String),
  ContentJson,
  ContentUrlEncoded,
}

impl Header {
  pub fn generate(&self) -> String {
    match self {
      Header::Auth((auth_type, token)) => {
        format!("Authorization: {} {}", auth_type.to_string(), token)
      }
      Header::ClientId(id) => {
        format!("Client-Id: {}", id)
      }
      Header::ContentJson => {
        format!("Content-Type: application/json")
      }
      Header::ContentUrlEncoded => {
        format!("Content-Type: application/x-www-form-urlencoded")
      }
    }
  }
}

#[derive(Clone, PartialEq, Debug)]
pub struct TwitchHttpRequest {
  url: String,
  headers: Vec<Header>,
  request_type: Option<RequestType>,
}

impl TwitchHttpRequest {
  pub fn new<S: Into<String>>(url: S) -> TwitchHttpRequest {
    TwitchHttpRequest {
      url: url.into(),
      headers: Vec::new(),
      request_type: None,
    }
  }

  #[must_use]
  pub fn full_auth<S: Into<String>, T: Into<String>>(
    self,
    access_token: S,
    client_id: T,
  ) -> TwitchHttpRequest {
    self
      .header_authorisation(access_token, AuthType::Bearer)
      .header_client_id(client_id)
  }

  #[must_use]
  pub fn add_header(mut self, header: Header) -> TwitchHttpRequest {
    self.headers.push(header);
    self
  }

  #[must_use]
  pub fn header_authorisation<S: Into<String>>(
    mut self,
    token: S,
    auth_type: AuthType,
  ) -> TwitchHttpRequest {
    self.headers.push(Header::Auth((auth_type, token.into())));
    self
  }

  #[must_use]
  pub fn header_client_id<S: Into<String>>(mut self, client_id: S) -> TwitchHttpRequest {
    self.headers.push(Header::ClientId(client_id.into()));
    self
  }

  #[must_use]
  pub fn json_content(mut self) -> TwitchHttpRequest {
    self.headers.push(Header::ContentJson);
    self
  }

  #[must_use]
  pub fn url_encoded_content(mut self) -> TwitchHttpRequest {
    self.headers.push(Header::ContentUrlEncoded);
    self
  }

  #[must_use]
  pub fn is_delete(mut self) -> TwitchHttpRequest {
    self.request_type = Some(RequestType::Delete);
    self
  }

  #[must_use]
  pub fn is_post<S: Into<String>>(mut self, data: S) -> TwitchHttpRequest {
    self.request_type = Some(RequestType::Post(data.into()));
    self
  }

  pub fn update_token<S: Into<String>>(&mut self, new_token: S) {
    for header in &mut self.headers {
      if let Header::Auth((_, ref mut token)) = header {
        *token = new_token.into();
        break;
      }
    }
  }

  pub fn run(&self) -> Result<String, EventSubError> {
    let mut data = Vec::new();

    #[cfg(feature = "logging")]
    info!("Running curl command with:");
    #[cfg(feature = "logging")]
    info!("    url: {}", self.url);
    let mut handle = Easy::new();
    {
      handle.url(&self.url).unwrap();
      if let Some(request) = &self.request_type {
        request.apply(&mut handle);
      }

      let mut headers = List::new();
      for header in &self.headers {
        headers.append(&header.generate()).unwrap();
      }

      handle.http_headers(headers).unwrap();

      let mut handle = handle.transfer();
      // getting data back
      // idk why its called write function
      // that silly
      // we are reading whats coming back
      let _ = handle.write_function(|new_data| {
        data.extend_from_slice(new_data);
        Ok(new_data.len())
      });

      if let Err(e) = handle.perform() {
        #[cfg(feature = "logging")]
        error!("Curl error: {}", e);
        return Err(EventSubError::CurlFailed(e));
      }
    }

    let data = String::from_utf8_lossy(&data).to_string();
    if let Ok(error) = serde_json::from_str::<Validation>(&data) {
      if error.is_error() {
        if error.status.unwrap() == 429 {
          return Err(EventSubError::MaximumWebsocketTransmissionsExceeded(
            error.error_msg(),
          ));
        }
        if error.status.unwrap() == 401 {
          // Regen access token
          // Re run the query

          let error = error.message.unwrap();
          if error.contains("Missing scope") {
            let scope = error.split_whitespace().nth(2).unwrap();
            if let Some(missing_subscription) = Subscription::from_scope(scope) {
              #[cfg(feature = "logging")]
              error!(
                "Token missing subscription: Subscription::{:?}",
                missing_subscription
              );
              return Err(EventSubError::TokenMissingSubscription(Box::new(
                missing_subscription,
              )));
            } else {
              #[cfg(feature = "logging")]
              error!("Token missing unimplemented subscription: {}", scope);
              return Err(EventSubError::TokenMissingUnimplementedSubscription(
                scope.to_owned(),
              ));
            }
          } else {
            #[cfg(feature = "logging")]
            info!("Token requires refresing, debug: {:?}", error);
            return Err(EventSubError::TokenRequiresRefreshing(Box::new(
              self.to_owned(),
            )));
          }
        }
        #[cfg(feature = "logging")]
        error!("Unhandled error: {}, {}", self.url, error.error_msg());
        return Err(EventSubError::InvalidOauthToken(error.error_msg()));
      }
    }

    Ok(data)
  }
}
