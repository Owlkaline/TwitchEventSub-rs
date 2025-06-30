use godot::prelude::*;
use twitcheventsub::prelude::*;

#[derive(GodotClass)]
#[class(init)]
pub struct GEmote {
  #[var]
  pub id: GString,
  #[var]
  pub emote_set_id: GString,
  #[var]
  pub owner_id: GString,
  #[var]
  pub format: Array<GString>,
}

#[derive(GodotClass)]
#[class(init)]
pub struct GRewardEmote {
  #[var]
  id: GString,
  #[var]
  begin: u32,
  #[var]
  end: u32,
}

#[derive(GodotClass)]
#[class(init)]
pub struct GEmoteStaticImages {
  #[var]
  url_1x: GString,
  #[var]
  url_2x: GString,
  #[var]
  url_4x: GString,
}

#[derive(GodotClass, Debug)]
#[class(init)]
pub struct GEmoteUrl {
  #[var]
  pub url: GString,
  #[var]
  pub animated: bool,
}

impl From<EmoteUrl> for GEmoteUrl {
  fn from(value: EmoteUrl) -> Self {
    GEmoteUrl {
      url: value.url.into(),
      animated: value.animated,
    }
  }
}

impl From<RewardEmote> for GRewardEmote {
  fn from(value: RewardEmote) -> Self {
    GRewardEmote {
      id: value.id.into(),
      begin: value.begin,
      end: value.end,
    }
  }
}

#[godot_api]
impl GEmote {
  #[func]
  pub fn has_animation(&self) -> bool {
    self.format.contains("animated")
  }

  pub fn convert_to_rust(&self) -> Emote {
    Emote {
      id: self.id.to_owned().into(),
      emote_set_id: self.emote_set_id.to_owned().into(),
      owner_id: if self.owner_id.is_empty() {
        None
      } else {
        Some(self.owner_id.to_owned().into())
      },
      format: Some(
        self
          .format
          .iter_shared()
          .map(|a| a.into())
          .collect::<Vec<_>>(),
      ),
    }
  }
}

impl From<Emote> for GEmote {
  fn from(value: Emote) -> Self {
    let mut format = Array::new();

    if let Some(some_format) = value.format {
      for value_format in some_format {
        format.push(&value_format);
      }
    }

    GEmote {
      id: value.id.into(),
      emote_set_id: value.emote_set_id.into(),
      owner_id: value.owner_id.unwrap_or("".to_string()).into(),
      format,
    }
  }
}

impl From<EmoteStaticImages> for GEmoteStaticImages {
  fn from(value: EmoteStaticImages) -> Self {
    GEmoteStaticImages {
      url_1x: value.url_1x.into(),
      url_2x: value.url_2x.into(),
      url_4x: value.url_4x.into(),
    }
  }
}
