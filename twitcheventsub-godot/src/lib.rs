use std::io::Read;
use std::panic;
use std::time::Duration;

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
struct TwitchEvent {
  #[export]
  connect_on_ready: bool,
  #[export]
  redirect_url: GString,
  #[export]
  user_token_file: GString,
  #[export]
  refresh_token_file: GString,

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
  moderator_deleted_message: bool,
  #[export]
  ad_break_begin: bool,
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
impl TwitchEvent {
  #[func]
  fn connect_api(&mut self) {
    //let mut keys_window = Window::new_alloc();
    //keys_window.set_title("Secrest not set".into());
    //keys_window.move_to_center();
    //keys_window.move_to_foreground();
    //keys_window.set_flag(Flags::ALWAYS_ON_TOP, true);
    //keys_window.set_flag(Flags::POPUP, true);
    //keys_window.request_attention();
    //keys_window.grab_focus();
    //keys_window.popup_centered();
    //keys_window.set_size(Vector2i::new(400, 150));
    //keys_window.set_position(Vector2i::new(500, 500));

    //let mut grid_two_columns = GridContainer::new_alloc();
    //grid_two_columns.set_columns(2);

    //let mut redirect_url_label = Label::new_alloc();
    //let mut redirect_url_input = TextEdit::new_alloc();
    //redirect_url_label.set_text("Redirect url:".into());
    //redirect_url_input.set_custom_minimum_size(Vector2::new(200.0, 20.0));
    //redirect_url_input.set_text("http://localhost:3000".into());

    //let mut client_id_label = Label::new_alloc();
    //let mut client_id_input = TextEdit::new_alloc();
    //client_id_input.set_custom_minimum_size(Vector2::new(200.0, 20.0));
    //client_id_label.set_text("Client id:".into());

    //let mut client_secret_label = Label::new_alloc();
    //let mut client_secret_input = TextEdit::new_alloc();
    //client_secret_input.set_custom_minimum_size(Vector2::new(200.0, 20.0));
    //client_secret_label.set_text("Client secret:".into());

    //let mut button = Button::new_alloc();

    //button.set_text("Confirm".into());

    //grid_two_columns.add_child(redirect_url_label.upcast());
    //grid_two_columns.add_child(redirect_url_input.upcast());
    //grid_two_columns.add_child(client_id_label.upcast());
    //grid_two_columns.add_child(client_id_input.upcast());
    //grid_two_columns.add_child(client_secret_label.upcast());
    //grid_two_columns.add_child(client_secret_input.upcast());
    //grid_two_columns.add_child(button.upcast());

    //keys_window.add_child(grid_two_columns.upcast());

    //self.base_mut().add_child(keys_window.upcast());
    //keys_window.free();

    let keys = match TwitchKeys::from_secrets_env() {
      Ok(keys) => keys,
      Err(_) => {
        return;
      }
    };

    let mut twitch = TwitchEventSubApi::builder(keys)
      .set_redirect_url(&self.redirect_url)
      .generate_new_token_if_insufficent_scope(true)
      .generate_new_token_if_none(true)
      .generate_access_token_on_expire(true)
      .auto_save_load_created_tokens(&self.user_token_file, &self.refresh_token_file);

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
    if self.moderator_deleted_message {
      twitch = twitch.add_subscription(Subscription::ModeratorDeletedMessage);
    }
    if self.ad_break_begin {
      twitch = twitch.add_subscription(Subscription::AdBreakBegin);
    }
    if self.chat_message {
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
  fn on_message_deleted(message_data: GdMessageDeletedContainer);

  #[signal]
  fn on_chat_message(message_data: GdMessageContainer);

  #[signal]
  fn chat_message_powerup_gigantified_emote(messaage_data: GdMessageContainer);

  #[signal]
  fn on_chat_message_powerup_message_effect(message_data: GdMessageContainer);

  #[signal]
  fn on_custom_point_reward_redeem(reward: GdCustomRewardRedeemContainer);

  #[signal]
  fn on_ad_break_start(ad_break_begin: GdAdBreakBeginContainer);

  #[signal]
  fn on_raid(raid_info: GdRaidContainer);

  #[signal]
  fn on_follow(follow_data: GdFollowContainer);

  #[signal]
  fn on_new_subscription(subscription: GdNewSubscriptionContainer);

  #[signal]
  fn on_subscription_gift(gift_data: GdGiftContainer);

  #[signal]
  fn on_resubscription(subscription: GdResubscriptionContainer);

  #[signal]
  fn on_cheer(cheer: GdCheerContainer);

  #[signal]
  fn on_poll_begin(poll_begin: GdPollBeginContainer);

  #[signal]
  fn on_poll_progress(poll_progress: GdPollProgressContainer);

  #[signal]
  fn on_poll_end(poll_end: GdPollEndContainer);

  #[func]
  pub fn get_chatters(&mut self) -> Gd<GGetChatters> {
    Gd::from_object(GGetChatters::from(
      self.twitch.as_mut().unwrap().get_chatters().unwrap(),
    ))
  }
}

#[godot_api]
impl INode for TwitchEvent {
  fn init(base: Base<Node>) -> Self {
    Self {
      connect_on_ready: true,
      redirect_url: "http://localhost:3000".into(),
      user_token_file: ".user_token.env".into(),
      refresh_token_file: ".refresh_token.env".into(),

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
      moderator_deleted_message: false,
      ad_break_begin: true,
      chat_message: true,
      permission_ban_timeout_user: false,
      permission_delete_message: false,
      permission_read_chatters: false,
      base,
    }
  }

  fn ready(&mut self) {
    if self.connect_on_ready {
      self.connect_api();
    }
  }

  fn process(&mut self, _delta: f64) {
    if let Some(ref mut api) = &mut self.twitch {
      if let Some(message) = api.receive_single_message(Duration::ZERO) {
        match message {
          ResponseType::Event(event) => match event {
            Event::ChatMessage(message_data) => {
              godot_print!("message recieved");
              self.base_mut().emit_signal(
                match message_data.message_type {
                  MessageType::PowerUpsGigantifiedEmote => {
                    "on_chat_message_powerup_gigantified_emote"
                  }
                  MessageType::PowerUpsMessageEffect => "on_chat_message_powerup_message_effect",
                  _ => "on_chat_message",
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
                "on_raid".into(),
                &[GdRaidContainer {
                  data: Gd::from_object(GRaid::from(raid_info)),
                }
                .to_variant()],
              );
            }
            Event::AdBreakBegin(ad_break_data) => {
              self.base_mut().emit_signal(
                "on_ad_break_start".into(),
                &[GdAdBreakBeginContainer {
                  data: Gd::from_object(GAdBreakBegin::from(ad_break_data)),
                }
                .to_variant()],
              );
            }
            Event::PointsCustomRewardRedeem(custom_reward_redeem) => {
              self.base_mut().emit_signal(
                "on_custom_point_reward_redeem".into(),
                &[GdCustomRewardRedeemContainer {
                  data: Gd::from_object(GCustomRewardRedeem::from(custom_reward_redeem)),
                }
                .to_variant()],
              );
            }
            Event::Follow(follow_data) => {
              self.base_mut().emit_signal(
                "on_follow".into(),
                &[GdFollowContainer {
                  data: Gd::from_object(GFollowData::from(follow_data)),
                }
                .to_variant()],
              );
            }
            Event::NewSubscription(new_sub_data) => {
              self.base_mut().emit_signal(
                "on_new_subscription".into(),
                &[GdNewSubscriptionContainer {
                  data: Gd::from_object(GNewSubscription::from(new_sub_data)),
                }
                .to_variant()],
              );
            }
            Event::GiftSubscription(gift_data) => {
              self.base_mut().emit_signal(
                "on_gift_subscription".into(),
                &[GdGiftContainer {
                  data: Gd::from_object(GGift::from(gift_data)),
                }
                .to_variant()],
              );
            }
            Event::Resubscription(resub_data) => {
              self.base_mut().emit_signal(
                "on_resubscription".into(),
                &[GdResubscriptionContainer {
                  data: Gd::from_object(GResubscription::from(resub_data)),
                }
                .to_variant()],
              );
            }
            Event::Cheer(cheer) => {
              self.base_mut().emit_signal(
                "on_cheer".into(),
                &[GdCheerContainer {
                  data: Gd::from_object(GCheerData::from(cheer)),
                }
                .to_variant()],
              );
            }
            Event::PollBegin(begin) => {
              self.base_mut().emit_signal(
                "on_poll_begin".into(),
                &[GdPollBeginContainer {
                  data: Gd::from_object(GPollBegin::from(begin)),
                }
                .to_variant()],
              );
            }
            Event::PollProgress(progress) => {
              self.base_mut().emit_signal(
                "on_poll_progress".into(),
                &[GdPollProgressContainer {
                  data: Gd::from_object(GPollProgress::from(progress)),
                }
                .to_variant()],
              );
            }
            Event::PollEnd(end) => {
              self.base_mut().emit_signal(
                "on_poll_end".into(),
                &[GdPollEndContainer {
                  data: Gd::from_object(GPollEnd::from(end)),
                }
                .to_variant()],
              );
            }
            Event::MessageDeleted(message_deleted) => {
              self.base_mut().emit_signal(
                "on_message_deleted".into(),
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
