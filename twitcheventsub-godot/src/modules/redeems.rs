use godot::prelude::*;
use twitcheventsub::*;

use crate::{modules::GUser, GEmoteStaticImages, GRewardMessage};

#[derive(GodotClass, Debug, Default)]
#[class(init)]
pub struct GReward {
  #[var]
  id: GString,
  #[var]
  title: GString,
  #[var]
  prompt: GString,
  #[var]
  cost: u32,
}

#[derive(GodotClass, Debug, Default)]
#[class(init)]
pub struct GCustomRewardRedeem {
  #[var]
  id: GString,
  #[var]
  user: Gd<GUser>,
  #[var]
  broadcaster: Gd<GUser>,
  #[var]
  user_input: GString,
  #[var]
  status: GString,
  #[var]
  reward: Gd<GReward>,
  #[var]
  redeemed_at: GString,
}

#[derive(GodotClass, Debug, Default)]
#[class(init)]
pub struct GGetCustomReward {
  #[var]
  id: GString,
  #[var]
  broadcaster_id: GString,
  #[var]
  broadcaster_login: GString,
  #[var]
  broadcaster_name: GString,
  #[var]
  title: GString,
  #[var]
  image: Gd<GEmoteStaticImages>,
  #[var]
  default_image: Gd<GEmoteStaticImages>,
  #[var]
  background_colour: GString,
  #[var]
  is_enabled: bool,
  #[var]
  cost: u32,
  #[var]
  prompt: GString,
  #[var]
  is_user_input_required: bool,
  #[var]
  is_paused: bool,
  #[var]
  is_in_stock: bool,
  #[var]
  max_per_stream_setting: Gd<GMaxPerStreamSetting>,
  #[var]
  max_per_user_per_stream_setting: Gd<GMaxPerUserPerStreamSetting>,
  #[var]
  global_cooldown_setting: Gd<GGlobalCooldownSetting>,
  #[var]
  should_redemptions_skip_request_queue: bool,
  #[var]
  redemptions_redeemed_current_stream: u32,
  #[var]
  cooldown_expires_at: GString,
}

#[derive(GodotClass, Debug, Default)]
#[class(init)]
pub struct GMaxPerStreamSetting {
  #[var]
  is_enabled: bool,
  #[var]
  max_per_stream: i64,
}

#[derive(GodotClass, Debug, Default)]
#[class(init)]
pub struct GMaxPerUserPerStreamSetting {
  #[var]
  is_enabled: bool,
  #[var]
  max_per_user_per_stream: u32,
}

#[derive(GodotClass, Debug, Default)]
#[class(init)]
pub struct GGlobalCooldownSetting {
  #[var]
  is_enabled: bool,
  #[var]
  global_cooldown_seconds: i64,
}

impl From<CustomPointsRewardRedeemData> for GCustomRewardRedeem {
  fn from(reward: CustomPointsRewardRedeemData) -> GCustomRewardRedeem {
    GCustomRewardRedeem {
      id: reward.id.to_owned().into(),
      user: Gd::from_object(GUser::from(reward.user)),
      broadcaster: Gd::from_object(GUser::from(reward.broadcaster)),
      user_input: reward.user_input.to_owned().into(),
      status: reward.status.to_owned().into(),
      reward: Gd::from_object(GReward::from(reward.reward)),
      redeemed_at: reward.redeemed_at.to_owned().into(),
    }
  }
}

impl From<Reward> for GReward {
  fn from(reward: Reward) -> GReward {
    GReward {
      id: reward.id.to_owned().into(),
      title: reward.title.to_owned().into(),
      prompt: reward.prompt.to_owned().into(),
      cost: reward.cost,
    }
  }
}

impl From<GetCustomReward> for GGetCustomReward {
  fn from(reward: GetCustomReward) -> Self {
    let default_image = Gd::from_object(GEmoteStaticImages::from(reward.default_image));
    GGetCustomReward {
      id: reward.id.into(),
      broadcaster_id: reward.broadcaster_id.into(),
      broadcaster_login: reward.broadcaster_login.into(),
      broadcaster_name: reward.broadcaster_name.into(),
      title: reward.title.into(),
      image: reward.image.map_or_else(
        || default_image.clone(),
        |image| Gd::from_object(image.into()),
      ),
      default_image,
      background_colour: reward.background_colour.into(),
      is_enabled: reward.is_enabled,
      cost: reward.cost,
      prompt: reward.prompt.into(),
      is_user_input_required: reward.is_user_input_required,
      is_paused: reward.is_paused,
      is_in_stock: reward.is_in_stock,
      max_per_stream_setting: Gd::from_object(reward.max_per_stream_setting.into()),
      max_per_user_per_stream_setting: Gd::from_object(
        reward.max_per_user_per_stream_setting.into(),
      ),
      global_cooldown_setting: Gd::from_object(reward.global_cooldown_setting.into()),
      should_redemptions_skip_request_queue: reward.should_redemptions_skip_request_queue,
      redemptions_redeemed_current_stream: reward
        .redemptions_redeemed_current_stream
        .unwrap_or_default(),
      cooldown_expires_at: reward.cooldown_expires_at.unwrap_or_default().into(),
    }
  }
}

impl From<MaxPerStreamSetting> for GMaxPerStreamSetting {
  fn from(setting: MaxPerStreamSetting) -> Self {
    GMaxPerStreamSetting {
      is_enabled: setting.is_enabled,
      max_per_stream: setting.max_per_stream,
    }
  }
}

impl From<MaxPerUserPerStreamSetting> for GMaxPerUserPerStreamSetting {
  fn from(setting: MaxPerUserPerStreamSetting) -> Self {
    GMaxPerUserPerStreamSetting {
      is_enabled: setting.is_enabled,
      max_per_user_per_stream: setting.max_per_user_per_stream,
    }
  }
}

impl From<GlobalCooldownSetting> for GGlobalCooldownSetting {
  fn from(setting: GlobalCooldownSetting) -> Self {
    GGlobalCooldownSetting {
      is_enabled: setting.is_enabled,
      global_cooldown_seconds: setting.global_cooldown_seconds,
    }
  }
}
