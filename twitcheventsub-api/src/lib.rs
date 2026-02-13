#![allow(clippy::uninlined_format_args)]
use std::{
  io::{BufRead, Read, stdin},
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
pub const CONNECTION_EVENTS: &str = "wss://eventsub.wss.twitch.tv/ws?keepalive_timeout_seconds=30";
pub const SUBSCRIBE_URL: &str = "https://api.twitch.tv/helix/eventsub/subscriptions";
pub const SEND_ANNOUNCEMENT_URL: &str = "https://api.twitch.tv/helix/chat/announcements";
pub const SEND_SHOUTOUT_URL: &str = "https://api.twitch.tv/helix/chat/shoutouts";
pub const TWITCH_BOT_AUTHORISE_URL: &str = "https://id.twitch.tv/oauth2/authorize?";
pub const TWITCH_BAN_URL: &str = "https://api.twitch.tv/helix/moderation/bans";
pub const TWITCH_DELETE_MESSAGE_URL: &str = "https://api.twitch.tv/helix/moderation/chat";
pub const GET_AD_SCHEDULE_URL: &str = "https://api.twitch.tv/helix/channels/ads";
pub const GET_CHATTERS_URL: &str = "https://api.twitch.tv/helix/chat/chatters";
pub const GET_CHANNEL_BADGES_URL: &str = "https://api.twitch.tv/helix/chat/badges";
pub const GET_MODERATORS_URL: &str = "https://api.twitch.tv/helix/moderation/moderators";
pub const GET_GLOBAL_BADGES_URL: &str = "https://api.twitch.tv/helix/chat/badges/global";
pub const CUSTOM_REWARDS_URL: &str = "https://api.twitch.tv/helix/channel_points/custom_rewards";
pub const GET_CLIPS_URL: &str = "https://api.twitch.tv/helix/clips";
pub const GET_HYPE_TRAIN_URL: &str = "https://api.twitch.tv/helix/hypetrain/status";

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

pub fn get_users<I: Into<String>, S: Into<String>>(
  user_token: &str,
  client_id: &str,
  id: Vec<I>,
  login: Vec<S>,
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
    .header_authorisation(user_token, AuthType::Bearer)
    .header_client_id(client_id)
    .run()
}

///
/// Returns Ok(Some(code)) when cpature code via localhost is set
///   and the redirect url is localhost / same machine
///
/// Otherwise, deal with user inputing code manually
///
pub fn get_authorisation_code_grant_flow_user_token<S: Into<String>, T: Into<String>>(
  client_id: S,
  redirect_url: T,
  scopes: &[Subscription],
  auto_open_browser: bool,
  capture_code_via_localhost: bool,
) -> Result<Option<String>, TwitchApiError> {
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
  dbg!(&redirect_url);

  let browser = open_browser(&get_authorisation_code_request, auto_open_browser);

  if browser.is_ok() {
    if capture_code_via_localhost {
      let url = redirect_url
        .split("http://")
        .map(|s| String::from(s))
        .collect::<Vec<String>>()[1]
        .clone();
      let listener = TcpListener::bind(&url).expect("Failed to create tcp listener.");

      // accept connections and process them serially
      return match listener.accept() {
        Ok((mut stream, _b)) => {
          let mut http_output = String::new();
          stream
            .read_to_string(&mut http_output)
            .expect("Failed to read tcp stream.");
          Ok(Some(
            http_output.split('&').collect::<Vec<_>>()[0]
              .split('=')
              .collect::<Vec<_>>()[1]
              .to_string(),
          ))
        }
        Err(e) => Err(TwitchApiError::HttpError(e.to_string())),
      }
    } else {
      Ok(None)
    }
  } else {
    Err(browser.err().unwrap())
  }
}

pub fn get_user_and_refresh_token_from_authorisation_code(
  client_id: &str,
  client_secret: &str,
  authorisation_code: &str,
  redirect_url: &str,
) -> Result<(String, String), TwitchApiError> {
  let post_data = format!(
    "client_id={}&client_secret={}&code={}&grant_type=authorization_code&redirect_uri={}",
    client_id, client_secret, authorisation_code, redirect_url
  );

  create_user_and_refresh_token(&post_data)
}

pub fn validate_token(token: &str) -> Result<Validation, TwitchApiError> {
  TwitchHttpRequest::new(VALIDATION_TOKEN_URL)
    .header_authorisation(token, AuthType::OAuth)
    .run()
    .and_then(|data| {
      serde_json::from_str::<Validation>(&data)
        .map_err(|e| TwitchApiError::DeserialisationError(e.to_string()))
    })
}

pub fn open_browser(browser_url: &str, auto_open_browser: bool) -> Result<(), TwitchApiError> {
  if let Err(response) = TwitchHttpRequest::new(browser_url).run() {
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

  Ok(())

  //let mut redirect_url = redirect_url.into().to_ascii_lowercase();

  //if redirect_url.contains("http") && redirect_url.contains("/") {
  //  redirect_url = redirect_url
  //    .split('/')
  //    .collect::<Vec<_>>()
  //    .last()
  //    .unwrap()
  //    .to_string();
  //}

  //if manual_code_input {
  //  println!("Please input your access token:");
  //  let stdin = stdin();
  //  for line in stdin.lock().lines() {
  //    let code = line.unwrap();
  //    return Ok(code);
  //  }

  //  Err(TwitchApiError::InputError(
  //    "Failed to get user input.".to_string(),
  //  ))
  //} else {
  //  println!("Starting local tcp listener for token generation");
  //  println!("Please click the blue redirect link if browser doesn't redirect.");
  //  dbg!(&redirect_url);
  //  let listener = TcpListener::bind(&redirect_url).expect("Failed to create tcp listener.");

  //  dbg!("After listener");

  //  // accept connections and process them serially
  //  match listener.accept() {
  //    Ok((mut stream, _b)) => {
  //      let mut http_output = String::new();
  //      stream
  //        .read_to_string(&mut http_output)
  //        .expect("Failed to read tcp stream.");
  //      dbg!(&http_output);
  //      Ok(http_output)
  //    }
  //    Err(e) => Err(TwitchApiError::HttpError(e.to_string())),
  //  }
  //}
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
  user_token: &str,
  client_id: &str,
  broadcaster_id: &str,
) -> Result<ChannelEmotes, TwitchApiError> {
  let url = RequestBuilder::new()
    .add_key_value("broadcaster_id", broadcaster_id)
    .build(GET_CHANNEL_EMOTES_URL);

  TwitchHttpRequest::new(url)
    .header_authorisation(user_token, AuthType::Bearer)
    .header_client_id(client_id)
    .run()
    .and_then(|data| {
      serde_json::from_str::<ChannelEmotes>(&data)
        .map_err(|e| TwitchApiError::DeserialisationError(e.to_string()))
    })
}

pub fn get_global_emotes(
  user_token: &str,
  client_id: &str,
) -> Result<GlobalEmotes, TwitchApiError> {
  let url = RequestBuilder::new().build(GET_GLOBAL_EMOTES_URL);

  TwitchHttpRequest::new(url)
    .header_authorisation(user_token, AuthType::Bearer)
    .header_client_id(client_id)
    .run()
    .and_then(|data| {
      serde_json::from_str::<GlobalEmotes>(&data)
        .map_err(|e| TwitchApiError::DeserialisationError(e.to_string()))
    })
}

pub fn get_emote_set(
  emote_set_id: &str,
  user_token: &str,
  client_id: &str,
) -> Result<GlobalEmotes, TwitchApiError> {
  let url = RequestBuilder::new()
    .add_key_value("emote_set_id", emote_set_id)
    .build(GET_EMOTE_SETS_URL);

  TwitchHttpRequest::new(url)
    .header_authorisation(user_token, AuthType::Bearer)
    .header_client_id(client_id)
    .run()
    .and_then(|data| {
      serde_json::from_str::<GlobalEmotes>(&data)
        .map_err(|e| TwitchApiError::DeserialisationError(e.to_string()))
    })
}

pub fn get_ad_schedule(
  broadcaster_id: &str,
  access_token: &str,
  client_id: &str,
) -> Result<String, TwitchApiError> {
  let url = RequestBuilder::new()
    .add_key_value("broadcaster_id", broadcaster_id)
    .build(GET_AD_SCHEDULE_URL);

  TwitchHttpRequest::new(url)
    .header_authorisation(access_token, AuthType::Bearer)
    .header_client_id(client_id)
    .run()
}

pub fn get_chatters(
  broadcaster_id: &str,
  moderator_id: &str,
  access_token: &str,
  client_id: &str,
) -> Result<String, TwitchApiError> {
  let url = RequestBuilder::new()
    .add_key_value("broadcaster_id", broadcaster_id)
    .add_key_value("moderator_id", moderator_id)
    .build(GET_CHATTERS_URL);

  TwitchHttpRequest::new(url)
    .header_authorisation(access_token, AuthType::Bearer)
    .header_client_id(client_id)
    .run()
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

pub fn send_announcement<P: Into<String>>(
  user_token: &str,
  client_id: &str,
  sender_account_id: &str,
  broadcaster_account_id: &str,
  message: &str,
  colour: Option<P>,
) -> Result<String, TwitchApiError> {
  if message.len() > 500 {
    return Err(TwitchApiError::InputError(String::from(
      "Message is too long.",
    )));
  }

  let url = RequestBuilder::new()
    .add_key_value("broadcaster_id", broadcaster_account_id)
    .add_key_value("moderator_id", sender_account_id)
    .build(SEND_ANNOUNCEMENT_URL);

  TwitchHttpRequest::new(url)
    .json_content()
    .full_auth(user_token, client_id)
    .is_post(
      serde_json::to_string(&AnnouncementMessage {
        message: message.to_owned(),
        colour: colour.map(|c| c.into()),
      })
      .unwrap(),
    )
    .run()
}

pub fn send_shoutout(
  user_token: &str,
  client_id: &str,
  moderator_id: &str,
  from_broadcaster_id: &str,
  to_broadcaster_id: &str,
) -> Result<String, TwitchApiError> {
  let url = RequestBuilder::new()
    .add_key_value("from_broadcaster_id", from_broadcaster_id)
    .add_key_value("to_broadcaster_id", to_broadcaster_id)
    .add_key_value("moderator_id", moderator_id)
    .build(SEND_SHOUTOUT_URL);

  TwitchHttpRequest::new(url)
    .header_authorisation(user_token, AuthType::Bearer)
    .header_client_id(client_id)
    .json_content()
    .is_post("")
    .run()
}

pub fn delete_message(
  user_token: &str,
  client_id: &str,
  sender_id: &str,
  broadcaster_id: &str,
  message_id: &str,
) -> Result<String, TwitchApiError> {
  let url = RequestBuilder::new()
    .add_key_value("broadcaster_id", broadcaster_id)
    .add_key_value("moderator_id", sender_id)
    .add_key_value("message_id", message_id)
    .build(TWITCH_DELETE_MESSAGE_URL);

  TwitchHttpRequest::new(url)
    .header_authorisation(user_token, AuthType::Bearer)
    .header_client_id(client_id)
    .is_delete()
    .run()
}

pub fn timeout_user(
  user_token: &str,
  client_id: &str,
  moderator_id: &str,
  broadcaster_id: &str,
  user_id: &str,
  duration_secs: Option<u32>,
  reason: &str,
) -> Result<String, TwitchApiError> {
  let url = RequestBuilder::new()
    .add_key_value("broadcaster_id", broadcaster_id)
    .add_key_value("moderator_id", moderator_id)
    .build(TWITCH_BAN_URL);

  let post_data = SendTimeoutRequest {
    data: TimeoutRequestData {
      user_id: user_id.to_owned(),
      duration: duration_secs,
      reason: reason.to_owned(),
    },
  };

  let post_data = serde_json::to_string(&post_data).unwrap();

  TwitchHttpRequest::new(url)
    .header_authorisation(user_token, AuthType::Bearer)
    .header_client_id(client_id)
    .json_content()
    .is_post(post_data)
    .run()
}

pub fn get_channel_badges(
  user_token: &str,
  client_id: &str,
  broadcaster_id: &str,
) -> Result<String, TwitchApiError> {
  let url = RequestBuilder::new()
    .add_key_value("broadcaster_id", broadcaster_id)
    .build(GET_CHANNEL_BADGES_URL);

  TwitchHttpRequest::new(url)
    .header_authorisation(user_token, AuthType::Bearer)
    .header_client_id(client_id)
    .run()
}

pub fn get_global_badges(user_token: &str, client_id: &str) -> Result<String, TwitchApiError> {
  let url = RequestBuilder::new().build(GET_GLOBAL_BADGES_URL);

  TwitchHttpRequest::new(url)
    .header_authorisation(user_token, AuthType::Bearer)
    .header_client_id(client_id)
    .run()
}

pub fn get_moderators(
  user_token: &str,
  client_id: &str,
  broadcaster_id: &str,
) -> Result<String, TwitchApiError> {
  let url = RequestBuilder::new()
    .add_key_value("broadcaster_id", broadcaster_id)
    .build(GET_MODERATORS_URL);

  TwitchHttpRequest::new(url)
    .header_authorisation(user_token, AuthType::Bearer)
    .header_client_id(client_id)
    .run()
}

pub fn get_custom_rewards(
  user_token: &str,
  client_id: &str,
  broadcaster_id: &str,
) -> Result<String, TwitchApiError> {
  let url = RequestBuilder::new()
    .add_key_value("broadcaster_id", broadcaster_id)
    .build(CUSTOM_REWARDS_URL);
  TwitchHttpRequest::new(url)
    .header_authorisation(user_token, AuthType::Bearer)
    .header_client_id(client_id)
    .run()
}

pub fn update_custom_rewards(
  user_token: &str,
  client_id: &str,
  broadcaster_id: &str,
  redeem_id: &str,
  update_redeem: &UpdateCustomReward,
) -> Result<String, TwitchApiError> {
  let url = RequestBuilder::new()
    .add_key_value("broadcaster_id", broadcaster_id)
    .add_key_value("id", redeem_id)
    .build(CUSTOM_REWARDS_URL);

  TwitchHttpRequest::new(url)
    .header_authorisation(user_token, AuthType::Bearer)
    .header_client_id(client_id)
    .json_content()
    .is_patch(serde_json::to_string(&update_redeem).unwrap())
    .run()
}

pub fn create_custom_reward(
  user_token: &str,
  client_id: &str,
  broadcaster_id: &str,
  custom_reward_data: CreateCustomReward,
) -> Result<String, TwitchApiError> {
  let url = RequestBuilder::new()
    .add_key_value("broadcaster_id", broadcaster_id)
    .build(CUSTOM_REWARDS_URL);
  let data = serde_json::to_string(&custom_reward_data).unwrap();
  TwitchHttpRequest::new(url)
    .header_authorisation(user_token, AuthType::Bearer)
    .header_client_id(client_id)
    .json_content()
    .is_post(data)
    .run()
}

pub fn delete_custom_reward(
  user_token: &str,
  client_id: &str,
  broadcaster_id: &str,
  reward_id: &str,
) -> Result<String, TwitchApiError> {
  let url = RequestBuilder::new()
    .add_key_value("broadcaster_id", broadcaster_id)
    .add_key_value("id", reward_id)
    .build(CUSTOM_REWARDS_URL);

  TwitchHttpRequest::new(url)
    .header_authorisation(user_token, AuthType::Bearer)
    .header_client_id(client_id)
    .is_delete()
    .run()
}

pub fn get_clips(
  user_token: &str,
  client_id: &str,
  broadcaster_id: &str,
) -> Result<String, TwitchApiError> {
  let url = RequestBuilder::new()
    .add_key_value("broadcaster_id", broadcaster_id)
    .build(GET_CLIPS_URL);
  TwitchHttpRequest::new(url)
    .header_authorisation(user_token, AuthType::Bearer)
    .header_client_id(client_id)
    .run()
}

pub fn get_hype_train_status(
  user_token: &str,
  client_id: &str,
  broadcaster_id: &str,
) -> Result<String, TwitchApiError> {
  let url = RequestBuilder::new()
    .add_key_value("broadcaster_id", broadcaster_id)
    .build(GET_HYPE_TRAIN_URL);

  TwitchHttpRequest::new(url)
    .header_authorisation(user_token, AuthType::Bearer)
    .header_client_id(client_id)
    .run()
}
