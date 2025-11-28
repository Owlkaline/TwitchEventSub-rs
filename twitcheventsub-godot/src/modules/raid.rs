use godot::prelude::*;
use twitcheventsub::prelude::*;

use crate::modules::GUser;

#[derive(GodotClass, Debug, Default, Clone)]
#[class(init)]
pub struct GRaid {
  #[var]
  pub from_broadcaster: Gd<GUser>,
  #[var]
  pub to_broadcaster: Gd<GUser>,
  #[var]
  pub viewers: u32,
}

impl From<RaidData> for GRaid {
  fn from(raid: RaidData) -> GRaid {
    GRaid {
      from_broadcaster: Gd::from_object(GUser::from(raid.from_broadcaster)),
      to_broadcaster: Gd::from_object(GUser::from(raid.to_broadcaster)),
      viewers: raid.viewers,
    }
  }
}
