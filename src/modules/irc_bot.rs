use std::net::TcpStream;

use log::{error, info};
use tungstenite::{connect, stream::MaybeTlsStream, Message as NetworkMessage, WebSocket};
use twitch_eventsub_structs::Event;

use std::sync::mpsc::Sender as SyncSender;

use crate::ResponseType;

pub const PRIV_MESSAGE: &str = "PRIVMSG";
pub const PASS: &str = "PASS";
pub const NICK: &str = "NICK";
pub const JOIN: &str = "JOIN";

pub const IRC_URL: &str = "ws://irc-ws.chat.twitch.tv:80";

pub enum IRCResponse {
  IRCMessage(IRCMessage),
  Error(String),
}

pub struct IRCChat {
  pub client: WebSocket<MaybeTlsStream<TcpStream>>,
  pub joined_channel: Option<String>,
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

impl From<String> for IRCMessage {
  fn from(value: String) -> Self {
    let elements = value.split(";").collect::<Vec<_>>();

    let mut message = IRCMessage::default();

    message.message = elements.last().unwrap().to_string();
    for element in elements {
      if element.contains("first-msg") {
        message.first_time_chatter =
          element.split("=").last().unwrap().parse::<u32>().unwrap() == 1;
      }

      if element.contains("mod=") {
        message.moderator = element.split("=").last().unwrap().parse::<u32>().unwrap() == 1;
      }

      if element.contains("display-name") {
        message.display_name = element.split("=").last().unwrap().to_owned();
      }

      if element.contains("returning-chatter") {
        message.returning_chatter = element.split("=").last().unwrap().parse::<u32>().unwrap() == 1;
      }

      if element.contains("subscriber=") {
        message.subscriber = element.split("=").last().unwrap().parse::<u32>().unwrap() == 1;
      }

      if element.contains("color") {
        message.colour = element.split("=").last().unwrap().to_owned();
      }
    }

    message
  }
}

impl IRCChat {
  pub fn new<T: Into<String>, S: Into<String>>(bot_name: T, oauth_token: S) -> IRCChat {
    let (mut irc_client, _) = connect(IRC_URL).unwrap();

    let _ = irc_client.send(NetworkMessage::text(
      "CAP REQ :twitch.tv/membership twitch.tv/tags twitch.tv/commands",
    ));

    let _ = irc_client.send(NetworkMessage::text(format!(
      "{} oauth:{}",
      PASS,
      oauth_token.into()
    )));
    let _ = irc_client.send(NetworkMessage::text(format!(
      "{} {}",
      NICK,
      bot_name.into()
    )));

    let mut welcome_recieved = false;
    while !welcome_recieved {
      if let Ok(message) = &irc_client.read() {
        if let NetworkMessage::Text(text) = message {
          welcome_recieved = text.contains("Welcome");
        }
      }
    }

    IRCChat {
      client: irc_client,
      joined_channel: None,
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
          return Some(IRCMessage::from(text));
        }
        NetworkMessage::Ping(data) => {
          println!("IRC: ping recieved, sending pong back");
          info!("IRC: ping recieved, sending pong back");
          let _ = self.client.send(NetworkMessage::Pong(data));
          None
        }
        NetworkMessage::Close(_) => {
          println!("IRC: connection Closed");
          error!("IRC: connection Closed");
          None
        }
        _ => None,
      },
      Err(e) => {
        println!("IRC: Error: {:?}", e);
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
