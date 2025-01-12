use crate::TwitchEventSubApi;

use twitcheventsub_structs::{
  EmoteData, EmoteFormat, EmoteScale, EmoteUrl, FragmentType, Fragments, ThemeMode,
};

pub struct EmoteBuilder {
  scale: EmoteScale,
  theme: ThemeMode,
  format: EmoteFormat,
  fallback_on_format_missing: bool,
}

impl EmoteBuilder {
  pub fn builder() -> EmoteBuilder {
    EmoteBuilder {
      scale: EmoteScale::Size1,
      theme: ThemeMode::Light,
      format: EmoteFormat::Static,
      fallback_on_format_missing: false,
    }
  }

  pub fn animate_or_fallback_on_static(mut self) -> EmoteBuilder {
    self.fallback_on_format_missing = true;
    self.animated()
  }

  pub fn format_static(mut self) -> EmoteBuilder {
    self.format = EmoteFormat::Static;
    self
  }

  pub fn animated(mut self) -> EmoteBuilder {
    self.format = EmoteFormat::Animated;
    self
  }

  pub fn light(mut self) -> EmoteBuilder {
    self.theme = ThemeMode::Light;
    self
  }

  pub fn dark(mut self) -> EmoteBuilder {
    self.theme = ThemeMode::Dark;
    self
  }

  pub fn scale1(mut self) -> EmoteBuilder {
    self.scale = EmoteScale::Size1;
    self
  }

  pub fn scale2(mut self) -> EmoteBuilder {
    self.scale = EmoteScale::Size2;
    self
  }

  pub fn scale3(mut self) -> EmoteBuilder {
    self.scale = EmoteScale::Size3;
    self
  }

  pub fn build(
    &mut self,
    twitch: &mut TwitchEventSubApi,
    fragment: &Fragments,
  ) -> Option<EmoteUrl> {
    match fragment.kind {
      FragmentType::Emote => {
        let channel_id: String = fragment
          .emote
          .as_ref()
          .unwrap()
          .owner_id
          .to_owned()
          .unwrap_or("".to_string());
        let mut template = String::new();

        let mut emote_data: Option<EmoteData> = None;

        let mut suffix = String::new();
        let id_array = fragment
          .emote
          .as_ref()
          .unwrap()
          .id
          .split('_')
          .collect::<Vec<_>>();

        let mut real_id = id_array[0].to_string();

        if id_array.len() > 1 {
          real_id = format!("{}_{}", id_array[0], id_array[1]);
          if id_array.len() > 2 {
            suffix = id_array[2].to_string();
          }
        }

        if let Ok(channel_emotes) = twitch.get_channel_emotes(channel_id) {
          template = channel_emotes.template;

          let mut valid_emotes = channel_emotes
            .data
            .into_iter()
            .filter_map(|d| if d.id == real_id { Some(d) } else { None })
            .collect::<Vec<_>>();

          if valid_emotes.len() > 0 {
            emote_data = Some(valid_emotes.remove(0).into());
          }
        }

        if emote_data.is_none() {
          if let Ok(emote_sets) =
            twitch.get_emote_sets(fragment.emote.as_ref().unwrap().emote_set_id.to_owned())
          {
            template = emote_sets.template;

            let mut valid_emotes = emote_sets
              .data
              .into_iter()
              .filter_map(|d| if d.id == real_id { Some(d) } else { None })
              .collect::<Vec<_>>();
            if valid_emotes.len() > 0 {
              emote_data = Some(valid_emotes.remove(0).into());
            }
          }

          if emote_data.is_none() {
            if let Ok(global_emotes) = twitch.get_global_emotes() {
              template = global_emotes.template;
              let mut valid_emotes = global_emotes
                .data
                .into_iter()
                .filter_map(|d| if d.id == real_id { Some(d) } else { None })
                .collect::<Vec<_>>();
              if valid_emotes.len() > 0 {
                emote_data = Some(valid_emotes.remove(0).into());
              }
            }
          }
        }

        if let Some(mut emote_data) = emote_data {
          if !suffix.is_empty() {
            emote_data.id = format!("{}_{}", real_id, suffix);
          }

          if !emote_data.format.contains(&self.format) {
            if self.fallback_on_format_missing {
              self.format = EmoteFormat::Static;
            } else {
              return None;
            }
          }

          let url = template
            .replace("{{id}}", &emote_data.id)
            .replace("{{format}}", &self.format.string())
            .replace("{{theme_mode}}", &self.theme.string())
            .replace("{{scale}}", &emote_data.scale[self.scale.idx()]);

          Some(EmoteUrl {
            url,
            animated: self.format == EmoteFormat::Animated,
          })
        } else {
          None
        }
      }
      FragmentType::BttvEmote => twitch.bttv.get_emote_url(&fragment.text, &self.scale),
      _ => None,
    }
  }
}
