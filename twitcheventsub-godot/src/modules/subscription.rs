use godot::prelude::*;
use twitcheventsub::prelude::*;

use crate::modules::{emote::GRewardEmote, GUser};

#[derive(GodotClass, Clone)]
#[class(init)]
pub struct GRewardMessage {
  #[var]
  text: GString,
  #[var]
  emotes: Array<Gd<GRewardEmote>>,
}

#[derive(GodotClass, Clone)]
#[class(init)]
pub struct GNewSubscription {
  #[var]
  user: Gd<GUser>,
  #[var]
  broadcaster: Gd<GUser>,
  #[var]
  tier: GString,
  #[var]
  is_gift: bool,
}

#[derive(GodotClass, Clone)]
#[class(init)]
pub struct GResubscription {
  #[var]
  user: Gd<GUser>,
  #[var]
  broadcaster: Gd<GUser>,
  #[var]
  tier: GString,
  #[var]
  message: Gd<GRewardMessage>,
  #[var]
  cumulative_months: u32,
  #[var]
  streak_months: u32,
  #[var]
  duration_months: u32,
}

#[derive(GodotClass, Clone)]
#[class(init)]
pub struct GGift {
  #[var]
  user: Gd<GUser>,
  #[var]
  broadcaster: Gd<GUser>,
  #[var]
  total: u32,
  #[var]
  tier: GString,
  #[var]
  cumulative_total: u32,
  #[var]
  is_anonymous: bool,
}

impl From<GiftData> for GGift {
  fn from(gift: GiftData) -> GGift {
    GGift {
      user: Gd::from_object(GUser::from(gift.user)),
      broadcaster: Gd::from_object(GUser::from(gift.broadcaster)),
      total: gift.total,
      tier: gift.tier.to_godot(),
      cumulative_total: gift.cumulative_total.unwrap_or(0),
      is_anonymous: gift.is_anonymous,
    }
  }
}

impl From<RewardMessageData> for GRewardMessage {
  fn from(reward_message: RewardMessageData) -> GRewardMessage {
    GRewardMessage {
      text: reward_message.text.to_godot(),
      emotes: reward_message
        .emotes
        .unwrap_or(Vec::new())
        .iter()
        .map(|e| Gd::from_object(GRewardEmote::from(e.clone())))
        .collect::<Array<_>>(),
    }
  }
}

impl From<NewSubscriptionData> for GNewSubscription {
  fn from(new_sub: NewSubscriptionData) -> GNewSubscription {
    GNewSubscription {
      user: Gd::from_object(GUser::from(new_sub.user)),
      broadcaster: Gd::from_object(GUser::from(new_sub.broadcaster)),
      tier: new_sub.tier.to_godot(),
      is_gift: new_sub.is_gift,
    }
  }
}

impl From<ResubscriptionData> for GResubscription {
  fn from(resub: ResubscriptionData) -> GResubscription {
    GResubscription {
      user: Gd::from_object(GUser::from(resub.user)),
      broadcaster: Gd::from_object(GUser::from(resub.broadcaster)),
      tier: resub.tier.to_godot(),
      message: Gd::from_object(GRewardMessage::from(resub.message)),
      cumulative_months: resub.cumulative_months,
      streak_months: resub.streak_months.unwrap_or(0),
      duration_months: resub.duration_months,
    }
  }
}
