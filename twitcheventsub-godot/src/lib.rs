use std::fs;
use std::io::Read;
use std::io::Write;
use std::panic;
use std::time::Duration;

use godot::classes::control::FocusMode;
use godot::classes::link_button::UnderlineMode;
use godot::classes::tween::EaseType;
use godot::classes::tween::TransitionType;
use godot::classes::window::WindowInitialPosition;
use godot::classes::AspectRatioContainer;
use godot::classes::Button;
use godot::classes::LinkButton;
use godot::classes::Panel;
use godot::classes::TextureRect;
use godot::classes::Tween;
use godot::classes::VBoxContainer;
use godot::classes::{AnimatedTexture, ConfirmationDialog, GridContainer, Label, LineEdit};
use godot::init::EditorRunBehavior;
use godot::meta::ParamType;
use godot::prelude::*;
use godot::{
  classes::{self, INode, Image, ImageTexture, Node},
  obj::WithBaseField,
};
use image::{EncodableLayout, ImageDecoder};
use log::LevelFilter;
use modules::badges::GBadgeVersion;
use modules::badges::GSetOfBadges;
use modules::banned::GUserBanned;
use twitcheventsub::prelude::twitcheventsub_api::TwitchApiError;
use twitcheventsub::prelude::twitcheventsub_tokens::TokenHandler;
use twitcheventsub::prelude::twitcheventsub_tokens::TokenHandlerBuilder;
use twitcheventsub::prelude::*;
use twitcheventsub::EventSubError;
use twitcheventsub::ResponseType;
use twitcheventsub::TwitchEventSubApi;
use twitcheventsub::TwitchEventSubApiBuilder;

mod modules;
use std::io::Cursor;

use image::{codecs::gif::GifDecoder, AnimationDecoder};

use crate::modules::{
  adbreak::*, cheer::*, emote::*, follow::*, getchatters::*, messages::*, poll::*, prediction::*,
  raid::*, redeems::*, subscription::*, GUser, GUserData,
};

const VISUAL_ERROR: &[u8] = include_bytes!("../assets/visual_error.png");

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

// TODO:
// Save in godot local area place
#[derive(GodotClass)]
#[class(base=Node)]
struct TwitchEventNode {
  #[export]
  start_onready: bool,
  #[export]
  show_connected_notification: bool,
  #[export]
  broadcaster_username: GString,
  #[export]
  redirect_url: GString,
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
  channel_user_banned: bool,
  #[export]
  permission_ban_timeout_user: bool,
  #[export]
  permission_delete_message: bool,
  #[export]
  permission_read_chatters: bool,
  #[export]
  env_file_name: GString,
  client_id_field: Option<Gd<LineEdit>>,
  client_secret_field: Option<Gd<LineEdit>>,
  broadcaster_id_field: Option<Gd<LineEdit>>,
  redirect_url_field: Option<Gd<LineEdit>>,
  need_help: Option<Gd<AspectRatioContainer>>,
  token: TokenHandler,
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
pub struct GdPredictionBeginContainer {
  pub data: Gd<GPredictionBegin>,
}

#[derive(GodotClass, Debug, GodotConvert)]
#[godot(transparent)]
#[class(init)]
pub struct GdPredictionProgressContainer {
  pub data: Gd<GPredictionProgress>,
}

#[derive(GodotClass, Debug, GodotConvert)]
#[godot(transparent)]
#[class(init)]
pub struct GdPredictionLockContainer {
  pub data: Gd<GPredictionLock>,
}

#[derive(GodotClass, Debug, GodotConvert)]
#[godot(transparent)]
#[class(init)]
pub struct GdPredictionEndContainer {
  pub data: Gd<GPredictionEnd>,
}

#[derive(GodotClass, Debug, GodotConvert)]
#[godot(transparent)]
#[class(init)]
pub struct GdMessageDeletedContainer {
  pub data: Gd<GMessageDeleted>,
}

#[derive(GodotClass, Debug, GodotConvert)]
#[godot(transparent)]
#[class(init)]
pub struct GdUserBannedContainer {
  pub data: Gd<GUserBanned>,
}

#[godot_api]
impl TwitchEventNode {
  //#[func]
  //fn get_generic_emote_texture_from_url(&mut self, url: Gd<GEmoteUrl>) -> Variant {
  //  if url.bind().animated {
  //    self
  //      .get_animated_texture_from_url(url.bind().url.to_owned())
  //      .to_variant()
  //  } else {
  //    self
  //      .get_static_texture_from_url(url.bind().url.to_owned())
  //      .to_variant()
  //  }
  //}

  //#[func]
  //fn get_animated_texture_from_url(&mut self, url: GString) -> Gd<AnimatedTexture> {
  //  let mut animated_texture = AnimatedTexture::new_gd();

  //  let data = TwitchEventSubApi::get_image_data_from_url(url);

  //  if let Ok(gif) = GifDecoder::new(Cursor::new(data.unwrap())) {
  //    let (width, height) = gif.dimensions();
  //    let frames = gif.into_frames().collect_frames().unwrap();
  //    let number_of_frames = frames.len();

  //    let mut textures = Vec::new();
  //    let mut frame_duartion_ms = Vec::new();
  //    for frame in frames.into_iter() {
  //      let (n, d) = frame.delay().numer_denom_ms();

  //      let delay = Duration::from_millis(
  //        if n == 0 || d == 0 {
  //          100
  //        } else {
  //          n as u64 / d as u64
  //        },
  //      )
  //      .as_secs_f32();

  //      let data = frame
  //        .buffer()
  //        .bytes()
  //        .map(|a| a.unwrap())
  //        .collect::<Vec<_>>();

  //      let image = Image::create_from_data(
  //        width as i32,
  //        height as i32,
  //        false,
  //        classes::image::Format::RGBA8,
  //        &PackedByteArray::from(data.as_bytes()),
  //      )
  //      .unwrap();
  //      let texture = ImageTexture::create_from_image(&image).unwrap();
  //      textures.push(texture);
  //      frame_duartion_ms.push(delay);
  //    }

  //    {
  //      animated_texture.set_frames(number_of_frames as i32);
  //      for i in 0..number_of_frames {
  //        animated_texture.set_frame_texture(i as i32, &textures[i]);
  //        animated_texture.set_frame_duration(i as i32, frame_duartion_ms[i]);
  //      }
  //    }
  //  }

  //  animated_texture
  //}

  //#[func]
  //fn get_static_texture_from_url(&mut self, url: GString) -> Gd<ImageTexture> {
  //  let data = TwitchEventSubApi::get_image_data_from_url(url);
  //  let image = image::ImageReader::new(Cursor::new(data.unwrap()))
  //    .with_guessed_format()
  //    .unwrap()
  //    .decode()
  //    .unwrap()
  //    .to_rgba8();

  //  let image = Image::create_from_data(
  //    image.width() as i32,
  //    image.height() as i32,
  //    false,
  //    classes::image::Format::RGBA8,
  //    &PackedByteArray::from(image.as_bytes()),
  //  )
  //  .unwrap();

  //  let texture = ImageTexture::create_from_image(&image);

  //  return texture.unwrap();
  //}

  //#[func]
  //fn get_badges_urls(&mut self, badges: Array<Gd<GBadge>>) -> Array<Gd<GBadgeVersion>> {
  //  let mut requested_badges = Array::new();

  //  let badges: Vec<Badge> = badges
  //    .iter_shared()
  //    .map(|b| (*b.bind()).clone().into())
  //    .collect::<Vec<_>>();
  //  if let Some(twitch) = &mut self.twitch {
  //    for badge in twitch.get_badge_urls_from_badges(badges) {
  //      requested_badges.push(&Gd::from_object(GBadgeVersion::from(badge)));
  //    }
  //  }

  //  requested_badges
  //}

  #[func]
  fn get_emote_url_1x(&mut self, fragment: Gd<GFragments>) -> Gd<GEmoteUrl> {
    self.get_emote_url(fragment, EmoteScale::Size1)
  }

  #[func]
  fn get_emote_url_2x(&mut self, fragment: Gd<GFragments>) -> Gd<GEmoteUrl> {
    self.get_emote_url(fragment, EmoteScale::Size2)
  }

  #[func]
  fn get_emote_url_3x(&mut self, fragment: Gd<GFragments>) -> Gd<GEmoteUrl> {
    self.get_emote_url(fragment, EmoteScale::Size3)
  }

  fn get_emote_url(&mut self, fragment: Gd<GFragments>, scale: EmoteScale) -> Gd<GEmoteUrl> {
    let mut url = GEmoteUrl {
      url: GString::new(),
      animated: false,
    };

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

      let id = twitch.broadcaster().id.clone();

      if let Some(emote_url) = builder.build(
        &twitch.get_tokens(),
        &id,
        &fragment.bind().convert_to_rust(),
        &mut twitch.bttv,
      ) {
        url.url = emote_url.url.into(); //.url;
        url.animated = emote_url.animated;
      }
    }

    Gd::from_object(url)
  }

  #[func]
  fn send_chat_message(&mut self, message: GString) {
    if let Some(twitch) = &mut self.twitch {
      let id = twitch.broadcaster().id.clone();
      let _ = twitch.api().send_chat_message(&id, &message.to_string());
    }
  }

  #[func]
  fn send_chat_message_with_reply(&mut self, message: GString, message_id: GString) {
    if let Some(twitch) = &mut self.twitch {
      let message: String = message.into();
      let id = twitch.broadcaster().id.clone();
      let _ = twitch.api().send_chat_message_with_reply(
        &id,
        &message.to_string(),
        Some(message_id.into()),
      );
    }
  }

  #[func]
  fn delete_message(&mut self, message_id: GString) {
    if let Some(twitch) = &mut self.twitch {
      let id = twitch.broadcaster().id.clone();
      let _ = twitch.api().delete_message(&id, &message_id.to_string());
    }
  }

  #[func]
  fn get_users_from_ids(&mut self, id: Array<GString>) -> Array<Gd<GUserData>> {
    let mut gusers = Array::new();

    if let Some(twitch) = &mut self.twitch {
      if let Ok(users) = twitch
        .api()
        .get_users(id.iter_shared().collect(), vec![] as Vec<String>)
      {
        for user in users.data {
          gusers.push(&Gd::from_object(GUserData::from(user)));
        }
      }
    }

    gusers
  }

  #[func]
  fn get_users_from_logins(&mut self, logins: Array<GString>) -> Array<Gd<GUserData>> {
    let mut gusers = Array::new();

    if let Some(twitch) = &mut self.twitch {
      if let Ok(users) = twitch
        .api()
        .get_users(vec![] as Vec<String>, logins.iter_shared().collect())
      {
        for user in users.data {
          gusers.push(&Gd::from_object(GUserData::from(user)));
        }
      }
    }

    gusers
  }

  #[func]
  fn get_ad_schedule(&mut self) -> Array<Gd<GAdDetails>> {
    let mut data = Array::new();

    if let Some(twitch) = &mut self.twitch {
      let id = twitch.broadcaster().id.clone();
      if let Ok(schedule) = twitch.api().get_ad_schedule(&id) {
        for details in schedule.data {
          data.push(&Gd::from_object(GAdDetails::from(details)));
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
      if let Ok(users) = twitch
        .api()
        .get_users(vec![] as Vec<String>, vec![] as Vec<String>)
      {
        for user in users.data {
          gusers.push(&Gd::from_object(GUserData::from(user)));
        }
      }
    }

    gusers
  }

  #[func]
  fn get_moderators(&mut self) -> Array<Gd<GUser>> {
    let mut moderators = Array::new();

    if let Some(twitch) = &mut self.twitch {
      let id = twitch.broadcaster().id.clone();
      if let Ok(mods) = twitch.api().get_moderators(&id) {
        for user in mods.data {
          moderators.push(&Gd::from_object(GUser::from(user)));
        }
      }
    }

    moderators
  }

  #[func]
  fn get_custom_rewards(&mut self) -> Array<Gd<GGetCustomReward>> {
    let mut rewards = Array::new();

    if let Some(twitch) = &mut self.twitch {
      let id = twitch.broadcaster().id.clone();
      if let Ok(custom_rewards) = twitch.api().get_custom_rewards(&id) {
        for reward in custom_rewards.data {
          rewards.push(&Gd::from_object(GGetCustomReward::from(reward)));
        }
      }
    }

    rewards
  }

  #[func]
  fn send_announcement(&mut self, message: GString, hex_colour: GString) {
    if let Some(twitch) = &mut self.twitch {
      let id = twitch.broadcaster().id.clone();
      let _ = twitch
        .api()
        .send_announcement(&id, &message.to_string(), hex_colour.into());
    }
  }

  #[func]
  fn send_shoutout(&mut self, to_broadcaster_id: GString) {
    if let Some(twitch) = &mut self.twitch {
      let id = twitch.broadcaster().id.clone();
      let _ = twitch
        .api()
        .send_shoutout(&id, &to_broadcaster_id.to_string());
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

  #[signal]
  fn prediction_begin(begin: GdPredictionBeginContainer);

  #[signal]
  fn prediction_progress(progress: GdPredictionProgressContainer);

  #[signal]
  fn prediction_lock(lock: GdPredictionLockContainer);

  #[signal]
  fn prediction_end(end: GdPredictionEndContainer);

  #[signal]
  fn user_banned(ban: GdUserBannedContainer);

  #[func]
  pub fn get_chatters(&mut self) -> Gd<GGetChatters> {
    let id = self.twitch.as_mut().unwrap().broadcaster().id.clone();
    Gd::from_object(GGetChatters::from(
      self
        .twitch
        .as_mut()
        .unwrap()
        .api()
        .get_chatters(&id)
        .unwrap(),
    ))
  }

  #[func]
  pub fn create_secrets(&mut self) {
    if let (Some(new_id), Some(new_secret), new_broadcaster_id, new_url) = (
      &self.client_id_field,
      &self.client_secret_field,
      &self.broadcaster_id_field,
      &self.redirect_url_field,
    ) {
      let new_client_id = new_id.get_text();
      let new_client_secret = new_secret.get_text();
      //let new_broadcaster_id = new_broadcaster_id.get_text();
      //let new_redirect_url = new_url.get_text();

      if new_client_id.is_empty() && new_client_secret.is_empty() {
        //&& new_broadcaster_id.is_empty() {
        self.create_popup(None, None);
        return;
      }

      self.token.client_id = new_client_id.to_string();
      self.token.client_secret = new_client_secret.to_string();

      self.token.save(&format!(".{}.env", self.env_file_name));

      //let mut file = fs::File::create().unwrap();

      //let secrets = format!(
      //  "TWITCH_CLIENT_ID = \"{}\"\
      //  \nTWITCH_CLIENT_SECRET = \"{}\"\
      //  \nTWITCH_BROADCASTER_ID = \"{}\"",
      //  new_client_id, new_client_secret, new_broadcaster_id
      //);

      //file.write_all(format!("{}\n", secrets).as_bytes()).unwrap();

      //self.redirect_url = new_redirect_url;
      self.start_twitchevents();
    }
  }

  #[func]
  fn show_hide_help_info(&mut self) {
    if let Some(need_help) = &mut self.need_help {
      let is_visible = need_help.is_visible();
      need_help.set_visible(!is_visible);
    }
  }

  pub fn create_popup(
    &mut self,
    possible_client_id: Option<String>,
    possible_client_secret: Option<String>,
  ) {
    let _ = fs::remove_file(".user_token.env");
    let _ = fs::remove_file(".refresh_token.env");

    let mut vbox = VBoxContainer::new_alloc();

    let mut confirmation = ConfirmationDialog::new_alloc();

    confirmation.set_title("Set Twitch Client Details");
    confirmation.set_initial_position(WindowInitialPosition::CENTER_PRIMARY_SCREEN);
    confirmation.set_visible(true);
    confirmation.move_to_foreground();
    //confirmation.set_max_size(Vector2i::new(800, 500));
    if let Some(mut okay_button) = confirmation.get_ok_button() {
      okay_button.connect("pressed", &self.to_gd().callable("create_secrets"));
    }

    let mut twitch_console_label = Label::new_alloc();
    twitch_console_label.set_text("Get Client ID and Secret here -> ");
    let mut twitch_console = LinkButton::new_alloc();
    twitch_console.set_text("Open Twitch Console");
    twitch_console.set_uri("https://dev.twitch.tv/console");
    twitch_console.set_underline_mode(UnderlineMode::ALWAYS);
    twitch_console.set_focus_mode(FocusMode::CLICK);

    let mut grid_container = GridContainer::new_alloc();
    grid_container.set_columns(2);

    let mut client_id_label = Label::new_alloc();
    client_id_label.set_text("Client ID:");

    let mut client_id_edit = LineEdit::new_alloc();
    client_id_edit.set_name("NewClientId");
    client_id_edit.set_placeholder("Client ID");
    client_id_edit.set_clear_button_enabled(true);
    client_id_edit.set_custom_minimum_size(Vector2 { x: 200.0, y: 0.0 });
    if let Some(assigned_id) = possible_client_id {
      client_id_edit.set_text(&assigned_id);
    }

    let mut client_secret_label = Label::new_alloc();
    client_secret_label.set_text("Client Secret:");

    let mut client_secret_edit = LineEdit::new_alloc();
    client_secret_edit.set_placeholder("Client Secret");
    client_secret_edit.set_secret(true);
    if let Some(assigned_secret) = possible_client_secret {
      if assigned_secret.is_empty() {
        client_secret_edit.set_placeholder("Invalid Secret Key");
      } else {
        client_secret_edit.set_text(&assigned_secret);
      }
    }

    let mut redirect_url_label = Label::new_alloc();
    redirect_url_label.set_text("Redirect Url:");

    let mut redirect_url_label2 = Label::new_alloc();
    redirect_url_label2.set_text("Set In Editor");

    let mut redirect_url_edit = LineEdit::new_alloc();
    redirect_url_edit.set_text(self.redirect_url.clone().owned_to_arg());
    redirect_url_edit.set_placeholder("http://localhost:3000");

    let mut twitch_console_label = Label::new_alloc();
    twitch_console_label.set_text("Get Client ID and Secret here -> ");

    let image = image::ImageReader::new(Cursor::new(VISUAL_ERROR))
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
      &PackedByteArray::from(image.as_bytes()),
    )
    .unwrap();

    let texture = ImageTexture::create_from_image(&image).unwrap();

    let mut need_help_button = Button::new_alloc();
    need_help_button.set_text("need help?");

    need_help_button.connect("pressed", &self.to_gd().callable("show_hide_help_info"));

    let mut visual_error = AspectRatioContainer::new_alloc();
    visual_error.set_visible(false);

    let mut visual_error_texture = TextureRect::new_alloc();
    visual_error_texture.set_texture(&texture);

    let mut ratio_error_texture = AspectRatioContainer::new_alloc();
    ratio_error_texture.add_child(&visual_error_texture);

    let mut error_description = Label::new_alloc();
    error_description.set_text(
      "If you are faced with this when the browser opens you have the incorrect redirect url set.",
    );

    let mut error_vbox = VBoxContainer::new_alloc();

    error_vbox.add_child(&error_description);
    error_vbox.add_child(&ratio_error_texture);

    visual_error.add_child(&error_vbox);

    grid_container.add_child(&twitch_console_label);
    grid_container.add_child(&twitch_console);
    grid_container.add_child(&client_id_label);
    grid_container.add_child(&client_id_edit);
    grid_container.add_child(&client_secret_label);
    grid_container.add_child(&client_secret_edit);
    grid_container.add_child(&redirect_url_label);
    grid_container.add_child(&redirect_url_label2);
    grid_container.add_child(&need_help_button);
    // grid_container.add_child(&error_description);
    // grid_container.add_child(&visual_error);

    vbox.add_child(&grid_container);
    vbox.add_child(&visual_error);

    confirmation.add_child(&vbox);
    //confirmation.add_child(&error_description);

    self.client_id_field = Some(client_id_edit);
    self.client_secret_field = Some(client_secret_edit);
    self.redirect_url_field = Some(redirect_url_edit);
    self.need_help = Some(visual_error);

    self.base_mut().add_child(&confirmation);
  }

  #[func]
  fn start_twitchevents(&mut self) {
    let mut token_builder = TokenHandlerBuilder::new()
      .env_file(&format!(".{}.env", self.env_file_name))
      .override_redirect_url(&self.redirect_url.to_string());
    if let Some(token) = token_builder.build_from_env_only() {
      self.token = token.clone();
      if token.client_id.is_empty() {
        self.create_popup(
          Some(String::from("No Id Set")),
          Some(self.token.client_secret.clone()),
        );
        return;
      }

      if token.client_secret.is_empty() {
        self.create_popup(Some(token.client_id), Some(String::new()));
        return;
      }
    } else {
      self.create_popup(None, None);
      return;
    }

    self.token = token_builder
      .generate_user_tokens(self.token.clone())
      .unwrap();

    // TokenHandlerBuilder::new()
    //   .env_file(&format!(".{}{}", self.env_file_name.to_string(), ".env"))
    //   .build_unsafe();

    //let keys = match TwitchKeys::from_secrets_env(vec![format!(
    //  ".{}.env",
    //  self.env_file_name.to_string()
    //)]) {
    //  Ok(keys) => keys,
    //  Err(_) => match TwitchKeys::from_secrets_env(vec![String::from(".example.env")]) {
    //    Ok(keys) => keys,
    //    Err(_) => {
    //      self.create_popup(None, None, None);
    //      return;
    //    }
    //  },
    //};

    let mut twitch = TwitchEventSubApi::builder(self.token.clone()).enable_irc();
    //let mut twitch = TwitchEventSubApi::builder(keys.clone())
    //  .set_redirect_url(self.redirect_url.to_string())
    //  .generate_new_token_if_insufficent_scope(true)
    //  .generate_new_token_if_none(true)
    //  .generate_access_token_on_expire(true)
    //  .auto_save_load_created_tokens(".user_token.env", ".refresh_token.env");

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
    if self.channel_user_banned {
      twitch = twitch.add_subscription(Subscription::ChannelUserBanned);
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

    match twitch.build(&self.broadcaster_username.to_string()) {
      Ok(twitch) => {
        self.token.save(&format!(".{}.env", self.env_file_name));
        self.twitch = Some(twitch);
      }
      Err(EventSubError::TwitchApiError(TwitchApiError::InvalidOauthToken(error)))
        if error.contains("are different") =>
      {
        panic!("Twitch Id doesnt match token user id: {:?}", error);
      }
      Err(e) => match e {
        //EventSubError::InvalidOauthToken(exact_error) => {
        //  let mut client_id = keys.client_id;
        //  let mut client_secret = keys.client_secret;
        //  let broadcaster_id = keys.broadcaster_account_id;

        //  if exact_error.contains("400") {
        //    client_id = "Invalid ID".to_string();
        //  }
        //  if exact_error.contains("403") {
        //    client_secret = String::new();
        //  }

        //  self.create_popup(Some(client_id), Some(client_secret), Some(broadcaster_id));
        //}
        e => {
          panic!("Test fail {:?}", e);
        }
      },
    }
  }
}

#[godot_api]
impl INode for TwitchEventNode {
  fn init(base: Base<Node>) -> Self {
    Self {
      twitch: None,
      token: TokenHandler::new(),
      start_onready: true,
      broadcaster_username: "".to_godot(),
      show_connected_notification: true,
      redirect_url: "http://localhost:3000".to_godot(),
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
      channel_user_banned: false,
      permission_ban_timeout_user: false,
      permission_delete_message: false,
      permission_read_chatters: false,
      env_file_name: "secrets".to_godot(),
      client_id_field: None,
      client_secret_field: None,
      broadcaster_id_field: None,
      redirect_url_field: None,
      need_help: None,
      base,
    }
  }

  fn ready(&mut self) {
    if self.start_onready {
      self.start_twitchevents();
    }
  }

  fn process(&mut self, _delta: f64) {
    if let Some(ref mut api) = &mut self.twitch {
      if let Some(message) = api.receive_single_message(Duration::ZERO) {
        println!("Event: {:?}", message);
        match message {
          ResponseType::Event(event) => match event {
            TwitchEvent::ChatMessage(message_data) => {
              self.base_mut().emit_signal(
                match message_data.message_type {
                  MessageType::PowerUpsGigantifiedEmote => "chat_message_powerup_gigantified_emote",
                  MessageType::PowerUpsMessageEffect => "chat_message_powerup_message_effect",
                  _ => "chat_message",
                },
                &[GdMessageContainer {
                  data: Gd::from_object(GMessageData::from(message_data)),
                }
                .to_variant()],
              );
            }
            TwitchEvent::Raid(raid_info) => {
              self.base_mut().emit_signal(
                "raid",
                &[GdRaidContainer {
                  data: Gd::from_object(GRaid::from(raid_info)),
                }
                .to_variant()],
              );
            }
            TwitchEvent::AdBreakBegin(ad_break_data) => {
              self.base_mut().emit_signal(
                "ad_break_start",
                &[GdAdBreakBeginContainer {
                  data: Gd::from_object(GAdBreakBegin::from(ad_break_data)),
                }
                .to_variant()],
              );
            }
            TwitchEvent::PointsCustomRewardRedeem(custom_reward_redeem) => {
              self.base_mut().emit_signal(
                "custom_point_reward_redeem",
                &[GdCustomRewardRedeemContainer {
                  data: Gd::from_object(GCustomRewardRedeem::from(custom_reward_redeem)),
                }
                .to_variant()],
              );
            }
            TwitchEvent::Follow(follow_data) => {
              self.base_mut().emit_signal(
                "follow",
                &[GdFollowContainer {
                  data: Gd::from_object(GFollowData::from(follow_data)),
                }
                .to_variant()],
              );
            }
            TwitchEvent::NewSubscription(new_sub_data) => {
              self.base_mut().emit_signal(
                "new_subscription",
                &[GdNewSubscriptionContainer {
                  data: Gd::from_object(GNewSubscription::from(new_sub_data)),
                }
                .to_variant()],
              );
            }
            TwitchEvent::GiftSubscription(gift_data) => {
              self.base_mut().emit_signal(
                "gift_subscription",
                &[GdGiftContainer {
                  data: Gd::from_object(GGift::from(gift_data)),
                }
                .to_variant()],
              );
            }
            TwitchEvent::Resubscription(resub_data) => {
              self.base_mut().emit_signal(
                "resubscription",
                &[GdResubscriptionContainer {
                  data: Gd::from_object(GResubscription::from(resub_data)),
                }
                .to_variant()],
              );
            }
            TwitchEvent::Cheer(cheer) => {
              self.base_mut().emit_signal(
                "cheer",
                &[GdCheerContainer {
                  data: Gd::from_object(GCheerData::from(cheer)),
                }
                .to_variant()],
              );
            }
            TwitchEvent::PollBegin(begin) => {
              self.base_mut().emit_signal(
                "poll_begin",
                &[GdPollBeginContainer {
                  data: Gd::from_object(GPollBegin::from(begin)),
                }
                .to_variant()],
              );
            }
            TwitchEvent::PollProgress(progress) => {
              self.base_mut().emit_signal(
                "poll_progress",
                &[GdPollProgressContainer {
                  data: Gd::from_object(GPollProgress::from(progress)),
                }
                .to_variant()],
              );
            }
            TwitchEvent::PollEnd(end) => {
              self.base_mut().emit_signal(
                "poll_end",
                &[GdPollEndContainer {
                  data: Gd::from_object(GPollEnd::from(end)),
                }
                .to_variant()],
              );
            }
            TwitchEvent::PredictionBegin(begin_data) => {
              self.base_mut().emit_signal(
                "prediction_begin",
                &[GdPredictionBeginContainer {
                  data: Gd::from_object(GPredictionBegin::from(begin_data)),
                }
                .to_variant()],
              );
            }
            TwitchEvent::PredictionProgress(progress_data) => {
              self.base_mut().emit_signal(
                "prediction_progress",
                &[GdPredictionProgressContainer {
                  data: Gd::from_object(GPredictionProgress::from(progress_data)),
                }
                .to_variant()],
              );
            }
            TwitchEvent::PredictionLock(lock_data) => {
              self.base_mut().emit_signal(
                "prediction_lock",
                &[GdPredictionLockContainer {
                  data: Gd::from_object(GPredictionLock::from(lock_data)),
                }
                .to_variant()],
              );
            }
            TwitchEvent::PredictionEnd(end_data) => {
              self.base_mut().emit_signal(
                "prediction_end",
                &[GdPredictionEndContainer {
                  data: Gd::from_object(GPredictionEnd::from(end_data)),
                }
                .to_variant()],
              );
            }
            TwitchEvent::MessageDeleted(message_deleted) => {
              self.base_mut().emit_signal(
                "message_deleted",
                &[GdMessageDeletedContainer {
                  data: Gd::from_object(GMessageDeleted::from(message_deleted)),
                }
                .to_variant()],
              );
            }
            TwitchEvent::UserBanned(ban_data) => {
              self.base_mut().emit_signal(
                "user_banned",
                &[GdUserBannedContainer {
                  data: Gd::from_object(GUserBanned::from(ban_data)),
                }
                .to_variant()],
              );
            }
            _ => {}
          },
          ResponseType::Ready => {
            if self.show_connected_notification {
              let r = 1.0;
              let g = 1.0;
              let b = 1.0;
              let coloured = Color::from_rgba(r, g, b, 1.0);
              let uncoloured = Color::from_rgba(r, g, b, 0.0);

              let mut panel = Panel::new_alloc();
              panel.set_size(Vector2::new(146.0, 31.0));
              panel.set_position(Vector2::new(6.0, 9.0));
              panel.add_theme_color_override("grey", Color::from_rgb(0.8, 0.8, 0.8));

              let mut label = Label::new_alloc();
              label.set_text("Twitch Connected!");
              label.set_size(Vector2::new(143.0, 23.0));
              label.set_position(Vector2::new(1.0, 3.0));

              panel.add_child(&label);

              self.base_mut().add_child(&panel);
              if let Some(mut tween) = self.base_mut().create_tween() {
                tween.tween_property(&panel, "modulate", &uncoloured.to_variant(), 0.0);

                tween.set_ease(EaseType::IN);
                tween.set_trans(TransitionType::QUINT);

                tween.tween_property(&panel, "modulate", &coloured.to_variant(), 0.6);

                tween.tween_interval(2.0);

                tween.set_ease(EaseType::OUT);
                tween.set_trans(TransitionType::QUINT);
                tween.tween_property(&panel, "modulate", &uncoloured.to_variant(), 3.0);
                tween.tween_callback(&panel.callable("queue_free"));
                tween.play();
              }
            }
          }
          ResponseType::Error(error) => match error {
            //EventSubError::InvalidOauthToken(_exact_error) => {
            //  let keys = api.get_twitch_keys();

            //  self.twitch = None;

            //  let client_id = keys.client_id;
            //  let client_secret = keys.client_secret;
            //  let broadcaster_id = "Invalid Id".to_owned();

            //  self.create_popup(Some(client_id), Some(client_secret), Some(broadcaster_id));
            //}
            e => {
              panic!("Error2: {:?}", e);
            }
          },
          _ => {}
        }
      }
    }
  }
}
