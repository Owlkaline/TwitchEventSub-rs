use crate::{Deserialise, Serialise};

use crate::*;

#[cfg(feature = "bevy")]
use bevy_ecs::prelude::Event as BevyEvent;

#[derive(Deserialise)]
pub struct NewAccessTokenResponse {
  pub access_token: String,
  pub expires_in: u32,
  pub token_type: String,
  pub refresh_token: Option<String>,
  pub scope: Option<Vec<String>>,
}

#[derive(Serialise, Deserialise, Debug, Clone)]
pub struct Validation {
  client_id: Option<String>,
  login: Option<String>,
  pub scopes: Option<Vec<String>>,
  user_id: Option<String>,
  pub expires_in: Option<u32>,
  pub status: Option<u32>,
  pub message: Option<String>,
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

#[derive(Serialise, Deserialise, Debug, Clone)]
pub struct TimeoutRequestData {
  pub user_id: String,
  pub duration: u32,
  pub reason: String,
}

#[derive(Serialise, Deserialise, Debug, Clone)]
pub struct SendTimeoutRequest {
  pub data: TimeoutRequestData,
}

#[derive(Serialise, Deserialise, Debug, Clone)]
pub struct SendMessage {
  pub broadcaster_id: String,
  pub sender_id: String,
  pub message: String,
  pub reply_parent_message_id: Option<String>,
}

#[derive(Serialise, Deserialise, Debug, Clone, PartialEq)]
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

#[derive(Serialise, Deserialise, Debug, Clone)]
pub struct Session {
  pub id: String,
  pub status: String,
  pub connected_at: String,
  pub keepalive_timeout_seconds: Option<u32>,
  pub reconnect_url: Option<String>, // is null
  pub recovery_url: Option<String>,  // is null
}

#[derive(Serialise, Deserialise, Debug, Clone)]
pub struct GMSubscription {
  pub id: String,
  pub status: Option<String>,
  #[serde(rename = "type")]
  pub kind: String,
  pub version: String,
  pub cost: i32,
  pub condition: Option<Condition>,
  pub transport: Transport,
  pub created_at: String,
  //pub event: Option<Event>,
}

#[repr(C)]
#[derive(Serialise, Deserialise, Debug, Clone)]
pub struct Mention {
  pub user_id: String,
  pub user_login: String,
  pub user_name: String,
}

#[repr(C)]
#[derive(Serialise, Deserialise, Debug, Clone)]
pub struct Emote {
  pub id: String,
  pub emote_set_id: String,
  pub owner_id: Option<String>,
  pub format: Option<Vec<String>>,
}

#[repr(C)]
#[derive(Serialise, Deserialise, Debug, Clone)]
pub struct CheerMote {
  pub prefix: String,
  pub bits: u32,
  pub tier: u32,
}
#[derive(Serialise, Deserialise, Clone, Debug, PartialEq)]
pub enum FragmentType {
  #[serde(rename = "text")]
  Text,
  #[serde(rename = "cheermote")]
  CheerMote,
  #[serde(rename = "emote")]
  Emote,
  #[serde(rename = "mention")]
  Mention,
}

impl Into<String> for FragmentType {
  fn into(self) -> String {
    match self {
      FragmentType::Text => "text",
      FragmentType::CheerMote => "cheermote",
      FragmentType::Emote => "emote",
      FragmentType::Mention => "mention",
    }
    .to_string()
  }
}

#[repr(C)]
#[derive(Serialise, Deserialise, Debug, Clone)]
pub struct Fragments {
  #[serde(rename = "type")]
  pub kind: FragmentType,
  pub text: String,
  pub cheermote: Option<CheerMote>,
  pub emote: Option<Emote>,
  pub mention: Option<Mention>,
}

impl Message {
  pub fn get_written_message(&self) -> Option<String> {
    let mut text = None;
    for fragment in &self.fragments {
      if fragment.kind != FragmentType::Mention {
        if let Some(ref mut text) = text {
          *text = format!("{} {}", text, fragment.text);
        } else {
          text = Some(fragment.text.to_string().trim().to_string());
        }
      }
    }
    text
  }
}

#[repr(C)]
#[derive(Serialise, Deserialise, Debug, Clone)]
pub struct Message {
  pub text: String,
  pub fragments: Vec<Fragments>,
}

#[repr(C)]
#[derive(Serialise, Deserialise, Debug, Clone)]
pub struct Badge {
  pub set_id: String,
  pub id: String,
  pub info: String,
}

#[repr(C)]
#[derive(Serialise, Deserialise, Debug, Clone)]
pub struct Reply {
  #[serde(flatten, with = "prefix_thread")]
  pub thread: User,
  #[serde(flatten, with = "prefix_parent")]
  pub parent: User,
  pub parent_message_id: String,
  pub parent_message_body: String,
  pub thread_message_id: String,
}

#[derive(Serialise, Deserialise, Debug, Clone, PartialEq)]
pub struct Reward {
  pub id: String,
  pub title: String,
  pub prompt: String,
  pub cost: u32,
}

#[repr(C)]
#[derive(Serialise, Deserialise, Debug, Clone)]
pub struct Cheer {
  pub bits: u32,
}

#[cfg_attr(
  feature = "bevy",
  derive(Serialise, Deserialise, Debug, Clone, BevyEvent)
)]
#[cfg_attr(not(feature = "bevy"), derive(Serialise, Deserialise, Debug, Clone))]
#[serde(untagged)]
pub enum Event {
  ChatMessage(MessageData),
  Raid(RaidData),
  Follow(FollowData),
  PointsCustomRewardRedeem(CustomPointsRewardRedeemData),
  AdBreakBegin(AdBreakBeginData),
  NewSubscription(NewSubscriptionData),
  GiftSubscription(GiftData),
  Resubscription(ResubscriptionData),
  Cheer(CheerData),
  ChannelPointsAutoRewardRedeem(ChannelPointsAutoRewardRedeemData),
  PollProgress(PollProgressData),
  PollBegin(PollBeginData),
  PollEnd(PollEndData),
  PredictionProgress(PredicitonProgressData),
  PredictionBegin(PredictionBeginData),
  PredictionLock(PredictionLockData),
  PredictionEnd(PredicitionEndData),
  HypeTrainProgress(HypeTrainProgressData),
  HypeTrainBegin(HypeTrainBeginData),
  HypeTrainEnd(HypeTrainEndData),
  MessageDeleted(MessageDeletedData),
}

#[derive(Serialise, Deserialise, Debug, Clone)]
pub struct Payload {
  pub session: Option<Session>,
  pub subscription: Option<GMSubscription>,
  pub event: Option<Event>,
}

#[derive(Serialise, Deserialise, Debug, Clone)]
pub struct MetaData {
  pub message_id: String,
  pub message_type: String,
  pub message_timestamp: String,
  pub subscription_type: Option<String>,
  pub subscription_version: Option<String>,
}

#[derive(Serialise, Deserialise, Debug, Clone)]
pub struct GenericMessage {
  pub metadata: MetaData,
  pub payload: Option<Payload>,
  pub subscription_type: Option<String>,
  pub subscription_version: Option<String>,
}

pub enum EventMessageType {
  Welcome,
  KeepAlive,
  Notification,
  Reconnect,
  Unknown,
}

impl EventMessageType {
  pub fn from_string(t: &str) -> EventMessageType {
    match t {
      "session_welcome" => EventMessageType::Welcome,
      "session_keepalive" => EventMessageType::KeepAlive,
      "notification" => EventMessageType::Notification,
      "session_reconnect" => EventMessageType::Reconnect,
      _ => EventMessageType::Unknown,
    }
  }
}

impl GenericMessage {
  pub fn event_type(&self) -> EventMessageType {
    EventMessageType::from_string(&self.metadata.message_type)
  }

  pub fn subscription_type(&self) -> Subscription {
    Subscription::from_string(&self.metadata.subscription_type.clone().unwrap()).unwrap()
  }
}
