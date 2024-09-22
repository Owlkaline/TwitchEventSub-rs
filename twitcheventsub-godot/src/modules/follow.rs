use godot::prelude::*;

use twitcheventsub::*;

use crate::modules::GUser;

#[derive(GodotClass, Debug)]
#[class(init)]
pub struct GFollowData {
  #[var]
  user: Gd<GUser>,
  #[var]
  broadcaster: Gd<GUser>,
  #[var]
  followed_at: GString,
}

impl From<FollowData> for GFollowData {
  fn from(follow_data: FollowData) -> GFollowData {
    GFollowData {
      user: Gd::from_object(GUser::from(follow_data.user)),
      broadcaster: Gd::from_object(GUser::from(follow_data.broadcaster)),
      followed_at: follow_data.followed_at.into(),
    }
  }
}
