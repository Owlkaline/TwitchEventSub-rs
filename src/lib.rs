use std::io::Write;

use std::fs;
use std::net::TcpStream;
use std::sync::mpsc::{channel, Receiver as SyncReceiver, Sender as SyncSender};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use std::sync::{Arc, Mutex};

use crate::modules::consts::*;
use open;
use std::io::Read;

use websocket::client::ClientBuilder;

use websocket::{sync::Client, OwnedMessage};

use std::net::TcpListener;

use websocket::stream::sync::TlsStream;

mod modules;

use crate::modules::generic_message::*;

use log::{error, info, warn, LevelFilter};
use simple_logging;

pub use websocket::WebSocketError;

pub const LOG_FILE: &str = "twitch_events.log";
pub const LOG_FILE_BUILDER: &str = "twitch_event_builder.log";

fn log_info() {
  let _ = simple_logging::log_to_file(LOG_FILE, LevelFilter::Info);
}

fn log_builder() {
  let _ = simple_logging::log_to_file(LOG_FILE_BUILDER, LevelFilter::Info);
}

pub use crate::modules::{
  generic_message::Reward,
  messages::MessageType,
  subscriptions::SubscriptionPermission,
  twitch_http::{AuthType, RequestType, TwitchApi, TwitchHttpRequest},
};

#[derive(Debug, PartialEq)]
pub enum EventSubError {
  TokenMissingScope,
  NoSubscriptionsRequested,
  AuthorisationError(String),
  WebsocketCreationFailed,
  UnhandledError(String),
  NoAccessTokenProvided,
  WriteError(String),
  // status 401 = invalid access token
  InvalidAccessToken(String),
  InvalidOauthToken(String),
  CurlFailed(curl::Error),
}

#[derive(Debug)]
pub enum TwitchKeysError {
  ClientIdNotFound,
  ClientSecretNotFound,
}

pub struct Token {
  pub access: TokenAccess,
  pub refresh: String,
  expires_in: f32,
  generation_time: Instant,
}

impl Token {
  pub fn new(access: TokenAccess, refresh: String, expires_in: f32) -> Token {
    Token {
      access,
      refresh,
      expires_in,
      generation_time: Instant::now(),
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

#[must_use]
pub struct TwitchEventSubApiBuilder {
  twitch_keys: TwitchKeys,
  subscriptions: Vec<SubscriptionPermission>,
  redirect_url: Option<String>,
  custom_subscription: Option<String>,
  generate_token_if_none: bool,
  generate_token_on_scope_error: bool,
  generate_access_token_on_expire: bool,
  auto_save_load_created_tokens: Option<(String, String)>,
}

impl TwitchEventSubApiBuilder {
  pub fn new(tk: TwitchKeys) -> TwitchEventSubApiBuilder {
    TwitchEventSubApiBuilder {
      twitch_keys: tk,
      subscriptions: Vec::new(),
      redirect_url: None,
      custom_subscription: None,
      generate_token_if_none: false,
      generate_token_on_scope_error: false,
      generate_access_token_on_expire: false,
      auto_save_load_created_tokens: None,
    }
  }

  pub fn add_subscription(mut self, sub: SubscriptionPermission) -> TwitchEventSubApiBuilder {
    self.subscriptions.push(sub);
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

  pub fn subscriptions(&self) -> Vec<SubscriptionPermission> {
    self.subscriptions.clone()
  }

  pub fn set_keys(&mut self, keys: TwitchKeys) {
    self.twitch_keys = keys;
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

        let mut generate_token = self.twitch_keys.refresh_token.is_none()
          && self.twitch_keys.access_token.is_none()
          || self.twitch_keys.access_token.is_none();

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
  receive_thread: JoinHandle<()>,
  send_thread: Option<JoinHandle<()>>,
  messages_received: SyncReceiver<MessageType>,
  twitch_keys: TwitchKeys,
}

impl TwitchEventSubApi {
  pub fn builder(twitch_keys: TwitchKeys) -> TwitchEventSubApiBuilder {
    TwitchEventSubApiBuilder::new(twitch_keys)
  }

  pub fn new(
    twitch_keys: TwitchKeys,
    subscriptions: Vec<SubscriptionPermission>,
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
      receive_thread,
      send_thread: None,
      messages_received: receive_message,
      twitch_keys,
    })
  }

  pub fn check_token_meets_requirements(
    access_token: TokenAccess,
    subs: &Vec<SubscriptionPermission>,
  ) -> Result<bool, EventSubError> {
    if let Ok(data) = TwitchHttpRequest::new(VALIDATION_TOKEN_URL)
      .header_authorisation(access_token.get_token(), AuthType::OAuth)
      .run()
    {
      match serde_json::from_str::<Validation>(&data) {
        Ok(validation) => {
          if validation.is_error() {
            Ok(false)
          } else {
            return Ok(
              subs
                .iter()
                .map(move |s| {
                  let r = s.required_scope();
                  let requirements = r.split('+').map(ToString::to_string).collect::<Vec<_>>();

                  for req in requirements {
                    if !validation.scopes.as_ref().unwrap().contains(&req) {
                      return 0;
                    }
                  }

                  1
                })
                .sum::<usize>()
                == subs.len(),
            );
          }
        }
        Err(e) => Err(EventSubError::AuthorisationError(e.to_string())),
      }
    } else {
      Ok(false)
    }
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

  pub fn receive_messages(&mut self) -> Vec<MessageType> {
    // check thread for new messages without waiting
    //
    // return new messages if any
    let mut messages = Vec::new();

    if let Ok(message) = self.messages_received.recv_timeout(Duration::ZERO) {
      messages.push(message);
    }

    messages
  }

  pub fn delete_message<S: Into<String>>(&self, message_id: S) {
    let broadcaster_account_id = self.twitch_keys.broadcaster_account_id.to_string();
    let moderator_account_id = broadcaster_account_id.to_owned();
    let access_token = self
      .twitch_keys
      .access_token
      .clone()
      .expect("No Access Token set")
      .get_token();
    let client_id = self.twitch_keys.client_id.to_string();

    TwitchApi::delete_message(
      broadcaster_account_id,
      moderator_account_id,
      message_id.into(),
      access_token,
      client_id,
    );
  }

  pub fn timeout_user<S: Into<String>, T: Into<String>>(
    &self,
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
    TwitchApi::timeout_user(
      access_token,
      client_id,
      broadcaster_account_id,
      moderator_account_id,
      user_id.into(),
      duration,
      reason.into(),
    );
  }

  pub fn send_chat_message<S: Into<String>>(&self, message: S) {
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

    TwitchApi::send_chat_message(
      message,
      access_token,
      client_id,
      broadcaster_account_id,
      Some(sender_id),
    );
  }

  pub fn wait_for_threads_to_close(self) {
    let _ = self.send_thread.unwrap().join();
    let _ = self.receive_thread.join();
  }

  fn event_sub_events(
    client: Arc<Mutex<Client<TlsStream<TcpStream>>>>,
    message_sender: SyncSender<MessageType>,
    subscriptions: Vec<SubscriptionPermission>,
    mut custom_subscriptions: Vec<String>,
    twitch_keys: TwitchKeys,
  ) {
    loop {
      let client = client.clone();
      let mut client = client.lock().unwrap();
      let message = match client.recv_message() {
        Ok(m) => m,
        Err(e) => {
          error!("recv message error: {:?}", e);
          let _ = client.send_message(&OwnedMessage::Close(None));
          message_sender.send(MessageType::Close).unwrap();

          return;
        }
      };

      if let OwnedMessage::Text(msg) = message.clone() {
        let message = serde_json::from_str(&msg);

        if let Err(e) = message {
          error!("Unimplemented twitch response: {}\n{}", msg, e);
          panic!("Unimplement Twitch Response {}\n{}", msg, e);
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
                .filter_map(Result::err)
                .for_each(|error| {
                  message_sender
                    .send(MessageType::SubscribeError(error))
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
          }
          EventMessageType::KeepAlive => {
            //println!("Keep alive receive message sent, !implemented");
          }
          EventMessageType::Notification => match message.subscription_type() {
            SubscriptionType::ChannelChatMessage => {
              let message_info = message.chat_message();

              message_sender
                .send(MessageType::Message(message_info))
                .unwrap();
            }
            SubscriptionType::CustomRedeem => {
              let custom_redeem = message.custom_redeem();

              message_sender
                .send(MessageType::CustomRedeem(custom_redeem))
                .unwrap();
            }
            SubscriptionType::AdBreakBegin => {
              let ad_break_duration = message.get_ad_duration();

              message_sender
                .send(MessageType::AdBreakNotification(ad_break_duration))
                .unwrap();
            }
            _ => {}
          },
          EventMessageType::Unknown => {
            if !custom_subscriptions.is_empty() {
              message_sender
                .send(MessageType::CustomSubscriptionResponse(msg))
                .unwrap();
            }
          }
          _ => {}
        }
        //      }
      }

      match message {
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
        // Say what we received
        _ => {
          // Already covered MessageType text
        }
      }
    }
  }
}
