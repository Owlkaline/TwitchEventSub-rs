use bevy::{platform::collections::HashMap, prelude::*};
use bevy_asset_loader::prelude::*;

use crate::Program;

pub struct LoadingPlugin;

/// This plugin loads all assets using [`AssetLoader`] from a third party bevy plugin
/// Alternatively you can write the logic to load assets yourself
/// If interested, take a look at <https://bevy-cheatbook.github.io/features/assets.html>
impl Plugin for LoadingPlugin {
  fn build(&self, app: &mut App) {
    app.add_loading_state(
      LoadingState::new(Program::Loading)
        .continue_to_state(Program::Connecting)
        .load_collection::<TextureAssets>(),
    );
  }
}

#[derive(AssetCollection, Resource)]
pub struct TextureAssets {
  #[asset(path = "emotes", collection(mapped))]
  pub emotes: HashMap<AssetFileStem, UntypedHandle>,
}
