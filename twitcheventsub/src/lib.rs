//#![doc = include_str!("../../../../README.md")]

use std::iter;
use std::sync::mpsc::{channel, Receiver as SyncReceiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use modules::bttv::BTTV;
pub use modules::errors::LOG_FILE;
use modules::irc_bot::IRCChat;
use tungstenite::connect;
use twitcheventsub_api::TwitchApiError;
use twitcheventsub_structs::prelude::{Subscription, TwitchEvent, UserData};
use twitcheventsub_tokens::TokenHandler;

use crate::modules::consts::*;

mod modules;

pub mod prelude {
  pub use twitcheventsub_api;
  pub use twitcheventsub_structs::prelude::*;
  pub use twitcheventsub_tokens;

  pub use crate::modules::{bttv::*, emotebuilder::*};
}

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
    broadcasters_login: &str,
  ) -> Result<TwitchEventSubApi, EventSubError> {
    let client_twitch_id = tokens.client_twitch_id.clone();
    dbg!(&client_twitch_id);
    dbg!(&broadcasters_login);
    let users = tokens.get_users(vec![&client_twitch_id], vec![broadcasters_login])?;

    // Or not all user details retrieved
    if users.data.is_empty() ||
      (users.data.len() < 2 &&
        (users.data[0].id != tokens.client_twitch_id ||
          users.data[0].login != broadcasters_login))
    {
      // Queried a invalid broadcaster
      return Err(EventSubError::InvalidBroadcaster);
    }

    let (broadcaster_user, token_user) = {
      let mut client_idx = 0;
      let broadcaster_idx;
      if users.data[0].id == tokens.client_twitch_id && users.data[0].login == broadcasters_login {
        broadcaster_idx = 0;
      } else if users.data[1].id == tokens.client_twitch_id {
        broadcaster_idx = 0;
        client_idx = 1;
      } else if users.data[0].id != tokens.client_twitch_id ||
        users.data[0].login != broadcasters_login
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

    let bttv = BTTV::new(&broadcaster_user.id);
    let bttv2 = BTTV::new(&broadcaster_user.id);

    let mut irc = None;

    if use_irc_channel {
      subscriptions.append(&mut vec![
        Subscription::PermissionIRCRead,
        Subscription::PermissionIRCWrite,
      ]);
    }

    if let Ok(false) = tokens.check_token_has_required_subscriptions(&subscriptions) {
      tokens.apply_subscriptions_to_tokens(&subscriptions, true);
    }

    if use_irc_channel {
      let mut new_irc = IRCChat::new(&token_user.login, &tokens.user_token);
      new_irc.join_channel(&broadcaster_user.login);

      irc = Some(new_irc);
    }

    #[cfg(feature = "logging")]
    info!("Starting websocket client.");
    let (client, _) = connect(CONNECTION_EVENTS).unwrap();

    let (transmit_messages, receive_message) = channel();
    let (send_quit_message, receive_quit_message) = channel();

    let thread_token = tokens.clone();
    let subscriptions_clone = subscriptions.clone();
    let custom_subscription_data_clone = custom_subscription_data.clone();

    let broadcaster_id = broadcaster_user.id.clone();
    let receive_thread = thread::spawn(move || {
      eventsub::events(
        client,
        transmit_messages,
        receive_quit_message,
        subscriptions_clone,
        custom_subscription_data_clone,
        thread_token,
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

  pub fn api(&mut self) -> &mut TokenHandler {
    &mut self.tokens
  }

  pub fn broadcaster(&self) -> &UserData {
    &self.broadcaster_user
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
}
