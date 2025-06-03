use std::sync::mpsc::Sender as SyncSender;
use std::{net::TcpStream, thread, time::Duration};

#[cfg(feature = "logging")]
use log::{error, info, warn};
use tungstenite::{
  connect, error::ProtocolError, stream::MaybeTlsStream, Error, Message as NetworkMessage,
  WebSocket,
};

pub const PRIV_MESSAGE: &str = "PRIVMSG";
pub const PASS: &str = "PASS";
pub const NICK: &str = "NICK";
pub const JOIN: &str = "JOIN";

pub const IRC_URL: &str = "ws://irc-ws.chat.twitch.tv:80";

pub enum IRCResponse {
  IRCMessage(IRCMessage),
  //  Error(String),
}

pub struct IRCChat {
  pub client: WebSocket<MaybeTlsStream<TcpStream>>,
  pub joined_channel: Option<String>,
  pub user_token: String,
  pub user_login: String,
}

#[derive(Default)]
pub struct IRCMessage {
  pub display_name: String,
  pub moderator: bool,
  pub subscriber: bool,
  pub returning_chatter: bool,
  pub first_time_chatter: bool,
  pub message: String,
  pub colour: String,
}

impl From<&str> for IRCMessage {
  fn from(value: &str) -> Self {
    let elements = value.split(";").collect::<Vec<_>>();

    let mut message = IRCMessage::default();

    if let Some(irc_message) = elements.last() {
      message.message = irc_message.to_string();
    }

    for element in elements {
      if element.contains("first-msg") {
        if let Some(number_element) = element.split("=").last() {
          if let Ok(first_time) = number_element.parse::<u32>() {
            message.first_time_chatter = first_time == 1;
          }
        }
      }

      if element.contains("mod=") {
        if let Some(number_element) = element.split("=").last() {
          if let Ok(moderator) = number_element.parse::<u32>() {
            message.moderator = moderator == 1;
          }
        }
      }

      if element.contains("display-name") {
        if let Some(display_name) = element.split("=").last() {
          message.display_name = display_name.to_owned();
        }
      }

      if element.contains("returning-chatter") {
        if let Some(number_element) = element.split("=").last() {
          if let Ok(returning_chatter) = number_element.parse::<u32>() {
            message.returning_chatter = returning_chatter == 1;
          }
        }
      }

      if element.contains("subscriber=") {
        if let Some(number_element) = element.split("=").last() {
          if let Ok(subscriber) = number_element.parse::<u32>() {
            message.subscriber = subscriber == 1;
          }
        }
      }

      if element.contains("color") {
        if let Some(colour) = element.split("=").last() {
          message.colour = colour.to_owned();
        }
      }
    }

    message
  }
}

impl IRCChat {
  pub fn new(user_login: &str, user_token: &str) -> IRCChat {
    println!("irc name: {}, oauth token: {}", user_login, user_token);

    let (mut irc_client, _) = connect(IRC_URL).unwrap();

    let _ = irc_client.send(NetworkMessage::text(
      "CAP REQ :twitch.tv/membership twitch.tv/tags twitch.tv/commands",
    ));

    let _ = irc_client.send(NetworkMessage::text(format!(
      "{} oauth:{}",
      PASS, &user_token
    )));
    let _ = irc_client.send(NetworkMessage::text(format!("{} {}", NICK, &user_login)));

    let mut welcome_recieved = false;
    while !welcome_recieved {
      if let Ok(NetworkMessage::Text(text)) = &irc_client.read() {
        welcome_recieved = text.as_str().contains("Welcome");
      }
    }

    IRCChat {
      client: irc_client,
      joined_channel: None,
      user_token: user_token.to_owned(),
      user_login: user_login.to_owned(),
    }
  }

  pub fn join_channel<S: Into<String>>(&mut self, channel_name: S) {
    let channel = channel_name.into();
    let _ = self.client.send(NetworkMessage::text(format!(
      "{} #{}",
      JOIN,
      channel.to_owned()
    )));

    self.joined_channel = Some(channel);
  }

  pub fn recv_message(&mut self) -> Option<IRCMessage> {
    match self.client.read() {
      Ok(message) => match message {
        NetworkMessage::Text(text) => {
          if text.as_str().contains("PING :tmi.twitch.tv") {
            #[cfg(feature = "logging")]
            info!("IRC: ping recieved, sending pong back");
            let _ = self
              .client
              .send(NetworkMessage::Pong(Vec::with_capacity(0).into()));
            let _ = self
              .client
              .send(NetworkMessage::Text("PONG :tmi.twitch.tv".into()));
            let _ = self.client.send(NetworkMessage::Text("PONG".into()));
            None
          } else {
            Some(IRCMessage::from(text.as_str()))
          }
        }
        NetworkMessage::Ping(data) => {
          #[cfg(feature = "logging")]
          info!("IRC: ping recieved, sending pong back");
          let _ = self.client.send(NetworkMessage::Pong(data));
          None
        }
        NetworkMessage::Close(_) => {
          #[cfg(feature = "logging")]
          error!("IRC: connection Closed");
          None
        }
        _ => None,
      },
      Err(Error::Protocol(ProtocolError::ResetWithoutClosingHandshake)) => {
        #[cfg(feature = "logging")]
        error!("Protocol error: Reset without closing handshake");
        #[cfg(feature = "logging")]
        warn!("Restarting IRC connection");
        let _ = self.client.close(None);

        #[cfg(feature = "logging")]
        warn!("Close connection requested");
        thread::sleep(Duration::from_secs(5));

        #[cfg(feature = "logging")]
        warn!("Attempting reconnect with IRC");
        *self = IRCChat::new(&self.user_login, &self.user_token);
        None
      }
      Err(e) => {
        #[cfg(feature = "logging")]
        error!("IRC: Error: {:?}", e);
        None
      }
    }
  }

  pub fn send_message<S: Into<String>>(&mut self, m: S) {
    if let Some(channel) = &self.joined_channel {
      let m = m.into();
      let _ = self.client.send(NetworkMessage::text(format!(
        "{} #{} :{}",
        PRIV_MESSAGE, channel, m
      )));
    }
  }
}

pub fn irc_thread(mut irc: IRCChat, message_sender: SyncSender<IRCResponse>) {
  loop {
    if let Some(irc_msg) = irc.recv_message() {
      let _ = message_sender.send(IRCResponse::IRCMessage(irc_msg));
    }
  }
}
