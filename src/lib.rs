use std::io::Write;

use std::fs;
use std::net::TcpStream;
use std::sync::mpsc::{channel, Receiver as SyncReceiver, Sender as SyncSender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

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

pub use crate::modules::{
  generic_message::SubscriptionPermission,
  twitch_http::{AuthType, RequestType, TwitchHttp},
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
  pub fn from_secrets_env() -> TwitchKeys {
    simple_env_load::load_env_from([".example.env", ".secrets.env"]);

    fn get(key: &str) -> Result<String, String> {
      std::env::var(key).map_err(|_| format!("please set {key} in .example.env"))
    }

    let client_id = get("TWITCH_CLIENT_ID").unwrap();
    let client_secret: String = get("TWITCH_CLIENT_SECRET").unwrap();
    let broadcaster_id = get("TWITCH_BROADCASTER_ID").unwrap();
    let bot_account_id = get("TWITCH_BOT_ID").unwrap_or(broadcaster_id.to_owned());

    let user_access_token = get("TWITCH_USER_ACCESS_TOKEN").ok().map(TokenAccess::User);

    TwitchKeys {
      authorisation_code: None,
      access_token: user_access_token,
      refresh_token: None,
      client_id,
      client_secret,

      broadcaster_account_id: broadcaster_id,
      sender_account_id: Some(bot_account_id),
    }
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
  save_created_tokens: Option<String>,
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
      save_created_tokens: None,
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

  pub fn save_created_tokens<S: Into<String>>(mut self, file: S) -> TwitchEventSubApiBuilder {
    self.save_created_tokens = Some(file.into());
    self
  }

  pub fn subscriptions(&self) -> Vec<SubscriptionPermission> {
    self.subscriptions.clone()
  }

  pub fn set_keys(&mut self, keys: TwitchKeys) {
    self.twitch_keys = keys;
  }

  pub fn build(mut self) -> Result<TwitchEventSubApi, EventSubError> {
    if self.subscriptions.is_empty() {
      return Err(EventSubError::NoSubscriptionsRequested);
    }

    if (self.generate_token_if_none || self.generate_token_on_scope_error)
      && self.redirect_url.is_none()
    {
      panic!("Redirect url was not set, when a generate token setting was enabled.");
    }

    if self.twitch_keys.access_token.is_none() {
      if self.generate_token_if_none {
        // Returns app access token
        match TwitchEventSubApi::generate_user_token(
          self.twitch_keys.client_id.to_owned(),
          self.twitch_keys.client_secret.to_owned(),
          self.redirect_url.clone().unwrap(),
          &self.subscriptions,
        ) {
          Ok(user_token) => {
            if let Some(ref file) = self.save_created_tokens {
              if let Ok(mut writer) = fs::OpenOptions::new().append(true).create(true).open(file) {
                if let Err(e) = writer.write(format!("{}\n", user_token.get_token()).as_bytes()) {
                  return Err(EventSubError::WriteError(e.to_string()));
                }
              }
            }

            self.twitch_keys.access_token = Some(user_token);
          }
          Err(e) => {
            return Err(e);
          }
        }
      } else {
        return Err(EventSubError::NoAccessTokenProvided);
      }
    }

    match TwitchEventSubApi::check_token_meets_requirements(
      self.twitch_keys.access_token.clone().unwrap(),
      &self.subscriptions,
    ) {
      Ok(is_valid) => {
        if !is_valid {
          if self.generate_token_on_scope_error {
            match TwitchEventSubApi::generate_user_token(
              self.twitch_keys.client_id.to_owned(),
              self.twitch_keys.client_secret.to_owned(),
              self.redirect_url.clone().unwrap(),
              &self.subscriptions,
            ) {
              Ok(user_token) => {
                if let Some(ref file) = self.save_created_tokens {
                  if let Ok(mut writer) =
                    fs::OpenOptions::new().append(true).create(true).open(file)
                  {
                    if let Err(e) = writer.write(format!("{}\n", user_token.get_token()).as_bytes())
                    {
                      return Err(EventSubError::WriteError(e.to_string()));
                    }
                  }
                }

                self.twitch_keys.access_token = Some(user_token);
              }
              Err(e) => {
                return Err(e);
              }
            }
          } else {
            return Err(EventSubError::TokenMissingScope);
          }
        }
      }
      Err(e) => {
        return Err(e);
      }
    }

    Ok(TwitchEventSubApi::new(
      self.twitch_keys,
      self.subscriptions,
      Vec::new(),
    ))
  }
}

pub struct TwitchEventSubApi {
  tx: Option<SyncSender<GenericMessage>>,
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
  ) -> TwitchEventSubApi {
    let client = ClientBuilder::new(CONNECTION_EVENTS)
      .unwrap()
      .add_protocol("rust-websocket-events")
      .connect_secure(None)
      .unwrap();

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

    TwitchEventSubApi {
      tx: None,
      receive_thread,
      send_thread: None,
      messages_received: receive_message,
      twitch_keys,
    }
  }

  pub fn check_token_meets_requirements(
    access_token: TokenAccess,
    subs: &Vec<SubscriptionPermission>,
  ) -> Result<bool, EventSubError> {
    if let Ok(data) = TwitchHttp::new(VALIDATION_TOKEN_URL)
      .header_authorisation(access_token.get_token(), AuthType::OAuth)
      .run()
    {
      match serde_json::from_str::<Validation>(&data) {
        Ok(validation) => {
          if validation.is_error() {
            return Err(EventSubError::InvalidAccessToken(validation.error_msg()));
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

    let listener = TcpListener::bind(&redirect_url).expect("Failed to create tcp listener.");

    // accept connections and process them serially
    match listener.accept() {
      Ok((mut stream, b)) => {
        let mut http_output = String::new();
        stream
          .read_to_string(&mut http_output)
          .expect("Failed to read tcp stream.");
        Ok(http_output)
      }
      Err(e) => Err(EventSubError::UnhandledError(e.to_string())),
    }
  }

  pub fn generate_user_token<S: Into<String>, T: Into<String>, V: Into<String>>(
    client_id: S,
    client_secret: T,
    redirect_url: V,
    subscriptions: &Vec<SubscriptionPermission>,
  ) -> Result<TokenAccess, EventSubError> {
    let client_id = client_id.into();
    let client_secret = client_secret.into();
    let redirect_url = redirect_url.into();

    // Returns app access token
    let authorisation_code_result = TwitchEventSubApi::get_authorisation_code(
      client_id.to_owned(),
      redirect_url.to_owned(),
      &subscriptions,
    );

    match authorisation_code_result {
      Ok(authorisation_code) => {
        match TwitchEventSubApi::get_user_token_from_authorisation_code(
          client_id.to_owned(),
          client_secret.to_owned(),
          authorisation_code.to_owned(),
          redirect_url.to_owned(),
        ) {
          Ok(user_token) => {
            return Ok(TokenAccess::User(user_token));
          }
          Err(e) => {
            return Err(e);
          }
        }
      }
      Err(e) => Err(e),
    }
  }

  pub fn get_authorisation_code<S: Into<String>, T: Into<String>>(
    client_id: S,
    redirect_url: T,
    scopes: &Vec<SubscriptionPermission>,
  ) -> Result<String, EventSubError> {
    let redirect_url = redirect_url.into();

    let scope = &scopes
      .iter()
      .map(|s| s.required_scope())
      .collect::<Vec<String>>()
      .join("+");

    let get_authorisation_code_request = format!(
      "{}authorize?response_type=code&client_id={}&redirect_uri={}&scope={}",
      TWITCH_AUTHORISE_URL,
      client_id.into(),
      redirect_url.to_owned(),
      scope
    );

    match TwitchEventSubApi::open_browser(get_authorisation_code_request, redirect_url) {
      Ok(http_response) => {
        if http_response.contains("error") {
          Err(EventSubError::UnhandledError(format!("{}", http_response)))
        } else {
          let auth_code = http_response.split('&').collect::<Vec<_>>()[0]
            .split('=')
            .collect::<Vec<_>>()[1];
          Ok(auth_code.to_string())
        }
      }
      e => e,
    }
  }

  pub fn get_user_token_from_authorisation_code<
    S: Into<String>,
    T: Into<String>,
    V: Into<String>,
    W: Into<String>,
  >(
    client_id: S,
    client_secret: T,
    authorisation_code: V,
    redirect_url: W,
  ) -> Result<String, EventSubError> {
    let post_data = format!(
      "client_id={}&client_secret={}&code={}&grant_type=authorization_code&redirect_uri={}",
      client_id.into(),
      client_secret.into(),
      authorisation_code.into(),
      redirect_url.into()
    );

    match TwitchHttp::new(TWITCH_TOKEN_URL)
      .url_encoded_content()
      .is_post(post_data)
      .run()
    {
      Ok(response) => match serde_json::from_str::<NewAccessTokenResponse>(&response) {
        Ok(new_token_data) => Ok(new_token_data.access_token),
        _ => Err(EventSubError::AuthorisationError(response)),
      },
      e => e,
    }
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

  pub fn send_chat_message<S: Into<String>>(&self, message: S) {
    let _ = TwitchHttp::new(SEND_MESSAGE_URL)
      .json_content()
      .full_auth(
        self
          .twitch_keys
          .access_token
          .clone()
          .expect("No Access Token set")
          .get_token(),
        self.twitch_keys.client_id.to_string(),
      )
      .is_post(
        serde_json::to_string(&SendMessage {
          broadcaster_id: self.twitch_keys.broadcaster_account_id.to_string(),
          sender_id: self
            .twitch_keys
            .sender_account_id
            .clone()
            .unwrap_or(self.twitch_keys.broadcaster_account_id.to_string()),
          message: message.into(),
        })
        .unwrap(),
      )
      .run();
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
          println!("message match Receive Loop: {:?}", e);
          let _ = client.send_message(&OwnedMessage::Close(None));
          message_sender.send(MessageType::Close).unwrap();

          return;
        }
      };

      if let OwnedMessage::Text(msg) = message.clone() {
        let message: GenericMessage = serde_json::from_str(&msg).unwrap();
        match message.event_type() {
          EventMessageType::Welcome => {
            let session_id = message.clone().payload.unwrap().session.unwrap().id;
            println!("Session id is: {}", session_id);

            let mut sub_data = subscriptions
              .iter()
              .filter_map(|s| {
                serde_json::to_string(&s.construct_data(&session_id, &twitch_keys)).ok()
              })
              .collect::<Vec<_>>();
            sub_data.append(&mut custom_subscriptions);

            if let Some(TokenAccess::User(ref token)) = twitch_keys.access_token {
              sub_data
                .iter()
                .map(|sub_data| {
                  let a = TwitchHttp::new(SUBSCRIBE_URL)
                    .full_auth(token.to_owned(), twitch_keys.client_id.to_string())
                    .json_content()
                    .is_post(sub_data)
                    .run();
                  a
                })
                .filter_map(Result::err)
                .for_each(|error| {
                  todo!("handled error in text");
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
              let (username, msg) = message.chat_message();

              message_sender
                .send(MessageType::Message((username, msg)))
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
          println!("Close message received: {:?}", a);
          // Got a close message, so send a close message and return
          let _ = client.send_message(&OwnedMessage::Close(None));
          return;
        }
        OwnedMessage::Ping(_) => {
          match client.send_message(&OwnedMessage::Pong(Vec::new())) {
            // Send a pong in response
            Ok(()) => {}
            Err(e) => {
              println!("Received an Error from Server: {:?}", e);
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

#[derive(Debug, PartialEq)]
pub enum MessageType {
  Message((String, String)),
  ChannelMessage(String),
  CustomSubscriptionResponse(String),
  SubscribeError(EventSubError),
  Error(EventSubError),
  Close,
}
