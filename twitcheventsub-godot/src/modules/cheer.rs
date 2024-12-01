use godot::prelude::*;

use twitcheventsub::*;

use crate::modules::GUser;

#[derive(GodotClass)]
#[class(init)]
pub struct GCheerMote {
  #[var]
  pub prefix: GString,
  #[var]
  pub bits: u32,
  #[var]
  pub tier: u32,
}

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
      user: Gd::from_object(GUser::from(cheer_data.user.unwrap_or(User::default()))),
      broadcaster: Gd::from_object(GUser::from(cheer_data.broadcaster)),
      is_anonymous: cheer_data.is_anonymous.into(),
      message: cheer_data.message.into(),
      bits: cheer_data.bits,
    }
  }
}
impl From<CheerMote> for GCheerMote {
  fn from(value: CheerMote) -> Self {
    GCheerMote {
      prefix: value.prefix.into(),
      bits: value.bits.into(),
      tier: value.tier.into(),
    }
  }
}
