use std::{
  fs::exists,
  sync::{Arc, Mutex, mpsc::Receiver},
  time::Duration,
};

use bevy_app::prelude::*;
use bevy_derive::{Deref, DerefMut};
use bevy_ecs::prelude::*;
use bevy_state::prelude::*;
use bevy_time::common_conditions::on_timer;
use bevy_time::prelude::*;
use twitcheventsub_structs::prelude::TwitchEvent::ChatMessage;
use twitcheventsub_tokens::TokenHandlerBuilder;

use crate::{
  EventSubError, ResponseType, TwitchEventSubApi,
  prelude::{
    Subscription, TwitchEvent, twitcheventsub_api::TwitchApiError,
    twitcheventsub_tokens::TokenHandler,
  },
};

#[derive(Resource)]
pub struct TwitchUsername(pub String);

struct TwitchInfo {
  subscriptions: Vec<Subscription>,
  recv_code: Option<Receiver<String>>,
  temp_handler: TokenHandler,
  builder: TokenHandlerBuilder,
}

impl TwitchInfo {
  pub fn new() -> TwitchInfo {
    TwitchInfo {
      subscriptions: Subscription::recommended(),
      recv_code: None,
      temp_handler: TokenHandler::new(),
      builder: TokenHandlerBuilder::new(),
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
      .insert_non_send_resource(TwitchInfo::new())
      .add_systems(OnEnter(self.state.clone()), setup)
      .add_systems(Update, check_for_code.run_if(in_state(self.state.clone())))
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
  mut commands: Commands,
) {
  if let Ok(mut twitch) = twitch.try_lock() {
    for message in twitch.receive_all_messages(None) {
      match message {
        ResponseType::Ready => {
          twitch_ready.0 = true;
          twitch_event_writer.write(TwitchEvent::Ready);
        }
        ResponseType::Event(event) => {
          twitch_event_writer.write(*event.clone());

          match *event {
            TwitchEvent::ChatMessage(value) => {
              commands.trigger(*value);
            }
            TwitchEvent::Raid(value) => {
              commands.trigger(value);
            }
            TwitchEvent::Follow(value) => {
              commands.trigger(value);
            }
            TwitchEvent::PointsCustomRewardRedeem(value) => {
              commands.trigger(value);
            }
            TwitchEvent::AdBreakBegin(value) => {
              commands.trigger(value);
            }
            TwitchEvent::NewSubscription(value) => {
              commands.trigger(value);
            }
            TwitchEvent::GiftSubscription(value) => {
              commands.trigger(value);
            }
            TwitchEvent::Resubscription(value) => {
              commands.trigger(value);
            }
            TwitchEvent::Cheer(value) => {
              commands.trigger(value);
            }
            TwitchEvent::ChannelPointsAutoRewardRedeem(value) => {
              commands.trigger(value);
            }
            TwitchEvent::PollProgress(value) => {
              commands.trigger(value);
            }
            TwitchEvent::PollBegin(value) => {
              commands.trigger(value);
            }
            TwitchEvent::PollEnd(value) => {
              commands.trigger(value);
            }
            TwitchEvent::PredictionProgress(value) => {
              commands.trigger(value);
            }
            TwitchEvent::PredictionBegin(value) => {
              commands.trigger(value);
            }
            TwitchEvent::PredictionLock(value) => {
              commands.trigger(value);
            }
            TwitchEvent::PredictionEnd(value) => {
              commands.trigger(value);
            }
            TwitchEvent::HypeTrainProgress(value) => {
              commands.trigger(value);
            }
            TwitchEvent::HypeTrainBegin(value) => {
              commands.trigger(value);
            }
            TwitchEvent::HypeTrainEnd(value) => {
              commands.trigger(value);
            }
            TwitchEvent::MessageDeleted(value) => {
              commands.trigger(value);
            }
            TwitchEvent::ShoutoutReceive(value) => {
              commands.trigger(value);
            }
            TwitchEvent::ShoutoutCreate(value) => {
              commands.trigger(value);
            }
            TwitchEvent::ModeratorEvent(value) => {
              commands.trigger(value);
            }
            TwitchEvent::UserBanned(value) => {
              commands.trigger(value);
            }
            TwitchEvent::StreamOnline(value) => {
              commands.trigger(value);
            }
            TwitchEvent::StreamOffline(value) => {
              commands.trigger(value);
            }
            _ => {}
          }
        }
        _ => {}
      }
    }
  }
}

fn check_for_code(
  twitch_info: Option<NonSendMut<TwitchInfo>>,
  username: Res<TwitchUsername>,
  mut commands: Commands,
) {
  if twitch_info.is_none() {
    //panic!("Please insert the TwitchInfo resource, before connecting to twitch.");
    return;
  }

  let mut twitch_info = twitch_info.unwrap();

  //if let Some(reciever) = twitch_info.reciever {
  //  reciever.recv()
  //}

  if let Some(recv) = &twitch_info.recv_code {
    if let Ok(code) = recv.try_recv() {
      //println!("subscriptions: {}", self.token.subscriptions.len());

      twitch_info.temp_handler = twitch_info
        .builder
        .build_token_from_authorisation_code(&code)
        .unwrap();
      twitch_info.recv_code = None;

      let mut twitch = TwitchEventSubApi::builder(twitch_info.temp_handler.clone()); //.enable_irc();
      match twitch.build(&username.0.to_string()) {
        Ok(twitch) => {
          twitch_info.temp_handler.save();

          commands.insert_resource(twitch_info.temp_handler.clone());
          commands.insert_resource(TwitchResource(Arc::new(Mutex::new(twitch))));
        }
        Err(EventSubError::TwitchApiError(TwitchApiError::InvalidOauthToken(error)))
          if error.contains("are different") =>
        {
          panic!("Twitch Id doesnt match token user id: {:?}", error);
        }
        Err(e) => match e {
          e => {
            panic!("Test fail {:?}", e);
          }
        },
      }
    }
  }
}

fn setup(
  mut twitch_info: NonSendMut<TwitchInfo>,
  username: Option<Res<TwitchUsername>>,
  mut commands: Commands,
) {
  if username.is_none() {
    panic!("Please insert the TwitchUsername resource, before connecting to twitch.");
  }
  let username = username.unwrap();

  //let mut twitch_info = twitch_info.unwrap();

  let mut builder = TokenHandler::builder().add_subscriptions(twitch_info.subscriptions.clone());
  let waiting = builder.build_nonblocking();

  let (tokens, reciever) = waiting.unwrap();

  if reciever.is_some() {
    twitch_info.recv_code = reciever;
    twitch_info.temp_handler = tokens;
    twitch_info.builder = builder;
  } else {
    twitch_info.temp_handler = tokens;
    twitch_info.builder = builder;

    let mut twitch = TwitchEventSubApi::builder(twitch_info.temp_handler.clone()); //.enable_irc();
    match twitch.build(&username.0.to_string()) {
      Ok(twitch) => {
        twitch_info.temp_handler.save();

        commands.insert_resource(twitch_info.temp_handler.clone());
        commands.insert_resource(TwitchResource(Arc::new(Mutex::new(twitch))));
      }
      Err(EventSubError::TwitchApiError(TwitchApiError::InvalidOauthToken(error)))
        if error.contains("are different") =>
      {
        panic!("Twitch Id doesnt match token user id: {:?}", error);
      }
      Err(e) => match e {
        e => {
          panic!("Test fail {:?}", e);
        }
      },
    }
  }
  //let tokens = TokenHandler::builder()
  //    .add_subscriptions(twitch_info.subscriptions.clone())
  //  .build();
  //let tokens = tokens.unwrap();

  //commands.insert_resource(tokens.clone());
  //let twitch = TwitchEventSubApi::builder(tokens)
  //  .build(&twitch_info.username)
  //  .unwrap();

  //commands.insert_resource(TwitchResource(Arc::new(Mutex::new(twitch))));
}
