use serde::{Deserialize as Deserialise, Serialize as Serialise};

use crate::{prefix_broadcaster, User};

#[derive(Serialise, Deserialise, Debug)]
pub struct GetCustomRewards {
  pub data: Vec<GetCustomReward>,
}

#[derive(Serialise, Deserialise, Debug)]
pub struct GetCustomReward {
  pub id: String,
  pub broadcaster_id: String,
  pub broadcaster_login: String,
  pub broadcaster_name: String,
  pub title: String,
  pub image: Option<EmoteStaticImages>,
  pub default_image: EmoteStaticImages,
  #[serde(rename = "background_color")]
  pub background_colour: String,
  pub is_enabled: bool,
  pub cost: u32,
  pub prompt: String,
  pub is_user_input_required: bool,
  pub is_paused: bool,
  pub is_in_stock: bool,
  pub max_per_stream_setting: MaxPerStreamSetting,
  pub max_per_user_per_stream_setting: MaxPerUserPerStreamSetting,
  pub global_cooldown_setting: GlobalCooldownSetting,
  pub should_redemptions_skip_request_queue: bool,
  pub redemptions_redeemed_current_stream: Option<u32>,
  pub cooldown_expires_at: Option<String>,
}

#[derive(Serialise, Deserialise, Debug)]
pub struct MaxPerUserPerStreamSetting {
  is_enabled: bool,
  max_per_user_per_stream: u32,
}

#[derive(Serialise, Deserialise, Debug)]
pub struct AnnouncementMessage {
  pub message: String,
  pub colour: Option<String>,
}

#[derive(Serialise, Deserialise, Debug)]
pub struct CreateCustomReward {
  pub title: String,
  pub cost: i64,
  pub prompt: bool,
  pub is_enabled: bool,
  pub is_user_input_required: bool,
  pub is_max_per_stream_enabled: bool,
  pub max_per_stream: i32,
  pub is_max_per_user_per_stream_enabled: bool,
  pub max_per_user_per_stream: i32,
  pub is_global_cooldown_enabled: bool,
  pub global_cooldown_seconds: i32,
}

impl Default for CreateCustomReward {
  fn default() -> Self {
    CreateCustomReward {
      title: "default".to_owned(),
      cost: 1,
      prompt: false,
      max_per_stream: 1,
      max_per_user_per_stream: 1,
      global_cooldown_seconds: 1,
      is_enabled: true,
      is_user_input_required: false,
      is_max_per_user_per_stream_enabled: false,
      is_max_per_stream_enabled: false,
      is_global_cooldown_enabled: false,
    }
  }
}

#[derive(Serialise, Deserialise, Debug)]
pub struct MaxPerStreamSetting {
  pub is_enabled: bool,
  pub max_per_stream: i64,
}

#[derive(Serialise, Deserialise, Debug)]
pub struct GlobalCooldownSetting {
  pub is_enabled: bool,
  pub global_cooldown_seconds: i64,
}

#[derive(Serialise, Deserialise, Debug)]
pub struct CreatedCustomRewardResponse {
  pub data: Vec<CreatedCustomReward>,
}

#[derive(Serialise, Deserialise, Debug)]
pub struct CreatedCustomReward {
  #[serde(flatten, with = "prefix_broadcaster")]
  pub broadcaster: User,
  pub id: String,
  pub title: String,
  pub prompt: String,
  pub cost: i32,
  pub image: Option<EmoteStaticImages>,
  pub default_image: Option<EmoteStaticImages>,
  pub background_color: String,
  pub is_enabled: bool,
  pub is_user_input_required: bool,
  pub max_per_stream_setting: MaxPerStreamSetting,
  pub max_per_user_per_stream: Option<i64>,
  pub global_cooldown_setting: GlobalCooldownSetting,
  pub is_paused: bool,
  pub is_in_stock: bool,
  pub should_redemptions_skip_request_queue: bool,
  pub redemptions_redeemed_current_stream: Option<i32>,
  pub cooldown_expires_at: Option<String>,
}

#[derive(Serialise, Deserialise, Debug)]
pub struct AdSchedule {
  pub data: Vec<AdDetails>,
}

#[derive(Serialise, Deserialise, Debug)]
pub struct AdDetails {
  pub next_ad_at: u32,
  pub last_ad_at: u32,
  pub duration: u32,
  pub preroll_free_time: u32,
  pub snooze_count: u32,
  pub snooze_refresh_at: u32,
}

#[derive(Serialise, Deserialise, Debug)]
pub struct Pagination {
  pub cursor: Option<String>,
}

#[derive(Serialise, Deserialise, Debug)]
pub struct GetChatters {
  pub data: Vec<User>,
  pub pagination: Pagination,
  pub total: i32,
}

#[derive(Serialise, Deserialise, Debug, Clone)]
pub struct EmoteStaticImages {
  pub url_1x: String,
  pub url_2x: String,
  pub url_4x: String,
}

#[derive(Serialise, Deserialise, Debug, Clone)]
pub enum EmoteType {
  #[serde(rename = "bitstier")]
  BitsTier,
  #[serde(rename = "follower")]
  Follower,
  #[serde(rename = "subscriptions")]
  Subscriptions,
  #[serde(rename = "globals")]
  Global,
  #[serde(rename = "prime")]
  Prime,
  #[serde(rename = "turbo")]
  Turbo,
  #[serde(rename = "smilies")]
  Smilies,
  #[serde(rename = "limitedtime")]
  LimitedTime,
  #[serde(rename = "rewards")]
  Rewards,
  #[serde(rename = "none")]
  None,
  #[serde(rename = "owl2019")]
  Owl2019,
  #[serde(rename = "hypetrain")]
  HypeTrain,
  #[serde(rename = "PSTgasm")]
  PSTGasm,
  #[serde(rename = "twofactor")]
  TwoFactor,
}

#[derive(Serialise, Deserialise, Debug, PartialEq, Clone)]
pub enum EmoteFormat {
  #[serde(rename = "static")]
  Static,
  #[serde(rename = "animated")]
  Animated,
}

impl EmoteFormat {
  pub fn string(&self) -> String {
    match self {
      EmoteFormat::Static => "static",
      EmoteFormat::Animated => "animated",
    }
    .to_string()
  }
}

#[derive(Serialise, Deserialise, Debug, Clone)]
pub enum ThemeMode {
  #[serde(rename = "light")]
  Light,
  #[serde(rename = "dark")]
  Dark,
}

impl ThemeMode {
  pub fn string(&self) -> String {
    match self {
      ThemeMode::Light => "light",
      ThemeMode::Dark => "dark",
    }
    .to_string()
  }
}

#[derive(Debug)]
pub struct EmoteData {
  pub id: String,
  pub name: String,
  pub images: EmoteStaticImages,
  pub format: Vec<EmoteFormat>,
  pub scale: Vec<String>,
  pub theme_mode: Vec<ThemeMode>,

  pub tier: Option<String>,
  pub emote_type: Option<EmoteType>,
  pub emote_set_id: Option<String>,
}

#[derive(Serialise, Deserialise, Debug, Clone)]
pub struct ChannelEmoteData {
  pub id: String,
  pub name: String,
  pub images: EmoteStaticImages,
  pub tier: String,
  pub emote_type: EmoteType,
  pub emote_set_id: String,
  pub format: Vec<EmoteFormat>,
  pub scale: Vec<String>,
  pub theme_mode: Vec<ThemeMode>,
}

#[derive(Serialise, Deserialise, Debug, Clone)]
pub struct GlobalEmoteData {
  pub id: String,
  pub name: String,
  pub images: EmoteStaticImages,
  pub format: Vec<EmoteFormat>,
  pub scale: Vec<String>,
  pub theme_mode: Vec<ThemeMode>,
}

#[derive(Serialise, Deserialise, Debug)]
pub struct ChannelEmotes {
  pub data: Vec<ChannelEmoteData>,
  pub template: String,
}

#[derive(Serialise, Deserialise, Debug)]
pub struct GlobalEmotes {
  pub data: Vec<GlobalEmoteData>,
  pub template: String,
}

#[derive(Serialise, Deserialise, Debug)]
pub struct Moderators {
  pub data: Vec<User>,
  pub pagination: Pagination,
}

impl Into<EmoteData> for ChannelEmoteData {
  fn into(self) -> EmoteData {
    EmoteData {
      id: self.id,
      name: self.name,
      images: self.images,
      tier: Some(self.tier),
      emote_type: Some(self.emote_type),
      emote_set_id: Some(self.emote_set_id),
      format: self.format,
      scale: self.scale,
      theme_mode: self.theme_mode,
    }
  }
}

impl Into<EmoteData> for GlobalEmoteData {
  fn into(self) -> EmoteData {
    EmoteData {
      id: self.id,
      name: self.name,
      images: self.images,
      tier: None,
      emote_type: None,
      emote_set_id: None,
      format: self.format,
      scale: self.scale,
      theme_mode: self.theme_mode,
    }
  }
}

#[derive(Serialise, Deserialise, Debug)]
pub enum EmoteScale {
  #[serde(rename = "1.0")]
  Size1,
  #[serde(rename = "2.0")]
  Size2,
  #[serde(rename = "3.0")]
  Size3,
}

impl EmoteScale {
  pub fn idx(&self) -> usize {
    match self {
      EmoteScale::Size1 => 0,
      EmoteScale::Size2 => 1,
      EmoteScale::Size3 => 2,
    }
  }
}

pub struct EmoteUrl {
  pub url: String,
  pub animated: bool,
}
