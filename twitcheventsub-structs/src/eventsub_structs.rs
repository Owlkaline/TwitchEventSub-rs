use std::fmt;

#[cfg(feature = "bevy")]
use bevy_ecs::prelude::Message as BevyMessage;
use serde_with::with_prefix;

use crate::prelude::*;
use crate::{Deserialise, Serialise};

with_prefix!(pub prefix_broadcaster "broadcaster_");
with_prefix!(pub prefix_source_broadcaster "source_broadcaster_");
with_prefix!(pub prefix_from_broadcaster "from_broadcaster_");
with_prefix!(pub prefix_to_broadcaster "to_broadcaster_");
with_prefix!(pub prefix_requester "requester_");
with_prefix!(pub prefix_request "request_");
with_prefix!(pub prefix_thread "thread_");
with_prefix!(pub prefix_parent "parent_");
with_prefix!(pub prefix_chatter "chatter_");
with_prefix!(pub prefix_target "target_");
with_prefix!(pub prefix_moderator "moderator_");

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct TopContributions {
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
  pub top_contributions: Vec<TopContributions>,
  pub shared_train_participants: Vec<Broadcaster>,
  pub started_at: String,
  pub ended_at: String,
  pub cooldown_ends_at: String,
  #[serde(rename = "type")]
  pub kind: HypeTrainType,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct Broadcaster {
  pub broadcaster_user_id: String,
  pub broadcaster_user_name: String,
  pub broadcaster_user_login: String,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub enum HypeTrainType {
  #[serde(rename = "treasure")]
  Treasure,
  #[serde(rename = "golden_kappa")]
  GoldenKappa,
  #[serde(rename = "regular")]
  Regular,
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
  pub top_contributions: Vec<TopContributions>,
  pub shared_train_participants: Vec<Broadcaster>,
  pub started_at: String,
  pub expires_at: String,
  pub is_shared_train: bool,
  #[serde(rename = "type")]
  pub kind: HypeTrainType,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct HypeTrainBeginData {
  pub id: String,
  #[serde(flatten, with = "prefix_broadcaster")]
  pub broadcaster: User,
  pub total: u32,
  pub progress: u32,
  pub top_contributions: Vec<TopContributions>,
  pub shared_train_participants: Vec<Broadcaster>,
  pub level: u32,
  pub started_at: String,
  pub exires_at: String,
  pub is_shared_train: bool,
  #[serde(rename = "type")]
  pub kind: HypeTrainType,
  all_time_high_level: u32,
  all_time_high_total: u32,
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
  pub id: String,
  pub title: String,
  #[serde(rename = "color")]
  pub colour: String,
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
pub struct PredictionProgressData {
  pub id: String,
  #[serde(flatten, with = "prefix_broadcaster")]
  pub broadcaster: User,
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
pub struct PredictionEndData {
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

#[cfg_attr(
  feature = "bevy",
  derive(Serialise, Deserialise, Debug, Clone, BevyMessage)
)]
#[cfg_attr(not(feature = "bevy"), derive(Serialise, Deserialise, Debug, Clone))]
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

#[cfg_attr(
  feature = "bevy",
  derive(Serialise, Deserialise, Debug, Clone, BevyMessage)
)]
#[cfg_attr(not(feature = "bevy"), derive(Serialise, Deserialise, Debug, Clone))]
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

#[cfg_attr(
  feature = "bevy",
  derive(Serialise, Deserialise, Debug, Clone, BevyMessage)
)]
#[cfg_attr(not(feature = "bevy"), derive(Serialise, Deserialise, Debug, Clone))]
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

#[cfg_attr(
  feature = "bevy",
  derive(Serialise, Deserialise, Debug, Clone, BevyMessage)
)]
#[cfg_attr(not(feature = "bevy"), derive(Serialise, Deserialise, Debug, Clone))]
pub struct CheerData {
  #[serde(flatten)]
  pub user: Option<User>,
  #[serde(flatten, with = "prefix_broadcaster")]
  pub broadcaster: User,
  pub is_anonymous: bool,
  pub message: String,
  pub bits: u32,
}

#[cfg_attr(
  feature = "bevy",
  derive(Serialise, Deserialise, Debug, Clone, BevyMessage)
)]
#[cfg_attr(not(feature = "bevy"), derive(Serialise, Deserialise, Debug, Clone))]
pub struct FollowData {
  #[serde(flatten)]
  pub user: User,
  #[serde(flatten, with = "prefix_broadcaster")]
  pub broadcaster: User,
  pub followed_at: String,
}

#[cfg_attr(
  feature = "bevy",
  derive(Serialise, Deserialise, Debug, Clone, BevyMessage)
)]
#[cfg_attr(not(feature = "bevy"), derive(Serialise, Deserialise, Debug, Clone))]
pub struct NewSubscriptionData {
  #[serde(flatten)]
  pub user: User,
  #[serde(flatten, with = "prefix_broadcaster")]
  pub broadcaster: User,
  pub tier: String,
  pub is_gift: bool,
}

#[cfg_attr(
  feature = "bevy",
  derive(Serialise, Deserialise, Debug, Clone, BevyMessage)
)]
#[cfg_attr(not(feature = "bevy"), derive(Serialise, Deserialise, Debug, Clone))]
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
#[derive(Serialise, Deserialise, Clone, Debug, Default)]
pub struct User {
  #[serde(rename = "user_id")]
  pub id: String,
  #[serde(rename = "user_name")]
  pub name: String,
  #[serde(rename = "user_login")]
  pub login: String,
}

#[cfg_attr(
  feature = "bevy",
  derive(Serialise, Deserialise, Debug, Clone, BevyMessage)
)]
#[cfg_attr(not(feature = "bevy"), derive(Serialise, Deserialise, Debug, Clone))]
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

impl fmt::Display for MessageType {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        MessageType::Text => "text",
        MessageType::ChannelPointsHighlighted => "channel_points_highlighted",
        MessageType::ChannelPointsSubOnly => "channel_points_sub_only",
        MessageType::UserIntro => "user_intro",
        MessageType::PowerUpsMessageEffect => "power_ups_message_effect",
        MessageType::PowerUpsGigantifiedEmote => "power_ups_gigantified_emote",
      }
    )
  }
}

#[repr(C)]
#[cfg_attr(
  feature = "bevy",
  derive(Serialise, Deserialise, Debug, Clone, BevyMessage)
)]
#[cfg_attr(not(feature = "bevy"), derive(Serialise, Deserialise, Debug, Clone))]
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
  #[serde(flatten, with = "prefix_source_broadcaster")]
  pub source_broadcaster: OptionalUser,
  pub source_message_id: Option<String>,
  pub source_badges: Option<Vec<Badge>>,
  pub is_source_only: Option<bool>,
  #[serde(skip)]
  pub first_time_chatter: bool,
  #[serde(skip)]
  pub returning_chatter: bool,
  #[serde(skip)]
  pub moderator: bool,
}

#[cfg_attr(
  feature = "bevy",
  derive(Serialise, Deserialise, Debug, Clone, BevyMessage)
)]
#[cfg_attr(not(feature = "bevy"), derive(Serialise, Deserialise, Debug, Clone))]
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

#[cfg_attr(
  feature = "bevy",
  derive(Serialise, Deserialise, Debug, Clone, BevyMessage)
)]
#[cfg_attr(not(feature = "bevy"), derive(Serialise, Deserialise, Debug, Clone))]
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

#[cfg_attr(
  feature = "bevy",
  derive(Serialise, Deserialise, Debug, Clone, BevyMessage)
)]
#[cfg_attr(not(feature = "bevy"), derive(Serialise, Deserialise, Debug, Clone))]
pub struct MessageDeletedData {
  #[serde(flatten, with = "prefix_broadcaster")]
  pub broadcaster: User,
  #[serde(flatten, with = "prefix_target")]
  pub target: User,
  pub message_id: String,
}

#[cfg_attr(
  feature = "bevy",
  derive(Serialise, Deserialise, Debug, Clone, BevyMessage)
)]
#[cfg_attr(not(feature = "bevy"), derive(Serialise, Deserialise, Debug, Clone))]
pub struct ShoutoutReceiveData {
  #[serde(flatten, with = "prefix_broadcaster")]
  pub broadcaster: User,
  #[serde(flatten, with = "prefix_from_broadcaster")]
  pub from_broadcaster: User,
  pub viewer_count: u32,
  pub started_at: String,
}

#[cfg_attr(
  feature = "bevy",
  derive(Serialise, Deserialise, Debug, Clone, BevyMessage)
)]
#[cfg_attr(not(feature = "bevy"), derive(Serialise, Deserialise, Debug, Clone))]
pub struct ShoutoutCreateData {
  #[serde(flatten, with = "prefix_broadcaster")]
  pub broadcaster: User,
  #[serde(flatten, with = "prefix_moderator")]
  pub moderator: User,
  #[serde(flatten, with = "prefix_to_broadcaster")]
  pub to_broadcaster: User,
  pub started_at: String,
  pub viewer_count: u32,
  pub cooldown_ends_at: String,
  pub target_cooldown_ends_at: String,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct UserBannedData {
  #[serde(flatten)]
  pub user: User,
  #[serde(flatten, with = "prefix_broadcaster")]
  pub broadcaster: User,
  #[serde(flatten, with = "prefix_moderator")]
  pub moderator: User,
  pub reason: String,
  pub banned_at: String,
  pub ends_at: Option<String>,
  pub is_permanent: bool,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct UserAction {
  #[serde(flatten)]
  pub user: User,
  reason: String,
  timeout: String,
  message_id: String,
  message_body: String,
  viewer_count: Option<u32>,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct StreamOnlineData {
  pub id: String,
  #[serde(flatten, with = "prefix_broadcaster")]
  pub broadcaster: User,
  #[serde(rename = "type")]
  pub kind: String,
  pub started_at: Option<String>,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct StreamOfflineData {
  #[serde(flatten, with = "prefix_broadcaster")]
  pub broadcaster: User,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub enum ModeratorAction {
  #[serde(rename = "ban")]
  Ban,
  #[serde(rename = "timeout")]
  Timeout,
  #[serde(rename = "unban")]
  Unban,
  #[serde(rename = "untimeout")]
  Untimeout,
  #[serde(rename = "clear")]
  Clear,
  #[serde(rename = "emoteonly")]
  EmoteOnly,
  #[serde(rename = "emoteonlyoff")]
  EmoteOnlyOff,
  #[serde(rename = "followers")]
  Followers,
  #[serde(rename = "followersoff")]
  FollowersOff,
  #[serde(rename = "uniquechat")]
  UniqueChat,
  #[serde(rename = "uniquechatoff")]
  UniqueChatOff,
  #[serde(rename = "slow")]
  Slow,
  #[serde(rename = "slowoff")]
  SlowOff,
  #[serde(rename = "subscribers")]
  Subscribers,
  #[serde(rename = "subscribersoff")]
  SubscribersOff,
  #[serde(rename = "unraid")]
  Unraid,
  #[serde(rename = "delete")]
  Delete,
  #[serde(rename = "unvip")]
  Unvip,
  #[serde(rename = "vip")]
  Vip,
  #[serde(rename = "raid")]
  Raid,
  #[serde(rename = "add_blocked_term")]
  AddBlockedTerm,
  #[serde(rename = "add_permitted_term")]
  AddPermittedTerm,
  #[serde(rename = "remove_blocked_term")]
  RemoveBlockedTerm,
  #[serde(rename = "remove_permitted_term")]
  RemovePermittedTerm,
  #[serde(rename = "mod")]
  Mod,
  #[serde(rename = "unmod")]
  Unmod,
  #[serde(rename = "approve_unban_request")]
  ApproveUnbanRequest,
  #[serde(rename = "deny_unban_request")]
  DenyUnbanRequest,
  #[serde(rename = "warn")]
  Warn,
  #[serde(rename = "shared_chat_ban")]
  SharedChatBan,
  #[serde(rename = "shared_chat_timeout")]
  SharedChatTimeout,
  #[serde(rename = "shared_chat_unban")]
  SharedChatUnban,
  #[serde(rename = "shared_chat_untimeout")]
  SharedChatUntimeout,
  #[serde(rename = "shared_chat_delete")]
  SharedChatDelete,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct Followers {
  follow_duration_minutes: u32,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct Slow {
  wait_time_seconds: u32,
}

#[derive(Serialise, Deserialise, Clone, Debug)]
pub struct ModeratorEvent {
  #[serde(flatten, with = "prefix_broadcaster")]
  pub broadcaster: User,
  #[serde(flatten, with = "prefix_source_broadcaster")]
  pub source: User,
  #[serde(flatten, with = "prefix_moderator")]
  pub moderator: User,
  pub action: ModeratorAction,
  pub followers: Option<Followers>,
  pub slow: Option<Slow>,
  pub vip: Option<User>,
  pub unvip: Option<User>,
  #[serde(rename = "mod")]
  pub mod_user: Option<User>,
  pub unmod: Option<User>,
  pub ban: Option<UserAction>,
  pub timeout: Option<UserAction>,
  pub untimeout: Option<UserAction>,
  pub raid: Option<UserAction>,
  pub unraid: Option<UserAction>,
  pub delete: Option<UserAction>,
  pub automod_terms: Option<String>,         // TODO:
  pub unban_request: Option<String>,         // TODO:
  pub warn: Option<String>,                  // TODO:
  pub shared_chat_ban: Option<String>,       // TODO:
  pub shared_chat_unban: Option<String>,     // TODO:
  pub shared_chat_timeout: Option<String>,   // TODO:
  pub shared_chat_untimeout: Option<String>, // TODO:
  pub shared_chat_delete: Option<String>,    // TODO:
}
