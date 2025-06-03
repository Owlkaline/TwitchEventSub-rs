//pub const BTTV_ENDPOINT: &str = "https://api.betterttv.net/3/";
//pub const BTTV_USER: &str = "/cached/users/";

use serde::Deserialize as Deserialise;
use serde_json;
use twitcheventsub_api::TwitchHttpRequest;
use twitcheventsub_structs::{EmoteScale, EmoteUrl};

pub const _BTTV_GLOBAL_EMOTES: &str = "https://api.betterttv.net/3/cached/emotes/global";
pub const _BTTV_CHANNEL_EMOTES: &str =
  "https://twitch.center/customapi/bttvemotes?channel={username}";
pub const BTTV_CHANNEL_EMOTES_FROM_ID: &str =
  "https://api.betterttv.net/3/cached/users/twitch/{id}";

pub const BTTV_EMOTE_URL: &str = "https://cdn.betterttv.net/emote/{id}/{scale}";

//825175324

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Debug)]
pub struct BTTV {
  pub response: Option<BttvUserResponse>,
  pub emote_names: Vec<String>,
}

impl BTTV {
  pub fn new<S: Into<String>>(id: S) -> BTTV {
    let mut bttv = BTTV {
      response: None,
      emote_names: Vec::new(),
    };

    if let Ok(user_emotes) =
      TwitchHttpRequest::new(BTTV_CHANNEL_EMOTES_FROM_ID.replace("{id}", &id.into())).run()
    {
      //pikaOMG
      //825175324
      if let Ok(output) = serde_json::from_str::<BttvUserResponse>(&user_emotes) {
        bttv.emote_names = output
          .channel_emotes
          .iter()
          .map(|e| e.code.to_lowercase().clone())
          .collect::<Vec<_>>();
        bttv.emote_names.append(
          &mut output
            .shared_emotes
            .iter()
            .map(|e| e.code.to_lowercase().clone())
            .collect::<Vec<_>>(),
        );

        bttv.response = Some(output);
      }
    }

    bttv
  }

  pub fn get_emote_url(&self, text: &str, scale: &EmoteScale) -> Option<EmoteUrl> {
    if let Some(response) = &self.response {
      let mut emote = response
        .channel_emotes
        .iter()
        .filter(|e| e.code.to_lowercase().eq(&text.to_lowercase()))
        .collect::<Vec<_>>();
      emote.append(
        &mut response
          .shared_emotes
          .iter()
          .filter(|e| e.code.to_lowercase().eq(&text.to_lowercase()))
          .collect::<Vec<_>>(),
      );

      if let Some(emote) = emote.into_iter().next() {
        return Some(EmoteUrl {
          url: BTTV_EMOTE_URL
            .replace("{id}", &emote.id)
            .replace("{scale}", &scale.to_string()),
          animated: emote.animated,
        });
      }
    } else {
      println!("failed to get bttv response");
    }

    None
  }
}

#[derive(Deserialise, Debug, Clone)]
pub struct BttvUserResponse {
  #[serde(rename = "id")]
  _id: String,
  #[serde(rename = "bots")]
  _bots: Vec<String>,
  #[serde(rename = "avatar")]
  _avatar: String,
  #[serde(rename = "channelEmotes")]
  channel_emotes: Vec<BttvEmote>,
  #[serde(rename = "sharedEmotes")]
  shared_emotes: Vec<BttvEmote>,
}

#[derive(Deserialise, Debug, Clone)]
struct BttvEmote {
  id: String,
  code: String,
  #[serde(rename = "imageType")]
  _image_type: String,
  animated: bool,
  #[serde(rename = "userId")]
  _user_id: Option<String>,
  #[serde(rename = "user")]
  _user: Option<BttvUser>,
}

#[derive(Deserialise, Debug, Clone)]
struct BttvUser {
  #[serde(rename = "id")]
  _id: String,
  #[serde(rename = "name")]
  _name: String,
  #[serde(rename = "displayName")]
  _display_name: String,
  #[serde(rename = "providerId")]
  _provider_id: String,
}
