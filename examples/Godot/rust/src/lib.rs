use std::process;

use godot::classes::ISprite2D;
use godot::classes::Sprite2D;
use godot::classes::{INode, INode2D, Node, Node2D};
use godot::engine::display_server::ExWindowGetVSyncMode;
use godot::engine::visual_shader_node_float_func;
use godot::init::EditorRunBehavior;
use godot::prelude::*;

use twitch_eventsub::*;

struct TwitchApi;

#[gdextension]
unsafe impl ExtensionLibrary for TwitchApi {
  fn editor_run_behavior() -> EditorRunBehavior {
    EditorRunBehavior::ToolClassesOnly
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
#[var]
#[export]
#[class(init)]
pub struct GdRaidContainer {
  pub data: Gd<GRaid>,
}

#[derive(GodotClass, Debug, Default)]
#[var]
#[export]
#[class(init)]
pub struct GRaid {
  #[var]
  pub raider_user_id: GString,

  #[var]
  pub raider_username: GString,

  #[var]
  pub viewers: u32,
}

impl From<RaidData> for GRaid {
  fn from(raid: RaidData) -> GRaid {
    GRaid {
      raider_user_id: raid.from_broadcaster.id.to_owned().into(),
      raider_username: raid.from_broadcaster.name.to_owned().into(),
      viewers: raid.viewers,
    }
  }
}

#[derive(GodotClass, Debug, GodotConvert)]
#[godot(transparent)]
#[var]
#[export]
#[class(init)]
pub struct GdMessageContainer {
  pub data: Gd<GMessageData>,
}

#[derive(GodotClass, Debug)]
#[var]
#[export]
#[class(init)]
pub struct GMessageData {
  #[var]
  pub user_id: GString,

  #[var]
  pub message_id: GString,

  #[var]
  pub message: GString,

  #[var]
  pub username: GString,
}

impl From<MessageData> for GMessageData {
  fn from(msg: MessageData) -> GMessageData {
    GMessageData {
      user_id: msg.chatter_user.id.to_owned().into(),
      message_id: msg.message_id.to_owned().into(),
      message: msg.message.text.to_owned().into(),
      username: msg.chatter_user.name.to_owned().into(),
    }
  }
}

#[godot_api]
impl TwitchEvent {
  #[signal]
  fn chat_message(message_data: GdMessageContainer);

  #[signal]
  fn point_redeem(
    username: GString,
    input: GString,
    redeem_cost: u32,
    redeem_id: GString,
    redeem_prompt: GString,
    title: GString,
  );

  #[signal]
  fn ad_break_start(duration: u32);

  #[signal]
  fn raid(raid_info: GdRaidContainer);
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
      .add_subscription(Subscription::BanTimeoutUser)
      .add_subscription(Subscription::DeleteMessage)
      .add_subscription(Subscription::AdBreakBegin)
      .add_subscription(Subscription::ChannelRaid)
      .build()
      .unwrap();
    self.twitch = Some(twitch);
  }

  fn process(&mut self, delta: f64) {
    if let Some(ref mut api) = &mut self.twitch {
      for message in api.receive_messages() {
        godot_print!("{:?}", message);
        match message {
          MessageType::Event(Event::ChatMessage(message_data)) => {
            self.base_mut().emit_signal(
              "chat_message".into(),
              &[GdMessageContainer {
                data: Gd::from_object(GMessageData::from(message_data)),
              }
              .to_variant()],
            );
          }
          MessageType::Event(Event::Raid(raid_info)) => {
            self.base_mut().emit_signal(
              "raid".into(),
              &[GdRaidContainer {
                data: Gd::from_object(GRaid::from(raid_info)),
              }
              .to_variant()],
            );
          }
          //MessageType::AdBreakNotification(duration) => {
          //  self
          //    .base_mut()
          //    .emit_signal("ad_break_start".into(), &[duration.to_variant()]);
          //}
          //MessageType::CustomRedeem((username, input, reward)) => {
          //  self.base_mut().emit_signal(
          //    "point_redeem".into(),
          //    &[
          //      username.to_variant(),
          //      input.to_variant(),
          //      reward.cost.to_variant(),
          //      reward.id.to_variant(),
          //      reward.prompt.to_variant(),
          //      reward.title.to_variant(),
          //    ],
          //  );
          //}
          _ => {}
        }
      }
    }
  }
}
