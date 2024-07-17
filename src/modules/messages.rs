use serde_with::with_prefix;

use crate::{
  Badge, Cheer, Deserialise, Emote, Event, EventSubError, Fragments, Message, Reply, Reward,
  Serialise, Subscription,
};

with_prefix!(prefix_broadcaster "broadcaster_");
with_prefix!(prefix_from_broadcaster "from_broadcaster_");
with_prefix!(prefix_to_broadcaster "to_broadcaster_");
with_prefix!(prefix_requester "requester_");
with_prefix!(prefix_request "request_");
with_prefix!(pub prefix_thread "thread_");
with_prefix!(pub prefix_parent "parent_");
with_prefix!(prefix_chatter "chatter_");

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
  pub outcomes: Vec<Outcome>,
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
  pub emotes: Vec<RewardEmote>,
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
  pub tier: String,
  pub message: RewardMessageData,
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
  pub bit_votes: u32,
  pub channel_points_votes: u32,
  pub votes: u32,
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
  pub status: String,
  pub started_at: String,
  pub ended_at: String,
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
  pub braodcaster: User,
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
  #[serde(flatten, with = "prefix_broadcaster")]
  pub broadcaster: User,
  pub user_input: String,
  pub status: String,
  pub reward: Reward,
  pub redeemed_at: String,
}

#[derive(Debug)]
pub enum ResponseType {
  Event(Event),
  Error(EventSubError),
  RawResponse(String),
  Close,
}
