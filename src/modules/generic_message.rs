use serde_derive::{Deserialize, Serialize};
use serde_json::Result;

use crate::TwitchKeys;

#[derive(Deserialize, Debug, Clone)]
pub struct NewAccessTokenResponse {
  pub access_token: String,
  expires_in: u32,
  token_type: String,
  refresh_token: Option<String>,
  scope: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Validation {
  client_id: Option<String>,
  login: Option<String>,
  pub scopes: Option<Vec<String>>,
  user_id: Option<String>,
  expires_in: Option<u32>,
  status: Option<u32>,
  message: Option<String>,
}

impl Validation {
  pub fn is_error(&self) -> bool {
    self.status.is_some()
  }

  pub fn error_msg(&self) -> String {
    if self.is_error() {
      format!(
        "status: {}, message: {}",
        self.status.unwrap(),
        self.message.clone().unwrap()
      )
    } else {
      panic!("Validation Error message requested, when it isnt a error!");
    }
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SendMessage {
  pub broadcaster_id: String,
  pub sender_id: String,
  pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transport {
  pub method: String,
  pub session_id: String,
}

impl Transport {
  pub fn new<S: Into<String>>(session_id: S) -> Transport {
    Transport {
      method: "websocket".to_string(),
      session_id: session_id.into(),
    }
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Session {
  pub id: String,
  pub status: String,
  pub connected_at: String,
  pub keepalive_timeout_seconds: u32,
  pub reconnect_url: Option<String>, // is null
  pub recovery_url: Option<String>,  // is null
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Subscription {
  pub id: String,
  pub status: Option<String>,
  #[serde(rename = "type")]
  pub kind: String,
  pub version: String,
  pub cost: i32,
  pub condition: Option<Condition>,
  pub transport: Transport,
  pub created_at: String,
  pub event: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Mention {
  user_id: String,
  user_login: String,
  user_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Emote {
  id: String,
  emote_set_id: String,
  owner_id: String,
  format: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Fragments {
  #[serde(rename = "type")]
  kind: String,
  text: String,
  cheer_mode: Option<String>,
  emote: Option<Emote>,
  mention: Option<Mention>,
}

impl Fragments {
  pub fn is_text(&self) -> bool {
    self.kind == "text"
  }

  pub fn is_mention(&self) -> bool {
    self.kind == "mention"
  }

  pub fn text(&self) -> String {
    self.text.to_string()
  }
}

impl Message {
  pub fn get_written_message(&self) -> Option<String> {
    let mut text = None;
    for fragment in &self.fragments {
      if fragment.is_text() {
        if let Some(ref mut text) = text {
          *text = format!("{} {}", text, fragment.text());
        } else {
          text = Some(fragment.text());
        }
      }
    }
    None
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
  text: String,
  fragments: Vec<Fragments>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Badge {
  set_id: String,
  id: String,
  info: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Reply {
  parent_message_id: String,
  parent_message_body: String,
  parent_user_id: String,
  parent_user_name: String,
  parent_user_login: String,
  thread_message_id: String,
  thread_user_id: String,
  thread_user_name: String,
  thread_user_login: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
  broadcaster_user_id: String,
  broadcaster_user_login: String,
  broadcaster_user_name: String,
  chatter_user_id: String,
  chatter_user_login: String,
  chatter_user_name: String,
  message_id: String,
  message: Message,
  color: String,
  badges: Vec<Badge>,
  message_type: String,
  cheer: Option<String>,
  reply: Option<Reply>,
  channel_points_custom_reward_id: Option<String>,
  channel_points_animation_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Payload {
  pub session: Option<Session>,
  pub subscription: Option<Subscription>,
  pub event: Option<Event>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MetaData {
  pub message_id: String,
  pub message_type: String,
  pub message_timestamp: String,
  pub subscription_type: Option<String>,
  pub subscription_version: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GenericMessage {
  pub metadata: MetaData,
  pub payload: Option<Payload>,
  pub subscription_type: Option<String>,
  pub subscription_version: Option<String>,
}

pub enum SubscriptionType {
  ChannelChatMessage,
  Unknown,
}

pub enum EventMessageType {
  Welcome,
  KeepAlive,
  Notification,
  Reconnect,
  Unknown,
}

impl SubscriptionType {
  pub fn from_string(t: &str) -> SubscriptionType {
    match t {
      "channel.chat.message" => SubscriptionType::ChannelChatMessage,
      _ => SubscriptionType::Unknown,
    }
  }
}

impl EventMessageType {
  pub fn from_string(t: &str) -> EventMessageType {
    match t {
      "session_welcome" => EventMessageType::Welcome,
      "session_keepalive" => EventMessageType::KeepAlive,
      "notification" => EventMessageType::Notification,
      _ => EventMessageType::Unknown,
    }
  }
}

impl GenericMessage {
  pub fn event_type(&self) -> EventMessageType {
    EventMessageType::from_string(&self.metadata.message_type)
  }

  pub fn subscription_type(&self) -> SubscriptionType {
    SubscriptionType::from_string(&self.metadata.subscription_type.clone().unwrap())
  }

  pub fn chat_message(&self) -> (String, String) {
    (
      self
        .payload
        .clone()
        .unwrap()
        .event
        .unwrap()
        .chatter_user_name,
      self
        .payload
        .clone()
        .unwrap()
        .event
        .unwrap()
        .message
        .get_written_message()
        .unwrap_or(self.payload.clone().unwrap().event.unwrap().message.text),
    )
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Condition {
  user_id: Option<String>,
  moderator_user_id: Option<String>,
  broadcaster_user_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventSubscription {
  #[serde(rename = "type")]
  kind: String,
  version: String,
  condition: Condition,
  transport: Transport,
}

#[derive(Clone)]
pub enum SubscriptionPermission {
  UserUpdate,
  ChannelFollow,
  ChatMessage,
  CustomRedeem,
}

impl SubscriptionPermission {
  pub fn tag(&self) -> String {
    match self {
      SubscriptionPermission::UserUpdate => "user.update",
      SubscriptionPermission::ChannelFollow => "channel.follow",
      SubscriptionPermission::ChatMessage => "channel.chat.message",
      SubscriptionPermission::CustomRedeem => "channel.channel_points_custom_reward_redemption.add",
    }
    .to_string()
  }

  pub fn required_scope(&self) -> String {
    match self {
      SubscriptionPermission::UserUpdate => "",
      SubscriptionPermission::ChannelFollow => "moderator:read:followers",
      SubscriptionPermission::ChatMessage => "user:read:chat+user:write:chat",
      SubscriptionPermission::CustomRedeem => "channel:read:redemptions",
    }
    .to_string()
  }

  pub fn construct_data(&self, session_id: &str, twitch_keys: &TwitchKeys) -> EventSubscription {
    let transport = Transport::new(session_id);
    match self {
      SubscriptionPermission::UserUpdate => EventSubscription {
        kind: self.tag(),
        version: "1".to_string(),
        condition: Condition {
          user_id: Some(twitch_keys.broadcaster_account_id.to_owned()),
          moderator_user_id: None,
          broadcaster_user_id: None,
        },
        transport,
      },
      SubscriptionPermission::ChannelFollow => EventSubscription {
        kind: self.tag(),
        version: "2".to_string(),
        condition: Condition {
          broadcaster_user_id: Some(twitch_keys.broadcaster_account_id.to_owned()),
          moderator_user_id: Some(twitch_keys.broadcaster_account_id.to_owned()),
          user_id: Some(twitch_keys.broadcaster_account_id.to_owned()),
        },
        transport,
      },
      SubscriptionPermission::ChatMessage => EventSubscription {
        kind: self.tag(),
        version: "1".to_string(),
        condition: Condition {
          broadcaster_user_id: Some(twitch_keys.broadcaster_account_id.to_owned()),
          moderator_user_id: None,
          user_id: Some(twitch_keys.broadcaster_account_id.to_owned()),
        },
        transport,
      },
      SubscriptionPermission::CustomRedeem => EventSubscription {
        kind: self.tag(),
        version: "1".to_string(),
        condition: Condition {
          user_id: None,
          moderator_user_id: None,
          broadcaster_user_id: Some(twitch_keys.broadcaster_account_id.to_owned()),
        },
        transport,
      },
    }
  }
}
