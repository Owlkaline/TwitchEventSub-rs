pub mod adbreak;
pub mod cheer;
pub mod emote;
pub mod follow;
pub mod getchatters;
pub mod messages;
pub mod raid;
pub mod redeems;
pub mod subscription;

use godot::prelude::*;

use twitch_eventsub::*;

#[derive(GodotClass, Debug)]
#[class(init)]
pub struct GUser {
  #[var]
  id: GString,
  #[var]
  name: GString,
  #[var]
  login: GString,
}

impl From<User> for GUser {
  fn from(value: User) -> Self {
    GUser {
      id: value.id.into(),
      login: value.login.into(),
      name: value.name.into(),
    }
  }
}

impl From<OptionalUser> for GUser {
  fn from(value: OptionalUser) -> Self {
    GUser {
      id: value.id.unwrap_or("anonymous".to_owned()).into(),
      login: value.login.unwrap_or("anonymous".to_owned()).into(),
      name: value.name.unwrap_or("anonymous".to_owned()).into(),
    }
  }
}
