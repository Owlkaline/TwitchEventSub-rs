use std::net::TcpStream;

use tungstenite::{connect, stream::MaybeTlsStream, Message as NetworkMessage, WebSocket};

pub const PRIV_MESSAGE: &str = "PRIVMSG";
pub const PASS: &str = "PASS";
pub const NICK: &str = "NICK";
pub const JOIN: &str = "JOIN";

pub const IRC_URL: &str = "ws://irc-ws.chat.twitch.tv:80";

pub struct IRCChat(pub WebSocket<MaybeTlsStream<TcpStream>>);

impl IRCChat {
  pub fn new<T: Into<String>, S: Into<String>>(bot_name: T, oauth_token: S) -> IRCChat {
    let (mut irc_client, _) = connect(IRC_URL).unwrap();

    let r = irc_client.send(NetworkMessage::text(
      "CAP REQ :twitch.tv/membership twitch.tv/tags twitch.tv/commands",
    ));
    println!("{:?}", r);

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
        println!("{:?}", message);
        if let NetworkMessage::Text(text) = message {
          welcome_recieved = text.contains("welcome");
        }
      }
    }

    let _ = irc_client.send(NetworkMessage::text(format!("{} Owlkalinevt", JOIN)));

    IRCChat(irc_client)
  }

  pub fn send_message<S: Into<String>>(&mut self, m: S) {
    let _ = self.0.send(NetworkMessage::text(format!(
      "{} {}",
      PRIV_MESSAGE,
      m.into()
    )));
  }
}
