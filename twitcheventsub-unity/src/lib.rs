extern crate rand;

use std::ffi::{c_char, CStr, CString};
use std::mem::transmute;

use serde::Deserialize as Deserialise;

use twitcheventsub::*;

pub struct TwitchEvents {
  api: TwitchEventSubApi,
}

#[derive(Debug, Deserialise, Default)]
pub struct UnitySubscriptions {
  chat_message: bool,
  user_update: bool,
  follow: bool,
  raid: bool,
  update: bool,
  new_subscription: bool,
  subscription_end: bool,
  gift_subscription: bool,
  resubscription: bool,
  cheer: bool,
  points_custom_reward_redeem: bool,
  points_auto_reward_redeem: bool,
  poll_begin: bool,
  poll_progress: bool,
  poll_end: bool,
  prediction_begin: bool,
  prediction_progress: bool,
  prediction_lock: bool,
  prediction_end: bool,
  goal_begin: bool,
  goal_progress: bool,
  goal_end: bool,
  hype_train_begin: bool,
  hype_train_progress: bool,
  hype_train_end: bool,
  shoutout_create: bool,
  shoutout_receive: bool,
  ban_timeout_user: bool,
  delete_message: bool,
  ad_break_begin: bool,
}

impl UnitySubscriptions {
  pub fn new() -> UnitySubscriptions {
    UnitySubscriptions {
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
    }
  }
}

#[no_mangle]
pub extern "C" fn create_twitch_events(subscriptions: *const c_char) -> *mut TwitchEvents {
  let keys = TwitchKeys::from_secrets_env().unwrap();
  let subscriptions: UnitySubscriptions =
    serde_json::from_str(unsafe { CStr::from_ptr(subscriptions) }.to_str().unwrap()).unwrap();

  let mut twitch = TwitchEventSubApi::builder(keys.clone())
    .set_redirect_url("http://localhost:3000")
    .generate_new_token_if_insufficent_scope(true)
    .generate_new_token_if_none(true)
    .generate_access_token_on_expire(true)
    .auto_save_load_created_tokens(".user_token.env", ".refresh_token.env");

  if subscriptions.user_update {
    twitch = twitch.add_subscription(Subscription::UserUpdate);
  }
  if subscriptions.follow {
    twitch = twitch.add_subscription(Subscription::ChannelFollow);
  }
  if subscriptions.raid {
    twitch = twitch.add_subscription(Subscription::ChannelRaid);
  }
  if subscriptions.update {
    twitch = twitch.add_subscription(Subscription::ChannelUpdate);
  }
  if subscriptions.new_subscription {
    twitch = twitch.add_subscription(Subscription::ChannelNewSubscription);
  }
  if subscriptions.subscription_end {
    twitch = twitch.add_subscription(Subscription::ChannelSubscriptionEnd);
  }
  if subscriptions.gift_subscription {
    twitch = twitch.add_subscription(Subscription::ChannelGiftSubscription);
  }
  if subscriptions.resubscription {
    twitch = twitch.add_subscription(Subscription::ChannelResubscription);
  }
  if subscriptions.cheer {
    twitch = twitch.add_subscription(Subscription::ChannelCheer);
  }
  if subscriptions.points_custom_reward_redeem {
    twitch = twitch.add_subscription(Subscription::ChannelPointsCustomRewardRedeem);
  }
  if subscriptions.points_auto_reward_redeem {
    twitch = twitch.add_subscription(Subscription::ChannelPointsAutoRewardRedeem);
  }
  if subscriptions.poll_begin {
    twitch = twitch.add_subscription(Subscription::ChannelPollBegin);
  }
  if subscriptions.poll_progress {
    twitch = twitch.add_subscription(Subscription::ChannelPollProgress);
  }
  if subscriptions.poll_end {
    twitch = twitch.add_subscription(Subscription::ChannelPollEnd);
  }
  if subscriptions.prediction_begin {
    twitch = twitch.add_subscription(Subscription::ChannelPredictionBegin);
  }
  if subscriptions.prediction_progress {
    twitch = twitch.add_subscription(Subscription::ChannelPredictionProgress);
  }
  if subscriptions.prediction_lock {
    twitch = twitch.add_subscription(Subscription::ChannelPredictionLock);
  }
  if subscriptions.prediction_end {
    twitch = twitch.add_subscription(Subscription::ChannelPredictionEnd);
  }
  if subscriptions.goal_begin {
    twitch = twitch.add_subscription(Subscription::ChannelGoalBegin);
  }
  if subscriptions.goal_progress {
    twitch = twitch.add_subscription(Subscription::ChannelGoalProgress);
  }
  if subscriptions.goal_end {
    twitch = twitch.add_subscription(Subscription::ChannelGoalEnd);
  }
  if subscriptions.hype_train_begin {
    twitch = twitch.add_subscription(Subscription::ChannelHypeTrainBegin);
  }
  if subscriptions.hype_train_progress {
    twitch = twitch.add_subscription(Subscription::ChannelHypeTrainProgress);
  }
  if subscriptions.hype_train_end {
    twitch = twitch.add_subscription(Subscription::ChannelHypeTrainEnd);
  }
  if subscriptions.shoutout_create {
    twitch = twitch.add_subscription(Subscription::ChannelShoutoutCreate);
  }
  if subscriptions.shoutout_receive {
    twitch = twitch.add_subscription(Subscription::ChannelShoutoutReceive);
  }
  if subscriptions.ban_timeout_user {
    twitch = twitch.add_subscription(Subscription::PermissionBanTimeoutUser);
  }
  if subscriptions.delete_message {
    twitch = twitch.add_subscription(Subscription::PermissionDeleteMessage);
  }
  if subscriptions.ad_break_begin {
    twitch = twitch.add_subscription(Subscription::AdBreakBegin);
  }
  if subscriptions.chat_message {
    twitch = twitch.add_subscription(Subscription::ChatMessage);
  }

  let twitch = twitch.build().unwrap();

  unsafe { transmute(Box::new(twitch)) }
}

#[repr(C)]
pub struct EventData {
  pub kind: CString,
  pub json: CString,
}

impl EventData {
  pub fn new() -> EventData {
    EventData {
      kind: CString::new("").unwrap(),
      json: CString::new("").unwrap(),
    }
  }

  pub fn kind(&self) -> CString {
    self.kind.to_owned()
  }

  pub fn json(&self) -> CString {
    self.json.to_owned()
  }
}

#[no_mangle]
pub extern "C" fn extract_type(event_data: *mut EventData) -> CString {
  let event = unsafe { &mut *event_data };

  event.kind()
}

#[no_mangle]
pub extern "C" fn extract_json(event_data: *mut EventData) -> CString {
  let event = unsafe { &mut *event_data };

  event.json()
}

#[no_mangle]
pub extern "C" fn get_event(twitch: *mut TwitchEvents) -> *mut EventData {
  let twitch = unsafe { &mut *twitch };
  let mut event = EventData::new();

  for response in twitch.api.receive_all_messages(None) {
    match response {
      ResponseType::Event(event_a) => match event_a {
        TwitchEvent::ChatMessage(message_data) => {
          println!("chat message recieved");
          event.kind = CString::new(match message_data.message_type {
            MessageType::PowerUpsGigantifiedEmote => "chat_message_powerup_gigantified_emote",
            MessageType::PowerUpsMessageEffect => "chat_message_powerup_message_effect",
            _ => "chat_message",
          })
          .unwrap();
          event.json = CString::new(serde_json::to_string(&message_data).unwrap()).unwrap();
        }
        TwitchEvent::PointsCustomRewardRedeem(custom_reward_redeem) => {
          event.kind = CString::new("custom_point_reward_redeem").unwrap();
          event.json = CString::new(serde_json::to_string(&custom_reward_redeem).unwrap()).unwrap();
        }
        TwitchEvent::AdBreakBegin(ad_break_begin) => {
          event.kind = CString::new("ad_break_start").unwrap();
          event.json = CString::new(serde_json::to_string(&ad_break_begin).unwrap()).unwrap();
        }
        TwitchEvent::Raid(raid) => {
          event.kind = CString::new("raid").unwrap();
          event.json = CString::new(serde_json::to_string(&raid).unwrap()).unwrap();
        }
        TwitchEvent::Follow(follow) => {
          event.kind = CString::new("follow").unwrap();
          event.json = CString::new(serde_json::to_string(&follow).unwrap()).unwrap();
        }
        TwitchEvent::NewSubscription(new_subscritpion) => {
          event.kind = CString::new("new_subscription").unwrap();
          event.json = CString::new(serde_json::to_string(&new_subscritpion).unwrap()).unwrap();
        }
        TwitchEvent::GiftSubscription(gift) => {
          event.kind = CString::new("subscription_gift").unwrap();
          event.json = CString::new(serde_json::to_string(&gift).unwrap()).unwrap();
        }
        TwitchEvent::Resubscription(resubscription) => {
          event.kind = CString::new("resubscritpion").unwrap();
          event.json = CString::new(serde_json::to_string(&resubscription).unwrap()).unwrap();
        }
        TwitchEvent::Cheer(cheer) => {
          event.kind = CString::new("cheer").unwrap();
          event.json = CString::new(serde_json::to_string(&cheer).unwrap()).unwrap();
        }
        _ => {}
      },

      ResponseType::RawResponse(_raw_string) => {
        //event.kind = CString::new("RawResponse").unwrap();
        //event.json = CString::new(raw_string).unwrap();
      }
      _ => {}
    }
  }

  unsafe { transmute(Box::new(event)) }
}

#[no_mangle]
pub extern "C" fn destroy_twitch_events(twitch_events: *mut TwitchEvents) {
  let _twitch: Box<TwitchEvents> = unsafe { transmute(twitch_events) };
}
