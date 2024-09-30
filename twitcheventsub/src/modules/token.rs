#[cfg(feature = "logging")]
use log::{error, info};

use crate::modules::errors::*;

use std::fs;
use std::io::Write;

#[derive(Debug)]
pub struct Token {
  pub access: TokenAccess,
  pub refresh: String,
}

impl Token {
  pub fn new(access: TokenAccess, refresh: String, _expires_in: f32) -> Token {
    Token { access, refresh }
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
          #[cfg(feature = "logging")]
          info!("Saving token failed: {}", e);
          return Err(EventSubError::WriteError(e.to_string()));
        }
      }
      Err(e) => {
        #[cfg(feature = "logging")]
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
        #[cfg(feature = "logging")]
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

#[derive(Clone, Debug)]
pub struct TwitchKeys {
  //  pub oauth: String,
  pub authorisation_code: Option<String>,
  pub access_token: Option<TokenAccess>,
  pub refresh_token: Option<String>,
  pub client_id: String,
  pub client_secret: String,

  pub broadcaster_account_id: String,
  pub this_account_id: String,
  pub sender_account_id: Option<String>,
}

impl Default for TwitchKeys {
  fn default() -> Self {
    TwitchKeys {
      authorisation_code: None,
      access_token: None,
      refresh_token: None,
      client_id: "".to_owned(),
      client_secret: "".to_owned(),
      broadcaster_account_id: "".to_owned(),
      this_account_id: "".to_owned(),
      sender_account_id: None,
    }
  }
}

impl TwitchKeys {
  pub fn token(&self) -> Option<Token> {
    if let (Some(access), Some(refresh)) =
      (self.access_token.to_owned(), self.refresh_token.to_owned())
    {
      Some(Token { access, refresh })
    } else {
      None
    }
  }

  pub fn from_secrets_env() -> Result<TwitchKeys, TwitchKeysError> {
    simple_env_load::load_env_from([".example.env", ".secrets.env"]);

    fn get(key: &str) -> Result<String, String> {
      std::env::var(key).map_err(|_| format!("please set {key} in .example.env"))
    }

    let client_id = match get("TWITCH_CLIENT_ID") {
      Ok(s) => s,
      Err(e) => {
        #[cfg(feature = "logging")]
        error!("{}", e);
        return Err(TwitchKeysError::ClientIdNotFound(e));
      }
    };

    let client_secret = match get("TWITCH_CLIENT_SECRET") {
      Ok(s) => s,
      Err(e) => {
        #[cfg(feature = "logging")]
        error!("{}", e);
        return Err(TwitchKeysError::ClientSecretNotFound(e));
      }
    };

    let broadcaster_id = match get("TWITCH_BROADCASTER_ID") {
      Ok(s) => s,
      Err(e) => {
        #[cfg(feature = "logging")]
        error!("{}", e);
        return Err(TwitchKeysError::ClientSecretNotFound(e));
      }
    };

    let bot_account_id = get("TWITCH_BOT_ID").ok();

    let user_access_token = get("TWITCH_USER_ACCESS_TOKEN").ok().map(TokenAccess::User);
    let user_refresh_token = get("TWITCH_USER_REFRESH_TOKEN").ok();

    Ok(TwitchKeys {
      authorisation_code: None,
      access_token: user_access_token,
      refresh_token: user_refresh_token,
      client_id,
      client_secret,
      broadcaster_account_id: broadcaster_id.clone(),
      this_account_id: broadcaster_id,
      sender_account_id: bot_account_id,
    })
  }
}
