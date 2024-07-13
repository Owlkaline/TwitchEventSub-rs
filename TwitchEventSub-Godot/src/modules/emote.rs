use godot::prelude::*;

use twitch_eventsub::*;

#[derive(GodotClass)]
#[class(init)]
pub struct GRewardEmote {
  #[var]
  id: GString,
  #[var]
  begin: u32,
  #[var]
  end: u32,
}

impl From<RewardEmote> for GRewardEmote {
  fn from(value: RewardEmote) -> Self {
    GRewardEmote {
      id: value.id.into(),
      begin: value.begin,
      end: value.end,
    }
  }
}
