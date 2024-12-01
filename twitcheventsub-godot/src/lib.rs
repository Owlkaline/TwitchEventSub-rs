use std::io::Read;
use std::panic;
use std::time::Duration;

use godot::engine::window::WindowInitialPosition;
use godot::engine::{ConfirmationDialog, LineEdit};
use godot::init::EditorRunBehavior;
use godot::prelude::*;
use godot::{
  classes::{self, INode, Image, ImageTexture, Node, SpriteFrames},
  engine::{window::Flags, Button, GridContainer, Label, TextEdit, Window},
  obj::WithBaseField,
};
use image::{EncodableLayout, ImageDecoder};
use log::LevelFilter;
use twitcheventsub::*;

mod modules;
use std::io::Cursor;

use image::{codecs::gif::GifDecoder, AnimationDecoder};

use crate::modules::{
  adbreak::*, cheer::*, emote::*, follow::*, getchatters::*, messages::*, poll::*, raid::*,
  redeems::*, subscription::*, GUser, GUserData,
};

struct TwitchApi;

#[gdextension]
unsafe impl ExtensionLibrary for TwitchApi {
  fn editor_run_behavior() -> EditorRunBehavior {
    EditorRunBehavior::ToolClassesOnly
  }

  fn on_level_init(level: InitLevel) {
    if level == InitLevel::Core {
      let _ = simple_logging::log_to_file("twitch_events.log", LevelFilter::Info);
      panic::set_hook(Box::new(|p| {
        godot_print!("Twitch Api Panic: {}", p);
      }));
    }
  }
}

#[derive(GodotClass)]
#[class(base=Node)]
struct TwitchEventNode {
  #[export]
  channel_chat_message: bool,
  #[export]
  channel_user_update: bool,
  #[export]
  channel_follow: bool,
  #[export]
  channel_raid: bool,
  #[export]
  channel_update: bool,
  #[export]
  channel_new_subscription: bool,
  #[export]
  channel_subscription_end: bool,
  #[export]
  channel_gift_subscription: bool,
  #[export]
  channel_resubscription: bool,
  #[export]
  channel_cheer: bool,
  #[export]
  channel_points_custom_reward_redeem: bool,
  #[export]
  channel_points_auto_reward_redeem: bool,
  #[export]
  channel_poll_begin: bool,
  #[export]
  channel_poll_progress: bool,
  #[export]
  channel_poll_end: bool,
  #[export]
  channel_prediction_begin: bool,
  #[export]
  channel_prediction_progress: bool,
  #[export]
  channel_prediction_lock: bool,
  #[export]
  channel_prediction_end: bool,
  #[export]
  channel_goal_begin: bool,
  #[export]
  channel_goal_progress: bool,
  #[export]
  channel_goal_end: bool,
  #[export]
  channel_hype_train_begin: bool,
  #[export]
  channel_hype_train_progress: bool,
  #[export]
  channel_hype_train_end: bool,
  #[export]
  channel_shoutout_created: bool,
  #[export]
  channel_shoutout_received: bool,
  #[export]
  channel_message_deleted: bool,
  #[export]
  channel_ad_break_begin: bool,
  #[export]
  permission_ban_timeout_user: bool,
  #[export]
  permission_delete_message: bool,
  #[export]
  permission_read_chatters: bool,
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

#[derive(GodotClass, Debug, GodotConvert)]
#[godot(transparent)]
#[class(init)]
pub struct GdGetChattersContainer {
  pub data: Gd<GGetChatters>,
}

#[derive(GodotClass, Debug, GodotConvert)]
#[godot(transparent)]
#[class(init)]
pub struct GdPollBeginContainer {
  pub data: Gd<GPollBegin>,
}

#[derive(GodotClass, Debug, GodotConvert)]
#[godot(transparent)]
#[class(init)]
pub struct GdPollProgressContainer {
  pub data: Gd<GPollProgress>,
}

#[derive(GodotClass, Debug, GodotConvert)]
#[godot(transparent)]
#[class(init)]
pub struct GdPollEndContainer {
  pub data: Gd<GPollEnd>,
}

#[derive(GodotClass, Debug, GodotConvert)]
#[godot(transparent)]
#[class(init)]
pub struct GdMessageDeletedContainer {
  pub data: Gd<GMessageDeleted>,
}

#[godot_api]
impl TwitchEventNode {
  #[func]
  fn get_animated_texture_from_url(
    &mut self,
    url: GString,
    mut animated_sprite: Gd<classes::AnimatedSprite2D>,
    mut sprite_frames: Gd<SpriteFrames>,
  ) {
    let animation_name = StringName::from("emote");
    sprite_frames.add_animation(animation_name.clone());
    sprite_frames.set_animation_loop(animation_name.clone(), true);

    let data = TwitchEventSubApi::get_image_data_from_url(url);
    let mut delay_ms = 0.0;
    if let Ok(gif) = GifDecoder::new(Cursor::new(data.unwrap())) {
      let (width, height) = gif.dimensions();
      let frames = gif.into_frames().collect_frames().unwrap();
      let number_of_frames = frames.len();
      for frame in frames {
        let (n, d) = frame.delay().numer_denom_ms();
        let delay = n as f32 / d as f32;
        delay_ms += delay;
        let data = frame
          .buffer()
          .bytes()
          .map(|a| a.unwrap())
          .collect::<Vec<_>>();
        godot_print!("{}x{}", width, height);
        let image = Image::create_from_data(
          width as i32,
          height as i32,
          false,
          classes::image::Format::RGBA8,
          PackedByteArray::from(data.as_bytes()),
        )
        .unwrap();
        let texture = ImageTexture::create_from_image(image).unwrap();

        //texture2d
        sprite_frames.add_frame(animation_name.clone(), texture.upcast());
      }
      delay_ms /= number_of_frames as f32;
      sprite_frames.set_animation_speed(animation_name, (1000.0 / delay_ms) as f64);
    }

    animated_sprite.set_sprite_frames(sprite_frames);
    animated_sprite.play();
  }

  #[func]
  fn get_static_texture_from_url(&mut self, url: GString) -> Gd<ImageTexture> {
    let data = TwitchEventSubApi::get_image_data_from_url(url);
    let image = image::ImageReader::new(Cursor::new(data.unwrap()))
      .with_guessed_format()
      .unwrap()
      .decode()
      .unwrap()
      .to_rgba8();

    let image = Image::create_from_data(
      image.width() as i32,
      image.height() as i32,
      false,
      classes::image::Format::RGBA8,
      PackedByteArray::from(image.as_bytes()),
    )
    .unwrap();

    let texture = ImageTexture::create_from_image(image);

    return texture.unwrap();
  }

  #[func]
  fn get_emote_url_1x(&mut self, emote: Gd<GEmote>) -> GString {
    self.get_emote_url(emote, EmoteScale::Size1)
  }

  #[func]
  fn get_emote_url_2x(&mut self, emote: Gd<GEmote>) -> GString {
    self.get_emote_url(emote, EmoteScale::Size2)
  }

  #[func]
  fn get_emote_url_3x(&mut self, emote: Gd<GEmote>) -> GString {
    self.get_emote_url(emote, EmoteScale::Size3)
  }

  fn get_emote_url(&mut self, emote: Gd<GEmote>, scale: EmoteScale) -> GString {
    let mut url = String::new();

    if let Some(twitch) = &mut self.twitch {
      let mut builder = EmoteBuilder::builder().animate_or_fallback_on_static();
      match scale {
        EmoteScale::Size1 => {
          builder = builder.scale1();
        }
        EmoteScale::Size2 => {
          builder = builder.scale2();
        }
        EmoteScale::Size3 => {
          builder = builder.scale3();
        }
      }

      if let Some(emote_url) = builder.build(twitch, &emote.bind().convert_to_rust()) {
        url = emote_url.url;
      }
    }

    url.into()
  }

  #[func]
  fn send_chat_message(&mut self, message: GString) {
    if let Some(twitch) = &mut self.twitch {
      let _ = twitch.send_chat_message(message);
    }
  }

  #[func]
  fn send_chat_message_with_reply(&mut self, message: GString, message_id: GString) {
    if let Some(twitch) = &mut self.twitch {
      let message: String = message.into();
      let _ = twitch.send_chat_message_with_reply(message, Some(message_id.into()));
    }
  }

  #[func]
  fn delete_message(&mut self, message_id: GString) {
    if let Some(twitch) = &mut self.twitch {
      let _ = twitch.delete_message(message_id.to_string());
    }
  }

  #[func]
  fn get_users_from_ids(&mut self, id: Array<GString>) -> Array<Gd<GUserData>> {
    let mut gusers = Array::new();

    if let Some(twitch) = &mut self.twitch {
      if let Ok(users) = twitch.get_users_from_ids(id.iter_shared().collect()) {
        for user in users.data {
          gusers.push(Gd::from_object(GUserData::from(user)));
        }
      }
    }

    gusers
  }

  #[func]
  fn get_users_from_logins(&mut self, logins: Array<GString>) -> Array<Gd<GUserData>> {
    let mut gusers = Array::new();

    if let Some(twitch) = &mut self.twitch {
      if let Ok(users) = twitch.get_users_from_logins(logins.iter_shared().collect()) {
        for user in users.data {
          gusers.push(Gd::from_object(GUserData::from(user)));
        }
      }
    }

    gusers
  }

  #[func]
  fn get_ad_schedule(&mut self) -> Array<Gd<GAdDetails>> {
    let mut data = Array::new();

    if let Some(twitch) = &mut self.twitch {
      if let Ok(schedule) = twitch.get_ad_schedule() {
        for details in schedule.data {
          data.push(Gd::from_object(GAdDetails::from(details)));
        }
      }
    }

    data
  }

  #[func]
  /// Get user data by ids or logins. See https://dev.twitch.tv/docs/api/reference/#get-users
  fn get_users_from_self(&mut self) -> Array<Gd<GUserData>> {
    let mut gusers = Array::new();

    if let Some(twitch) = &mut self.twitch {
      if let Ok(users) = twitch.get_users_self() {
        for user in users.data {
          gusers.push(Gd::from_object(GUserData::from(user)));
        }
      }
    }

    gusers
  }

  #[func]
  fn get_moderators(&mut self) -> Array<Gd<GUser>> {
    let mut moderators = Array::new();

    if let Some(twitch) = &mut self.twitch {
      if let Ok(mods) = twitch.get_moderators() {
        for user in mods.data {
          moderators.push(Gd::from_object(GUser::from(user)));
        }
      }
    }

    moderators
  }

  #[func]
  fn get_custom_rewards(&mut self) -> Array<Gd<GGetCustomReward>> {
    let mut rewards = Array::new();

    if let Some(twitch) = &mut self.twitch {
      if let Ok(custom_rewards) = twitch.get_custom_rewards() {
        for reward in custom_rewards.data {
          rewards.push(Gd::from_object(GGetCustomReward::from(reward)));
        }
      }
    }

    rewards
  }

  #[func]
  fn send_announcement(&mut self, message: GString, hex_colour: GString) {
    if let Some(twitch) = &mut self.twitch {
      let _ = twitch.send_announcement(message.to_string(), hex_colour.into());
    }
  }

  #[func]
  fn send_shoutout(&mut self, to_broadcaster_id: GString) {
    if let Some(twitch) = &mut self.twitch {
      let _ = twitch.send_shoutout(to_broadcaster_id);
    }
  }

  #[signal]
  fn message_deleted(message_data: GdMessageDeletedContainer);

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

  #[signal]
  fn poll_begin(poll_begin: GdPollBeginContainer);

  #[signal]
  fn poll_progress(poll_progress: GdPollProgressContainer);

  #[signal]
  fn poll_end(poll_end: GdPollEndContainer);

  #[func]
  pub fn get_chatters(&mut self) -> Gd<GGetChatters> {
    Gd::from_object(GGetChatters::from(
      self.twitch.as_mut().unwrap().get_chatters().unwrap(),
    ))
  }

  pub fn create_popup(&mut self) {
    let mut confirmation = ConfirmationDialog::new_alloc();

    confirmation.set_title("Set Twitch Client Details".into());
    confirmation.set_initial_position(WindowInitialPosition::CENTER_PRIMARY_SCREEN);
    confirmation.set_visible(true);

    let mut grid_container = GridContainer::new_alloc();
    grid_container.set_columns(1);

    let mut client_id_label = Label::new_alloc();
    client_id_label.set_text("Client ID:".into());

    let mut client_id_edit = LineEdit::new_alloc();
    client_id_edit.set_placeholder("Client ID".into());
    client_id_edit.set_clear_button_enabled(true);
    client_id_edit.set_custom_minimum_size(Vector2 { x: 200.0, y: 0.0 });

    let mut client_secret_label = Label::new_alloc();
    client_secret_label.set_text("Client Secret:".into());

    let mut client_secret_edit = LineEdit::new_alloc();
    client_secret_edit.set_placeholder("Client Secret".into());
    client_secret_edit.set_secret(true);

    grid_container.add_child(client_id_label.upcast());
    grid_container.add_child(client_id_edit.upcast());
    grid_container.add_child(client_secret_label.upcast());
    grid_container.add_child(client_secret_edit.upcast());

    confirmation.add_child(grid_container.upcast());

    self.base_mut().add_child(confirmation.upcast());
  }
}

#[godot_api]
impl INode for TwitchEventNode {
  fn init(base: Base<Node>) -> Self {
    Self {
      twitch: None,
      channel_user_update: false,
      channel_follow: true,
      channel_raid: true,
      channel_update: false,
      channel_new_subscription: true,
      channel_subscription_end: false,
      channel_gift_subscription: true,
      channel_resubscription: true,
      channel_cheer: true,
      channel_points_custom_reward_redeem: true,
      channel_points_auto_reward_redeem: true,
      channel_poll_begin: false,
      channel_poll_progress: false,
      channel_poll_end: false,
      channel_prediction_begin: false,
      channel_prediction_progress: false,
      channel_prediction_lock: false,
      channel_prediction_end: false,
      channel_goal_begin: false,
      channel_goal_progress: false,
      channel_goal_end: false,
      channel_hype_train_begin: false,
      channel_hype_train_progress: false,
      channel_hype_train_end: false,
      channel_message_deleted: false,
      channel_shoutout_created: false,
      channel_shoutout_received: false,
      channel_ad_break_begin: true,
      channel_chat_message: true,
      permission_ban_timeout_user: false,
      permission_delete_message: false,
      permission_read_chatters: false,
      base,
    }
  }

  fn ready(&mut self) {
    let keys = match TwitchKeys::from_secrets_env() {
      Ok(keys) => keys,
      Err(_) => {
        self.create_popup();
        return;
      }
    };

    let redirect_url = "http://localhost:3000";

    let mut twitch = TwitchEventSubApi::builder(keys)
      .set_redirect_url(redirect_url)
      .generate_new_token_if_insufficent_scope(true)
      .generate_new_token_if_none(true)
      .generate_access_token_on_expire(true)
      .auto_save_load_created_tokens(".user_token.env", ".refresh_token.env");

    if self.channel_user_update {
      twitch = twitch.add_subscription(Subscription::UserUpdate);
    }
    if self.channel_follow {
      twitch = twitch.add_subscription(Subscription::ChannelFollow);
    }
    if self.channel_raid {
      twitch = twitch.add_subscription(Subscription::ChannelRaid);
    }
    if self.channel_update {
      twitch = twitch.add_subscription(Subscription::ChannelUpdate);
    }
    if self.channel_new_subscription {
      twitch = twitch.add_subscription(Subscription::ChannelNewSubscription);
    }
    if self.channel_subscription_end {
      twitch = twitch.add_subscription(Subscription::ChannelSubscriptionEnd);
    }
    if self.channel_gift_subscription {
      twitch = twitch.add_subscription(Subscription::ChannelGiftSubscription);
    }
    if self.channel_resubscription {
      twitch = twitch.add_subscription(Subscription::ChannelResubscription);
    }
    if self.channel_cheer {
      twitch = twitch.add_subscription(Subscription::ChannelCheer);
    }
    if self.channel_points_custom_reward_redeem {
      twitch = twitch.add_subscription(Subscription::ChannelPointsCustomRewardRedeem);
    }
    if self.channel_points_auto_reward_redeem {
      twitch = twitch.add_subscription(Subscription::ChannelPointsAutoRewardRedeem);
    }
    if self.channel_poll_begin {
      twitch = twitch.add_subscription(Subscription::ChannelPollBegin);
    }
    if self.channel_poll_progress {
      twitch = twitch.add_subscription(Subscription::ChannelPollProgress);
    }
    if self.channel_poll_end {
      twitch = twitch.add_subscription(Subscription::ChannelPollEnd);
    }
    if self.channel_prediction_begin {
      twitch = twitch.add_subscription(Subscription::ChannelPredictionBegin);
    }
    if self.channel_prediction_progress {
      twitch = twitch.add_subscription(Subscription::ChannelPredictionProgress);
    }
    if self.channel_prediction_lock {
      twitch = twitch.add_subscription(Subscription::ChannelPredictionLock);
    }
    if self.channel_prediction_end {
      twitch = twitch.add_subscription(Subscription::ChannelPredictionEnd);
    }
    if self.channel_goal_begin {
      twitch = twitch.add_subscription(Subscription::ChannelGoalBegin);
    }
    if self.channel_goal_progress {
      twitch = twitch.add_subscription(Subscription::ChannelGoalProgress);
    }
    if self.channel_goal_end {
      twitch = twitch.add_subscription(Subscription::ChannelGoalEnd);
    }
    if self.channel_hype_train_begin {
      twitch = twitch.add_subscription(Subscription::ChannelHypeTrainBegin);
    }
    if self.channel_hype_train_progress {
      twitch = twitch.add_subscription(Subscription::ChannelHypeTrainProgress);
    }
    if self.channel_hype_train_end {
      twitch = twitch.add_subscription(Subscription::ChannelHypeTrainEnd);
    }
    if self.channel_shoutout_created {
      twitch = twitch.add_subscription(Subscription::ChannelShoutoutCreate);
    }
    if self.channel_shoutout_received {
      twitch = twitch.add_subscription(Subscription::ChannelShoutoutReceive);
    }
    if self.channel_message_deleted {
      twitch = twitch.add_subscription(Subscription::ChannelMessageDeleted);
    }
    if self.channel_ad_break_begin {
      twitch = twitch.add_subscription(Subscription::AdBreakBegin);
    }
    if self.channel_chat_message {
      twitch = twitch.add_subscription(Subscription::ChatMessage);
    }
    if self.permission_ban_timeout_user {
      twitch = twitch.add_subscription(Subscription::PermissionBanTimeoutUser);
    }
    if self.permission_delete_message {
      twitch = twitch.add_subscription(Subscription::PermissionDeleteMessage);
    }
    if self.permission_read_chatters {
      twitch = twitch.add_subscription(Subscription::PermissionReadChatters);
    }

    let twitch = twitch.build().unwrap();
    self.twitch = Some(twitch);
  }

  fn process(&mut self, _delta: f64) {
    if let Some(ref mut api) = &mut self.twitch {
      if let Some(message) = api.receive_single_message(Duration::ZERO) {
        match message {
          ResponseType::Event(event) => match event {
            TwitchEvent::ChatMessage(message_data) => {
              godot_print!("message recieved");
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
            TwitchEvent::Raid(raid_info) => {
              self.base_mut().emit_signal(
                "raid".into(),
                &[GdRaidContainer {
                  data: Gd::from_object(GRaid::from(raid_info)),
                }
                .to_variant()],
              );
            }
            TwitchEvent::AdBreakBegin(ad_break_data) => {
              self.base_mut().emit_signal(
                "ad_break_start".into(),
                &[GdAdBreakBeginContainer {
                  data: Gd::from_object(GAdBreakBegin::from(ad_break_data)),
                }
                .to_variant()],
              );
            }
            TwitchEvent::PointsCustomRewardRedeem(custom_reward_redeem) => {
              self.base_mut().emit_signal(
                "custom_point_reward_redeem".into(),
                &[GdCustomRewardRedeemContainer {
                  data: Gd::from_object(GCustomRewardRedeem::from(custom_reward_redeem)),
                }
                .to_variant()],
              );
            }
            TwitchEvent::Follow(follow_data) => {
              self.base_mut().emit_signal(
                "follow".into(),
                &[GdFollowContainer {
                  data: Gd::from_object(GFollowData::from(follow_data)),
                }
                .to_variant()],
              );
            }
            TwitchEvent::NewSubscription(new_sub_data) => {
              self.base_mut().emit_signal(
                "new_subscription".into(),
                &[GdNewSubscriptionContainer {
                  data: Gd::from_object(GNewSubscription::from(new_sub_data)),
                }
                .to_variant()],
              );
            }
            TwitchEvent::GiftSubscription(gift_data) => {
              self.base_mut().emit_signal(
                "gift_subscription".into(),
                &[GdGiftContainer {
                  data: Gd::from_object(GGift::from(gift_data)),
                }
                .to_variant()],
              );
            }
            TwitchEvent::Resubscription(resub_data) => {
              self.base_mut().emit_signal(
                "resubscription".into(),
                &[GdResubscriptionContainer {
                  data: Gd::from_object(GResubscription::from(resub_data)),
                }
                .to_variant()],
              );
            }
            TwitchEvent::Cheer(cheer) => {
              self.base_mut().emit_signal(
                "cheer".into(),
                &[GdCheerContainer {
                  data: Gd::from_object(GCheerData::from(cheer)),
                }
                .to_variant()],
              );
            }
            TwitchEvent::PollBegin(begin) => {
              self.base_mut().emit_signal(
                "poll_begin".into(),
                &[GdPollBeginContainer {
                  data: Gd::from_object(GPollBegin::from(begin)),
                }
                .to_variant()],
              );
            }
            TwitchEvent::PollProgress(progress) => {
              self.base_mut().emit_signal(
                "poll_progress".into(),
                &[GdPollProgressContainer {
                  data: Gd::from_object(GPollProgress::from(progress)),
                }
                .to_variant()],
              );
            }
            TwitchEvent::PollEnd(end) => {
              self.base_mut().emit_signal(
                "poll_end".into(),
                &[GdPollEndContainer {
                  data: Gd::from_object(GPollEnd::from(end)),
                }
                .to_variant()],
              );
            }
            TwitchEvent::MessageDeleted(message_deleted) => {
              self.base_mut().emit_signal(
                "message_deleted".into(),
                &[GdMessageDeletedContainer {
                  data: Gd::from_object(GMessageDeleted::from(message_deleted)),
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
