use godot::prelude::*;

use twitch_eventsub::*;

use crate::modules::GUser;

#[derive(GodotClass, Debug, Default)]
#[class(init)]
pub struct GReward {
  #[var]
  id: GString,
  #[var]
  title: GString,
  #[var]
  prompt: GString,
  #[var]
  cost: u32,
}

#[derive(GodotClass, Debug, Default)]
#[class(init)]
pub struct GCustomRewardRedeem {
  #[var]
  id: GString,
  #[var]
  user: Gd<GUser>,
  #[var]
  broadcaster: Gd<GUser>,
  #[var]
  user_input: GString,
  #[var]
  status: GString,
  #[var]
  reward: Gd<GReward>,
  #[var]
  redeemed_at: GString,
}

impl From<CustomPointsRewardRedeemData> for GCustomRewardRedeem {
  fn from(reward: CustomPointsRewardRedeemData) -> GCustomRewardRedeem {
    GCustomRewardRedeem {
      id: reward.id.to_owned().into(),
      user: Gd::from_object(GUser::from(reward.user)),
      broadcaster: Gd::from_object(GUser::from(reward.broadcaster)),
      user_input: reward.user_input.to_owned().into(),
      status: reward.status.to_owned().into(),
      reward: Gd::from_object(GReward::from(reward.reward)),
      redeemed_at: reward.redeemed_at.to_owned().into(),
    }
  }
}

impl From<Reward> for GReward {
  fn from(reward: Reward) -> GReward {
    GReward {
      id: reward.id.to_owned().into(),
      title: reward.title.to_owned().into(),
      prompt: reward.prompt.to_owned().into(),
      cost: reward.cost,
    }
  }
}
