use godot::prelude::*;
use twitcheventsub::UserBannedData;

use super::GUser;

#[derive(GodotClass, Debug)]
#[class(init)]
pub struct GUserBanned {
  #[var]
  user: Gd<GUser>,
  #[var]
  broadcaster: Gd<GUser>,
  #[var]
  moderator: Gd<GUser>,
  #[var]
  reason: GString,
  #[var]
  banned_at: GString,
  #[var]
  ends_at: Array<GString>,
  #[var]
  is_permanent: bool,
}

impl From<UserBannedData> for GUserBanned {
  fn from(value: UserBannedData) -> Self {
    let mut ends_at = Array::new();

    if let Some(at) = value.ends_at {
      ends_at.push(&at.to_godot());
    }

    GUserBanned {
      user: Gd::from_object(GUser::from(value.user)),
      broadcaster: Gd::from_object(GUser::from(value.broadcaster)),
      moderator: Gd::from_object(GUser::from(value.moderator)),
      reason: value.reason.to_godot(),
      banned_at: value.reason.to_godot(),
      ends_at,
      is_permanent: value.is_permanent,
    }
  }
}
