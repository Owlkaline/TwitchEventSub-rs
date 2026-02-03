use std::{
  fs::File,
  io::{Cursor, Write},
  path::Path,
};

use twitcheventsub::prelude::*;

//Use bevy::{
//  asset::{self, AssetPath, RenderAssetUsages},
//  input::common_conditions::input_just_pressed,
//  prelude::*,
//  render::render_resource::Texture,
//};
//Use bevy_asset_loader::mapped::{AssetFileName, AssetFileStem, MapKey};
//Use bevy_easy_gif::{Gif, GifAsset, GifNode, GifPlugin};
//Use bevy_text_animation::{TextAnimationFinished, TextAnimatorPlugin, TextSimpleAnimator};
//Use image::{
//  codecs::gif::{self, GifDecoder, GifEncoder},
//  AnimationDecoder, ImageReader,
//};
//Use twitcheventsub::{
//  prelude::{
//    twitch_is_ready, twitcheventsub_tokens::TokenHandler, FragmentType, Fragments, MessageType,
//    TwitchEvent, TwitchInfo, TwitchPlugin, BTTV,
//  },
//  EmoteBuilder,
//};

#[derive(States, PartialEq, Eq, Debug, Clone, Copy, Hash, Default)]
pub enum Program {
  #[default]
  Loading,
  Connecting,
  Chat,
}

fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(TwitchPlugin::<Program>::connect_on_enter(
      Program::Connecting,
    ))
    .init_state::<Program>()
    .add_systems(Startup, setup)
    .add_systems(Update, new_message.run_if(twitch_is_ready))
    .run();
}

fn setup(
  mut commands: Commands,
  asset_server: Res<AssetServer>,
  next_state: ResMut<NextState<Program>>,
) {
  commands.insert_resource(TwitchInfo::recommended("owlkalinevt"));
  next_state.set(Program::Connecting);
}

fn new_message(
  mut event_reader: MessageReader<TwitchEvent>,
  chat_box: Single<Entity, With<ChatBox>>,
  mut commands: Commands,
  tokens: Res<TokenHandler>,
  textures: ResMut<TextureAssets>,
  assets: Res<AssetServer>,
) {
  for event in event_reader.read() {
    match event {
      TwitchEvent::Ready => {
        // Everything has been subscribed and connected to twitch
        // After recieving this you should get all event updates from twitch
        println!("Twitch connected!")
      }
      TwitchEvent::ChatMessage(msg) => {
        println!("{}: {}", msg.chatter.name, msg.message.text);
      }
      // Many more events
      _ => {}
    }
  }
}
