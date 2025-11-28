use godot::prelude::*;
use twitcheventsub::prelude::*;

use crate::modules::GUser;

#[derive(GodotClass, Debug, Clone)]
#[class(init)]
pub struct GAdBreakBegin {
  #[var]
  broadcaster: Gd<GUser>,
  #[var]
  requester: Gd<GUser>,
  #[var]
  duration_seconds: u32,
  #[var]
  started_at: GString,
  #[var]
  is_automatic: bool,
}

#[derive(GodotClass, Debug, Clone)]
#[class(init)]
pub struct GAdDetails {
  #[var]
  next_ad_at: u32,
  #[var]
  last_ad_at: u32,
  #[var]
  duration: u32,
  #[var]
  preroll_free_time: u32,
  #[var]
  snooze_count: u32,
  #[var]
  snooze_refresh_at: u32,
}

impl From<AdBreakBeginData> for GAdBreakBegin {
  fn from(ad: AdBreakBeginData) -> Self {
    GAdBreakBegin {
      broadcaster: Gd::from_object(GUser::from(ad.broadcaster)),
      requester: Gd::from_object(GUser::from(ad.requester)),
      duration_seconds: ad.duration_seconds,
      started_at: ad.started_at.to_godot(),
      is_automatic: ad.is_automatic,
    }
  }
}

impl From<AdDetails> for GAdDetails {
  fn from(ad: AdDetails) -> Self {
    GAdDetails {
      next_ad_at: ad.next_ad_at,
      last_ad_at: ad.last_ad_at,
      duration: ad.duration,
      preroll_free_time: ad.preroll_free_time,
      snooze_count: ad.snooze_count,
      snooze_refresh_at: ad.snooze_refresh_at,
    }
  }
}
