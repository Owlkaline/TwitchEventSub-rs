use std::{
  io::{stdin, BufRead, Read},
  net::TcpListener,
};

use curl::Error;
use request::*;
use twitcheventsub_structs::prelude::*;

pub const GET_USERS_URL: &str = "https://api.twitch.tv/helix/users";
pub const TWITCH_AUTHORISE_URL: &str = "https://id.twitch.tv/oauth2/";
pub const TWITCH_TOKEN_URL: &str = "https://id.twitch.tv/oauth2/token";
pub const VALIDATION_TOKEN_URL: &str = "https://id.twitch.tv/oauth2/validate";
pub const GET_GLOBAL_EMOTES_URL: &str = "https://api.twitch.tv/helix/chat/emotes/global";
pub const GET_EMOTE_SETS_URL: &str = "https://api.twitch.tv/helix/chat/emotes/set";
pub const GET_CHANNEL_EMOTES_URL: &str = "https://api.twitch.tv/helix/chat/emotes";
pub const SEND_MESSAGE_URL: &str = "https://api.twitch.tv/helix/chat/messages";

mod request;
pub use request::TwitchHttpRequest;

#[derive(Debug, PartialEq)]
pub enum TwitchApiError {
  CurlFailed(Error),
  MaximumWebsocketTransmissionsExceeded(String),
  TokenMissingSubscription(Box<Subscription>),
  TokenMissingUnimplementedSubscription(String),
  TokenRequiresRefreshing(Box<TwitchHttpRequest>),
  InvalidOauthToken(String),
  InvalidAuthorisationCode(String),
  BrowserError(String),
  HttpError(String),
  InputError(String),
  DeserialisationError(String),
}

pub fn get_users<T: Into<String>, I: Into<String>, S: Into<String>, V: Into<String>>(
  access_token: T,
  id: Vec<I>,
  login: Vec<S>,
  client_id: V,
) -> Result<String, TwitchApiError> {
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

pub fn get_implicit_grant_flow_user_token(
  client_id: &str,
  redirect_url: &str,
  scopes: &[Subscription],
  auto_open_browser: bool,
  manual_code_input: bool,
) -> Result<String, TwitchApiError> {
  let scope = &scopes
    .iter()
    .map(|s| s.required_scope())
    .filter(|s| !s.is_empty())
    .collect::<Vec<String>>()
    .join("+");

  let get_authorisation_code_request = format!(
    "{}authorize?response_type=token&client_id={}&redirect_uri={}&scope={}&force_verify=true",
    TWITCH_AUTHORISE_URL,
    client_id,
    redirect_url.to_owned(),
    scope
  );

  match open_browser(
    get_authorisation_code_request,
    redirect_url,
    auto_open_browser,
    manual_code_input,
  ) {
    Ok(http_response) => {
      if http_response.contains("error") {
        Err(TwitchApiError::HttpError(format!("{}", http_response)))
      } else {
        let auth_code = if manual_code_input {
          http_response.trim()
        } else {
          http_response.split('&').collect::<Vec<_>>()[0]
            .split('=')
            .collect::<Vec<_>>()[1]
        };

        Ok(auth_code.to_owned())
      }
    }
    e => e,
  }
}

pub fn get_authorisation_code_grant_flow_user_token<S: Into<String>, T: Into<String>>(
  client_id: S,
  redirect_url: T,
  scopes: &[Subscription],
  auto_open_browser: bool,
  manual_code_input: bool,
) -> Result<String, TwitchApiError> {
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

  match open_browser(
    get_authorisation_code_request,
    redirect_url,
    auto_open_browser,
    manual_code_input,
  ) {
    Ok(http_response) => {
      if http_response.contains("error") {
        Err(TwitchApiError::HttpError(http_response))
      } else {
        if manual_code_input {
          Ok(http_response)
        } else {
          let auth_code = http_response.split('&').collect::<Vec<_>>()[0]
            .split('=')
            .collect::<Vec<_>>()[1];
          Ok(auth_code.to_string())
        }
      }
    }
    e => e,
  }
}

pub fn get_user_and_refresh_token_from_authorisation_code<
  S: Into<String>,
  T: Into<String>,
  V: Into<String>,
  W: Into<String>,
>(
  client_id: S,
  client_secret: T,
  authorisation_code: V,
  redirect_url: W,
) -> Result<(String, String), TwitchApiError> {
  let post_data = format!(
    "client_id={}&client_secret={}&code={}&grant_type=authorization_code&redirect_uri={}",
    client_id.into(),
    client_secret.into(),
    authorisation_code.into(),
    redirect_url.into()
  );

  create_user_and_refresh_token(&post_data)
}

pub fn validate_token<S: Into<String>>(token: S) -> Result<Validation, TwitchApiError> {
  TwitchHttpRequest::new(VALIDATION_TOKEN_URL)
    .header_authorisation(token.into(), AuthType::OAuth)
    .run()
    .and_then(|data| {
      serde_json::from_str::<Validation>(&data)
        .map_err(|e| TwitchApiError::DeserialisationError(e.to_string()))
    })
}

//fn create_user_and_refresh_token_from_data(post_data: &str) -> Result<(String, String), TwitchApiError> {
//  TwitchHttpRequest::new(TWITCH_TOKEN_URL)
//    .url_encoded_content()
//    .is_post(post_data)
//    .run()
//    .and_then(|twitch_response| {
//      serde_json::from_str::<NewAccessTokenResponse>(&twitch_response)
//        .map_err(|_|
//          TwitchApiError::FailedToCreateFre(twitch_response)
//        .map(|new_token_data| {
//          // Token::new_user_token(
//          (
//            new_token_data.access_token,
//            new_token_data.refresh_token.unwrap(),
//          )
//          //  new_token_data.expires_in as f32,
//          // )
//        })
//    })
//}

pub fn open_browser<S: Into<String>, T: Into<String>>(
  browser_url: S,
  redirect_url: T,
  auto_open_browser: bool,
  manual_code_input: bool,
) -> Result<String, TwitchApiError> {
  let browser_url = browser_url.into();

  if let Err(response) = TwitchHttpRequest::new(&browser_url).run() {
    return Err(response);
  }

  if auto_open_browser {
    if let Err(e) = open::that_detached(browser_url) {
      #[cfg(feature = "logging")]
      error!("Failed to open browser: {}", e);
      return Err(TwitchApiError::BrowserError(e.to_string()));
    }
  } else {
    println!(
      "Please visit the following link to have your token be authorised and generated:\n{}",
      browser_url
    );
  }

  let mut redirect_url = redirect_url.into().to_ascii_lowercase();

  if redirect_url.contains("http") && redirect_url.contains("/") {
    redirect_url = redirect_url
      .split('/')
      .collect::<Vec<_>>()
      .last()
      .unwrap()
      .to_string();
  }

  if manual_code_input {
    println!("Please input your access token:");
    let stdin = stdin();
    for line in stdin.lock().lines() {
      let code = line.unwrap();
      return Ok(code);
    }

    Err(TwitchApiError::InputError(
      "Failed to get user input.".to_string(),
    ))
  } else {
    println!("Starting local tcp listener for token generation");
    println!("Please click the blue redirect link if browser doesn't redirect.");
    dbg!(&redirect_url);
    let listener = TcpListener::bind(&redirect_url).expect("Failed to create tcp listener.");

    dbg!("After listener");

    // accept connections and process them serially
    match listener.accept() {
      Ok((mut stream, _b)) => {
        let mut http_output = String::new();
        stream
          .read_to_string(&mut http_output)
          .expect("Failed to read tcp stream.");
        dbg!(&http_output);
        Ok(http_output)
      }
      Err(e) => Err(TwitchApiError::HttpError(e.to_string())),
    }
  }
}

pub fn create_user_and_refresh_token(post_data: &str) -> Result<(String, String), TwitchApiError> {
  TwitchHttpRequest::new(TWITCH_TOKEN_URL)
    .url_encoded_content()
    .is_post(post_data)
    .run()
    .and_then(|twitch_response| {
      serde_json::from_str::<NewAccessTokenResponse>(&twitch_response)
        .map_err(|_| TwitchApiError::DeserialisationError(twitch_response))
        .map(|new_token_data| {
          (
            new_token_data.access_token,
            new_token_data.refresh_token.unwrap(),
          )
        })
    })
}

pub fn get_channel_emotes(
  access_token: &str,
  client_id: &str,
  broadcaster_id: &str,
) -> Result<ChannelEmotes, TwitchApiError> {
  let url = RequestBuilder::new()
    .add_key_value("broadcaster_id", broadcaster_id)
    .build(GET_CHANNEL_EMOTES_URL);

  TwitchHttpRequest::new(url)
    .header_authorisation(access_token, AuthType::Bearer)
    .header_client_id(client_id)
    .run()
    .and_then(|data| {
      serde_json::from_str::<ChannelEmotes>(&data)
        .map_err(|e| TwitchApiError::DeserialisationError(e.to_string()))
    })
}

pub fn get_global_emotes(
  access_token: &str,
  client_id: &str,
) -> Result<GlobalEmotes, TwitchApiError> {
  let url = RequestBuilder::new().build(GET_GLOBAL_EMOTES_URL);

  TwitchHttpRequest::new(url)
    .header_authorisation(access_token, AuthType::Bearer)
    .header_client_id(client_id)
    .run()
    .and_then(|data| {
      serde_json::from_str::<GlobalEmotes>(&data)
        .map_err(|e| TwitchApiError::DeserialisationError(e.to_string()))
    })
}

pub fn get_emote_set(
  emote_set_id: &str,
  access_token: &str,
  client_id: &str,
) -> Result<GlobalEmotes, TwitchApiError> {
  let url = RequestBuilder::new()
    .add_key_value("emote_set_id", emote_set_id)
    .build(GET_EMOTE_SETS_URL);

  TwitchHttpRequest::new(url)
    .header_authorisation(access_token, AuthType::Bearer)
    .header_client_id(client_id)
    .run()
    .and_then(|data| {
      serde_json::from_str::<GlobalEmotes>(&data)
        .map_err(|e| TwitchApiError::DeserialisationError(e.to_string()))
    })
}

pub fn send_chat_message(
  user_token: &str,
  client_id: &str,
  sender_id: &str,
  broadcaster_id: &str,
  message: &str,
) -> Result<String, TwitchApiError> {
  send_chat_message_with_reply(
    user_token,
    client_id,
    sender_id,
    broadcaster_id,
    message,
    None,
  )
}

pub fn send_chat_message_with_reply(
  user_token: &str,
  client_id: &str,
  sender_id: &str,
  broadcaster_id: &str,
  message: &str,
  reply_message_parent_id: Option<String>,
) -> Result<String, TwitchApiError> {
  if message.len() > 500 {
    return Err(TwitchApiError::InputError(String::from(
      "Message Length is too long.",
    )));
  }

  TwitchHttpRequest::new(SEND_MESSAGE_URL)
    .json_content()
    .full_auth(user_token, client_id)
    .is_post(
      serde_json::to_string(&SendMessage {
        broadcaster_id: broadcaster_id.to_owned(),
        sender_id: sender_id.to_owned(),
        message: message.into(),
        reply_parent_message_id: reply_message_parent_id,
      })
      .unwrap(),
    )
    .run()
}
