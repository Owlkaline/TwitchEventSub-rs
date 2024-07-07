use crate::{
  Badge, Cheer, Deserialise, Emote, Event, EventSubError, Fragments, Message, Reply, Reward,
  Serialise, Subscription,
};

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct TopContributions {
  #[serde(flatten)]
  pub user: User,
  #[serde(rename = "type")]
  pub kind: String,
  pub total: u32,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct LastContribution {
  #[serde(flatten)]
  pub user: User,
  #[serde(rename = "type")]
  pub kind: String,
  pub total: u32,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct HypeTrainEndData {
  pub id: String,
  #[serde(flatten)]
  pub broadcaster: BroadcasterUser,
  pub level: u32,
  pub total: u32,
  pub top_contributions: TopContributions,
  pub started_at: String,
  pub ended_at: String,
  pub cooldown_ends_at: String,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct HypeTrainProgressData {
  pub id: String,
  #[serde(flatten)]
  pub broadcaster: BroadcasterUser,
  pub level: u32,
  pub total: u32,
  pub progress: u32,
  pub goal: u32,
  pub top_contributions: TopContributions,
  pub last_contribution: LastContribution,
  pub started_at: String,
  pub expires_at: String,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct HypeTrainBeginData {
  pub id: String,
  #[serde(flatten)]
  pub broadcaster: BroadcasterUser,
  pub total: u32,
  pub progress: u32,
  pub top_contributions: TopContributions,
  pub last_contribution: LastContribution,
  pub level: u32,
  pub started_at: String,
  pub exires_at: String,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct TopPredictors {
  #[serde(flatten)]
  pub user: User,
  pub channel_points_won: Option<u32>,
  pub channel_points_used: u32,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct Outcome {
  pub id: String,
  pub title: String,
  #[serde(rename = "color")]
  pub colour: String,
  pub users: u32,
  pub channel_points: u32,
  pub top_predictors: Vec<TopPredictors>,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct PredictionBeginData {
  pub id: String,
  #[serde(flatten)]
  pub broadcaster: BroadcasterUser,
  pub title: String,
  pub outcomes: Vec<Outcome>,
  pub started_at: String,
  pub locks_at: String,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct PredicitonProgressData {
  pub id: String,
  #[serde(flatten)]
  broadcaster: BroadcasterUser,
  pub title: String,
  pub outcomes: Vec<Outcome>,
  pub started_at: String,
  pub locks_at: String,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct PredictionLockData {
  pub id: String,
  #[serde(flatten)]
  pub broadcaster: BroadcasterUser,
  pub title: String,
  pub outcomes: Vec<Outcome>,
  pub started_at: String,
  pub locked_at: String,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct PredicitionEndData {
  pub id: String,
  #[serde(flatten)]
  pub broadcaster: BroadcasterUser,
  pub title: String,
  pub winning_outcome_id: String,
  pub outcomes: Vec<Outcome>,
  pub status: String,
  pub started_at: String,
  pub ended_at: String,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct GiftData {
  #[serde(flatten)]
  pub user: User,
  #[serde(flatten)]
  pub broadcaster: BroadcasterUser,
  pub total: u32,
  pub tier: String,
  pub cumulative_total: u32,
  pub is_anonymous: bool,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct UnlockedEmote {
  pub id: String,
  pub name: String,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct RewardEmote {
  pub id: String,
  pub begin: u32,
  pub end: u32,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct RewardMessageData {
  pub text: String,
  pub emotes: Vec<Emote>,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct AutoRewardData {
  #[serde(rename = "type")]
  pub kind: String,
  pub cost: u32,
  pub unlocked_emote: Option<UnlockedEmote>,
  pub message: RewardMessageData,
  pub user_input: Option<String>,
  pub redeemed_at: String,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct ChannelPointsAutoRewardRedeemData {
  #[serde(flatten)]
  pub broadcaster: BroadcasterUser,
  #[serde(flatten)]
  pub user: User,
  pub id: String,
  pub reward: Reward,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct CheerData {
  #[serde(flatten)]
  pub user: User,
  #[serde(flatten)]
  pub broadcaster: BroadcasterUser,
  pub is_anonymous: bool,
  pub message: String,
  pub bits: u32,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct FollowData {
  #[serde(flatten)]
  pub user: User,
  #[serde(flatten)]
  pub broadcaster: BroadcasterUser,
  pub followed_at: String,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct SubscribeData {
  #[serde(flatten)]
  pub user: User,
  #[serde(flatten)]
  pub braodcaster: BroadcasterUser,
  pub tier: String,
  pub is_gift: bool,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct SubscribeMessageData {
  #[serde(flatten)]
  pub user: User,
  #[serde(flatten)]
  pub broadcaster: BroadcasterUser,
  pub tier: String,
  pub message: RewardMessageData,
  pub cumulative_months: u32,
  pub streak_months: Option<u32>,
  pub duration_months: u32,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct FromBroadcasterUser {
  #[serde(rename = "from_broadcaster_user_id")]
  pub id: String,
  #[serde(rename = "from_broadcaster_user_login")]
  pub login: String,
  #[serde(rename = "from_broadcaster_user_name")]
  pub name: String,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct ToBroadcasterUser {
  #[serde(rename = "to_broadcaster_user_id")]
  pub id: String,
  #[serde(rename = "to_broadcaster_user_login")]
  pub login: String,
  #[serde(rename = "to_broadcaster_user_name")]
  pub name: String,
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
pub struct RequesterUser {
  #[serde(rename = "requester_user_id")]
  pub id: String,
  #[serde(rename = "requester_user_login")]
  pub login: String,
  #[serde(rename = "requester_user_name")]
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
pub struct AdBreakBeginData {
  #[serde(flatten)]
  pub broadcaster: BroadcasterUser,
  #[serde(flatten)]
  pub requester: RequesterUser,
  pub duration_seconds: u32,
  pub started_at: String,
  pub is_automatic: bool,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct MessageData {
  #[serde(flatten)]
  pub broadcaster: BroadcasterUser,
  #[serde(flatten)]
  pub chatter: ChatterUser,
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
  #[serde(flatten)]
  pub from_broadcaster: FromBroadcasterUser,
  #[serde(flatten)]
  pub to_broadcaster: ToBroadcasterUser,
  pub viewers: u32,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct BitsVotingData {
  pub is_enabled: bool,
  pub amount_per_vote: u32,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct ChannelPointsVoting {
  pub is_enabled: bool,
  pub amount_per_vote: u32,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct Choices {
  pub id: String,
  pub title: String,
  pub bit_votes: u32,
  pub channel_points_votes: u32,
  pub votes: u32,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct PollEndData {
  pub id: String,
  #[serde(flatten)]
  pub broadcaster: BroadcasterUser,
  pub title: String,
  pub choices: Vec<Choices>,
  pub bits_voting: BitsVotingData,
  pub channel_points_voting: ChannelPointsVoting,
  pub status: String,
  pub started_at: String,
  pub ended_at: String,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct PollProgressData {
  pub id: String,
  #[serde(flatten)]
  pub broadcaster: BroadcasterUser,
  pub title: String,
  pub choices: Vec<Choices>,
  pub bits_voting: BitsVotingData,
  pub channel_points_voting: ChannelPointsVoting,
  pub started_at: String,
  pub ends_at: String,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct PollBeginData {
  pub id: String,
  #[serde(flatten)]
  pub braodcaster: BroadcasterUser,
  pub title: String,
  pub choices: Vec<Choices>,
  pub bits_voting: BitsVotingData,
  pub channel_points_voting: ChannelPointsVoting,
  pub started_at: String,
  pub ends_at: String,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct CustomPointsRewardRedeemData {
  pub id: String,
  #[serde(flatten)]
  pub user: User,
  #[serde(flatten)]
  pub broadcaster: BroadcasterUser,
  pub user_input: String,
  pub status: String,
  pub reward: Reward,
  pub redeemed_at: String,
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
