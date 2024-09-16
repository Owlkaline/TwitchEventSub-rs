use crate::serde::{self, Deserialize as Deserialise, Serialize as Serialise};
use crate::TwitchEventSubApi;

use twitch_eventsub_structs::User;
use twitch_eventsub_structs::{prefix_broadcaster, Emote};

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

#[derive(Serialise, Deserialise, Debug)]
#[serde(crate = "self::serde")]
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

pub struct EmoteBuilder {
  scale: EmoteScale,
  theme: ThemeMode,
  format: EmoteFormat,
  fallback_on_format_missing: bool,
}

impl EmoteBuilder {
  pub fn builder() -> EmoteBuilder {
    EmoteBuilder {
      scale: EmoteScale::Size1,
      theme: ThemeMode::Light,
      format: EmoteFormat::Static,
      fallback_on_format_missing: false,
    }
  }

  pub fn animate_or_fallback_on_static(mut self) -> EmoteBuilder {
    self.fallback_on_format_missing = true;
    self.animated()
  }

  pub fn format_static(mut self) -> EmoteBuilder {
    self.format = EmoteFormat::Static;
    self
  }

  pub fn animated(mut self) -> EmoteBuilder {
    self.format = EmoteFormat::Animated;
    self
  }

  pub fn light(mut self) -> EmoteBuilder {
    self.theme = ThemeMode::Light;
    self
  }

  pub fn dark(mut self) -> EmoteBuilder {
    self.theme = ThemeMode::Dark;
    self
  }

  pub fn scale1(mut self) -> EmoteBuilder {
    self.scale = EmoteScale::Size1;
    self
  }

  pub fn scale2(mut self) -> EmoteBuilder {
    self.scale = EmoteScale::Size2;
    self
  }

  pub fn scale3(mut self) -> EmoteBuilder {
    self.scale = EmoteScale::Size3;
    self
  }

  pub fn build(&mut self, twitch: &mut TwitchEventSubApi, emote: &Emote) -> Option<EmoteUrl> {
    let channel_id: String = emote.owner_id.to_owned().unwrap_or("".to_string());
    let mut template = String::new();

    let mut emote_data: Option<EmoteData> = None;

    let mut suffix = String::new();
    let id_array = emote.id.split('_').collect::<Vec<_>>();

    let mut real_id = id_array[0].to_string();

    if id_array.len() > 1 {
      real_id = format!("{}_{}", id_array[0], id_array[1]);
      if id_array.len() > 2 {
        suffix = id_array[2].to_string();
      }
    }

    if let Ok(channel_emotes) = twitch.get_channel_emotes(channel_id) {
      template = channel_emotes.template;

      let mut valid_emotes = channel_emotes
        .data
        .into_iter()
        .filter_map(|d| if d.id == real_id { Some(d) } else { None })
        .collect::<Vec<_>>();

      if valid_emotes.len() > 0 {
        emote_data = Some(valid_emotes.remove(0).into());
      }
    }

    if emote_data.is_none() {
      if let Ok(emote_sets) = twitch.get_emote_sets(emote.emote_set_id.to_owned()) {
        template = emote_sets.template;

        let mut valid_emotes = emote_sets
          .data
          .into_iter()
          .filter_map(|d| if d.id == real_id { Some(d) } else { None })
          .collect::<Vec<_>>();
        if valid_emotes.len() > 0 {
          emote_data = Some(valid_emotes.remove(0).into());
        }
      }

      if emote_data.is_none() {
        if let Ok(global_emotes) = twitch.get_global_emotes() {
          template = global_emotes.template;
          let mut valid_emotes = global_emotes
            .data
            .into_iter()
            .filter_map(|d| if d.id == real_id { Some(d) } else { None })
            .collect::<Vec<_>>();
          if valid_emotes.len() > 0 {
            emote_data = Some(valid_emotes.remove(0).into());
          }
        }
      }
    }

    if let Some(mut emote_data) = emote_data {
      if !suffix.is_empty() {
        emote_data.id = format!("{}_{}", real_id, suffix);
      }

      if !emote_data.format.contains(&self.format) {
        if self.fallback_on_format_missing {
          self.format = EmoteFormat::Static;
        } else {
          return None;
        }
      }

      let url = template
        .replace("{{id}}", &emote_data.id)
        .replace("{{format}}", &self.format.string())
        .replace("{{theme_mode}}", &self.theme.string())
        .replace("{{scale}}", &emote_data.scale[self.scale.idx()]);

      Some(EmoteUrl {
        url,
        animated: self.format == EmoteFormat::Animated,
      })
    } else {
      None
    }
  }
}
