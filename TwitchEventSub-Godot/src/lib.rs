use std::time::Duration;

use godot::classes::{INode, Node};

use godot::init::EditorRunBehavior;
use godot::prelude::*;

use std::panic;
use twitch_eventsub::*;

mod modules;
use crate::modules::{
  adbreak::*, cheer::*, follow::*, messages::*, raid::*, redeems::*, subscription::*,
};

struct TwitchApi;

#[gdextension]
unsafe impl ExtensionLibrary for TwitchApi {
  fn editor_run_behavior() -> EditorRunBehavior {
    EditorRunBehavior::ToolClassesOnly
  }

  fn on_level_init(level: InitLevel) {
    if level == InitLevel::Core {
      panic::set_hook(Box::new(|p| {
        godot_print!("Twitch Api Panic: {}", p);
      }));
    }
  }
}

#[derive(GodotClass)]
#[class(base=Node)]
struct TwitchEvent {
  #[export]
  chat_message: bool,
  #[export]
  user_update: bool,
  #[export]
  follow: bool,
  #[export]
  raid: bool,
  #[export]
  update: bool,
  #[export]
  new_subscription: bool,
  #[export]
  subscription_end: bool,
  #[export]
  gift_subscription: bool,
  #[export]
  resubscription: bool,
  #[export]
  cheer: bool,
  #[export]
  points_custom_reward_redeem: bool,
  #[export]
  points_auto_reward_redeem: bool,
  #[export]
  poll_begin: bool,
  #[export]
  poll_progress: bool,
  #[export]
  poll_end: bool,
  #[export]
  prediction_begin: bool,
  #[export]
  prediction_progress: bool,
  #[export]
  prediction_lock: bool,
  #[export]
  prediction_end: bool,
  #[export]
  goal_begin: bool,
  #[export]
  goal_progress: bool,
  #[export]
  goal_end: bool,
  #[export]
  hype_train_begin: bool,
  #[export]
  hype_train_progress: bool,
  #[export]
  hype_train_end: bool,
  #[export]
  shoutout_create: bool,
  #[export]
  shoutout_receive: bool,
  #[export]
  ban_timeout_user: bool,
  #[export]
  delete_message: bool,
  #[export]
  ad_break_begin: bool,
  twitch: Option<TwitchEventSubApi>,
  base: Base<Node>,
}

#[derive(GodotClass, Debug, GodotConvert)]
#[godot(transparent)]
#[class(init)]
pub struct GdCustomRewardRedeemContainer {
  pub data: Gd<GCustomRewardRedeem>,
}

#[derive(GodotClass, Debug, GodotConvert)]
#[godot(transparent)]
#[class(init)]
pub struct GdRaidContainer {
  pub data: Gd<GRaid>,
}

#[derive(GodotClass, Debug, GodotConvert)]
#[godot(transparent)]
#[class(init)]
pub struct GdMessageContainer {
  pub data: Gd<GMessageData>,
}

#[derive(GodotClass, Debug, GodotConvert)]
#[godot(transparent)]
#[class(init)]
pub struct GdAdBreakBeginContainer {
  pub data: Gd<GAdBreakBegin>,
}

#[derive(GodotClass, Debug, GodotConvert)]
#[godot(transparent)]
#[class(init)]
pub struct GdFollowContainer {
  pub data: Gd<GFollowData>,
}

#[derive(GodotClass, Debug, GodotConvert)]
#[godot(transparent)]
#[class(init)]
pub struct GdNewSubscriptionContainer {
  pub data: Gd<GNewSubscription>,
}

#[derive(GodotClass, Debug, GodotConvert)]
#[godot(transparent)]
#[class(init)]
pub struct GdGiftContainer {
  pub data: Gd<GGift>,
}

#[derive(GodotClass, Debug, GodotConvert)]
#[godot(transparent)]
#[class(init)]
pub struct GdResubscriptionContainer {
  pub data: Gd<GResubscription>,
}

#[derive(GodotClass, Debug, GodotConvert)]
#[godot(transparent)]
#[class(init)]
pub struct GdCheerContainer {
  pub data: Gd<GCheerData>,
}

#[godot_api]
impl TwitchEvent {
  #[signal]
  fn chat_message(message_data: GdMessageContainer);

  #[signal]
  fn chat_message_powerup_gigantified_emote(messaage_data: GdMessageContainer);

  #[signal]
  fn chat_message_powerup_message_effect(message_data: GdMessageContainer);

  #[signal]
  fn custom_point_reward_redeem(reward: GdCustomRewardRedeemContainer);

  #[signal]
  fn ad_break_start(ad_break_begin: GdAdBreakBeginContainer);

  #[signal]
  fn raid(raid_info: GdRaidContainer);

  #[signal]
  fn follow(follow_data: GdFollowContainer);

  #[signal]
  fn new_subscription(subscription: GdNewSubscriptionContainer);

  #[signal]
  fn subscription_gift(gift_data: GdGiftContainer);

  #[signal]
  fn resubscription(subscription: GdResubscriptionContainer);

  #[signal]
  fn cheer(cheer: GdCheerContainer);
}

#[godot_api]
impl INode for TwitchEvent {
  fn init(base: Base<Node>) -> Self {
    Self {
      twitch: None,
      user_update: false,
      follow: true,
      raid: true,
      update: false,
      new_subscription: true,
      subscription_end: false,
      gift_subscription: true,
      resubscription: true,
      cheer: true,
      points_custom_reward_redeem: true,
      points_auto_reward_redeem: true,
      poll_begin: false,
      poll_progress: false,
      poll_end: false,
      prediction_begin: false,
      prediction_progress: false,
      prediction_lock: false,
      prediction_end: false,
      goal_begin: false,
      goal_progress: false,
      goal_end: false,
      hype_train_begin: false,
      hype_train_progress: false,
      hype_train_end: false,
      shoutout_create: false,
      shoutout_receive: false,
      ban_timeout_user: false,
      delete_message: false,
      ad_break_begin: true,
      chat_message: true,
      base,
    }
  }

  fn ready(&mut self) {
    let keys = TwitchKeys::from_secrets_env().unwrap();
    let redirect_url = "http://localhost:3000";

    let mut twitch = TwitchEventSubApi::builder(keys)
      .set_redirect_url(redirect_url)
      .generate_new_token_if_insufficent_scope(true)
      .generate_new_token_if_none(true)
      .generate_access_token_on_expire(true)
      .auto_save_load_created_tokens(".user_token.env", ".refresh_token.env");

    if self.user_update {
      twitch = twitch.add_subscription(Subscription::UserUpdate);
    }
    if self.follow {
      twitch = twitch.add_subscription(Subscription::ChannelFollow);
    }
    if self.raid {
      twitch = twitch.add_subscription(Subscription::ChannelRaid);
    }
    if self.update {
      twitch = twitch.add_subscription(Subscription::ChannelUpdate);
    }
    if self.new_subscription {
      twitch = twitch.add_subscription(Subscription::ChannelNewSubscription);
    }
    if self.subscription_end {
      twitch = twitch.add_subscription(Subscription::ChannelSubscriptionEnd);
    }
    if self.gift_subscription {
      twitch = twitch.add_subscription(Subscription::ChannelGiftSubscription);
    }
    if self.resubscription {
      twitch = twitch.add_subscription(Subscription::ChannelResubscription);
    }
    if self.cheer {
      twitch = twitch.add_subscription(Subscription::ChannelCheer);
    }
    if self.points_custom_reward_redeem {
      twitch = twitch.add_subscription(Subscription::ChannelPointsCustomRewardRedeem);
    }
    if self.points_auto_reward_redeem {
      twitch = twitch.add_subscription(Subscription::ChannelPointsAutoRewardRedeem);
    }
    if self.poll_begin {
      twitch = twitch.add_subscription(Subscription::ChannelPollBegin);
    }
    if self.poll_progress {
      twitch = twitch.add_subscription(Subscription::ChannelPollProgress);
    }
    if self.poll_end {
      twitch = twitch.add_subscription(Subscription::ChannelPollEnd);
    }
    if self.prediction_begin {
      twitch = twitch.add_subscription(Subscription::ChannelPredictionBegin);
    }
    if self.prediction_progress {
      twitch = twitch.add_subscription(Subscription::ChannelPredictionProgress);
    }
    if self.prediction_lock {
      twitch = twitch.add_subscription(Subscription::ChannelPredictionLock);
    }
    if self.prediction_end {
      twitch = twitch.add_subscription(Subscription::ChannelPredictionEnd);
    }
    if self.goal_begin {
      twitch = twitch.add_subscription(Subscription::ChannelGoalBegin);
    }
    if self.goal_progress {
      twitch = twitch.add_subscription(Subscription::ChannelGoalProgress);
    }
    if self.goal_end {
      twitch = twitch.add_subscription(Subscription::ChannelGoalEnd);
    }
    if self.hype_train_begin {
      twitch = twitch.add_subscription(Subscription::ChannelHypeTrainBegin);
    }
    if self.hype_train_progress {
      twitch = twitch.add_subscription(Subscription::ChannelHypeTrainProgress);
    }
    if self.hype_train_end {
      twitch = twitch.add_subscription(Subscription::ChannelHypeTrainEnd);
    }
    if self.shoutout_create {
      twitch = twitch.add_subscription(Subscription::ChannelShoutoutCreate);
    }
    if self.shoutout_receive {
      twitch = twitch.add_subscription(Subscription::ChannelShoutoutReceive);
    }
    if self.ban_timeout_user {
      twitch = twitch.add_subscription(Subscription::BanTimeoutUser);
    }
    if self.delete_message {
      twitch = twitch.add_subscription(Subscription::DeleteMessage);
    }
    if self.ad_break_begin {
      twitch = twitch.add_subscription(Subscription::AdBreakBegin);
    }
    if self.chat_message {
      twitch = twitch.add_subscription(Subscription::ChatMessage);
    }

    let twitch = twitch.build().unwrap();
    self.twitch = Some(twitch);
  }

  fn process(&mut self, _delta: f64) {
    if let Some(ref mut api) = &mut self.twitch {
      if let Some(message) = api.receive_single_message(Duration::ZERO) {
        match message {
          ResponseType::Event(event) => match event {
            Event::ChatMessage(message_data) => {
              self.base_mut().emit_signal(
                match message_data.message_type {
                  MessageType::PowerUpsGigantifiedEmote => "chat_message_powerup_gigantified_emote",
                  MessageType::PowerUpsMessageEffect => "chat_message_powerup_message_effect",
                  _ => "chat_message",
                }
                .into(),
                &[GdMessageContainer {
                  data: Gd::from_object(GMessageData::from(message_data)),
                }
                .to_variant()],
              );
            }
            Event::Raid(raid_info) => {
              self.base_mut().emit_signal(
                "raid".into(),
                &[GdRaidContainer {
                  data: Gd::from_object(GRaid::from(raid_info)),
                }
                .to_variant()],
              );
            }
            Event::AdBreakBegin(ad_break_data) => {
              self.base_mut().emit_signal(
                "ad_break_start".into(),
                &[GdAdBreakBeginContainer {
                  data: Gd::from_object(GAdBreakBegin::from(ad_break_data)),
                }
                .to_variant()],
              );
            }
            Event::PointsCustomRewardRedeem(custom_reward_redeem) => {
              self.base_mut().emit_signal(
                "custom_point_reward_redeem".into(),
                &[GdCustomRewardRedeemContainer {
                  data: Gd::from_object(GCustomRewardRedeem::from(custom_reward_redeem)),
                }
                .to_variant()],
              );
            }
            Event::Follow(follow_data) => {
              self.base_mut().emit_signal(
                "follow".into(),
                &[GdFollowContainer {
                  data: Gd::from_object(GFollowData::from(follow_data)),
                }
                .to_variant()],
              );
            }
            Event::NewSubscription(new_sub_data) => {
              self.base_mut().emit_signal(
                "new_subscription".into(),
                &[GdNewSubscriptionContainer {
                  data: Gd::from_object(GNewSubscription::from(new_sub_data)),
                }
                .to_variant()],
              );
            }
            Event::GiftSubscription(gift_data) => {
              self.base_mut().emit_signal(
                "gift_subscription".into(),
                &[GdGiftContainer {
                  data: Gd::from_object(GGift::from(gift_data)),
                }
                .to_variant()],
              );
            }
            Event::Resubscription(resub_data) => {
              self.base_mut().emit_signal(
                "resubscription".into(),
                &[GdResubscriptionContainer {
                  data: Gd::from_object(GResubscription::from(resub_data)),
                }
                .to_variant()],
              );
            }
            Event::Cheer(cheer) => {
              self.base_mut().emit_signal(
                "cheer".into(),
                &[GdCheerContainer {
                  data: Gd::from_object(GCheerData::from(cheer)),
                }
                .to_variant()],
              );
            }
            _ => {}
          },
          _ => {}
        }
      }
    }
  }
}
