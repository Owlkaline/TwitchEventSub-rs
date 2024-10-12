//#![doc = include_str!("../../../../README.md")]

use std::fs;
use std::iter;
use std::sync::mpsc::{channel, Receiver as SyncReceiver};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use std::sync::{Arc, Mutex};

use crate::modules::consts::*;

use modules::irc_bot::IRCChat;
use open;

use std::io::Read;
use tungstenite::connect;
use tungstenite::Error;

use serde_json;

use std::net::TcpListener;

pub use modules::errors::LOG_FILE;
pub use twitcheventsub_structs::*;

mod modules;

use crate::modules::token::Token;

#[cfg(feature = "logging")]
pub use log::{error, info, warn};

pub use crate::modules::{
  emotebuilder::*,
  errors::EventSubError,
  eventsub,
  token::{TokenAccess, TwitchKeys},
  twitch_http::{AuthType, RequestType, TwitchApi, TwitchHttpRequest},
};

#[derive(Debug)]
pub enum ResponseType {
  Event(Event),
  Error(EventSubError),
  RawResponse(String),
  Close,
  Ready,
}

#[must_use]
pub struct TwitchEventSubApiBuilder {
  twitch_keys: TwitchKeys,
  subscriptions: Vec<Subscription>,
  redirect_url: Option<String>,

  bot_is_local: bool,
  generate_token_if_none: bool,
  generate_token_on_scope_error: bool,
  generate_access_token_on_expire: bool,
  auto_save_load_created_tokens: Option<(String, String)>,
  only_raw_responses: bool,
  enable_irc: Option<(String, String)>,
}

impl TwitchEventSubApiBuilder {
  pub fn new(tk: TwitchKeys) -> TwitchEventSubApiBuilder {
    TwitchEventSubApiBuilder {
      twitch_keys: tk,
      subscriptions: Vec::new(),
      redirect_url: None,

      bot_is_local: true,
      generate_token_if_none: false,
      generate_token_on_scope_error: false,
      generate_access_token_on_expire: false,
      auto_save_load_created_tokens: None,
      only_raw_responses: false,
      enable_irc: None,
    }
  }

  pub fn enable_irc<S: Into<String>, T: Into<String>>(
    mut self,
    username: T,
    channel: S,
  ) -> TwitchEventSubApiBuilder {
    self.enable_irc = Some((username.into(), channel.into()));
    self
  }

  pub fn is_run_remotely(mut self) -> TwitchEventSubApiBuilder {
    self.bot_is_local = false;
    self
  }

  pub fn add_subscription(mut self, sub: Subscription) -> TwitchEventSubApiBuilder {
    self.subscriptions.push(sub);
    self
  }

  pub fn add_subscriptions(mut self, mut subs: Vec<Subscription>) -> TwitchEventSubApiBuilder {
    self.subscriptions.append(&mut subs);
    self
  }

  pub fn set_redirect_url<S: Into<String>>(mut self, url: S) -> TwitchEventSubApiBuilder {
    self.redirect_url = Some(url.into());
    self
  }

  pub fn generate_new_token_if_insufficent_scope(
    mut self,
    should_generate: bool,
  ) -> TwitchEventSubApiBuilder {
    self.generate_token_on_scope_error = should_generate;
    self
  }

  pub fn generate_new_token_if_none(mut self, should_generate: bool) -> TwitchEventSubApiBuilder {
    self.generate_token_if_none = should_generate;
    self
  }

  pub fn generate_access_token_on_expire(
    mut self,
    should_generate_access_token_on_expire: bool,
  ) -> TwitchEventSubApiBuilder {
    self.generate_access_token_on_expire = should_generate_access_token_on_expire;
    self
  }

  pub fn auto_save_load_created_tokens<S: Into<String>, T: Into<String>>(
    mut self,
    user_token_file: S,
    refresh_token_file: T,
  ) -> TwitchEventSubApiBuilder {
    self.auto_save_load_created_tokens = Some((user_token_file.into(), refresh_token_file.into()));
    self
  }

  pub fn subscriptions(&self) -> Vec<Subscription> {
    self.subscriptions.clone()
  }

  pub fn set_keys(&mut self, keys: TwitchKeys) {
    self.twitch_keys = keys;
  }

  pub fn receive_all_responses_raw(&mut self, receive_raw_data: bool) {
    self.only_raw_responses = receive_raw_data;
  }

  pub fn build(mut self) -> Result<TwitchEventSubApi, EventSubError> {
    let mut newly_generated_token = None;

    if self.subscriptions.is_empty() {
      #[cfg(feature = "logging")]
      error!("No Subscriptions selected.");
      return Err(EventSubError::NoSubscriptionsRequested);
    }

    if (self.generate_token_if_none || self.generate_token_on_scope_error)
      && self.redirect_url.is_none()
    {
      #[cfg(feature = "logging")]
      error!("No redirect url given when generate token is enabled.");
      return Err(EventSubError::UnhandledError(
        "Redirect url was not set, when a generate token setting was enabled.".to_owned(),
      ));
    }

    let mut save_new_tokens = false;
    // If there is no access token
    if self.twitch_keys.access_token.is_none() {
      // If auto save and load created tokens is enabled
      // We check those files for the relevant keys
      if let Some((ref token_file, ref refresh_file)) = self.auto_save_load_created_tokens {
        let mut new_token = String::new();
        let mut refresh_token = String::new();

        if let Ok(mut reader) = fs::OpenOptions::new()
          .append(false)
          .create(false)
          .read(true)
          .open(token_file)
        {
          let _ = reader.read_to_string(&mut new_token);
        }

        if let Ok(mut reader) = fs::OpenOptions::new()
          .append(false)
          .create(false)
          .read(true)
          .open(refresh_file)
        {
          let _ = reader.read_to_string(&mut refresh_token);
        }

        if !new_token.is_empty() {
          #[cfg(feature = "logging")]
          info!("Found user access token!");
          self.twitch_keys.access_token = Some(TokenAccess::User(new_token));
        }
        if !refresh_token.is_empty() {
          #[cfg(feature = "logging")]
          info!("Found refresh token!");
          self.twitch_keys.refresh_token = Some(refresh_token);
        }

        let mut generate_token =
          self.twitch_keys.refresh_token.is_none() || self.twitch_keys.access_token.is_none();

        if self.generate_token_if_none {
          if generate_token {
            if let Some(refresh_token) = &self.twitch_keys.refresh_token {
              #[cfg(feature = "logging")]
              info!(
                "No access token provided, attempting to generate access token from refresh token."
              );
              // Try to create new token from refresh token
              if let Ok(token) = TwitchApi::generate_token_from_refresh_token(
                self.twitch_keys.client_id.to_owned(),
                self.twitch_keys.client_secret.to_owned(),
                refresh_token,
              ) {
                #[cfg(feature = "logging")]
                info!("Generated user access token from refresh key.");
                self.twitch_keys.access_token = Some(token.access.clone());
                self.twitch_keys.refresh_token = Some(token.refresh.to_owned());
                newly_generated_token = Some(token);
                save_new_tokens = true;
                generate_token = false;
              } else {
                #[cfg(feature = "logging")]
                warn!("Couldn't generate access token from refresh token.");
              }
            }

            // If there are no refresh tokens or refresh token could created
            // a new access token, then get a completely new user token
            if generate_token {
              #[cfg(feature = "logging")]
              info!("Generating new user token.");
              // Returns app access token
              match TwitchApi::generate_user_token(
                self.twitch_keys.client_id.to_owned(),
                self.twitch_keys.client_secret.to_owned(),
                self.redirect_url.clone().unwrap(),
                self.bot_is_local,
                &self.subscriptions,
              ) {
                Ok(user_token) => {
                  #[cfg(feature = "logging")]
                  info!("Token created!");
                  self.twitch_keys.access_token = Some(user_token.access.clone());
                  self.twitch_keys.refresh_token = Some(user_token.refresh.to_owned());
                  newly_generated_token = Some(user_token);
                  save_new_tokens = true;
                }
                Err(e) => {
                  #[cfg(feature = "logging")]
                  error!("Failed to generate token: {:?}", e);
                  return Err(e);
                }
              }
            }
          }
        } else {
          #[cfg(feature = "logging")]
          error!("No access token provided");
          return Err(EventSubError::NoAccessTokenProvided);
        }
      }
    }

    match TwitchEventSubApi::check_token_meets_requirements(
      self.twitch_keys.access_token.clone().unwrap(),
      &self.subscriptions,
    ) {
      Ok(token_meets_requirements) => {
        if !token_meets_requirements {
          if self.generate_token_on_scope_error {
            #[cfg(feature = "logging")]
            info!("Generating new token because current token doesn't have correct scope.");
            match TwitchApi::generate_user_token(
              self.twitch_keys.client_id.to_owned(),
              self.twitch_keys.client_secret.to_owned(),
              self.redirect_url.clone().unwrap(),
              self.bot_is_local,
              &self.subscriptions,
            ) {
              Ok(user_token) => {
                #[cfg(feature = "logging")]
                info!("Token Generated!");
                self.twitch_keys.refresh_token = Some(user_token.refresh.clone());
                self.twitch_keys.access_token = Some(user_token.access.to_owned());
                newly_generated_token = Some(user_token);
                save_new_tokens = true;
              }
              Err(e) => {
                #[cfg(feature = "logging")]
                error!("Generating token failed: {:?}", e);
                return Err(e);
              }
            }
          } else {
            #[cfg(feature = "logging")]
            error!("Token missing required scope!");
            return Err(EventSubError::TokenMissingScope);
          }
        }
      }
      Err(EventSubError::TokenRequiresRefreshing(http)) => {
        #[cfg(feature = "logging")]
        warn!("Checking validation failed as token needs refreshing");

        TwitchEventSubApi::regen_token_if_401(
          Err(EventSubError::TokenRequiresRefreshing(http)),
          &mut self.twitch_keys,
          &self.auto_save_load_created_tokens,
        )?;
      }
      Err(e) => {
        #[cfg(feature = "logging")]
        error!("Failed parsing validation response: {:?}", e);
        return Err(e);
      }
    }

    if save_new_tokens {
      if let Some((token_file, refresh_file)) = self.auto_save_load_created_tokens {
        #[cfg(feature = "logging")]
        info!("saving tokens");
        if let Some(new_token) = newly_generated_token {
          if let Err(e) = new_token.save_to_file(token_file, refresh_file) {
            #[cfg(feature = "logging")]
            warn!("Failed to save tokens to file!");
            return Err(e);
          }
        }
      }
    }

    TwitchEventSubApi::new(
      self.twitch_keys,
      self.subscriptions,
      Vec::new(),
      self.enable_irc,
    )
    .map_err(|e| EventSubError::UnhandledError(e.to_string()))
  }
}

#[derive(Debug)]
pub struct TwitchEventSubApi {
  _receive_thread: JoinHandle<()>,
  messages_received: SyncReceiver<ResponseType>,
  twitch_keys: TwitchKeys,
  _token: Arc<Mutex<Token>>,
  save_locations: Option<(String, String)>,
  subscriptions: Vec<Subscription>,
  subscription_data: Vec<String>,
}

impl TwitchEventSubApi {
  pub fn builder(twitch_keys: TwitchKeys) -> TwitchEventSubApiBuilder {
    TwitchEventSubApiBuilder::new(twitch_keys)
  }

  pub fn new(
    mut twitch_keys: TwitchKeys,
    subscriptions: Vec<Subscription>,
    custom_subscription_data: Vec<String>,
    irc_channel: Option<(String, String)>,
  ) -> Result<TwitchEventSubApi, Error> {
    let mut irc = None;

    let this_user = TwitchEventSubApi::regen_token_if_401(
      TwitchApi::get_users(
        twitch_keys
          .access_token
          .clone()
          .expect("Access token not set")
          .get_token(),
        Vec::<String>::new(),
        Vec::<String>::new(),
        twitch_keys.client_id.to_string(),
      ),
      &mut twitch_keys,
      &None,
    )
    .and_then(|x| {
      serde_json::from_str::<Users>(&x).map_err(|e| EventSubError::ParseError(e.to_string()))
    });
    twitch_keys.this_account_id = match this_user {
      Ok(users) => users.data.into_iter().next().unwrap().id,
      Err(err) => {
        #[cfg(feature = "logging")]
        error!("Failed to get account id: {:?}", err);
        dbg!(&err);
        twitch_keys.broadcaster_account_id.clone()
      }
    };

    if let Some((username, channel)) = irc_channel {
      let mut new_irc = IRCChat::new(
        username,
        twitch_keys.access_token.clone().unwrap().get_token(),
      );
      new_irc.join_channel(channel);

      irc = Some(new_irc);
    }

    #[cfg(feature = "logging")]
    info!("Starting websocket client.");
    let (client, _) = match connect(CONNECTION_EVENTS) {
      Ok(a) => a,
      Err(e) => return Err(e),
    };

    let receiver = client;

    let (transmit_messages, receive_message) = channel();

    let token = Arc::new(Mutex::new(Token::new(
      twitch_keys
        .access_token
        .to_owned()
        .unwrap_or(TokenAccess::User("".to_owned())),
      twitch_keys
        .refresh_token
        .to_owned()
        .unwrap_or("".to_owned()),
      0.0,
    )));

    let keys_clone = twitch_keys.clone();
    let subscriptions_clone = subscriptions.clone();
    let custom_subscription_data_clone = custom_subscription_data.clone();
    let receive_thread = thread::spawn(move || {
      eventsub::events(
        receiver,
        transmit_messages,
        subscriptions_clone,
        custom_subscription_data_clone,
        keys_clone,
        None,
        irc,
      )
    });

    Ok(TwitchEventSubApi {
      _receive_thread: receive_thread,
      messages_received: receive_message,
      twitch_keys,
      _token: token,
      save_locations: None,
      subscriptions,
      subscription_data: custom_subscription_data,
    })
  }

  pub fn restart_websockets(&mut self) -> Result<(), EventSubError> {
    let keys_clone = self.twitch_keys.clone();
    let subscriptions = self.subscriptions.clone();
    let custom_subscription_data = self.subscription_data.clone();
    match TwitchEventSubApi::new(keys_clone, subscriptions, custom_subscription_data, None) {
      Ok(t) => {
        *self = t;
        Ok(())
      }
      Err(e) => Err(EventSubError::WebsocketRestartFailed(e.to_string())),
    }
  }

  pub fn validate_token<S: Into<String>>(token: S) -> Result<Validation, EventSubError> {
    TwitchHttpRequest::new(VALIDATION_TOKEN_URL)
      .header_authorisation(token.into(), AuthType::OAuth)
      .run()
      .and_then(|data| {
        serde_json::from_str::<Validation>(&data)
          .map_err(|e| EventSubError::ParseError(e.to_string()))
      })
  }

  pub fn check_token_meets_requirements(
    access_token: TokenAccess,
    subs: &[Subscription],
  ) -> Result<bool, EventSubError> {
    TwitchEventSubApi::validate_token(access_token.get_token()).map(|validation| {
      if validation.is_error() {
        false
      } else {
        subs
          .iter()
          .filter(|s| !s.required_scope().is_empty())
          .all(move |s| {
            let r = s.required_scope();

            let requirements = r.split('+').map(ToString::to_string).collect::<Vec<_>>();

            for req in requirements {
              if !validation.scopes.as_ref().unwrap().contains(&req) {
                return false;
              }
            }
            true
          })
      }
    })
  }

  pub fn open_browser<S: Into<String>, T: Into<String>>(
    browser_url: S,
    redirect_url: T,
    is_local: bool,
  ) -> Result<String, EventSubError> {
    let browser_url = browser_url.into();
    if is_local {
      if let Err(e) = open::that_detached(browser_url) {
        #[cfg(feature = "logging")]
        error!("Failed to open browser: {}", e);
        return Err(EventSubError::UnhandledError(e.to_string()));
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

    #[cfg(feature = "logging")]
    info!("Starting local tcp listener for token generation");
    let listener = TcpListener::bind(&redirect_url).expect("Failed to create tcp listener.");

    // accept connections and process them serially
    match listener.accept() {
      Ok((mut stream, _b)) => {
        let mut http_output = String::new();
        stream
          .read_to_string(&mut http_output)
          .expect("Failed to read tcp stream.");
        Ok(http_output)
      }
      Err(e) => Err(EventSubError::UnhandledError(e.to_string())),
    }
  }

  fn regen_token_if_401(
    result: Result<String, EventSubError>,
    twitch_keys: &mut TwitchKeys,
    auto_save_load_created_tokens: &Option<(String, String)>,
  ) -> Result<String, EventSubError> {
    if let Err(EventSubError::TokenRequiresRefreshing(mut http_request)) = result {
      #[cfg(feature = "logging")]
      warn!("Token requires refreshing return!");
      if let Ok(token) = TwitchApi::generate_token_from_refresh_token(
        twitch_keys.client_id.to_owned(),
        twitch_keys.client_secret.to_owned(),
        twitch_keys.refresh_token.clone().unwrap().to_owned(),
      ) {
        #[cfg(feature = "logging")]
        info!("Generated new keys as 401 was returned!");
        twitch_keys.access_token = Some(token.access);
        twitch_keys.refresh_token = Some(token.refresh.to_owned());
      }

      // TODO: SAVE
      if let Some((token_file, refresh_file)) = auto_save_load_created_tokens {
        #[cfg(feature = "logging")]
        info!("saving tokens");
        if let Some(new_token) = twitch_keys.token() {
          if let Err(e) = new_token.save_to_file(token_file, refresh_file) {
            #[cfg(feature = "logging")]
            warn!("Failed to save tokens to file!");
            return Err(e);
          }
        }
      }

      let access_token = twitch_keys.access_token.as_ref().unwrap();
      http_request.update_token(access_token.get_token());
      http_request.run()
    } else {
      if result.is_err() {
        #[cfg(feature = "logging")]
        warn!(
          "regen 401 called with result being an error, but wasnt token refresh required: {:?}",
          result
        );
      }
      result
    }
  }

  fn process_token_query<S: Into<String>>(post_data: S) -> Result<Token, EventSubError> {
    TwitchHttpRequest::new(TWITCH_TOKEN_URL)
      .url_encoded_content()
      .is_post(post_data)
      .run()
      .and_then(|twitch_response| {
        serde_json::from_str::<NewAccessTokenResponse>(&twitch_response)
          .map_err(|_| EventSubError::AuthorisationError(twitch_response))
          .map(|new_token_data| {
            Token::new_user_token(
              new_token_data.access_token,
              new_token_data.refresh_token.unwrap(),
              new_token_data.expires_in as f32,
            )
          })
      })
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

  pub fn delete_message<S: Into<String>>(
    &mut self,
    message_id: S,
  ) -> Result<String, EventSubError> {
    let broadcaster_account_id = self.twitch_keys.broadcaster_account_id.to_string();
    let moderator_account_id = self.twitch_keys.this_account_id.to_string();
    let access_token = self
      .twitch_keys
      .access_token
      .clone()
      .expect("No Access Token set")
      .get_token();
    let client_id = self.twitch_keys.client_id.to_string();

    TwitchEventSubApi::regen_token_if_401(
      TwitchApi::delete_message(
        broadcaster_account_id,
        moderator_account_id,
        message_id.into(),
        access_token,
        client_id,
      ),
      &mut self.twitch_keys,
      &self.save_locations,
    )
  }

  pub fn timeout_user<S: Into<String>, T: Into<String>>(
    &mut self,
    user_id: S,
    duration: u32,
    reason: T,
  ) {
    let broadcaster_account_id = self.twitch_keys.broadcaster_account_id.to_string();
    let moderator_account_id = self.twitch_keys.this_account_id.to_string();

    let access_token = self
      .twitch_keys
      .access_token
      .clone()
      .expect("No Access Token set")
      .get_token();
    let client_id = self.twitch_keys.client_id.to_string();
    let _ = TwitchEventSubApi::regen_token_if_401(
      TwitchApi::timeout_user(
        access_token,
        client_id,
        broadcaster_account_id,
        moderator_account_id,
        user_id.into(),
        duration,
        reason.into(),
      ),
      &mut self.twitch_keys,
      &self.save_locations,
    );
  }

  pub fn get_ad_schedule(&mut self) -> Result<AdSchedule, EventSubError> {
    let access_token = self
      .twitch_keys
      .access_token
      .clone()
      .expect("Access token not set")
      .get_token();
    let client_id = self.twitch_keys.client_id.to_string();
    let broadcaster_id = self.twitch_keys.broadcaster_account_id.to_string();

    TwitchEventSubApi::regen_token_if_401(
      TwitchApi::get_ad_schedule(broadcaster_id, access_token, client_id),
      &mut self.twitch_keys,
      &self.save_locations,
    )
    .and_then(|x| serde_json::from_str(&x).map_err(|e| EventSubError::ParseError(e.to_string())))
  }

  pub fn get_chatters(&mut self) -> Result<GetChatters, EventSubError> {
    let access_token = self
      .twitch_keys
      .access_token
      .clone()
      .expect("Access token not set")
      .get_token();
    let broadcaster_id = self.twitch_keys.broadcaster_account_id.to_string();
    let client_id = self.twitch_keys.client_id.to_string();

    TwitchEventSubApi::regen_token_if_401(
      TwitchApi::get_chatters(
        broadcaster_id.to_owned(),
        broadcaster_id,
        access_token,
        client_id,
      ),
      &mut self.twitch_keys,
      &self.save_locations,
    )
    .and_then(|x| serde_json::from_str(&x).map_err(|e| EventSubError::ParseError(e.to_string())))
  }

  pub fn get_moderators(&mut self) -> Result<Moderators, EventSubError> {
    let access_token = self
      .twitch_keys
      .access_token
      .clone()
      .expect("Access token not set")
      .get_token();
    let broadcaster_id = self.twitch_keys.broadcaster_account_id.to_string();
    let client_id = self.twitch_keys.client_id.to_string();

    TwitchEventSubApi::regen_token_if_401(
      TwitchApi::get_moderators(access_token, client_id, broadcaster_id),
      &mut self.twitch_keys,
      &self.save_locations,
    )
    .and_then(|x| serde_json::from_str(&x).map_err(|e| EventSubError::ParseError(e.to_string())))
  }

  pub fn get_custom_rewards(&mut self) -> Result<GetCustomRewards, EventSubError> {
    let access_token = self
      .twitch_keys
      .access_token
      .clone()
      .expect("Access token not set")
      .get_token();
    let broadcaster_id = self.twitch_keys.broadcaster_account_id.to_string();
    let client_id = self.twitch_keys.client_id.to_string();

    TwitchEventSubApi::regen_token_if_401(
      TwitchApi::get_custom_rewards(access_token, client_id, broadcaster_id),
      &mut self.twitch_keys,
      &self.save_locations,
    )
    .and_then(|x| serde_json::from_str(&x).map_err(|e| EventSubError::ParseError(e.to_string())))
  }

  pub fn create_custom_reward(
    &mut self,
    custom_reward: CreateCustomReward,
  ) -> Result<CreatedCustomRewardResponse, EventSubError> {
    let access_token = self
      .twitch_keys
      .access_token
      .clone()
      .expect("Access token not set")
      .get_token();
    let broadcaster_id = self.twitch_keys.broadcaster_account_id.to_string();
    let client_id = self.twitch_keys.client_id.to_string();

    TwitchEventSubApi::regen_token_if_401(
      TwitchApi::create_custom_reward(access_token, client_id, broadcaster_id, custom_reward),
      &mut self.twitch_keys,
      &self.save_locations,
    )
    .and_then(|x| serde_json::from_str(&x).map_err(|e| EventSubError::ParseError(e.to_string())))
  }

  // come back later
  pub fn delete_custom_reward<T: Into<String>>(&mut self, id: T) -> Result<(), EventSubError> {
    let access_token = self
      .twitch_keys
      .access_token
      .clone()
      .expect("Access token not set")
      .get_token();
    let broadcaster_id = self.twitch_keys.broadcaster_account_id.to_string();
    let client_id = self.twitch_keys.client_id.to_string();

    TwitchEventSubApi::regen_token_if_401(
      TwitchApi::delete_custom_reward(access_token, client_id, broadcaster_id, id),
      &mut self.twitch_keys,
      &self.save_locations,
    )
    .and_then(|_| Ok(()))
  }

  pub fn get_users_from_ids<S: Into<String>>(
    &mut self,
    ids: Vec<S>,
  ) -> Result<Users, EventSubError> {
    self.get_users(
      ids
        .into_iter()
        .map(|a| a.into())
        .collect::<Vec<String>>()
        .into(),
      Vec::with_capacity(0),
    )
  }

  pub fn get_users_from_logins<S: Into<String>>(
    &mut self,
    logins: Vec<S>,
  ) -> Result<Users, EventSubError> {
    self.get_users(
      Vec::with_capacity(0),
      logins
        .into_iter()
        .map(|a| a.into())
        .collect::<Vec<String>>()
        .into(),
    )
  }

  pub fn get_users_self(&mut self) -> Result<Users, EventSubError> {
    self.get_users(Vec::with_capacity(0), Vec::with_capacity(0))
  }

  ///
  /// It is recommended to use the get_users_from_* methods instead.
  ///
  /// How ever if you wish to use it manuall please
  /// Refer to https://dev.twitch.tv/docs/api/reference/#get-users for specifics how to use
  ///
  pub fn get_users(&mut self, id: Vec<String>, login: Vec<String>) -> Result<Users, EventSubError> {
    let access_token = self
      .twitch_keys
      .access_token
      .clone()
      .expect("Access token not set")
      .get_token();
    let client_id = self.twitch_keys.client_id.to_string();

    TwitchEventSubApi::regen_token_if_401(
      TwitchApi::get_users(access_token, id, login, client_id),
      &mut self.twitch_keys,
      &self.save_locations,
    )
    .and_then(|x| serde_json::from_str(&x).map_err(|e| EventSubError::ParseError(e.to_string())))
  }

  pub fn get_channel_emotes<S: Into<String>>(
    &mut self,
    broadcaster_id: S,
  ) -> Result<ChannelEmotes, EventSubError> {
    let access_token = self
      .twitch_keys
      .access_token
      .clone()
      .expect("Access token not set")
      .get_token();
    let client_id = self.twitch_keys.client_id.to_string();

    let broadcaster_id: String = broadcaster_id.into();
    if let Err(_) = broadcaster_id.parse::<u32>() {
      return Err(EventSubError::UnhandledError(
        "Broadcaster id must be numeric!".to_string(),
      ));
    }

    TwitchEventSubApi::regen_token_if_401(
      TwitchApi::get_channel_emotes(access_token, client_id, broadcaster_id),
      &mut self.twitch_keys,
      &self.save_locations,
    )
    .and_then(|x| serde_json::from_str(&x).map_err(|e| EventSubError::ParseError(e.to_string())))
  }

  pub fn get_global_emotes(&mut self) -> Result<GlobalEmotes, EventSubError> {
    let access_token = self
      .twitch_keys
      .access_token
      .clone()
      .expect("Access token not set")
      .get_token();
    let client_id = self.twitch_keys.client_id.to_string();

    TwitchEventSubApi::regen_token_if_401(
      TwitchApi::get_global_emotes(access_token, client_id),
      &mut self.twitch_keys,
      &self.save_locations,
    )
    .and_then(|x| serde_json::from_str(&x).map_err(|e| EventSubError::ParseError(e.to_string())))
  }

  pub fn get_image_data_from_url<S: Into<String>>(emote_url: S) -> Result<Vec<u8>, EventSubError> {
    match attohttpc::get(emote_url.into()).send() {
      Ok(new_image_data) => Ok(new_image_data.bytes().unwrap()),
      Err(e) => Err(EventSubError::HttpFailed(e.to_string())),
    }
  }

  pub fn get_emote_sets<S: Into<String>>(
    &mut self,
    emote_set_id: S,
  ) -> Result<GlobalEmotes, EventSubError> {
    let access_token = self
      .twitch_keys
      .access_token
      .clone()
      .expect("Access token not set")
      .get_token();
    let client_id = self.twitch_keys.client_id.to_string();

    TwitchEventSubApi::regen_token_if_401(
      TwitchApi::get_emote_set(emote_set_id.into(), access_token, client_id),
      &mut self.twitch_keys,
      &self.save_locations,
    )
    .and_then(|x| serde_json::from_str(&x).map_err(|e| EventSubError::ParseError(e.to_string())))
  }

  pub fn send_chat_message<S: Into<String>>(
    &mut self,
    message: S,
  ) -> Result<String, EventSubError> {
    self.send_chat_message_with_reply(message, None)
  }

  pub fn send_chat_message_with_reply<S: Into<String>>(
    &mut self,
    message: S,
    reply_message_parent_id: Option<String>,
  ) -> Result<String, EventSubError> {
    let message: String = message.into();
    let access_token = self
      .twitch_keys
      .access_token
      .clone()
      .expect("No Access Token set")
      .get_token();
    let client_id = self.twitch_keys.client_id.to_string();
    let broadcaster_account_id = self.twitch_keys.broadcaster_account_id.to_string();
    let sender_id = self
      .twitch_keys
      .sender_account_id
      .clone()
      .unwrap_or(self.twitch_keys.this_account_id.to_string());

    TwitchEventSubApi::regen_token_if_401(
      TwitchApi::send_chat_message(
        message,
        access_token,
        client_id,
        broadcaster_account_id,
        Some(sender_id),
        reply_message_parent_id,
      ),
      &mut self.twitch_keys,
      &self.save_locations,
    )
  }

  pub fn send_announcement<S: Into<String>, T: Into<String>>(
    &mut self,
    message: S,
    colour: Option<T>,
  ) -> Result<String, EventSubError> {
    let message: String = message.into();
    let access_token = self
      .twitch_keys
      .access_token
      .clone()
      .expect("No Access Token set")
      .get_token();
    let client_id = self.twitch_keys.client_id.to_string();
    let broadcaster_account_id = self.twitch_keys.broadcaster_account_id.to_string();
    let sender_id = self
      .twitch_keys
      .sender_account_id
      .clone()
      .unwrap_or(self.twitch_keys.this_account_id.to_string());

    TwitchEventSubApi::regen_token_if_401(
      TwitchApi::send_announcement(
        message,
        access_token,
        client_id,
        broadcaster_account_id,
        sender_id,
        colour,
      ),
      &mut self.twitch_keys,
      &self.save_locations,
    )
  }

  pub fn send_shoutout<S: Into<String>>(&mut self, to_broadcaster_id: S) {
    let broadcaster_account_id = self.twitch_keys.broadcaster_account_id.to_string();
    let moderator_account_id = self.twitch_keys.this_account_id.to_string();

    let access_token = self
      .twitch_keys
      .access_token
      .clone()
      .expect("No Access Token set")
      .get_token();
    let client_id = self.twitch_keys.client_id.to_string();
    let _ = TwitchEventSubApi::regen_token_if_401(
      TwitchApi::send_shoutout(
        access_token,
        client_id,
        broadcaster_account_id,
        to_broadcaster_id,
        moderator_account_id,
      ),
      &mut self.twitch_keys,
      &self.save_locations,
    );
  }
}
