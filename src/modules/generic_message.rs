use serde_derive::{Deserialize, Serialize};
//use serde_json::Result;

use crate::{
  modules::messages::{MessageData, RaidInfo},
  EventSubError, SubscriptionPermission, Token,
};

#[derive(Deserialize, Debug, Clone)]
pub struct NewAccessTokenResponse {
  pub access_token: String,
  pub expires_in: u32,
  token_type: String,
  pub refresh_token: Option<String>,
  scope: Option<Vec<String>>,
}

impl NewAccessTokenResponse {
  pub fn get_token_from_data(raw_data: &str) -> Result<Token, EventSubError> {
    serde_json::from_str::<NewAccessTokenResponse>(raw_data)
      .map(|validation| {
        Token::new_user_token(
          validation.access_token,
          validation.refresh_token.unwrap(),
          validation.expires_in as f32,
        )
      })
      .map_err(|e| EventSubError::AuthorisationError(e.to_string()))
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Validation {
  client_id: Option<String>,
  login: Option<String>,
  pub scopes: Option<Vec<String>>,
  user_id: Option<String>,
  pub expires_in: Option<u32>,
  pub status: Option<u32>,
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
pub struct TimeoutRequestData {
  pub user_id: String,
  pub duration: u32,
  pub reason: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SendTimeoutRequest {
  pub data: TimeoutRequestData,
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
  pub keepalive_timeout_seconds: Option<u32>,
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
  pub event: Option<Event>,
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
pub struct CheerMote {
  prefix: String,
  bits: u32,
  tier: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Fragments {
  #[serde(rename = "type")]
  kind: String,
  text: String,
  cheermote: Option<CheerMote>,
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
      if !fragment.is_mention() {
        if let Some(ref mut text) = text {
          *text = format!("{} {}", text, fragment.text());
        } else {
          text = Some(fragment.text().to_string().trim().to_string());
        }
      }
    }
    text
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Reward {
  pub id: String,
  pub title: String,
  pub prompt: String,
  pub cost: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Cheer {
  bits: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
  broadcaster_user_id: Option<String>,
  broadcaster_user_login: Option<String>,
  broadcaster_user_name: Option<String>,
  from_broadcaster_user_id: Option<String>,
  from_broadcaster_user_login: Option<String>,
  from_broadcaster_user_name: Option<String>,
  to_broadcaster_user_id: Option<String>,
  to_broadcaster_user_login: Option<String>,
  to_broadcaster_user_name: Option<String>,
  chatter_user_id: Option<String>,
  chatter_user_login: Option<String>,
  chatter_user_name: Option<String>,
  id: Option<String>,
  user_id: Option<String>,
  user_login: Option<String>,
  user_name: Option<String>,
  requester_user_id: Option<String>,
  requester_user_login: Option<String>,
  requester_user_name: Option<String>,
  user_input: Option<String>,
  status: Option<String>,
  redeemed_at: Option<String>,
  message_id: Option<String>,
  message: Option<Message>,
  viewers: Option<u32>,
  color: Option<String>,
  badges: Option<Vec<Badge>>,
  message_type: Option<String>,
  cheer: Option<Cheer>,
  reply: Option<Reply>,
  reward: Option<Reward>,
  channel_points_custom_reward_id: Option<String>,
  channel_points_animation_id: Option<String>,
  is_automatic: Option<bool>,
  started_at: Option<String>,
  duration_seconds: Option<u32>,
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
  ChannelRaid,
  ChannelChatMessage,
  CustomRedeem,
  AdBreakBegin,
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
    let chat_message = &SubscriptionPermission::ChatMessage.tag();
    let channel_raid = &SubscriptionPermission::ChannelRaid.tag();
    let custom_redeem = &SubscriptionPermission::CustomRedeem.tag();
    let ad_break_bagin = &SubscriptionPermission::AdBreakBegin.tag();

    match t {
      t if t == chat_message => SubscriptionType::ChannelChatMessage,
      t if t == custom_redeem => SubscriptionType::CustomRedeem,
      t if t == ad_break_bagin => SubscriptionType::AdBreakBegin,
      t if t == channel_raid => SubscriptionType::ChannelRaid,
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

  pub fn chat_message(&self) -> MessageData {
    let payload = self.payload.clone().unwrap();
    let event = payload.clone().event.unwrap();

    MessageData {
      message_id: event.message_id.unwrap().to_owned(),
      user_id: event.chatter_user_id.unwrap(),
      message: event.message.unwrap().get_written_message().unwrap_or(
        self
          .payload
          .clone()
          .unwrap()
          .event
          .unwrap()
          .message
          .unwrap()
          .text,
      ),
      username: event.chatter_user_name.unwrap(),
    }
  }

  pub fn custom_redeem(&self) -> (String, String, Reward) {
    (
      self
        .payload
        .clone()
        .unwrap()
        .event
        .unwrap()
        .user_name
        .unwrap(),
      self
        .payload
        .clone()
        .unwrap()
        .event
        .unwrap()
        .user_input
        .unwrap(),
      self.payload.clone().unwrap().event.unwrap().reward.unwrap(),
    )
  }

  pub fn get_raid_info(&self) -> RaidInfo {
    let payload = self.payload.clone().unwrap();
    let event = payload.event.clone().unwrap();

    RaidInfo {
      raider_user_id: event.from_broadcaster_user_id.unwrap(),
      raider_username: event.from_broadcaster_user_name.unwrap(),
      viewers: event.viewers.unwrap(),
    }
  }

  pub fn get_ad_duration(&self) -> u32 {
    self
      .payload
      .clone()
      .unwrap()
      .event
      .unwrap()
      .duration_seconds
      .unwrap()
  }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Condition {
  pub user_id: Option<String>,
  pub moderator_user_id: Option<String>,
  pub broadcaster_user_id: Option<String>,
  pub reward_id: Option<String>,
  pub from_broadcaster_user_id: Option<String>,
  pub to_broadcaster_user_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventSubscription {
  #[serde(rename = "type")]
  pub kind: String,
  pub version: String,
  pub condition: Condition,
  pub transport: Transport,
}
