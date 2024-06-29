use log::{error, info};

use crate::modules::errors::*;

use std::fs;
use std::io::Write;

pub struct Token {
  pub access: TokenAccess,
  pub refresh: String,
  expires_in: f32,
}

impl Token {
  pub fn new(access: TokenAccess, refresh: String, expires_in: f32) -> Token {
    Token {
      access,
      refresh,
      expires_in,
    }
  }

  pub fn save_to_file<S: Into<String>, T: Into<String>>(
    &self,
    token_file: S,
    refresh_file: T,
  ) -> Result<(), EventSubError> {
    match fs::OpenOptions::new()
      .append(false)
      .create(true)
      .write(true)
      .truncate(true)
      .open(format!("./{}", token_file.into()))
    {
      Ok(mut writer) => {
        if let Err(e) = writer.write(format!("{}", self.access.get_token()).as_bytes()) {
          info!("Saving token failed: {}", e);
          return Err(EventSubError::WriteError(e.to_string()));
        }
      }
      Err(e) => {
        error!("Writing failed: {}", e);
      }
    }

    if let Ok(mut writer) = fs::OpenOptions::new()
      .append(false)
      .create(true)
      .write(true)
      .truncate(true)
      .open(refresh_file.into())
    {
      if let Err(e) = writer.write(format!("{}", self.refresh).as_bytes()) {
        info!("Saving token failed: {}", e);
        return Err(EventSubError::WriteError(e.to_string()));
      }
    }

    Ok(())
  }

  pub fn new_user_token(access_token: String, refresh: String, expires_in: f32) -> Token {
    Token::new(TokenAccess::User(access_token), refresh, expires_in)
  }

  pub fn new_app_token(access_token: String, refresh: String, expires_in: f32) -> Token {
    Token::new(TokenAccess::App(access_token), refresh, expires_in)
  }
}

#[derive(Clone, Debug)]
pub enum TokenAccess {
  App(String),
  User(String),
}

impl TokenAccess {
  pub fn get_token(&self) -> String {
    match self {
      TokenAccess::User(token) => token.to_string(),
      TokenAccess::App(token) => token.to_string(),
    }
  }
}

/// When subscribing to events, webhooks uses app access tokens and WebSockets uses user access tokens.
/// If you use app access tokens with WebSockets, the subscriptions will fail.

#[derive(Clone)]
pub struct TwitchKeys {
  //  pub oauth: String,
  pub authorisation_code: Option<String>,
  pub access_token: Option<TokenAccess>,
  pub refresh_token: Option<String>,
  pub client_id: String,
  pub client_secret: String,

  pub broadcaster_account_id: String,
  pub sender_account_id: Option<String>,
}

impl TwitchKeys {
  pub fn from_secrets_env() -> Result<TwitchKeys, TwitchKeysError> {
    simple_env_load::load_env_from([".example.env", ".secrets.env"]);

    fn get(key: &str) -> Result<String, String> {
      std::env::var(key).map_err(|_| format!("please set {key} in .example.env"))
    }

    let client_id = match get("TWITCH_CLIENT_ID") {
      Ok(s) => s,
      Err(e) => {
        error!("{}", e);
        return Err(TwitchKeysError::ClientIdNotFound);
      }
    };

    let client_secret = match get("TWITCH_CLIENT_SECRET") {
      Ok(s) => s,
      Err(e) => {
        error!("{}", e);
        return Err(TwitchKeysError::ClientSecretNotFound);
      }
    };

    let broadcaster_id = match get("TWITCH_BROADCASTER_ID") {
      Ok(s) => s,
      Err(e) => {
        error!("{}", e);
        return Err(TwitchKeysError::ClientSecretNotFound);
      }
    };

    let bot_account_id = get("TWITCH_BOT_ID").unwrap_or(broadcaster_id.to_owned());

    let user_access_token = get("TWITCH_USER_ACCESS_TOKEN").ok().map(TokenAccess::User);
    let user_refresh_token = get("TWITCH_USER_REFRESH_TOKEN").ok();

    Ok(TwitchKeys {
      authorisation_code: None,
      access_token: user_access_token,
      refresh_token: user_refresh_token,
      client_id,
      client_secret,
      broadcaster_account_id: broadcaster_id,
      sender_account_id: Some(bot_account_id),
    })
  }
}
