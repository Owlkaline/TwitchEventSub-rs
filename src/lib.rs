use std::fs;
use std::net::TcpStream;
use std::sync::mpsc::{channel, Receiver as SyncReceiver, Sender as SyncSender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use std::sync::{Arc, Mutex};

use crate::modules::consts::*;
use open;
use serde::Serialize;
use std::io::{ErrorKind, Read};

use websocket::client::ClientBuilder;
use websocket::stream::sync::TlsStream;
pub use websocket::WebSocketError;
use websocket::{sync::Client, OwnedMessage};

use std::net::TcpListener;

mod modules;

use crate::modules::{errors::*, generic_message::*, token::Token};

pub use log::{error, info, warn};

use serde_derive::{Deserialize as Deserialise, Serialize as Serialise};

pub use crate::modules::{
  errors::EventSubError,
  generic_message::{Badge, Cheer, Event, Reply, Reward, Transport},
  messages::{AdBreakBeginData, CustomPointsRewardRedeemData, MessageData, MessageType, RaidData},
  subscriptions::{Condition, EventSubscription, Subscription},
  token::{TokenAccess, TwitchKeys},
  twitch_http::{AuthType, RequestType, TwitchApi, TwitchHttpRequest},
};

#[must_use]
pub struct TwitchEventSubApiBuilder {
  twitch_keys: TwitchKeys,
  subscriptions: Vec<Subscription>,
  redirect_url: Option<String>,

  generate_token_if_none: bool,
  generate_token_on_scope_error: bool,
  generate_access_token_on_expire: bool,
  auto_save_load_created_tokens: Option<(String, String)>,
  only_raw_responses: bool,
}

impl TwitchEventSubApiBuilder {
  pub fn new(tk: TwitchKeys) -> TwitchEventSubApiBuilder {
    TwitchEventSubApiBuilder {
      twitch_keys: tk,
      subscriptions: Vec::new(),
      redirect_url: None,

      generate_token_if_none: false,
      generate_token_on_scope_error: false,
      generate_access_token_on_expire: false,
      auto_save_load_created_tokens: None,
      only_raw_responses: false,
    }
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

  pub fn recieve_all_responses_raw(&mut self, recieve_raw_data: bool) {
    self.only_raw_responses = recieve_raw_data;
  }

  pub fn build(mut self) -> Result<TwitchEventSubApi, EventSubError> {
    log_builder();
    let mut newly_generated_token = None;

    if self.subscriptions.is_empty() {
      error!("No Subscriptions selected.");
      return Err(EventSubError::NoSubscriptionsRequested);
    }

    if (self.generate_token_if_none || self.generate_token_on_scope_error)
      && self.redirect_url.is_none()
    {
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
          info!("Found user access token!");
          self.twitch_keys.access_token = Some(TokenAccess::User(new_token));
        }
        if !refresh_token.is_empty() {
          info!("Found refresh token!");
          self.twitch_keys.refresh_token = Some(refresh_token);
        }

        let mut generate_token =
          self.twitch_keys.refresh_token.is_none() || self.twitch_keys.access_token.is_none();

        if self.generate_token_if_none {
          if generate_token {
            if let Some(refresh_token) = &self.twitch_keys.refresh_token {
              info!(
                "No access token provided, attempting to generate access token from refresh token."
              );
              // Try to create new token from refresh token
              if let Ok(token) = TwitchApi::generate_token_from_refresh_token(
                self.twitch_keys.client_id.to_owned(),
                self.twitch_keys.client_secret.to_owned(),
                refresh_token,
              ) {
                info!("Generated user access token from refresh key.");
                self.twitch_keys.access_token = Some(token.access.clone());
                self.twitch_keys.refresh_token = Some(token.refresh.to_owned());
                newly_generated_token = Some(token);
                save_new_tokens = true;
                generate_token = false;
              } else {
                warn!("Couldn't generate access token from refresh token.");
              }
            }

            // If there are no refresh tokens or refresh token could created
            // a new access token, then get a completely new user token
            if generate_token {
              info!("Generating new user token.");
              // Returns app access token
              match TwitchApi::generate_user_token(
                self.twitch_keys.client_id.to_owned(),
                self.twitch_keys.client_secret.to_owned(),
                self.redirect_url.clone().unwrap(),
                &self.subscriptions,
              ) {
                Ok(user_token) => {
                  info!("Token created!");
                  self.twitch_keys.access_token = Some(user_token.access.clone());
                  self.twitch_keys.refresh_token = Some(user_token.refresh.to_owned());
                  newly_generated_token = Some(user_token);
                  save_new_tokens = true;
                }
                Err(e) => {
                  error!("Failed to generate token: {:?}", e);
                  return Err(e);
                }
              }
            }
          }
        } else {
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
            info!("Generating new token because current token doesn't have correct scope.");
            match TwitchApi::generate_user_token(
              self.twitch_keys.client_id.to_owned(),
              self.twitch_keys.client_secret.to_owned(),
              self.redirect_url.clone().unwrap(),
              &self.subscriptions,
            ) {
              Ok(user_token) => {
                info!("Token Generated!");
                self.twitch_keys.refresh_token = Some(user_token.refresh.clone());
                self.twitch_keys.access_token = Some(user_token.access.to_owned());
                newly_generated_token = Some(user_token);
                save_new_tokens = true;
              }
              Err(e) => {
                error!("Generating token failed: {:?}", e);
                return Err(e);
              }
            }
          } else {
            error!("Token missing required scope!");
            return Err(EventSubError::TokenMissingScope);
          }
        }
      }
      Err(e) => {
        error!("Failed parsing validation response: {:?}", e);
        return Err(e);
      }
    }

    if save_new_tokens {
      if let Some((token_file, refresh_file)) = self.auto_save_load_created_tokens {
        info!("saving tokens");
        if let Some(new_token) = newly_generated_token {
          if let Err(e) = new_token.save_to_file(token_file, refresh_file) {
            warn!("Failed to save tokens to file!");
            return Err(e);
          }
        }
      }
    }

    TwitchEventSubApi::new(self.twitch_keys, self.subscriptions, Vec::new())
      .map_err(|e| EventSubError::UnhandledError(e.to_string()))
  }
}

pub struct TwitchEventSubApi {
  _receive_thread: JoinHandle<()>,

  messages_received: SyncReceiver<MessageType>,
  twitch_keys: TwitchKeys,
  _token: Arc<Mutex<Token>>,
}

impl TwitchEventSubApi {
  pub fn builder(twitch_keys: TwitchKeys) -> TwitchEventSubApiBuilder {
    TwitchEventSubApiBuilder::new(twitch_keys)
  }

  pub fn new(
    twitch_keys: TwitchKeys,
    subscriptions: Vec<Subscription>,
    custom_subscription_data: Vec<String>,
  ) -> Result<TwitchEventSubApi, WebSocketError> {
    log_info();
    info!("Starting websocket client.");
    let client = match ClientBuilder::new(CONNECTION_EVENTS)
      .unwrap()
      .add_protocol("rust-websocket-events")
      .connect_secure(None)
    {
      Ok(c) => c,
      Err(e) => return Err(e),
    };

    let receiver = Arc::new(Mutex::new(client));

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
    let receive_thread = thread::spawn(move || {
      TwitchEventSubApi::event_sub_events(
        receiver,
        transmit_messages,
        subscriptions,
        custom_subscription_data,
        keys_clone,
      )
    });

    Ok(TwitchEventSubApi {
      _receive_thread: receive_thread,
      messages_received: receive_message,
      twitch_keys,
      _token: token,
    })
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
    subs: &Vec<Subscription>,
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
  ) -> Result<String, EventSubError> {
    if let Err(e) = open::that(browser_url.into()) {
      error!("Failed to open browser: {}", e);
      return Err(EventSubError::UnhandledError(e.to_string()));
    }

    let mut redirect_url = redirect_url.into().to_ascii_lowercase();

    if redirect_url.contains("http") {
      redirect_url = redirect_url
        .split('/')
        .collect::<Vec<_>>()
        .last()
        .unwrap()
        .to_string();
    }

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
  ) -> Result<String, EventSubError> {
    //warn!("Token return 401!");
    if let Err(EventSubError::TokenRequiresRefreshing(mut http_request)) = result {
      warn!("Token requires refreshing return!");
      if let Ok(token) = TwitchApi::generate_token_from_refresh_token(
        twitch_keys.client_id.to_owned(),
        twitch_keys.client_secret.to_owned(),
        twitch_keys.refresh_token.clone().unwrap().to_owned(),
      ) {
        println!("Generated new keys as 401 was returned!");
        info!("Generated new keys as 401 was returned!");
        twitch_keys.access_token = Some(token.access);
        twitch_keys.refresh_token = Some(token.refresh.to_owned());
      }

      let access_token = twitch_keys.access_token.as_ref().unwrap();
      http_request.update_token(access_token.get_token());
      http_request.run()
    } else {
      if result.is_err() {
        warn!(
          "regen 401 called with result being an error, but wasnt token refresh required: {:?}",
          result
        );
        println!("BIG ERROR!");
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

  ///
  /// Set duration to Duration::ZERO for completely non-blocking
  ///
  /// Recommended to use Duration::ZERO when used in games and
  /// in engines like Godot and Unity.
  ///
  /// For a chat bot I found setting duration to 1 millis was enough
  /// to reduce cpu from 3.2% to 0.8% maximum
  ///
  pub fn receive_messages(&mut self, duration: Duration) -> Vec<MessageType> {
    // check thread for new messages without waiting
    //
    // return new messages if any
    let mut messages = Vec::new();

    if let Ok(message) = self.messages_received.recv_timeout(duration) {
      messages.push(message);
    }

    messages
  }

  pub fn delete_message<S: Into<String>>(&mut self, message_id: S) {
    let broadcaster_account_id = self.twitch_keys.broadcaster_account_id.to_string();
    let moderator_account_id = broadcaster_account_id.to_owned();
    let access_token = self
      .twitch_keys
      .access_token
      .clone()
      .expect("No Access Token set")
      .get_token();
    let client_id = self.twitch_keys.client_id.to_string();

    let _ = TwitchEventSubApi::regen_token_if_401(
      TwitchApi::delete_message(
        broadcaster_account_id,
        moderator_account_id,
        message_id.into(),
        access_token,
        client_id,
      ),
      &mut self.twitch_keys,
    );
  }

  pub fn timeout_user<S: Into<String>, T: Into<String>>(
    &mut self,
    user_id: S,
    duration: u32,
    reason: T,
  ) {
    let broadcaster_account_id = self.twitch_keys.broadcaster_account_id.to_string();
    let moderator_account_id = broadcaster_account_id.to_owned();

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
    );
  }

  pub fn send_chat_message<S: Into<String>>(&mut self, message: S) {
    self.send_chat_message_with_reply(message, None);
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
      .unwrap_or(self.twitch_keys.broadcaster_account_id.to_string());

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
    )
  }

  #[cfg(feature = "only_raw_responses")]
  fn event_sub_events(
    client: Arc<Mutex<Client<TlsStream<TcpStream>>>>,
    message_sender: SyncSender<MessageType>,
    subscriptions: Vec<Subscription>,
    mut custom_subscriptions: Vec<String>,
    twitch_keys: TwitchKeys,
  ) {
    loop {
      let client = client.clone();
      let mut client = client.lock().unwrap();
      let message = match client.recv_message() {
        Ok(m) => m,
        Err(WebSocketError::IoError(e)) if e.kind() == ErrorKind::WouldBlock => {
          continue;
        }
        Err(e) => {
          error!("recv message error: {:?}", e);
          let _ = client.send_message(&OwnedMessage::Close(None));
          message_sender.send(MessageType::Close).unwrap();

          return;
        }
      };

      if let OwnedMessage::Text(msg) = message.clone() {
        message_sender.send(MessageType::RawResponse(msg)).unwrap();
        continue;
      }
    }
  }

  #[cfg(not(feature = "only_raw_responses"))]
  fn event_sub_events(
    client: Arc<Mutex<Client<TlsStream<TcpStream>>>>,
    message_sender: SyncSender<MessageType>,
    subscriptions: Vec<Subscription>,
    mut custom_subscriptions: Vec<String>,
    mut twitch_keys: TwitchKeys,
  ) {
    loop {
      let client = client.clone();
      let mut client = client.lock().unwrap();
      let message = match client.recv_message() {
        Ok(m) => m,
        Err(WebSocketError::IoError(e)) if e.kind() == ErrorKind::WouldBlock => {
          continue;
        }
        Err(e) => {
          error!("recv message error: {:?}", e);
          let _ = client.send_message(&OwnedMessage::Close(None));
          message_sender.send(MessageType::Close).unwrap();

          return;
        }
      };

      match message {
        OwnedMessage::Text(msg) => {
          let message = serde_json::from_str(&msg);

          if let Err(e) = message {
            error!("Unimplemented twitch response: {}\n{}", msg, e);
            message_sender.send(MessageType::RawResponse(msg)).unwrap();
            continue;
          }

          let message: GenericMessage = message.unwrap();

          match message.event_type() {
            EventMessageType::Welcome => {
              let session_id = message.clone().payload.unwrap().session.unwrap().id;

              let mut sub_data = subscriptions
                .iter()
                .filter_map(|s| {
                  serde_json::to_string(&s.construct_data(&session_id, &twitch_keys)).ok()
                })
                .collect::<Vec<_>>();
              sub_data.append(&mut custom_subscriptions);

              info!("Subscribing to events!");
              let mut clone_twitch_keys = twitch_keys.clone();
              if let Some(TokenAccess::User(ref token)) = twitch_keys.access_token {
                sub_data
                  .iter()
                  .map(|sub_data| {
                    let a = TwitchHttpRequest::new(SUBSCRIBE_URL)
                      .full_auth(token.to_owned(), twitch_keys.client_id.to_string())
                      .json_content()
                      .is_post(sub_data)
                      .run();
                    a
                  })
                  .map(|a| TwitchEventSubApi::regen_token_if_401(a, &mut clone_twitch_keys))
                  .filter_map(Result::err)
                  .for_each(|error| {
                    message_sender
                      .send(MessageType::Error(error))
                      .expect("Failed to send error Message back to main thread.");
                  });
              } else {
                let _ = message_sender.send(MessageType::Error(EventSubError::InvalidAccessToken(
                  format!(
                    "Expected TokenAccess::User(TOKENHERE) but found {:?}",
                    twitch_keys.access_token
                  ),
                )));
              }

              twitch_keys = clone_twitch_keys;
            }
            EventMessageType::KeepAlive => {
              //println!("Keep alive receive message sent, !implemented");
            }
            EventMessageType::Notification => {
              message_sender
                .send(MessageType::Event(message.payload.unwrap().event.unwrap()))
                .unwrap();
            }
            EventMessageType::Unknown => {
              if !custom_subscriptions.is_empty() {
                message_sender.send(MessageType::RawResponse(msg)).unwrap();
              }
            }
          }
        }
        OwnedMessage::Close(a) => {
          warn!("Close message received: {:?}", a);
          // Got a close message, so send a close message and return
          let _ = client.send_message(&OwnedMessage::Close(None));
          return;
        }
        OwnedMessage::Ping(_) => {
          match client.send_message(&OwnedMessage::Pong(Vec::new())) {
            // Send a pong in response
            Ok(()) => {}
            Err(e) => {
              error!("Received an Error from Server: {:?}", e);
              return;
            }
          }
        }
        _ => {}
      }
    }
  }
}
