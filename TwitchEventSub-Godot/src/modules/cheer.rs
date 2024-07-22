use godot::prelude::*;

use twitch_eventsub::*;

use crate::modules::GUser;

#[derive(GodotClass, Debug)]
#[class(init)]
pub struct GCheerData {
  #[var]
  user: Gd<GUser>,
  #[var]
  broadcaster: Gd<GUser>,
  #[var]
  is_anonymous: bool,
  #[var]
  message: GString,
  #[var]
  bits: u32,
}

impl From<CheerData> for GCheerData {
  fn from(cheer_data: CheerData) -> GCheerData {
    GCheerData {
      user: Gd::from_object(GUser::from(cheer_data.user)),
      broadcaster: Gd::from_object(GUser::from(cheer_data.broadcaster)),
      is_anonymous: cheer_data.is_anonymous.into(),
      message: cheer_data.message.into(),
      bits: cheer_data.bits,
    }
  }
}
