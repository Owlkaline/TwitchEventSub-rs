pub mod adbreak;
pub mod cheer;
pub mod emote;
pub mod follow;
pub mod getchatters;
pub mod messages;
pub mod poll;
pub mod raid;
pub mod redeems;
pub mod subscription;

use godot::prelude::*;

use twitcheventsub::*;

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

#[derive(GodotClass, Debug)]
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
      id: value.id.into(),
      login: value.login.into(),
      name: value.name.into(),
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
      description: value.description.into(),
      profile_image_url: value.profile_image_url.into(),
      offline_image_url: value.offline_image_url.into(),
      view_count: value.view_count,
      email: value.email.unwrap_or_default().into(),
      created_at: value.created_at.into(),
    }
  }
}
