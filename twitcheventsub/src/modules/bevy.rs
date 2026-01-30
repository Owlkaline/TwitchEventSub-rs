use std::{
  fs::exists,
  sync::{Arc, Mutex},
  time::Duration,
};

use bevy_app::prelude::*;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;
use bevy_state::prelude::*;
use bevy_time::common_conditions::on_timer;
use bevy_time::prelude::*;

use crate::{
  prelude::{
    twitcheventsub_api::TwitchApiError, twitcheventsub_tokens::TokenHandler, Subscription,
    TwitchEvent,
  },
  EventSubError, ResponseType, TwitchEventSubApi,
};

#[derive(Resource)]
pub struct TwitchInfo {
  subscriptions: Vec<Subscription>,
  username: String,
}

impl TwitchInfo {
  pub fn recommended(username: &str) -> TwitchInfo {
    TwitchInfo {
      subscriptions: vec![
        Subscription::ChatMessage,
        Subscription::AdBreakBegin,
        Subscription::ChannelPointsCustomRewardRedeem,
        Subscription::ChannelPointsAutoRewardRedeem,
        Subscription::ChannelFollow,
        Subscription::ChannelRaid,
        Subscription::ChannelNewSubscription,
        Subscription::ChannelGiftSubscription,
        Subscription::ChannelResubscription,
        Subscription::ChannelCheer,
        Subscription::ChannelHypeTrainBegin,
        Subscription::ChannelHypeTrainProgress,
        Subscription::ChannelHypeTrainEnd,
        Subscription::ChannelUserBanned,
        Subscription::ChannelMessageDeleted,
        Subscription::PermissionReadModerator,
        Subscription::PermissionDeleteMessage,
        Subscription::PermissionReadChatters,
        Subscription::PermissionSendAnnouncements,
        Subscription::PermissionManageRewards,
        Subscription::PermissionIRCRead,
        Subscription::PermissionIRCWrite,
      ],
      username: username.to_string(),
    }
  }
}

#[derive(Resource, Deref, DerefMut)]
pub struct TwitchReady(bool);

impl Default for TwitchReady {
  fn default() -> Self {
    TwitchReady(false)
  }
}

pub struct TwitchPlugin<S: States> {
  pub state: S,
}

impl<S: States> TwitchPlugin<S> {
  pub fn connect_on_enter(state: S) -> TwitchPlugin<S> {
    TwitchPlugin { state }
  }
}

#[derive(Resource, Deref, DerefMut)]
struct TwitchResource(Arc<Mutex<TwitchEventSubApi>>);

impl<S: States> Plugin for TwitchPlugin<S> {
  fn build(&self, app: &mut App) {
    app
      .add_message::<TwitchEvent>()
      .init_resource::<TwitchReady>()
      .add_systems(OnEnter(self.state.clone()), setup)
      .add_systems(
        PreUpdate,
        check_for_messages.run_if(resource_exists::<TwitchResource>),
      )
      .add_systems(
        Update,
        update_token
          .run_if(on_timer(Duration::from_mins(5)).and(resource_exists::<TwitchResource>)),
      );
  }
}

fn update_token(twitch: Res<TwitchResource>, mut token: ResMut<TokenHandler>) {
  if let Ok(twitch) = twitch.lock() {
    *token = twitch.get_tokens();
  }
}

pub fn twitch_is_ready(ready: Res<TwitchReady>) -> bool {
  ready.0
}

fn check_for_messages(
  twitch: ResMut<TwitchResource>,
  mut twitch_event_writer: MessageWriter<TwitchEvent>,
  mut twitch_ready: ResMut<TwitchReady>,
) {
  if let Ok(mut twitch) = twitch.try_lock() {
    for message in twitch.receive_all_messages(None) {
      match message {
        ResponseType::Ready => {
          twitch_ready.0 = true;
          twitch_event_writer.write(TwitchEvent::Ready);
        }
        ResponseType::Event(event) => {
          twitch_event_writer.write(*event);
        }
        _ => {}
      }
    }
  }
}

pub fn setup(twitch_info: Option<Res<TwitchInfo>>, mut commands: Commands) {
  if twitch_info.is_none() {
    panic!("Please insert the TwitchInfo resource, before connecting to twitch.");
  }

  let twitch_info = twitch_info.unwrap();

  let tokens = TokenHandler::builder()
    .add_subscriptions(twitch_info.subscriptions.clone())
    .build();
  let tokens = tokens.unwrap();

  commands.insert_resource(tokens.clone());
  let twitch = TwitchEventSubApi::builder(tokens)
    .build(&twitch_info.username)
    .unwrap();

  commands.insert_resource(TwitchResource(Arc::new(Mutex::new(twitch))));
}
