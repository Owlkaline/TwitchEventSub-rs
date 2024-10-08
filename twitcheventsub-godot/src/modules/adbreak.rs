use godot::prelude::*;

use twitcheventsub::*;

use crate::modules::GUser;

#[derive(GodotClass, Debug)]
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

impl From<AdBreakBeginData> for GAdBreakBegin {
  fn from(ad: AdBreakBeginData) -> Self {
    GAdBreakBegin {
      broadcaster: Gd::from_object(GUser::from(ad.broadcaster)),
      requester: Gd::from_object(GUser::from(ad.requester)),
      duration_seconds: ad.duration_seconds,
      started_at: ad.started_at.to_owned().into(),
      is_automatic: ad.is_automatic,
    }
  }
}
