use crate::*;
use crate::{Deserialise, Serialise};
use serde_with::with_prefix;

with_prefix!(pub prefix_broadcaster "broadcaster_");
with_prefix!(pub prefix_from_broadcaster "from_broadcaster_");
with_prefix!(pub prefix_to_broadcaster "to_broadcaster_");
with_prefix!(pub prefix_requester "requester_");
with_prefix!(pub prefix_request "request_");
with_prefix!(pub prefix_thread "thread_");
with_prefix!(pub prefix_parent "parent_");
with_prefix!(pub prefix_chatter "chatter_");
with_prefix!(pub prefix_target "target_");

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
  #[serde(flatten, with = "prefix_broadcaster")]
  pub broadcaster: User,
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
  #[serde(flatten, with = "prefix_broadcaster")]
  pub broadcaster: User,
  pub level: u32,
  pub total: u32,
  pub progress: u32,
  pub goal: u32,
  pub top_contributions: TopContributions,
  pub last_contribution: LastContribution,
  pub started_at: String,
  pub expires_at: String,
}
//RAW response: {"metadata":{"message_id":"bSxSXPGSqWJeZKQCFi5U1pU9idS1xzkV7ijsoNE6CEU=","message_type":"notification","message_timestamp":"2024-09-12T15:10:17.454792032Z","subscription_type":"channel.hype_train.progress","subscription_version":"1"},"payload":{"subscription":{"id":"a3a8087b-1796-428f-803c-efc6345c62a7","status":"enabled","type":"channel.hype_train.progress","version":"1","condition":{"broadcaster_user_id":"122297321"},"transport":{"method":"websocket","session_id":"AgoQpICHjI14T5i7mniZfQpuEhIGY2VsbC1h"},"created_at":"2024-09-12T07:48:13.157489194Z","cost":0},"event":{"id":"57c43e5a-a90d-4e0e-956f-8d8fa1190a3f","broadcaster_user_id":"122297321","broadcaster_user_login":"owlkalinevt","broadcaster_user_name":"OwlkalineVT","total":3000,"top_contributions":[{"user_id":"613487069","user_login":"hiimneil9000","user_name":"hiimNeil9000","type":"subscription","total":2500}],"started_at":"2024-09-12T15:09:08.800986839Z","last_contribution":{"user_id":"37104029","user_login":"daagr","user_name":"Daagr","type":"subscription","total":500},"level":2,"goal":1800,"progress":1400,"expires_at":"2024-09-12T15:14:08.800986839Z"}}}

//RAW response: {"metadata":{"message_id":"hdGW3Cg1S2rJ7SBKm0Hv8T7FXGKDQoH2cVypZNXeApk=","message_type":"notification","message_timestamp":"2024-09-12T15:16:20.192135738Z","subscription_type":"channel.hype_train.end","subscription_version":"1"},"payload":{"subscription":{"id":"5e39932f-85d6-484c-981a-b00100fec4a6","status":"enabled","type":"channel.hype_train.end","version":"1","condition":{"broadcaster_user_id":"122297321"},"transport":{"method":"websocket","session_id":"AgoQpICHjI14T5i7mniZfQpuEhIGY2VsbC1h"},"created_at":"2024-09-12T07:48:13.368718467Z","cost":0},"event":{"id":"57c43e5a-a90d-4e0e-956f-8d8fa1190a3f","broadcaster_user_id":"122297321","broadcaster_user_login":"owlkalinevt","broadcaster_user_name":"OwlkalineVT","total":3600,"top_contributions":[{"user_id":"613487069","user_login":"hiimneil9000","user_name":"hiimNeil9000","type":"subscription","total":2500},{"user_id":"189444487","user_login":"exia_quantavt","user_name":"Exia_QuantaVT","type":"bits","total":100}],"started_at":"2024-09-12T15:09:08.800986839Z","level":3,"ended_at":"2024-09-12T15:16:19.167649013Z","cooldown_ends_at":"2024-09-12T16:16:19.167649013Z"}}}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct HypeTrainBeginData {
  pub id: String,
  #[serde(flatten, with = "prefix_broadcaster")]
  pub broadcaster: User,
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
pub struct BeginOutcome {
  id: String,
  title: String,
  #[serde(rename = "color")]
  colour: String,
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
  #[serde(flatten, with = "prefix_broadcaster")]
  pub broadcaster: User,
  pub title: String,
  pub outcomes: Vec<BeginOutcome>,
  pub started_at: String,
  pub locks_at: String,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct PredicitonProgressData {
  pub id: String,
  #[serde(flatten, with = "prefix_broadcaster")]
  broadcaster: User,
  pub title: String,
  pub outcomes: Vec<Outcome>,
  pub started_at: String,
  pub locks_at: String,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct PredictionLockData {
  pub id: String,
  #[serde(flatten, with = "prefix_broadcaster")]
  pub broadcaster: User,
  pub title: String,
  pub outcomes: Vec<Outcome>,
  pub started_at: String,
  pub locked_at: String,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct PredicitionEndData {
  pub id: String,
  #[serde(flatten, with = "prefix_broadcaster")]
  pub broadcaster: User,
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
  pub user: OptionalUser,
  #[serde(flatten, with = "prefix_broadcaster")]
  pub broadcaster: User,
  pub total: u32,
  pub tier: String,
  pub cumulative_total: Option<u32>,
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
  pub emotes: Option<Vec<RewardEmote>>,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub enum AutoRewardType {
  #[serde(rename = "send_highlighted_message")]
  SendHighlightedMessage,
  #[serde(rename = "single_message_bypass_ub_mode")]
  SingleMessageBypassSubMode,
  #[serde(rename = "random_sub_emote_unlock")]
  RandomSubEmoteUnlock,
  #[serde(rename = "chosen_sub_emote_unlock")]
  ChosenSubEmoteUnlock,
  #[serde(rename = "chosen_modified_sub_emote_unlock")]
  ChosenModifiedSubEmoteUnlock,
  #[serde(rename = "message_effect")]
  MessageEffect,
  #[serde(rename = "gigantify_an_emote")]
  GigantifyAnEmote,
  #[serde(rename = "celebration")]
  Celebration,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct AutoRewardData {
  #[serde(rename = "type")]
  pub kind: AutoRewardType,
  pub cost: u32,
  pub unlocked_emote: Option<UnlockedEmote>,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct ChannelPointsAutoRewardRedeemData {
  #[serde(flatten, with = "prefix_broadcaster")]
  pub broadcaster: User,
  #[serde(flatten)]
  pub user: User,
  pub id: String,
  pub reward: AutoRewardData,
  pub message: RewardMessageData,
  pub user_input: Option<String>,
  pub redeemed_at: String,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct CheerData {
  #[serde(flatten)]
  pub user: User,
  #[serde(flatten, with = "prefix_broadcaster")]
  pub broadcaster: User,
  pub is_anonymous: bool,
  pub message: String,
  pub bits: u32,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct FollowData {
  #[serde(flatten)]
  pub user: User,
  #[serde(flatten, with = "prefix_broadcaster")]
  pub broadcaster: User,
  pub followed_at: String,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct NewSubscriptionData {
  #[serde(flatten)]
  pub user: User,
  #[serde(flatten, with = "prefix_broadcaster")]
  pub broadcaster: User,
  pub tier: String,
  pub is_gift: bool,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct ResubscriptionData {
  #[serde(flatten)]
  pub user: User,
  #[serde(flatten, with = "prefix_broadcaster")]
  pub broadcaster: User,
  pub message: RewardMessageData,
  pub tier: String,
  pub cumulative_months: u32,
  pub streak_months: Option<u32>,
  pub duration_months: u32,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct OptionalUser {
  #[serde(rename = "user_id")]
  pub id: Option<String>,
  #[serde(rename = "user_name")]
  pub name: Option<String>,
  #[serde(rename = "user_login")]
  pub login: Option<String>,
}

#[repr(C)]
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
  #[serde(flatten, with = "prefix_broadcaster")]
  pub broadcaster: User,
  #[serde(flatten, with = "prefix_requester")]
  pub requester: User,
  pub duration_seconds: u32,
  pub started_at: String,
  pub is_automatic: bool,
}

#[repr(u8)]
#[derive(Serialise, Deserialise, Clone, Debug)]
pub enum MessageType {
  #[serde(rename = "text")]
  Text,
  #[serde(rename = "channel_points_highlighted")]
  ChannelPointsHighlighted,
  #[serde(rename = "channel_points_sub_only")]
  ChannelPointsSubOnly,
  #[serde(rename = "user_intro")]
  UserIntro,
  #[serde(rename = "power_ups_message_effect")]
  PowerUpsMessageEffect,
  #[serde(rename = "power_ups_gigantified_emote")]
  PowerUpsGigantifiedEmote,
}

#[repr(C)]
#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct MessageData {
  #[serde(flatten, with = "prefix_broadcaster")]
  pub broadcaster: User,
  #[serde(flatten, with = "prefix_chatter")]
  pub chatter: User,
  pub message_id: String,
  pub message: Message,
  #[serde(rename = "color")]
  pub colour: String,
  pub badges: Vec<Badge>,
  pub message_type: MessageType,
  pub cheer: Option<Cheer>,
  pub reply: Option<Reply>,
  pub channel_points_custom_reward_id: Option<String>,
  pub channel_points_animation_id: Option<String>,
  // Adding afterwards with data from irc
  #[serde(skip)]
  pub first_time_chatter: bool,
  #[serde(skip)]
  pub returning_chatter: bool,
  #[serde(skip)]
  pub moderator: bool,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct RaidData {
  #[serde(flatten, with = "prefix_from_broadcaster")]
  pub from_broadcaster: User,
  #[serde(flatten, with = "prefix_to_broadcaster")]
  pub to_broadcaster: User,
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
  pub votes: u32,
  pub channel_points_votes: u32,
  pub bits_votes: u32,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct BeginChoices {
  pub id: String,
  pub title: String,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct PollEndData {
  pub id: String,
  #[serde(flatten, with = "prefix_broadcaster")]
  pub broadcaster: User,
  pub title: String,
  pub choices: Vec<Choices>,
  pub bits_voting: BitsVotingData,
  pub channel_points_voting: ChannelPointsVoting,
  pub started_at: String,
  pub ended_at: String,
  pub status: String,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct PollProgressData {
  pub id: String,
  #[serde(flatten, with = "prefix_broadcaster")]
  pub broadcaster: User,
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
  #[serde(flatten, with = "prefix_broadcaster")]
  pub broadcaster: User,
  pub title: String,
  pub choices: Vec<BeginChoices>,
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
  #[serde(flatten, with = "prefix_broadcaster")]
  pub broadcaster: User,
  pub user_input: String,
  pub status: String,
  pub reward: Reward,
  pub redeemed_at: String,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct MessageDeletedData {
  #[serde(flatten, with = "prefix_broadcaster")]
  pub broadcaster: User,
  #[serde(flatten, with = "prefix_target")]
  pub target: User,
  pub message_id: String,
}
