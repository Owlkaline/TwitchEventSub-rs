use std::{fs::File, io::Cursor};

use bevy::prelude::*;
use bevy_easy_gif::{GifAsset, GifNode, GifPlugin};
use bevy_text_animation::{TextAnimationFinished, TextAnimatorPlugin, TextSimpleAnimator};
use image::{
  codecs::gif::{GifDecoder, GifEncoder},
  AnimationDecoder, ImageReader,
};
use twitcheventsub::{
  prelude::{
    twitch_is_ready, twitcheventsub_tokens::TokenHandler, FragmentType, TwitchEvent, TwitchInfo,
    TwitchPlugin, BTTV,
  },
  EmoteBuilder,
};

use crate::loading::{LoadingPlugin, TextureAssets};

#[derive(States, PartialEq, Eq, Debug, Clone, Copy, Hash, Default)]
pub enum Program {
  #[default]
  Loading,
  Connecting,
  Chat,
}

mod loading;

#[derive(Component)]
pub struct ChatBox;

#[derive(Component)]
pub struct RainbowAnimation;

fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(TextAnimatorPlugin)
    .add_plugins(LoadingPlugin)
    .add_plugins(GifPlugin)
    .add_plugins(TwitchPlugin::<Program>::connect_on_enter(
      Program::Connecting,
    ))
    .init_state::<Program>()
    .add_systems(Startup, setup)
    .add_systems(Update, key_handler)
    .add_systems(Update, event_handler)
    .add_systems(Update, text_colour_system)
    .add_systems(Update, new_message.run_if(twitch_is_ready))
    .run();
}

fn create_text(
  text: &str,
) -> (
  Text,
  TextFont,
  TextColor,
  TextLayout,
  RainbowAnimation,
  TextSimpleAnimator,
) {
  (
    Text::new(text),
    TextFont {
      font_size: 20.0,
      //                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
      ..default()
    },
    TextColor(Color::WHITE),
    TextLayout::new_with_linebreak(LineBreak::WordBoundary),
    RainbowAnimation,
    TextSimpleAnimator::new(text, 80.0),
  )
}

fn check_file_exists(
  emote_id: &str,
  assets: &Res<AssetServer>,
  commands: &mut Commands,
) -> Option<Entity> {
  let mut entity = None;
  let path = format!("./assets/emotes/{}", emote_id);
  let png_path = format!("{}.png", path);
  let gif_path = format!("{}.gif", path);
  if File::open(&png_path).is_ok() {
    let image = assets.load(format!("emotes/{}.png", emote_id));
    entity = Some(commands.spawn(ImageNode::new(image)).id());
  } else if File::open(&gif_path).is_ok() {
    let image = assets.load(format!("emotes/{}.gif", emote_id));
    entity = Some(commands.spawn(GifNode { handle: image }).id());
  }

  entity
}

fn check_already_loaded(
  textures: &ResMut<TextureAssets>,
  emote_id: &str,
  commands: &mut Commands,
) -> Option<Entity> {
  let mut entity = None;
  if let Some(image) = textures.emotes.get(emote_id) {
    if let Ok(image) = image.clone().try_typed::<Image>() {
      entity = Some(commands.spawn(ImageNode::new(image)).id());
    } else if let Ok(gif) = image.clone().try_typed::<GifAsset>() {
      entity = Some(commands.spawn(GifNode { handle: gif }).id());
    }
  }

  entity
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
    if let TwitchEvent::Ready = event {
      let text = commands.spawn(create_text("Twitch Ready!")).id();

      commands.entity(chat_box.entity()).add_child(text);
    }

    if let TwitchEvent::ChatMessage(msg) = event {
      let mut elements = Vec::new();

      let name = &msg.chatter.name;
      let name = commands.spawn(create_text(&format!("{}: ", name))).id();

      elements.push(name);

      let mut emote_chain = None;
      for fragment in &msg.message.fragments {
        match fragment.kind {
          FragmentType::Text => {
            if let Some(entity) = emote_chain {
              elements.push(entity);
              emote_chain = None;
            }

            let text = commands.spawn(create_text(&fragment.text)).id();
            elements.push(text);
          }
          FragmentType::Emote => {
            if emote_chain.is_none() {
              emote_chain = Some(
                commands
                  .spawn(Node {
                    display: Display::Flex,
                    flex_direction: FlexDirection::Row,
                    width: Val::Px(50.0),
                    max_width: Val::Px(50.0),
                    // min_width: Val::Px(72.0),
                    ..default()
                  })
                  .id(),
              );
            }

            let mut new_node = None;

            if let Some(emote) = &fragment.emote {
              if let Some(entity) = check_already_loaded(&textures, &emote.id, &mut commands) {
                new_node = Some(entity);
              } else if let Some(entity) = check_file_exists(&emote.id, &assets, &mut commands) {
                new_node = Some(entity);
              } else if let Some(url) = EmoteBuilder::builder()
                .animate_or_fallback_on_static()
                .scale3()
                .build(
                  &tokens,
                  &tokens.client_twitch_id,
                  fragment,
                  &mut BTTV {
                    response: None,
                    emote_names: Vec::new(),
                  },
                )
              {
                if let Ok(new_image_data) = attohttpc::get(url.url).send() {
                  dbg!(new_image_data.headers());
                  let is_gif = new_image_data
                    .headers()
                    .get("content-type")
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .contains("gif");

                  let data = &new_image_data.bytes().unwrap();
                  if is_gif {
                    if let Ok(gif) = GifDecoder::new(Cursor::new(data)) {
                      let path = format!("emotes/{}.gif", emote.id);

                      let frames = gif.into_frames();
                      let frames = frames.collect_frames().expect("error decoding gif");
                      let file_out = File::create(format!("./assets/{}", path)).unwrap();
                      let mut encoder = GifEncoder::new(file_out);
                      let _ = encoder.encode_frames(frames.into_iter());

                      let texture = assets.load(path);
                      new_node = Some(commands.spawn(GifNode { handle: texture }).id());
                    }
                  } else if let Ok(image) = ImageReader::new(Cursor::new(&data))
                    .with_guessed_format()
                    .unwrap()
                    .decode()
                  {
                    let path = format!("emotes/{}.png", emote.id);
                    let _ = image.save(format!("./assets/{}", path));
                    let texture = assets.load(path);
                    new_node = Some(commands.spawn(ImageNode::new(texture)).id());
                  }
                }
              }
            }

            if let Some(emote) = new_node {
              commands.entity(emote).insert(Node {
                justify_self: JustifySelf::Center,
                max_height: Val::Px(72.0),
                max_width: Val::Px(72.0),
                ..default()
              });

              if let Some(chain) = emote_chain {
                commands.entity(chain).add_child(emote);
              }
            }
          }
          _ => {}
        }
      }
      if let Some(entity) = emote_chain {
        elements.push(entity);
      }

      let message_box = commands
        .spawn((Node {
          max_width: Val::Px(1300.0),
          align_items: AlignItems::Center,
          width: Val::Px(1300.0),
          ..default()
        },))
        .add_children(&elements)
        .id();

      commands.entity(chat_box.entity()).add_child(message_box);
    }
  }
}

fn setup(mut commands: Commands) {
  commands.insert_resource(TwitchInfo::recommended("owlkalinevt"));
  commands.spawn(Camera2d::default());

  commands.spawn((
    Node {
      display: Display::Flex,
      max_width: Val::Px(1300.0),
      flex_direction: FlexDirection::ColumnReverse,
      ..default()
    },
    ChatBox,
    children![(
      Text::new(""),
      //Text2d::new(""),
      TextFont {
        font_size: 60.0,
        ..default()
      },
      TextColor(Color::WHITE),
      TextSimpleAnimator::new("Loading...", 8.0)
    )],
  ));
}

fn key_handler(
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut query: Query<&mut TextSimpleAnimator>,
) {
  for mut animator in query.iter_mut() {
    if keyboard_input.just_pressed(KeyCode::Space) {
      animator.play();
    }
  }
}

fn event_handler(mut events: MessageReader<TextAnimationFinished>) {
  for event in events.read() {
    println!(
      "Text Animation finished for entity (id: {:?})",
      event.entity
    );
  }
}

fn text_colour_system(time: Res<Time>, mut query: Query<&mut TextColor, With<RainbowAnimation>>) {
  for mut text_color in &mut query {
    let seconds = time.elapsed_secs();

    // Update the color of the ColorText span.
    text_color.0 = Color::srgb(
      ops::sin(1.25 * seconds) / 2.0 + 0.5,
      ops::sin(0.75 * seconds) / 2.0 + 0.5,
      ops::sin(0.50 * seconds) / 2.0 + 0.5,
    );
  }
}
