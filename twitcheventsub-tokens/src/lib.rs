use std::{
  fs::{self, OpenOptions},
  io::{stdin, Write},
  process::exit,
};

use env_file_reader::read_file;
use env_handler::EnvHandler;
use log::{error, info, warn};
use twitcheventsub_api::{self, TwitchApiError};
use twitcheventsub_structs::{Subscription, UserData, UserDataSet};

mod builder;
mod env_handler;

pub use builder::TokenHandlerBuilder;

use crate::builder::get_user_and_refresh_tokens;

#[derive(Default, Clone, Debug)]
pub struct TokenHandler {
  pub user_token: String,
  pub refresh_token: String,

  pub redirect_url: String,

  pub client_id: String,
  pub client_secret: String,
  pub client_twitch_id: String,

  user_token_env: String,
  refresh_token_env: String,
}

impl TokenHandler {
  pub fn builder() -> TokenHandlerBuilder {
    TokenHandlerBuilder::default()
  }

  pub fn new() -> TokenHandler {
    TokenHandler::default()
  }

  pub fn check_token_has_required_subscriptions(
    &self,
    subs: &[Subscription],
  ) -> Result<bool, TwitchApiError> {
    twitcheventsub_api::validate_token(&self.user_token).and_then(|validation| {
      Ok(
        subs
          .iter()
          .filter(|s| !s.required_scope().is_empty())
          .all(move |s| {
            let r = s.required_scope();

            let requirements = r.split('+').map(ToString::to_string).collect::<Vec<_>>();

            for req in requirements {
              if !validation
                .scopes
                .as_ref()
                .unwrap_or(&Vec::new())
                .contains(&req)
              {
                return false;
              }
            }
            true
          }),
      )
    })
  }

  pub fn apply_subscriptions_to_tokens(&mut self, scopes: &[Subscription], open_browser: bool) {
    let (user_token, refresh_token) = get_user_and_refresh_tokens(
      false,
      &self.client_id,
      &self.client_secret,
      &self.redirect_url,
      scopes,
      open_browser,
    );

    self.user_token = user_token;
    self.refresh_token = refresh_token;

    EnvHandler::save_user_token(&self.user_token_env, &self.user_token);
    EnvHandler::save_refresh_token(&self.refresh_token_env, &self.refresh_token);
  }

  pub fn generate_user_token_from_refresh_token(&mut self) -> Result<(), TwitchApiError> {
    let post_data = format!(
      "grant_type=refresh_token&refresh_token={}&client_id={}&client_secret={}",
      self.refresh_token, self.client_id, self.client_secret
    );

    twitcheventsub_api::create_user_and_refresh_token(&post_data).map(
      |(user_token, refresh_token)| {
        self.user_token = user_token;
        self.refresh_token = refresh_token;
        Ok(())
      },
    )?
  }

  pub fn regen_tokens_on_fail(
    &mut self,
    twitch_result: Result<String, TwitchApiError>,
  ) -> Result<String, TwitchApiError> {
    if let Err(TwitchApiError::TokenRequiresRefreshing(mut http_request)) = twitch_result {
      self.generate_user_token_from_refresh_token().map(|()| {
        http_request.update_token(&self.refresh_token);
        http_request.run()
      })?
    } else {
      if twitch_result.is_err() {
        warn!(
          "regen 401 called with result being an error, but wasnt token refresh required: {:?}",
          twitch_result
        );
      }

      twitch_result
    }
  }

  pub fn get_users<I: Into<String>, S: Into<String>>(
    &mut self,
    id: Vec<I>,
    login: Vec<S>,
  ) -> Result<UserDataSet, TwitchApiError> {
    self
      .regen_tokens_on_fail(twitcheventsub_api::get_users(
        &self.user_token,
        id,
        login,
        &self.client_id,
      ))
      .and_then(|user_data| match serde_json::from_str(&user_data) {
        Ok(users) => Ok(users),
        Err(e) => Err(TwitchApiError::DeserialisationError(e.to_string())),
      })
  }

  fn get_token_user_id(&mut self) -> Result<String, TwitchApiError> {
    self
      .get_users(vec![] as Vec<String>, vec![] as Vec<String>)
      .and_then(|user| {
        if user.data.is_empty() {
          Err(TwitchApiError::InputError(
            "Failed to Deserialise user information from get_user endpoint".to_owned(),
          ))
        } else {
          Ok(user.data[0].id.to_owned())
        }
      })
  }
}
