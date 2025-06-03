use std::iter;
use std::sync::mpsc::{channel, Receiver as SyncReceiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use modules::bttv::BTTV;
pub use modules::errors::LOG_FILE;
use modules::irc_bot::IRCChat;
use serde_json;
use tungstenite::connect;
use twitcheventsub_api::TwitchApiError;
pub use twitcheventsub_structs::*;
use twitcheventsub_tokens::TokenHandler;

use crate::modules::consts::*;

mod modules;

#[cfg(feature = "logging")]
pub use log::{error, info, warn};

pub use crate::modules::{emotebuilder::*, errors::EventSubError, eventsub};

impl From<TwitchApiError> for EventSubError {
  fn from(value: TwitchApiError) -> Self {
    EventSubError::TwitchApiError(value)
  }
}

#[derive(Debug)]
pub enum ResponseType {
  Event(TwitchEvent),
  Error(EventSubError),
  RawResponse(String),
  Close,
  Ready,
}

#[must_use]
pub struct TwitchEventSubApiBuilder {
  tokens: TokenHandler,
  subscriptions: Vec<Subscription>,
  enable_irc: bool,
  bot_tokens: Option<TokenHandler>,
}

impl TwitchEventSubApiBuilder {
  pub fn new(tokens: TokenHandler) -> TwitchEventSubApiBuilder {
    TwitchEventSubApiBuilder {
      tokens,
      subscriptions: Vec::new(),
      enable_irc: false,
      bot_tokens: None,
    }
  }

  pub fn use_bot_account(mut self, tokens: TokenHandler) -> TwitchEventSubApiBuilder {
    self.bot_tokens = Some(tokens);
    self
  }

  pub fn enable_irc(mut self) -> TwitchEventSubApiBuilder {
    self.enable_irc = true;
    self
  }

  pub fn add_subscription(mut self, sub: Subscription) -> TwitchEventSubApiBuilder {
    self.subscriptions.push(sub);
    self
  }

  pub fn add_subscriptions<I: IntoIterator<Item = Subscription>>(
    mut self,
    subs: I,
  ) -> TwitchEventSubApiBuilder {
    self.subscriptions.extend(subs);
    self
  }

  pub fn subscriptions(&self) -> Vec<Subscription> {
    self.subscriptions.clone()
  }

  pub fn build(self, broadcasters_username: &str) -> Result<TwitchEventSubApi, EventSubError> {
    TwitchEventSubApi::new(
      self.tokens,
      self.bot_tokens,
      self.subscriptions,
      Vec::new(),
      self.enable_irc,
      broadcasters_username,
    )
  }
}

#[derive(Debug)]
pub struct TwitchEventSubApi {
  _receive_thread: JoinHandle<()>,
  messages_received: SyncReceiver<ResponseType>,
  send_quit_message: Sender<bool>,
  tokens: TokenHandler,
  bot_tokens: Option<TokenHandler>,
  subscriptions: Vec<Subscription>,
  subscription_data: Vec<String>,
  pub bttv: BTTV,
  pub broadcaster_user: UserData,
}

impl TwitchEventSubApi {
  pub fn builder(tokens: TokenHandler) -> TwitchEventSubApiBuilder {
    TwitchEventSubApiBuilder::new(tokens)
  }

  pub fn new(
    mut tokens: TokenHandler,
    bot_tokens: Option<TokenHandler>,
    mut subscriptions: Vec<Subscription>,
    custom_subscription_data: Vec<String>,
    use_irc_channel: bool,
    broadcasters_username: &str,
  ) -> Result<TwitchEventSubApi, EventSubError> {
    dbg!("Start of twitcheventsubapi new");

    let client_twitch_id = tokens.client_twitch_id.clone();
    let users = tokens.get_users(vec![&client_twitch_id], vec![broadcasters_username])?;

    //let users = tokens.regen_tokens_on_fail(twitcheventsub_api::get_users(
    //  &user_token,
    //  vec![&client_twitch_id],
    //  vec![broadcasters_username],
    //  &client_id,
    //))?;
    dbg!("after get users");

    // Or not all user details retrieved
    if users.data.is_empty() ||
      (users.data.len() < 2 &&
        (users.data[0].id != tokens.client_twitch_id ||
          users.data[0].login != broadcasters_username))
    {
      // Queried a invalid broadcaster
      return Err(EventSubError::InvalidBroadcaster);
    }

    let (broadcaster_user, token_user) = {
      let mut client_idx = 0;
      let broadcaster_idx;
      if users.data[0].id == tokens.client_twitch_id && users.data[0].login == broadcasters_username
      {
        broadcaster_idx = 0;
      } else if users.data[1].id == tokens.client_twitch_id {
        broadcaster_idx = 0;
        client_idx = 1;
      } else if users.data[0].id != tokens.client_twitch_id ||
        users.data[0].login != broadcasters_username
      {
        broadcaster_idx = 1;
      } else {
        return Err(EventSubError::InvalidBroadcaster);
      }

      (
        users.data[broadcaster_idx].clone(),
        users.data[client_idx].clone(),
      )
    };

    dbg!("start bttv");
    let bttv = BTTV::new(&broadcaster_user.id);
    let bttv2 = BTTV::new(&broadcaster_user.id);
    dbg!("end bttv");

    let mut irc = None;

    if use_irc_channel {
      subscriptions.append(&mut vec![
        Subscription::PermissionIRCRead,
        Subscription::PermissionIRCWrite,
      ]);
    }

    dbg!("Check token has requirement");
    if let Ok(false) = tokens.check_token_has_required_subscriptions(&subscriptions) {
      dbg!("Apply subscription tokens");
      tokens.apply_subscriptions_to_tokens(&subscriptions, true);
    }
    dbg!("End check token requirements");

    dbg!("start irc");
    if use_irc_channel {
      let mut new_irc = IRCChat::new(&token_user.login, &tokens.user_token);
      dbg!("after irc new");
      new_irc.join_channel(&broadcaster_user.login);

      irc = Some(new_irc);
    }
    dbg!("end irc");

    #[cfg(feature = "logging")]
    info!("Starting websocket client.");
    let (twitch_receiver, _) = match connect(CONNECTION_EVENTS) {
      Ok(a) => a,
      Err(e) => return Err(EventSubError::WebSocketFailed(e.to_string())),
    };

    dbg!("After websocket start");

    let (transmit_messages, receive_message) = channel();
    let (send_quit_message, receive_quit_message) = channel();

    let thread_token = tokens.clone();
    let subscriptions_clone = subscriptions.clone();
    let custom_subscription_data_clone = custom_subscription_data.clone();

    let broadcaster_id = broadcaster_user.id.clone();
    let receive_thread = thread::spawn(move || {
      eventsub::events(
        twitch_receiver,
        transmit_messages,
        receive_quit_message,
        subscriptions_clone,
        custom_subscription_data_clone,
        thread_token,
        None,
        irc,
        bttv,
        &broadcaster_id,
      )
    });

    Ok(TwitchEventSubApi {
      _receive_thread: receive_thread,
      messages_received: receive_message,
      send_quit_message,
      tokens,
      bot_tokens,
      subscriptions,
      subscription_data: custom_subscription_data,
      bttv: bttv2,
      broadcaster_user,
    })
  }

  pub fn get_tokens(&self) -> TokenHandler {
    self.tokens.clone()
  }

  pub fn restart_websockets(&mut self) -> Result<(), EventSubError> {
    let _ = self.send_quit_message.send(true);

    let tokens = self.tokens.clone();
    let bot_tokens = self.bot_tokens.clone();
    let subscriptions = self.subscriptions.clone();
    let custom_subscription_data = self.subscription_data.clone();
    let broadcasters_username = &self.broadcaster_user.login;
    let new_webscoket = TwitchEventSubApi::new(
      tokens,
      bot_tokens,
      subscriptions,
      custom_subscription_data,
      false,
      broadcasters_username,
    )?;

    *self = new_webscoket;

    Ok(())
  }

  // fn regen_token_if_401(
  //   result: Result<String, EventSubError>,
  //   twitch_keys: &mut TwitchKeys,
  //   auto_save_load_created_tokens: &Option<(String, String)>,
  // ) -> Result<String, EventSubError> {
  //   if let Err(EventSubError::TokenRequiresRefreshing(mut http_request)) = result {
  //     #[cfg(feature = "logging")]
  //     warn!("Token requires refreshing return!");
  //     if let Ok(token) = TwitchApi::generate_token_from_refresh_token(
  //       twitch_keys.client_id.to_owned(),
  //       twitch_keys.client_secret.to_owned(),
  //       twitch_keys.refresh_token.clone().unwrap().to_owned(),
  //     ) {
  //       #[cfg(feature = "logging")]
  //       info!("Generated new keys as 401 was returned!");
  //       twitch_keys.access_token = Some(token.access);
  //       twitch_keys.refresh_token = Some(token.refresh.to_owned());
  //     }

  //     // TODO: SAVE
  //     if let Some((token_file, refresh_file)) = auto_save_load_created_tokens {
  //       #[cfg(feature = "logging")]
  //       info!("saving tokens");
  //       if let Some(new_token) = twitch_keys.token() {
  //         if let Err(e) = new_token.save_to_file(token_file, refresh_file) {
  //           #[cfg(feature = "logging")]
  //           warn!("Failed to save tokens to file!");
  //           return Err(e);
  //         }
  //       }
  //     }

  //     let access_token = twitch_keys.access_token.as_ref().unwrap();
  //     http_request.update_token(access_token.get_token());
  //     http_request.run()
  //   } else {
  //     if result.is_err() {
  //       #[cfg(feature = "logging")]
  //       warn!(
  //         "regen 401 called with result being an error, but wasnt token refresh required: {:?}",
  //         result
  //       );
  //     }
  //     result
  //   }
  // }

  /// Collect all pending message immediately
  /// Should be non-blocking
  ///
  /// WARNING: Override the duration is just for circumstances
  /// where you don't want it to consume your entire cpu
  /// if its the only expensive function running on this thread
  ///
  /// Do note setting duration on this function, doesn't mean it will wait
  /// exactly DURATION time, it has the possibility of never returning if
  /// duration is high enough and at least one message is sent within the
  /// set timeout
  pub fn receive_all_messages(&mut self, override_duration: Option<Duration>) -> Vec<ResponseType> {
    debug_assert!(
      override_duration.is_some(),
      "Warning: This isn't recommended to be setup unless you know what you are doing."
    );
    // check thread for new messages without waiting
    //
    // return new messages if any
    iter::from_fn(|| self.receive_single_message(override_duration.unwrap_or(Duration::ZERO)))
      .collect::<Vec<_>>()
  }

  ///
  /// This is the recommended function to use for getting twitch events
  ///
  /// This retrieves a single message if there are any messages waiting
  ///
  /// Set duration to Duration::ZERO for completely non-blocking
  ///
  /// Recommended to use Duration::ZERO when used in games and
  /// in engines like Godot and Unity.
  ///
  /// For a chat bot I found setting duration to 1 millis was enough
  /// to reduce cpu from 3.2% to 0.8% maximum
  ///
  pub fn receive_single_message(&mut self, duration: Duration) -> Option<ResponseType> {
    self.messages_received.recv_timeout(duration).ok()
  }

  //pub fn delete_message<S: Into<String>>(
  //  &mut self,
  //  message_id: S,
  //) -> Result<String, EventSubError> {
  //  let broadcaster_account_id = self.broadcaster_account_id.to_string();
  //  let moderator_account_id = broadcaster_account_id.to_owned();
  //  let access_token = self
  //    .twitch_keys
  //    .access_token
  //    .clone()
  //    .expect("No Access Token set")
  //    .get_token();
  //  let client_id = self.twitch_keys.client_id.to_string();

  //  TwitchEventSubApi::regen_token_if_401(
  //    TwitchApi::delete_message(
  //      broadcaster_account_id,
  //      moderator_account_id,
  //      message_id.into(),
  //      access_token,
  //      client_id,
  //    ),
  //    &mut self.twitch_keys,
  //    &self.save_locations,
  //  )
  //}

  //pub fn timeout_user<S: Into<String>, T: Into<String>>(
  //  &mut self,
  //  user_id: S,
  //  duration: u32,
  //  reason: T,
  //) {
  //  let broadcaster_account_id = self.twitch_keys.broadcaster_account_id.to_string();
  //  let moderator_account_id = broadcaster_account_id.to_owned();

  //  let access_token = self
  //    .twitch_keys
  //    .access_token
  //    .clone()
  //    .expect("No Access Token set")
  //    .get_token();
  //  let client_id = self.twitch_keys.client_id.to_string();
  //  let _ = TwitchEventSubApi::regen_token_if_401(
  //    TwitchApi::timeout_user(
  //      access_token,
  //      client_id,
  //      broadcaster_account_id,
  //      moderator_account_id,
  //      user_id.into(),
  //      duration,
  //      reason.into(),
  //    ),
  //    &mut self.twitch_keys,
  //    &self.save_locations,
  //  );
  //}

  //pub fn get_ad_schedule(&mut self) -> Result<AdSchedule, EventSubError> {
  //  let access_token = self
  //    .twitch_keys
  //    .access_token
  //    .clone()
  //    .expect("Access token not set")
  //    .get_token();
  //  let client_id = self.twitch_keys.client_id.to_string();
  //  let broadcaster_id = self.twitch_keys.broadcaster_account_id.to_string();

  //  TwitchEventSubApi::regen_token_if_401(
  //    TwitchApi::get_ad_schedule(broadcaster_id, access_token, client_id),
  //    &mut self.twitch_keys,
  //    &self.save_locations,
  //  )
  //  .and_then(|x| serde_json::from_str(&x).map_err(|e| EventSubError::ParseError(e.to_string())))
  //}

  //pub fn get_chatters(&mut self) -> Result<GetChatters, EventSubError> {
  //  let access_token = self
  //    .twitch_keys
  //    .access_token
  //    .clone()
  //    .expect("Access token not set")
  //    .get_token();
  //  let broadcaster_id = self.twitch_keys.broadcaster_account_id.to_string();
  //  let moderator_id = self.twitch_keys.token_user_id.to_owned();
  //  let client_id = self.twitch_keys.client_id.to_string();

  //  TwitchEventSubApi::regen_token_if_401(
  //    TwitchApi::get_chatters(
  //      broadcaster_id.to_owned(),
  //      moderator_id,
  //      access_token,
  //      client_id,
  //    ),
  //    &mut self.twitch_keys,
  //    &self.save_locations,
  //  )
  //  .and_then(|x| serde_json::from_str(&x).map_err(|e| EventSubError::ParseError(e.to_string())))
  //}

  //pub fn get_moderators(&mut self) -> Result<Moderators, EventSubError> {
  //  let access_token = self
  //    .twitch_keys
  //    .access_token
  //    .clone()
  //    .expect("Access token not set")
  //    .get_token();
  //  let broadcaster_id = self.twitch_keys.broadcaster_account_id.to_string();
  //  let client_id = self.twitch_keys.client_id.to_string();

  //  if !self.twitch_keys.token_user_id.is_empty() &&
  //    !broadcaster_id.eq(&self.twitch_keys.token_user_id)
  //  {
  //    return Err(EventSubError::TokenDoesntBelongToBroadcaster);
  //  }

  //  TwitchEventSubApi::regen_token_if_401(
  //    TwitchApi::get_moderators(access_token, client_id, broadcaster_id),
  //    &mut self.twitch_keys,
  //    &self.save_locations,
  //  )
  //  .and_then(|x| serde_json::from_str(&x).map_err(|e| EventSubError::ParseError(e.to_string())))
  //}

  //pub fn get_custom_rewards(&mut self) -> Result<GetCustomRewards, EventSubError> {
  //  let access_token = self
  //    .twitch_keys
  //    .access_token
  //    .clone()
  //    .expect("Access token not set")
  //    .get_token();
  //  let broadcaster_id = self.twitch_keys.broadcaster_account_id.to_string();
  //  let client_id = self.twitch_keys.client_id.to_string();

  //  TwitchEventSubApi::regen_token_if_401(
  //    TwitchApi::get_custom_rewards(access_token, client_id, broadcaster_id),
  //    &mut self.twitch_keys,
  //    &self.save_locations,
  //  )
  //  .and_then(|x| serde_json::from_str(&x).map_err(|e| EventSubError::ParseError(e.to_string())))
  //}

  //pub fn update_custom_reward<S: Into<String>>(
  //  &mut self,
  //  redeem_id: S,
  //  update_redeem: UpdateCustomReward,
  //) -> Result<CreatedCustomRewardResponse, EventSubError> {
  //  let access_token = self
  //    .twitch_keys
  //    .access_token
  //    .clone()
  //    .expect("Access token not set")
  //    .get_token();
  //  let broadcaster_id = self.twitch_keys.broadcaster_account_id.to_string();
  //  let client_id = self.twitch_keys.client_id.to_string();

  //  TwitchEventSubApi::regen_token_if_401(
  //    TwitchApi::update_custom_rewards(
  //      access_token,
  //      client_id,
  //      broadcaster_id,
  //      redeem_id.into(),
  //      update_redeem,
  //    ),
  //    &mut self.twitch_keys,
  //    &self.save_locations,
  //  )
  //  .and_then(|x| serde_json::from_str(&x).map_err(|e| EventSubError::ParseError(e.to_string())))
  //}

  //pub fn create_custom_reward(
  //  &mut self,
  //  custom_reward: CreateCustomReward,
  //) -> Result<CreatedCustomRewardResponse, EventSubError> {
  //  let access_token = self
  //    .twitch_keys
  //    .access_token
  //    .clone()
  //    .expect("Access token not set")
  //    .get_token();
  //  let broadcaster_id = self.twitch_keys.broadcaster_account_id.to_string();
  //  let client_id = self.twitch_keys.client_id.to_string();

  //  TwitchEventSubApi::regen_token_if_401(
  //    TwitchApi::create_custom_reward(access_token, client_id, broadcaster_id, custom_reward),
  //    &mut self.twitch_keys,
  //    &self.save_locations,
  //  )
  //  .and_then(|x| serde_json::from_str(&x).map_err(|e| EventSubError::ParseError(e.to_string())))
  //}

  //// come back later
  //pub fn delete_custom_reward<T: Into<String>>(&mut self, id: T) -> Result<(), EventSubError> {
  //  let access_token = self
  //    .twitch_keys
  //    .access_token
  //    .clone()
  //    .expect("Access token not set")
  //    .get_token();
  //  let broadcaster_id = self.twitch_keys.broadcaster_account_id.to_string();
  //  let client_id = self.twitch_keys.client_id.to_string();

  //  TwitchEventSubApi::regen_token_if_401(
  //    TwitchApi::delete_custom_reward(access_token, client_id, broadcaster_id, id),
  //    &mut self.twitch_keys,
  //    &self.save_locations,
  //  )
  //  .and_then(|_| Ok(()))
  //}

  //pub fn get_clips_for_broadcaster<T: Into<String>>(
  //  &mut self,
  //  broadcaster_id: T,
  //) -> Result<Clips, EventSubError> {
  //  let access_token = self
  //    .twitch_keys
  //    .access_token
  //    .clone()
  //    .expect("Access token not set")
  //    .get_token();
  //  let client_id = self.twitch_keys.client_id.to_string();

  //  TwitchEventSubApi::regen_token_if_401(
  //    TwitchApi::get_clips(access_token, client_id, broadcaster_id),
  //    &mut self.twitch_keys,
  //    &self.save_locations,
  //  )
  //  .and_then(|x| serde_json::from_str(&x).map_err(|e| EventSubError::ParseError(e.to_string())))
  //}

  //pub fn api_user_id(&self) -> String {
  //  self
  //    .twitch_keys
  //    .sender_account_id
  //    .as_deref()
  //    .unwrap_or(&self.twitch_keys.broadcaster_account_id)
  //    .to_owned()
  //}

  //pub fn get_users_from_ids<S: Into<String>>(
  //  &mut self,
  //  ids: Vec<S>,
  //) -> Result<Users, EventSubError> {
  //  self.get_users(
  //    ids
  //      .into_iter()
  //      .map(|a| a.into())
  //      .collect::<Vec<String>>()
  //      .into(),
  //    Vec::with_capacity(0),
  //  )
  //}

  //pub fn get_users_from_logins<S: Into<String>>(
  //  &mut self,
  //  logins: Vec<S>,
  //) -> Result<Users, EventSubError> {
  //  self.get_users(
  //    Vec::with_capacity(0),
  //    logins
  //      .into_iter()
  //      .map(|a| a.into())
  //      .collect::<Vec<String>>()
  //      .into(),
  //  )
  //}

  //pub fn get_users_self(&mut self) -> Result<Users, EventSubError> {
  //  self.get_users(Vec::with_capacity(0), Vec::with_capacity(0))
  //}

  /////
  ///// It is recommended to use the get_users_from_* methods instead.
  /////
  ///// How ever if you wish to use it manuall please
  ///// Refer to https://dev.twitch.tv/docs/api/reference/#get-users for specifics how to use
  /////
  //pub fn get_users(&mut self, id: Vec<String>, login: Vec<String>) -> Result<Users, EventSubError> {
  //  let access_token = self
  //    .twitch_keys
  //    .access_token
  //    .clone()
  //    .expect("Access token not set")
  //    .get_token();
  //  let client_id = self.twitch_keys.client_id.to_string();

  //  TwitchEventSubApi::regen_token_if_401(
  //    TwitchApi::get_users(access_token, id, login, client_id),
  //    &mut self.twitch_keys,
  //    &self.save_locations,
  //  )
  //  .and_then(|x| serde_json::from_str(&x).map_err(|e| EventSubError::ParseError(e.to_string())))
  //}

  //pub fn get_badge_urls_from_badges(&mut self, badges: Vec<Badge>) -> Vec<BadgeVersion> {
  //  let mut badges_requested = Vec::new();

  //  let broadcaster_id = self.twitch_keys.broadcaster_account_id.clone();

  //  if let Ok(channel_badges) = self.get_channel_badges(broadcaster_id) {
  //    for channel_badge in channel_badges.data {
  //      for badge in &badges {
  //        if badge.set_id.eq(&channel_badge.set_id) {
  //          for version in &channel_badge.versions {
  //            if version.id.eq(&badge.id) {
  //              // correct url version
  //              badges_requested.push(version.clone());
  //            }
  //          }
  //        }
  //      }
  //    }
  //  }

  //  if let Ok(global_badges) = self.get_global_badges() {
  //    for global_badge in global_badges.data {
  //      for badge in &badges {
  //        if badge.set_id.eq(&global_badge.set_id) {
  //          for version in &global_badge.versions {
  //            if version.id.eq(&badge.id) {
  //              // correct url version
  //              badges_requested.push(version.clone());
  //            }
  //          }
  //        }
  //      }
  //    }
  //  }

  //  badges_requested
  //}

  //pub fn get_channel_badges<S: Into<String>>(
  //  &mut self,
  //  broadcaster_id: S,
  //) -> Result<SetOfBadges, EventSubError> {
  //  let access_token = self
  //    .twitch_keys
  //    .access_token
  //    .clone()
  //    .expect("Access token not set")
  //    .get_token();
  //  let client_id = self.twitch_keys.client_id.to_string();

  //  let broadcaster_id: String = broadcaster_id.into();
  //  if let Err(_) = broadcaster_id.parse::<u32>() {
  //    return Err(EventSubError::UnhandledError(
  //      "Broadcaster id must be numeric!".to_string(),
  //    ));
  //  }

  //  TwitchEventSubApi::regen_token_if_401(
  //    TwitchApi::get_channel_badges(access_token, client_id, broadcaster_id),
  //    &mut self.twitch_keys,
  //    &self.save_locations,
  //  )
  //  .and_then(|data| {
  //    serde_json::from_str(&data).map_err(|e| EventSubError::ParseError(e.to_string()))
  //  })
  //}

  //pub fn get_global_badges(&mut self) -> Result<SetOfBadges, EventSubError> {
  //  let access_token = self
  //    .twitch_keys
  //    .access_token
  //    .clone()
  //    .expect("Access token not set")
  //    .get_token();
  //  let client_id = self.twitch_keys.client_id.to_string();

  //  TwitchEventSubApi::regen_token_if_401(
  //    TwitchApi::get_global_badges(access_token, client_id),
  //    &mut self.twitch_keys,
  //    &self.save_locations,
  //  )
  //  .and_then(|data| {
  //    serde_json::from_str(&data).map_err(|e| EventSubError::ParseError(e.to_string()))
  //  })
  //}

  //pub fn get_channel_emotes<S: Into<String>>(
  //  &mut self,
  //  broadcaster_id: S,
  //) -> Result<ChannelEmotes, EventSubError> {
  //  let access_token = self
  //    .twitch_keys
  //    .access_token
  //    .clone()
  //    .expect("Access token not set")
  //    .get_token();
  //  let client_id = self.twitch_keys.client_id.to_string();

  //  let broadcaster_id: String = broadcaster_id.into();
  //  if let Err(_) = broadcaster_id.parse::<u32>() {
  //    return Err(EventSubError::UnhandledError(
  //      "Broadcaster id must be numeric!".to_string(),
  //    ));
  //  }

  //  TwitchEventSubApi::regen_token_if_401(
  //    TwitchApi::get_channel_emotes(access_token, client_id, broadcaster_id),
  //    &mut self.twitch_keys,
  //    &self.save_locations,
  //  )
  //  .and_then(|x| serde_json::from_str(&x).map_err(|e| EventSubError::ParseError(e.to_string())))
  //}

  //pub fn get_global_emotes(&mut self) -> Result<GlobalEmotes, EventSubError> {
  //  let access_token = self
  //    .twitch_keys
  //    .access_token
  //    .clone()
  //    .expect("Access token not set")
  //    .get_token();
  //  let client_id = self.twitch_keys.client_id.to_string();

  //  TwitchEventSubApi::regen_token_if_401(
  //    TwitchApi::get_global_emotes(access_token, client_id),
  //    &mut self.twitch_keys,
  //    &self.save_locations,
  //  )
  //  .and_then(|x| serde_json::from_str(&x).map_err(|e| EventSubError::ParseError(e.to_string())))
  //}

  //pub fn get_image_data_from_url<S: Into<String>>(emote_url: S) -> Result<Vec<u8>, EventSubError> {
  //  match attohttpc::get(emote_url.into()).send() {
  //    Ok(new_image_data) => Ok(new_image_data.bytes().unwrap()),
  //    Err(e) => Err(EventSubError::HttpFailed(e.to_string())),
  //  }
  //}

  //pub fn get_emote_sets<S: Into<String>>(
  //  &mut self,
  //  emote_set_id: S,
  //) -> Result<GlobalEmotes, EventSubError> {
  //  let access_token = self
  //    .twitch_keys
  //    .access_token
  //    .clone()
  //    .expect("Access token not set")
  //    .get_token();
  //  let client_id = self.twitch_keys.client_id.to_string();

  //  TwitchEventSubApi::regen_token_if_401(
  //    TwitchApi::get_emote_set(emote_set_id.into(), access_token, client_id),
  //    &mut self.twitch_keys,
  //    &self.save_locations,
  //  )
  //  .and_then(|x| serde_json::from_str(&x).map_err(|e| EventSubError::ParseError(e.to_string())))
  //}

  pub fn send_chat_message(&mut self, message: &str) -> Result<String, EventSubError> {
    self.send_chat_message_with_reply(message, None)
  }

  pub fn send_chat_message_with_reply(
    &mut self,
    message: &str,
    reply_message_parent_id: Option<String>,
  ) -> Result<String, EventSubError> {
    let tokens = self.bot_tokens.clone().unwrap_or(self.tokens.clone());
    let sender_id = &self.tokens.client_twitch_id;

    //  TwitchEventSubApi::regen_token_if_401(
    twitcheventsub_api::send_chat_message_with_reply(
      &message,
      &tokens.user_token,
      &tokens.client_id,
      &self.broadcaster_user.id,
      &sender_id,
      reply_message_parent_id,
    )
    .map_err(EventSubError::from)
    //,
    //   &mut self.twitch_keys,
    //    &self.save_locations,
    //  )
  }

  //pub fn send_announcement<S: Into<String>, T: Into<String>>(
  //  &mut self,
  //  message: S,
  //  colour: Option<T>,
  //) -> Result<String, EventSubError> {
  //  let message: String = message.into();
  //  let access_token = self
  //    .twitch_keys
  //    .access_token
  //    .clone()
  //    .expect("No Access Token set")
  //    .get_token();
  //  let client_id = self.twitch_keys.client_id.to_string();
  //  let broadcaster_account_id = self.twitch_keys.broadcaster_account_id.to_string();
  //  let sender_id = self
  //    .twitch_keys
  //    .sender_account_id
  //    .clone()
  //    .unwrap_or(self.twitch_keys.broadcaster_account_id.to_string());

  //  TwitchEventSubApi::regen_token_if_401(
  //    TwitchApi::send_announcement(
  //      message,
  //      access_token,
  //      client_id,
  //      broadcaster_account_id,
  //      sender_id,
  //      colour,
  //    ),
  //    &mut self.twitch_keys,
  //    &self.save_locations,
  //  )
  //}

  //pub fn send_shoutout<S: Into<String>>(&mut self, to_broadcaster_id: S) {
  //  let broadcaster_account_id = self.twitch_keys.broadcaster_account_id.to_string();

  //  let access_token = self
  //    .twitch_keys
  //    .access_token
  //    .clone()
  //    .expect("No Access Token set")
  //    .get_token();
  //  let client_id = self.twitch_keys.client_id.to_string();
  //  let moderator_id = self.twitch_keys.token_user_id.to_owned();

  //  let _ = TwitchEventSubApi::regen_token_if_401(
  //    TwitchApi::send_shoutout(
  //      access_token,
  //      client_id,
  //      broadcaster_account_id,
  //      to_broadcaster_id,
  //      moderator_id,
  //    ),
  //    &mut self.twitch_keys,
  //    &self.save_locations,
  //  );
  //}
}
