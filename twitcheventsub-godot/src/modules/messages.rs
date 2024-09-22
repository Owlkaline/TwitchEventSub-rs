use godot::prelude::*;

use twitcheventsub::*;

use crate::modules::{cheer::GCheerMote, emote::GEmote, GUser};

#[derive(GodotClass, Debug)]
#[class(init)]
pub struct GBadge {
  #[var]
  set_id: GString,
  #[var]
  id: GString,
  #[var]
  info: GString,
}

#[derive(GodotClass, Debug)]
#[class(init)]
pub struct GCheer {
  #[var]
  bits: u32,
}

#[derive(GodotClass, Debug)]
#[class(init)]
pub struct GReply {
  #[var]
  thread: Gd<GUser>,
  #[var]
  parent: Gd<GUser>,
  #[var]
  parent_message_id: GString,
  #[var]
  parent_message_body: GString,
  #[var]
  thread_message_id: GString,
}

#[derive(GodotClass)]
#[class(init)]
pub struct GMention {
  #[var]
  pub user_id: GString,
  #[var]
  pub user_login: GString,
  #[var]
  pub user_name: GString,
}

#[derive(GodotClass, Debug)]
#[class(init)]
pub struct GFragments {
  #[var]
  pub kind: GString,
  #[var]
  pub text: GString,
  #[var]
  pub cheermote: Array<Gd<GCheerMote>>,
  #[var]
  pub emote: Array<Gd<GEmote>>,
  #[var]
  pub mention: Array<Gd<GMention>>,
}

#[derive(GodotClass, Debug)]
#[class(init)]
pub struct GMessage {
  #[var]
  text: GString,
  #[var]
  fragments: Array<Gd<GFragments>>,
}

#[derive(GodotClass, Debug)]
#[class(init)]
pub struct GMessageData {
  #[var]
  broadcaster: Gd<GUser>,
  #[var]
  chatter: Gd<GUser>,
  #[var]
  pub message_id: GString,
  #[var]
  pub message: Gd<GMessage>,
  #[var]
  colour: GString,
  #[var]
  badges: Array<Gd<GBadge>>,
  #[var]
  cheer: Array<Gd<GCheer>>,
  #[var]
  reply: Array<Gd<GReply>>,
  #[var]
  channel_points_custom_reward_id: GString,
  #[var]
  channel_points_animation_id: GString,
  #[var]
  first_time_chatter: bool,
  #[var]
  returning_chater: bool,
  #[var]
  moderator: bool,
}

impl From<Badge> for GBadge {
  fn from(badge: Badge) -> GBadge {
    GBadge {
      set_id: badge.set_id.to_owned().into(),
      id: badge.id.to_owned().into(),
      info: badge.info.to_owned().into(),
    }
  }
}

impl From<Cheer> for GCheer {
  fn from(cheer: Cheer) -> GCheer {
    GCheer {
      bits: cheer.clone().bits,
    }
  }
}

impl From<Reply> for GReply {
  fn from(reply: Reply) -> Self {
    GReply {
      thread: Gd::from_object(GUser::from(reply.thread)),
      parent: Gd::from_object(GUser::from(reply.parent)),
      parent_message_id: reply.parent_message_id.to_owned().into(),
      parent_message_body: reply.parent_message_body.to_owned().into(),
      thread_message_id: reply.thread_message_id.to_owned().into(),
    }
  }
}

impl From<MessageData> for GMessageData {
  fn from(msg: MessageData) -> GMessageData {
    let mut cheer = Array::new();
    let mut reply = Array::new();
    let mut badges = Array::new();

    if let Some(cheer_data) = msg.cheer {
      cheer.push(Gd::from_object(GCheer::from(cheer_data)));
    }

    if let Some(reply_data) = msg.reply {
      reply.push(Gd::from_object(GReply::from(reply_data)));
    }

    for i in 0..msg.badges.len() {
      let badge = msg.badges[i].to_owned();
      badges.push(Gd::from_object(GBadge::from(badge)));
    }

    GMessageData {
      broadcaster: Gd::from_object(GUser::from(msg.broadcaster)),
      chatter: Gd::from_object(GUser::from(msg.chatter)),
      message_id: msg.message_id.to_owned().into(),
      message: Gd::from_object(GMessage::from(msg.message)),
      colour: msg.colour.to_owned().into(),
      channel_points_custom_reward_id: msg
        .channel_points_custom_reward_id
        .unwrap_or("".to_owned())
        .to_owned()
        .into(),
      channel_points_animation_id: msg
        .channel_points_animation_id
        .unwrap_or("".to_owned())
        .to_owned()
        .into(),
      cheer,
      reply,
      badges,
      first_time_chatter: msg.first_time_chatter,
      returning_chater: msg.returning_chatter,
      moderator: msg.moderator,
    }
  }
}

impl From<Fragments> for GFragments {
  fn from(value: Fragments) -> Self {
    let mut cheermote = Array::new();
    let mut emote = Array::new();
    let mut mention = Array::new();

    if let Some(frag_cheermote) = value.cheermote {
      cheermote.push(Gd::from_object(GCheerMote::from(frag_cheermote)));
    }

    if let Some(frag_emote) = value.emote {
      emote.push(Gd::from_object(GEmote::from(frag_emote)));
    }

    if let Some(frag_mention) = value.mention {
      mention.push(Gd::from_object(GMention::from(frag_mention)));
    }

    let value_kind: String = value.kind.into();
    GFragments {
      kind: value_kind.into(),
      text: value.text.into(),
      cheermote,
      emote,
      mention,
    }
  }
}

impl From<Message> for GMessage {
  fn from(value: Message) -> Self {
    let mut fragments = Array::new();

    for frag in value.fragments {
      fragments.push(Gd::from_object(GFragments::from(frag)));
    }

    GMessage {
      text: value.text.into(),
      fragments,
    }
  }
}

impl From<Mention> for GMention {
  fn from(value: Mention) -> Self {
    GMention {
      user_id: value.user_id.into(),
      user_login: value.user_login.into(),
      user_name: value.user_name.into(),
    }
  }
}
