#![allow(clippy::uninlined_format_args)]
use env_handler::EnvHandler;
use log::warn;
use twitcheventsub_api::{
  self, get_user_and_refresh_token_from_authorisation_code, validate_token, TwitchApiError,
};
use twitcheventsub_structs::prelude::{
  AdSchedule, ChannelEmotes, Clips, CreateCustomReward, CreatedCustomRewardResponse, GetChatters,
  GetCustomRewards, GlobalEmotes, Moderators, Subscription, UpdateCustomReward, UserDataSet,
};

mod builder;
mod custom_redeems;
mod env_handler;

pub use builder::{TokenBuilderError, TokenHandlerBuilder};
pub use custom_redeems::*;

use crate::builder::{generate_authorisation_code, get_input};

#[derive(Default, Clone, Debug)]
pub struct TokenHandler {
  pub user_token: String,
  pub refresh_token: String,

  pub redirect_url: String,

  pub client_id: String,
  pub client_secret: String,
  pub client_twitch_id: String,

  env: String,
  user_token_env: String,
  refresh_token_env: String,

  pub subscriptions: Vec<Subscription>,
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
    twitcheventsub_api::validate_token(&self.user_token).map(|validation| {
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
        })
    })
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
        http_request.update_token(&self.user_token);
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
        &self.client_id,
        id,
        login,
      ))
      .and_then(|user_data| match serde_json::from_str(&user_data) {
        Ok(users) => Ok(users),
        Err(e) => Err(TwitchApiError::DeserialisationError(e.to_string())),
      })
  }

  pub fn get_token_user_id(&mut self) -> Result<String, TwitchApiError> {
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

  pub fn get_chatters(&mut self, broadcaster_id: &str) -> Result<GetChatters, TwitchApiError> {
    self
      .regen_tokens_on_fail(twitcheventsub_api::get_chatters(
        broadcaster_id,
        &self.client_twitch_id,
        &self.user_token,
        &self.client_id,
      ))
      .and_then(|data| match serde_json::from_str(&data) {
        Ok(data) => Ok(data),
        Err(e) => Err(TwitchApiError::DeserialisationError(e.to_string())),
      })
  }

  pub fn send_chat_message(
    &mut self,
    broadcaster_id: &str,
    message: &str,
  ) -> Result<String, TwitchApiError> {
    self.send_chat_message_with_reply(broadcaster_id, message, None)
  }

  pub fn send_chat_message_with_reply(
    &mut self,
    broadcaster_id: &str,
    message: &str,
    reply_message_parent_id: Option<String>,
  ) -> Result<String, TwitchApiError> {
    self.regen_tokens_on_fail(twitcheventsub_api::send_chat_message_with_reply(
      &self.user_token,
      &self.client_id,
      &self.client_twitch_id,
      broadcaster_id,
      message,
      reply_message_parent_id,
    ))
  }

  pub fn send_announcement<P: Into<String>>(
    &mut self,
    broadcaster_id: &str,
    message: &str,
    colour: Option<P>,
  ) -> Result<String, TwitchApiError> {
    self
      .regen_tokens_on_fail(twitcheventsub_api::send_announcement(
        &self.user_token,
        &self.client_id,
        &self.client_twitch_id,
        broadcaster_id,
        message,
        colour,
      ))
      .and_then(|data| match serde_json::from_str(&data) {
        Ok(data) => Ok(data),
        Err(e) => Err(TwitchApiError::DeserialisationError(e.to_string())),
      })
  }

  pub fn send_shoutout(
    &mut self,
    from_broadcaster_id: &str,
    to_broadcaster_id: &str,
  ) -> Result<String, TwitchApiError> {
    self.regen_tokens_on_fail(twitcheventsub_api::send_shoutout(
      &self.user_token,
      &self.client_id,
      &self.client_twitch_id,
      from_broadcaster_id,
      to_broadcaster_id,
    ))
  }

  pub fn delete_message(
    &mut self,
    broadcaster_id: &str,
    message_id: &str,
  ) -> Result<String, TwitchApiError> {
    self.regen_tokens_on_fail(twitcheventsub_api::delete_message(
      &self.user_token,
      &self.client_id,
      &self.client_twitch_id,
      broadcaster_id,
      message_id,
    ))
  }

  pub fn timeout_user(
    &mut self,
    broadcaster_id: &str,
    user_id: &str,
    duration_secs: u32,
    reason: &str,
  ) -> Result<String, TwitchApiError> {
    self.regen_tokens_on_fail(twitcheventsub_api::timeout_user(
      &self.user_token,
      &self.client_id,
      &self.client_twitch_id,
      broadcaster_id,
      user_id,
      duration_secs,
      reason,
    ))
  }

  pub fn get_channel_badges(
    &mut self,
    broadcaster_id: &str,
  ) -> Result<ChannelEmotes, TwitchApiError> {
    self
      .regen_tokens_on_fail(twitcheventsub_api::get_channel_badges(
        &self.user_token,
        &self.client_id,
        broadcaster_id,
      ))
      .and_then(|data| match serde_json::from_str(&data) {
        Ok(data) => Ok(data),
        Err(e) => Err(TwitchApiError::DeserialisationError(e.to_string())),
      })
  }

  pub fn get_global_badges(&mut self) -> Result<GlobalEmotes, TwitchApiError> {
    self
      .regen_tokens_on_fail(twitcheventsub_api::get_global_badges(
        &self.user_token,
        &self.client_id,
      ))
      .and_then(|data| match serde_json::from_str(&data) {
        Ok(data) => Ok(data),
        Err(e) => Err(TwitchApiError::DeserialisationError(e.to_string())),
      })
  }

  pub fn get_moderators(&mut self, broadcaster_id: &str) -> Result<Moderators, TwitchApiError> {
    self
      .regen_tokens_on_fail(twitcheventsub_api::get_moderators(
        &self.user_token,
        &self.client_id,
        broadcaster_id,
      ))
      .and_then(|data| match serde_json::from_str(&data) {
        Ok(data) => Ok(data),
        Err(e) => Err(TwitchApiError::DeserialisationError(e.to_string())),
      })
  }

  pub fn get_custom_rewards(
    &mut self,
    broadcaster_id: &str,
  ) -> Result<GetCustomRewards, TwitchApiError> {
    self
      .regen_tokens_on_fail(twitcheventsub_api::get_custom_rewards(
        &self.user_token,
        &self.client_id,
        broadcaster_id,
      ))
      .and_then(|data| match serde_json::from_str(&data) {
        Ok(data) => Ok(data),
        Err(e) => Err(TwitchApiError::DeserialisationError(e.to_string())),
      })
  }

  pub fn update_custom_rewards(
    &mut self,
    broadcaster_id: &str,
    redeem_id: &str,
    update_redeem: &UpdateCustomReward,
  ) -> Result<CreatedCustomRewardResponse, TwitchApiError> {
    self
      .regen_tokens_on_fail(twitcheventsub_api::update_custom_rewards(
        &self.user_token,
        &self.client_id,
        broadcaster_id,
        redeem_id,
        update_redeem,
      ))
      .and_then(|data| match serde_json::from_str(&data) {
        Ok(data) => Ok(data),
        Err(e) => Err(TwitchApiError::DeserialisationError(e.to_string())),
      })
  }

  pub fn create_custom_reward(
    &mut self,
    broadcaster_id: &str,
    custom_reward_data: CreateCustomReward,
  ) -> Result<CreatedCustomRewardResponse, TwitchApiError> {
    self
      .regen_tokens_on_fail(twitcheventsub_api::create_custom_reward(
        &self.user_token,
        &self.client_id,
        broadcaster_id,
        custom_reward_data,
      ))
      .and_then(|data| match serde_json::from_str(&data) {
        Ok(data) => Ok(data),
        Err(e) => Err(TwitchApiError::DeserialisationError(e.to_string())),
      })
  }

  pub fn delete_custom_reward(
    &mut self,
    broadcaster_id: &str,
    reward_id: &str,
  ) -> Result<String, TwitchApiError> {
    self
      .regen_tokens_on_fail(twitcheventsub_api::delete_custom_reward(
        &self.user_token,
        &self.client_id,
        broadcaster_id,
        reward_id,
      ))
      .and_then(|data| match serde_json::from_str(&data) {
        Ok(data) => Ok(data),
        Err(e) => Err(TwitchApiError::DeserialisationError(e.to_string())),
      })
  }

  pub fn get_clips(&mut self, broadcaster_id: &str) -> Result<Clips, TwitchApiError> {
    self
      .regen_tokens_on_fail(twitcheventsub_api::get_clips(
        &self.user_token,
        &self.client_id,
        broadcaster_id,
      ))
      .and_then(|data| match serde_json::from_str(&data) {
        Ok(data) => Ok(data),
        Err(e) => Err(TwitchApiError::DeserialisationError(e.to_string())),
      })
  }

  pub fn get_ad_schedule(&mut self, broadcaster_id: &str) -> Result<AdSchedule, TwitchApiError> {
    self
      .regen_tokens_on_fail(twitcheventsub_api::get_ad_schedule(
        &self.user_token,
        &self.client_id,
        broadcaster_id,
      ))
      .and_then(|data| match serde_json::from_str(&data) {
        Ok(data) => Ok(data),
        Err(e) => Err(TwitchApiError::DeserialisationError(e.to_string())),
      })
  }

  pub fn generate_user_and_refresh_tokens(
    &mut self,
    scopes: &[Subscription],
  ) -> Option<TokenBuilderError> {
    let open_browser = true;

    let authorisation_code;
    match generate_authorisation_code(&self.client_id, &self.redirect_url, scopes, open_browser) {
      Ok(code) => {
        authorisation_code = code;
      }
      Err(TokenBuilderError::ManuallyInputAuthorisationCode) => {
        // Listen for input
        // or manually set input
        println!("Please input generated authorisation code from browser url:");
        authorisation_code = get_input();
      }
      Err(e) => {
        return Some(e);
      }
    }

    let (user_token, refresh_token) = get_user_and_refresh_token_from_authorisation_code(
      &self.client_id,
      &self.client_secret,
      &authorisation_code,
      &self.redirect_url,
    )
    .map_err(|e| TokenBuilderError::TwitchApiError(e))
    .ok()?;

    self.user_token = user_token;
    self.refresh_token = refresh_token;

    if let Ok(valid) = validate_token(&self.user_token) {
      if valid.status.is_some() {
        // it is not valid
        return Some(TokenBuilderError::InvalidUserToken)
      }
    } else {
      // not valid
      return Some(TokenBuilderError::InvalidUserToken)
    }

    if let Ok(user_id) = self.get_token_user_id() {
      self.client_twitch_id = user_id;
    } else {
      return Some(TokenBuilderError::InvalidClientIdOrSecret);
    }

    self.subscriptions = scopes.to_vec();

    self.save();
    None
  }

  pub fn save(&mut self) {
    EnvHandler::save_env(
      &self.env,
      &self.client_id,
      &self.client_secret,
      &self.client_twitch_id,
      &self.redirect_url,
    );
    EnvHandler::save_user_token(&self.user_token_env, &self.user_token);
    EnvHandler::save_refresh_token(&self.refresh_token_env, &self.refresh_token);
  }
}
