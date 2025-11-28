pub mod adbreak;
pub mod badges;
pub mod banned;
pub mod cheer;
pub mod emote;
pub mod follow;
pub mod getchatters;
pub mod messages;
pub mod poll;
pub mod prediction;
pub mod raid;
pub mod redeems;
pub mod subscription;

use godot::prelude::*;
use twitcheventsub::prelude::*;

#[derive(GodotClass, Debug, Clone)]
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
      id: value.id.to_godot(),
      login: value.login.to_godot(),
      name: value.name.to_godot(),
    }
  }
}

impl From<OptionalUser> for GUser {
  fn from(value: OptionalUser) -> Self {
    GUser {
      id: value.id.unwrap_or("anonymous".to_owned()).to_godot(),
      login: value.login.unwrap_or("anonymous".to_owned()).to_godot(),
      name: value.name.unwrap_or("anonymous".to_owned()).to_godot(),
    }
  }
}

#[derive(GodotClass, Debug, Clone)]
#[class(init)]
pub struct GUserData {
  #[var]
  id: GString,
  #[var]
  login: GString,
  #[var]
  name: GString,
  #[var]
  /// User type, might be "admin", "global_mod", "staff" or ""
  user_type: GString,
  #[var]
  /// Broadcaster type, might be "affiliate", "partner" or ""
  broadcaster_type: GString,
  #[var]
  description: GString,
  #[var]
  profile_image_url: GString,
  #[var]
  offline_image_url: GString,
  #[var]
  view_count: u32,
  #[var]
  email: GString,
  #[var]
  created_at: GString,
}

impl From<UserData> for GUserData {
  fn from(value: UserData) -> Self {
    GUserData {
      id: value.id.to_godot(),
      login: value.login.to_godot(),
      name: value.name.to_godot(),
      user_type: match value.user_type {
        UserType::Admin => "admin",
        UserType::GlobalMod => "global_mod",
        UserType::Staff => "staff",
        UserType::Normal => "",
      }
      .into(),
      broadcaster_type: match value.broadcaster_type {
        BroadcasterType::Affiliate => "affiliate",
        BroadcasterType::Partner => "partner",
        BroadcasterType::Normal => "",
      }
      .into(),
      description: value.description.to_godot(),
      profile_image_url: value.profile_image_url.to_godot(),
      offline_image_url: value.offline_image_url.to_godot(),
      view_count: value.view_count,
      email: value.email.unwrap_or_default().to_godot(),
      created_at: value.created_at.to_godot(),
    }
  }
}
