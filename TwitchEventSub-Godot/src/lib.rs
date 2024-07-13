use std::time::Duration;

use godot::classes::{INode, Node};

use godot::init::EditorRunBehavior;
use godot::prelude::*;

use std::panic;
use twitch_eventsub::*;

mod modules;
use crate::modules::{adbreak::*, follow::*, messages::*, raid::*, redeems::*, subscription::*};

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
}

#[godot_api]
impl INode for TwitchEvent {
  fn init(base: Base<Node>) -> Self {
    Self { twitch: None, base }
  }

  fn ready(&mut self) {
    let keys = TwitchKeys::from_secrets_env().unwrap();
    let redirect_url = "http://localhost:3000";

    let twitch = TwitchEventSubApi::builder(keys)
      .set_redirect_url(redirect_url)
      .generate_new_token_if_insufficent_scope(true)
      .generate_new_token_if_none(true)
      .generate_access_token_on_expire(true)
      .auto_save_load_created_tokens(".user_token.env", ".refresh_token.env")
      .add_subscription(Subscription::ChatMessage)
      .add_subscription(Subscription::ChannelPointsCustomRewardRedeem)
      .add_subscription(Subscription::AdBreakBegin)
      .add_subscription(Subscription::ChannelRaid)
      .add_subscription(Subscription::ChannelFollow)
      .build()
      .unwrap();
    self.twitch = Some(twitch);
  }

  fn process(&mut self, _delta: f64) {
    if let Some(ref mut api) = &mut self.twitch {
      for message in api.receive_messages(Duration::ZERO) {
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
            _ => {}
          },
          _ => {}
        }
      }
    }
  }
}
