use std::net::TcpStream;

use websocket::{
  client::ClientBuilder,
  sync::{Reader, Writer},
  Message, OwnedMessage,
};

pub const PRIV_MESSAGE: &str = "PRIVMSG";
pub const PASS: &str = "PASS";
pub const NICK: &str = "NICK";
pub const JOIN: &str = "JOIN";

pub const IRC_URL: &str = "ws://irc-ws.chat.twitch.tv:80";

pub struct IRCChat {
  sender: Writer<TcpStream>,
  reciever: Reader<TcpStream>,
}

impl IRCChat {
  pub fn new<T: Into<String>, S: Into<String>>(bot_name: T, oauth_token: S) -> IRCChat {
    let client = ClientBuilder::new(IRC_URL)
      .unwrap()
      .connect_insecure()
      .unwrap();

    let (mut reciever, mut sender) = client.split().unwrap();
    let _ = sender.send_message(&Message::text(
      "CAP; REQ :twitch.tv/membership twitch.tv/tags twitch.tv/commands",
    ));

    let _ = sender.send_message(&Message::text(format!(
      "{} oauth:{}",
      PASS,
      oauth_token.into()
    )));
    let _ = sender.send_message(&Message::text(format!("{} {}", NICK, bot_name.into())));

    let mut welcome_recieved = false;
    while !welcome_recieved {
      if let Ok(message) = &reciever.recv_message() {
        println!("{:?}", message);
        if let OwnedMessage::Text(text) = message {
          welcome_recieved = text.contains("welcome");
        }
      }
    }

    let _ = sender.send_message(&Message::text(format!("{} Owlkalinevt", JOIN)));

    IRCChat { reciever, sender }
  }

  pub fn send_message<S: Into<String>>(&mut self, m: S) {
    let _ = self
      .sender
      .send_message(&Message::text(format!("{} {}", PRIV_MESSAGE, m.into())));
  }
}
