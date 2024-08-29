use crate::serde::{self, Deserialize as Deserialise, Serialize as Serialise};

use twitch_eventsub_structs::prefix_broadcaster;
use twitch_eventsub_structs::User;

#[derive(Serialise, Deserialise, Debug)]
#[serde(crate = "self::serde")]
pub struct AnnouncementMessage {
  pub message: String,
  pub colour: Option<String>,
}

#[derive(Serialise, Deserialise, Debug)]
#[serde(crate = "self::serde")]
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
#[serde(crate = "self::serde")]
pub struct MaxPerStreamSetting {
  pub is_enabled: bool,
  pub max_per_stream: i64,
}

#[derive(Serialise, Deserialise, Debug)]
#[serde(crate = "self::serde")]
pub struct GlobalCooldownSetting {
  pub is_enabled: bool,
  pub global_cooldown_seconds: i64,
}

#[derive(Serialise, Deserialise, Debug)]
#[serde(crate = "self::serde")]
pub struct CustomRewardResponse {
  pub data: Vec<CustomReward>,
}

#[derive(Serialise, Deserialise, Debug)]
#[serde(crate = "self::serde")]
pub struct CustomReward {
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
#[serde(crate = "self::serde")]
pub struct AdSchedule {
  pub data: Vec<AdDetails>,
}

#[derive(Serialise, Deserialise, Debug)]
#[serde(crate = "self::serde")]
pub struct AdDetails {
  pub next_ad_at: u32,
  pub last_ad_at: u32,
  pub duration: u32,
  pub preroll_free_time: u32,
  pub snooze_count: u32,
  pub snooze_refresh_at: u32,
}

#[derive(Serialise, Deserialise, Debug)]
#[serde(crate = "self::serde")]
pub struct Pagination {
  pub cursor: Option<String>,
}

#[derive(Serialise, Deserialise, Debug)]
#[serde(crate = "self::serde")]
pub struct GetChatters {
  pub data: Vec<User>,
  pub pagination: Pagination,
  pub total: i32,
}

#[derive(Serialise, Deserialise, Debug, Clone)]
#[serde(crate = "self::serde")]
pub struct EmoteStaticImages {
  pub url_1x: String,
  pub url_2x: String,
  pub url_4x: String,
}

#[derive(Serialise, Deserialise, Debug, Clone)]
#[serde(crate = "self::serde")]
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
#[serde(crate = "self::serde")]
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
#[serde(crate = "self::serde")]
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
#[serde(crate = "self::serde")]
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
#[serde(crate = "self::serde")]
pub struct GlobalEmoteData {
  pub id: String,
  pub name: String,
  pub images: EmoteStaticImages,
  pub format: Vec<EmoteFormat>,
  pub scale: Vec<String>,
  pub theme_mode: Vec<ThemeMode>,
}

#[derive(Serialise, Deserialise, Debug)]
#[serde(crate = "self::serde")]
pub struct ChannelEmotes {
  pub data: Vec<ChannelEmoteData>,
  pub template: String,
}

#[derive(Serialise, Deserialise, Debug)]
#[serde(crate = "self::serde")]
pub struct GlobalEmotes {
  pub data: Vec<GlobalEmoteData>,
  pub template: String,
}

#[derive(Serialise, Deserialise, Debug)]
#[serde(crate = "self::serde")]
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

pub trait EmoteTemplateBuilder {
  fn from_id<S: Into<String>>(&self, id: S) -> EmoteData;
  fn from_idx(&self, idx: usize, animated: bool) -> String;
  fn from_emote(&self, emote: &EmoteData, animated: bool) -> String;
  fn get_emote<S: Into<String>, T: Into<String>, X: Into<String>, Z: Into<String>>(
    &self,
    id: S,
    format: T,
    theme_mode: X,
    scale: Z,
  ) -> String;
}

impl EmoteTemplateBuilder for ChannelEmotes {
  fn from_id<S: Into<String>>(&self, id: S) -> EmoteData {
    let id = id.into();
    self
      .data
      .iter()
      .filter(|emote| id == emote.id)
      .nth(0)
      .unwrap()
      .clone()
      .into()
  }

  fn from_idx(&self, idx: usize, animated: bool) -> String {
    self.from_emote(&self.data[idx].clone().into(), animated)
  }

  fn from_emote(&self, emote: &EmoteData, animated: bool) -> String {
    self.get_emote(
      &emote.id,
      emote.format[if animated { 1 } else { 0 }].string(),
      emote.theme_mode[0].string(),
      &emote.scale[0],
    )
  }

  fn get_emote<S: Into<String>, T: Into<String>, X: Into<String>, Z: Into<String>>(
    &self,
    id: S,
    format: T,
    theme_mode: X,
    scale: Z,
  ) -> String {
    self
      .template
      .replace("{{id}}", &id.into())
      .replace("{{format}}", &format.into())
      .replace("{{theme_mode}}", &theme_mode.into())
      .replace("{{scale}}", &scale.into())
  }
}

impl EmoteTemplateBuilder for GlobalEmotes {
  fn from_id<S: Into<String>>(&self, id: S) -> EmoteData {
    let id = id.into();
    self
      .data
      .iter()
      .filter(|emote| id == emote.id)
      .nth(0)
      .unwrap()
      .clone()
      .into()
  }

  fn from_idx(&self, idx: usize, animated: bool) -> String {
    self.from_emote(&self.data[idx].clone().into(), animated)
  }

  fn from_emote(&self, emote: &EmoteData, animated: bool) -> String {
    self.get_emote(
      &emote.id,
      emote.format[if animated { 1 } else { 0 }].string(),
      emote.theme_mode[0].string(),
      &emote.scale[0],
    )
  }

  fn get_emote<S: Into<String>, T: Into<String>, X: Into<String>, Z: Into<String>>(
    &self,
    id: S,
    format: T,
    theme_mode: X,
    scale: Z,
  ) -> String {
    self
      .template
      .replace("{{id}}", &id.into())
      .replace("{{format}}", &format.into())
      .replace("{{theme_mode}}", &theme_mode.into())
      .replace("{{scale}}", &scale.into())
  }
}
