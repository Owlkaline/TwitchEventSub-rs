pub mod adbreak;
pub mod emote;
pub mod follow;
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

impl From<BroadcasterUser> for GUser {
  fn from(value: BroadcasterUser) -> Self {
    GUser {
      id: value.id.into(),
      login: value.login.into(),
      name: value.name.into(),
    }
  }
}

impl From<FromBroadcasterUser> for GUser {
  fn from(value: FromBroadcasterUser) -> Self {
    GUser {
      id: value.id.into(),
      login: value.login.into(),
      name: value.name.into(),
    }
  }
}

impl From<ToBroadcasterUser> for GUser {
  fn from(value: ToBroadcasterUser) -> Self {
    GUser {
      id: value.id.into(),
      login: value.login.into(),
      name: value.name.into(),
    }
  }
}

impl From<RequesterUser> for GUser {
  fn from(value: RequesterUser) -> Self {
    GUser {
      id: value.id.into(),
      login: value.login.into(),
      name: value.name.into(),
    }
  }
}

impl From<RequestUser> for GUser {
  fn from(value: RequestUser) -> Self {
    GUser {
      id: value.id.into(),
      login: value.login.into(),
      name: value.name.into(),
    }
  }
}

impl From<ThreadUser> for GUser {
  fn from(value: ThreadUser) -> Self {
    GUser {
      id: value.id.into(),
      login: value.login.into(),
      name: value.name.into(),
    }
  }
}

impl From<ParentUser> for GUser {
  fn from(value: ParentUser) -> Self {
    GUser {
      id: value.id.into(),
      login: value.login.into(),
      name: value.name.into(),
    }
  }
}

impl From<ChatterUser> for GUser {
  fn from(value: ChatterUser) -> Self {
    GUser {
      id: value.id.into(),
      login: value.login.into(),
      name: value.name.into(),
    }
  }
}
