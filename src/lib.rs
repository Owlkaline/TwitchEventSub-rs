use curl::easy::{Easy, List};
use std::borrow::Borrow;
use std::net::TcpStream;
use std::sync::mpsc::{channel, Receiver as SyncReceiver, Sender as SyncSender};
use std::thread::{self, JoinHandle, Scope};
use std::time::Duration;
use websocket::futures::executor::NotifyHandle;

use std::sync::{Arc, Mutex};

use crate::modules::consts::*;
use open;
use std::io::{Read, Write};
use std::ops::Deref;
use websocket::client::ClientBuilder;
use websocket::sync::stream::Splittable;
use websocket::ws::dataframe::DataFrame;
use websocket::ws::Message;
use websocket::ws::Receiver;
use websocket::{
  receiver::Receiver as StructReceiver,
  sender::Sender,
  stream::sync::AsTcpStream,
  sync::{Client, Reader, Writer},
  Message as StructMessage, OwnedMessage,
};

use std::net::TcpListener;

use serde_derive::{Deserialize, Serialize};
use websocket::stream::sync::TlsStream;

mod modules;

use crate::modules::generic_message::*;

#[derive(PartialEq)]
pub enum RequestType {
  Post(String),
}

impl RequestType {
  pub fn apply(&self, handle: &mut Easy) {
    match self {
      RequestType::Post(data) => {
        handle.post(true).unwrap();
        handle.post_fields_copy(data.as_bytes()).unwrap();
      }
    }
  }
}

#[derive(Clone)]
pub struct TwitchKeys {
  oauth: String,
  scoped_oauth: String,
  client_id: String,
  client_secret: String,
  app_token: String,
  broadcaster_account_id: String,
  sender_account_id: Option<String>,
}

impl TwitchKeys {
  pub fn from_secrets_env() -> TwitchKeys {
    simple_env_load::load_env_from([".example.env", ".secrets.env"]);

    fn get(key: &str) -> Result<String, String> {
      std::env::var(key).map_err(|_| format!("please set {key} in .example.env"))
    }

    let oauth = get("TWITCH_OAUTH_TOKEN").unwrap();
    let client_id = get("TWITCH_CLIENT_ID").unwrap();
    let client_secret: String = get("TWITCH_CLIENT_SECRET").unwrap();
    let broadcaster_id = get("TWITCH_BROADCASTER_ID").unwrap();
    let bot_account_id = get("TWITCH_BOT_ID").unwrap();
    let temp_app_token = get("TWITCH_TEMP_APP_TOKEN").unwrap_or_else(|e| {
      println!("{}", e);
      "".into()
    });
    let scoped_oauth = get("TWITCH_OAUTH_SCOPED_TOKEN").unwrap_or_else(|e| {
      println!("{}", e);
      "".into()
    });

    TwitchKeys {
      oauth,
      scoped_oauth,
      client_id,
      client_secret,
      app_token: temp_app_token,
      broadcaster_account_id: broadcaster_id,
      sender_account_id: Some(bot_account_id),
    }
  }
}

pub struct TwitchHttp {
  url: String,
  header: Vec<String>,
  request_type: Option<RequestType>,
}

impl TwitchHttp {
  pub fn new<S: Into<String>>(url: S) -> TwitchHttp {
    TwitchHttp {
      url: url.into(),
      header: Vec::new(),
      request_type: None,
    }
  }

  #[must_use]
  pub fn auth<S: Into<String>, T: Into<String>>(
    mut self,
    scoped_oauth: S,
    client_id: T,
  ) -> TwitchHttp {
    self
      .header
      .push(format!("Authorization: Bearer {}", scoped_oauth.into()));
    self.header.push(format!("Client-Id: {}", client_id.into()));

    self
  }

  #[must_use]
  pub fn json_content(mut self) -> TwitchHttp {
    self.header.push(format!("Content-Type: application/json"));

    self
  }

  #[must_use]
  pub fn is_post<S: Into<String>>(mut self, data: S) -> TwitchHttp {
    self.request_type = Some(RequestType::Post(data.into()));
    self
  }

  pub fn run(&self) {
    let mut handle = Easy::new();

    handle.url(&self.url).unwrap();
    if let Some(request) = &self.request_type {
      request.apply(&mut handle);
    }

    let mut headers = List::new();
    for header in &self.header {
      headers.append(header).unwrap();
    }

    handle.http_headers(headers).unwrap();

    // getting data back
    // idk why its called write function
    // that silly
    // we are reading whats coming back
    let _ = handle.write_function(|new_data| {
      println!(
        "HTTP REQUEST RESPONSE: {}",
        String::from_utf8_lossy(&new_data)
      );
      Ok(new_data.len())
    });

    if let Err(e) = handle.perform() {
      println!("Error: {:?}", e);
    }
  }
}

#[must_use]
pub struct TwitchEventSubApiBuilder {
  twitch_keys: TwitchKeys,
  subscriptions: Vec<Subscription>,
  local_server_ip: Option<String>,
}

impl TwitchEventSubApiBuilder {
  pub fn new(tk: TwitchKeys) -> TwitchEventSubApiBuilder {
    TwitchEventSubApiBuilder {
      twitch_keys: tk,
      subscriptions: Vec::new(),
      local_server_ip: None,
    }
  }

  pub fn set_local_authentication_server<S: Into<String>>(
    mut self,
    ip: S,
  ) -> TwitchEventSubApiBuilder {
    self.local_server_ip = Some(ip.into());
    self
  }

  pub fn add_subscription(mut self, sub: Subscription) -> TwitchEventSubApiBuilder {
    self.subscriptions.push(sub.into());
    self
  }

  pub fn build(self) -> Option<TwitchEventSubApi> {
    if self.subscriptions.len() == 0 {
      None
    } else {
      Some(TwitchEventSubApi::new(
        self.twitch_keys,
        self.subscriptions,
        self.local_server_ip,
      ))
    }
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
    subscriptions: Vec<String>,
    local_server_ip: Option<String>,
    // oauth: string,
    // scoped_oauth: string,
    // client_id: string,
    // client_secret: string,
    // app_token: string,
    // broadcaster_account_id: string,
    // sender_account_id: Option<String>,
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
      TwitchEventSubApi::event_sub_events(receiver, transmit_messages, keys_clone)
    });

    TwitchEventSubApi {
      tx: None,
      receive_thread,
      send_thread: None,
      messages_received: receive_message,
      twitch_keys,
    }
  }

  pub fn web_browser_authorisation<S: Into<String>>(
    twitch_keys: TwitchKeys,
    oauth_redirect_url: S,
  ) {
    let oauth_redirect_url = oauth_redirect_url.into();

    let twitch_broswer_url = format!("{}authorize?response_type=token&client_id={}&client_secret={}&redirect_uri={}&scope=user:read:chat+user:write:chat", TWITCH_AUTHORISE_URL, twitch_keys.client_id, twitch_keys.client_secret, oauth_redirect_url);
    open::that(twitch_broswer_url).unwrap();

    let listener = TcpListener::bind(&oauth_redirect_url).unwrap();

    // accept connections and process them serially
    if let Ok((mut stream, b)) = listener.accept() {
      let mut a = String::new();
      stream.read_to_string(&mut a).unwrap();
      println!("{}", a);
      // TODO: Actually get code in the body.
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

  pub fn send_chat_message(&self, message: MessageType) {
    if let MessageType::ChannelMessage(ref msg) = message {
      TwitchHttp::new(SEND_MESSAGE_URL)
        .json_content()
        .auth(
          self.twitch_keys.scoped_oauth.to_string(),
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
            message: msg.to_string(),
          })
          .unwrap(),
        )
        .run();
    }
  }

  pub fn wait_for_threads_to_close(self) {
    let _ = self.send_thread.unwrap().join();
    let _ = self.receive_thread.join();
  }

  fn event_sub_events(
    client: Arc<Mutex<Client<TlsStream<TcpStream>>>>,
    message_sender: SyncSender<MessageType>,
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
        println!("{}", msg);
        match message.event_type() {
          EventMessageType::Welcome => {
            let session_id = message.clone().payload.unwrap().session.unwrap().id;
            println!("Session id is: {}", session_id);

            let event_subscription = serde_json::to_string(&EventSubscription::chat_message(
              twitch_keys.broadcaster_account_id.to_string(),
              session_id.to_string(),
            ))
            .unwrap();
            println!("event_subscription: {}", event_subscription);

            TwitchHttp::new(SUBSCRIBE_URL)
              .auth(
                twitch_keys.scoped_oauth.clone(),
                twitch_keys.client_id.to_string(),
              )
              .json_content()
              .is_post(event_subscription)
              .run();
          }
          EventMessageType::KeepAlive => {
            println!("Keep alive recieve message sent, !implemented");
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
          _ => {}
        }
        //      }
      }

      match message {
        OwnedMessage::Close(a) => {
          println!("Close message recieved: {:?}", a);
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
  Capability,
  Password(String),
  Username(String),
  JoinChannel(String),
  JoinMessage(String),
  Message((String, String)),
  ChannelMessage(String),
  Ping,
  Pong,
  Close,
}
#[cfg(test)]
mod tests {
  use super::*;

  // #[test]
  // fn token_missing_requested_subscription() {
  //   if let Some(mut api) = TwitchEventSubApiBuilder::new(TwitchKeys::from_secrets_env())
  //     .set_local_authentication_server("localhost:3000".into())
  //     .add_subscription("user_update".into())
  //     .build()
  //   {
  //     loop {
  //       let mut force_break = false;
  //       for msg in api.receive_messages() {
  //         assert_eq!(msg, MessageType::Close);
  //         force_break = true;
  //         break;
  //       }
  //       if force_break {
  //         break;
  //       }
  //     }
  //   } else {
  //     assert!(false, "Failed to create TwitchApi from builder");
  //   }
  // }

  #[test]
  fn token_has_subscription() {
    if let Some(mut api) = TwitchEventSubApiBuilder::new(TwitchKeys::from_secrets_env())
      .set_local_authentication_server("localhost:3000")
      .add_subscription("chat_message")
      .build()
    {
      // users program main loop simulation
      'test_loop: loop {
        // non block for loop of messages
        for msg in api.receive_messages() {
          break 'test_loop;
          // assert_ne!(MessageType::Close, msg);
          // force_break = true;
          // break;
        }
      }
    } else {
      assert!(false, "Failed to create TwitchApi from builder");
    }
  }
}
