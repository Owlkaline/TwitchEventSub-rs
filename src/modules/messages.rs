use crate::{
  Badge, Cheer, Deserialise, Event, EventSubError, Fragments, Message, Reward, Serialise,
  Subscription,
};

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct SubscribeData {
  #[serde(flatten)]
  user: User,
  #[serde(flatten)]
  broadcaster: BroadcasterUser,
  tier: String,
  is_gift: bool,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct BroadcasterUser {
  #[serde(rename = "broadcaster_user_id")]
  pub id: String,
  #[serde(rename = "broadcaster_user_login")]
  pub login: String,
  #[serde(rename = "broadcaster_user_name")]
  pub name: String,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct RequestUser {
  #[serde(rename = "request_user_id")]
  pub id: String,
  #[serde(rename = "request_user_login")]
  pub login: String,
  #[serde(rename = "request_user_name")]
  pub name: String,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct ThreadUser {
  #[serde(rename = "thread_user_id")]
  pub id: String,
  #[serde(rename = "thread_user_login")]
  pub login: String,
  #[serde(rename = "thread_user_name")]
  pub name: String,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct ParentUser {
  #[serde(rename = "parent_user_id")]
  pub id: String,
  #[serde(rename = "parent_user_login")]
  pub login: String,
  #[serde(rename = "parent_user_name")]
  pub name: String,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct ChatterUser {
  #[serde(rename = "chatter_user_id")]
  pub id: String,
  #[serde(rename = "chatter_user_name")]
  pub name: String,
  #[serde(rename = "chatter_user_login")]
  pub login: String,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct User {
  #[serde(rename = "user_id")]
  pub id: String,
  #[serde(rename = "user_name")]
  pub name: String,
  #[serde(rename = "user_login")]
  pub login: String,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct AdBreakBeginInfo {
  #[serde(flatten)]
  broadcast_user: BroadcasterUser,
  #[serde(flatten)]
  request_user: RequestUser,
  duration_seconds: u32,
  started_at: String,
  is_automatic: bool,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct Reply {
  #[serde(flatten)]
  parent_user: ParentUser,
  #[serde(flatten)]
  thread_user: ThreadUser,
  parent_message_id: String,
  parent_message_body: String,
  thread_message_id: String,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct MessageData {
  #[serde(flatten)]
  pub broadcaster_user: BroadcasterUser,
  #[serde(flatten)]
  pub chatter_user: ChatterUser,
  pub message_id: String,
  pub message: Message,
  #[serde(rename = "color")]
  pub colour: String,
  pub badges: Vec<Badge>,
  pub message_type: String,
  pub cheer: Option<Cheer>,
  pub reply: Option<Reply>,
  pub channel_points_custom_reward_id: Option<String>,
  pub channel_points_animation_id: Option<String>,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct RaidData {
  pub raider_user_id: String,
  pub raider_username: String,
  pub viewers: u32,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct CustomPointsRewardRedeemData {
  id: String,
  #[serde(flatten)]
  user: User,
  #[serde(flatten)]
  broadcaster: BroadcasterUser,
  user_input: String,
  status: String,
  reward: Reward,
  redeemed_at: String,
}

#[derive(Debug)]
pub enum MessageType {
  Event(Event),
  BanTimeoutUser,
  DeleteMessage,
  Error(EventSubError),
  RawResponse(String),
  Close,
}
