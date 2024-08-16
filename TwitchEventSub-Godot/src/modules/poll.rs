use godot::prelude::*;

use twitch_eventsub::*;

use crate::modules::GUser;

#[derive(GodotClass, Debug)]
#[class(init)]
pub struct GChoices {
  id: GString,
  title: GString,
  votes: u32,
  channel_points_votes: u32,
  bits_votes: u32,
}

#[derive(GodotClass, Debug)]
#[class(init)]
pub struct GBeginChoices {
  id: GString,
  title: GString,
}

#[derive(GodotClass, Debug, Default)]
#[class(init)]
pub struct GBitsVotingData {
  is_enabled: bool,
  amount_per_vote: u32,
}

#[derive(GodotClass, Debug, Default)]
#[class(init)]
pub struct GChannelPointsVoting {
  is_enabled: bool,
  amount_per_vote: u32,
}

#[derive(GodotClass, Debug)]
#[class(init)]
pub struct GPollBeginData {
  #[var]
  id: GString,
  broadcaster: Gd<GUser>,
  title: GString,
  choices: Array<Gd<GBeginChoices>>,
  bits_voting: GBitsVotingData,
  channel_points_voting: GChannelPointsVoting,
  started_at: GString,
  ends_at: GString,
}

#[derive(GodotClass, Debug)]
#[class(init)]
pub struct GPollProgressData {
  #[var]
  id: GString,
  broadcaster: Gd<GUser>,
  title: GString,
  choices: Array<Gd<GChoices>>,
  bits_voting: GBitsVotingData,
  channel_points_voting: GChannelPointsVoting,
  started_at: GString,
  ends_at: GString,
}

#[derive(GodotClass, Debug)]
#[class(init)]
pub struct GPollEndData {
  #[var]
  id: GString,
  broadcaster: Gd<GUser>,
  title: GString,
  choices: Array<Gd<GChoices>>,
  bits_voting: GBitsVotingData,
  channel_points_voting: GChannelPointsVoting,
  started_at: GString,
  ended_at: GString,
  status: GString,
}

impl From<PollBeginData> for GPollBeginData {
  fn from(poll: PollBeginData) -> Self {
    let mut choices = Array::new();

    for choice in poll.choices {
      choices.push(Gd::from_object(GBeginChoices::from(choice)));
    }

    GPollBeginData {
      id: poll.id.into(),
      broadcaster: Gd::from_object(GUser::from(poll.broadcaster)),
      title: poll.title.into(),
      choices,
      bits_voting: GBitsVotingData::from(poll.bits_voting),
      channel_points_voting: GChannelPointsVoting::from(poll.channel_points_voting),
      started_at: poll.started_at.into(),
      ends_at: poll.ends_at.into(),
    }
  }
}

impl From<PollProgressData> for GPollProgressData {
  fn from(poll: PollProgressData) -> Self {
    let mut choices = Array::new();

    for choice in poll.choices {
      choices.push(Gd::from_object(GChoices::from(choice)));
    }

    GPollProgressData {
      id: poll.id.into(),
      broadcaster: Gd::from_object(GUser::from(poll.broadcaster)),
      title: poll.title.into(),
      choices,
      bits_voting: GBitsVotingData::from(poll.bits_voting),
      channel_points_voting: GChannelPointsVoting::from(poll.channel_points_voting),
      started_at: poll.started_at.into(),
      ends_at: poll.ends_at.into(),
    }
  }
}

impl From<PollEndData> for GPollEndData {
  fn from(poll: PollEndData) -> Self {
    let mut choices = Array::new();

    for choice in poll.choices {
      choices.push(Gd::from_object(GChoices::from(choice)));
    }

    GPollEndData {
      id: poll.id.into(),
      broadcaster: Gd::from_object(GUser::from(poll.broadcaster)),
      title: poll.title.into(),
      choices,
      bits_voting: GBitsVotingData::from(poll.bits_voting),
      channel_points_voting: GChannelPointsVoting::from(poll.channel_points_voting),
      started_at: poll.started_at.into(),
      ended_at: poll.ended_at.into(),
      status: poll.status.into(),
    }
  }
}

impl From<BitsVotingData> for GBitsVotingData {
  fn from(bits_voting: BitsVotingData) -> GBitsVotingData {
    GBitsVotingData {
      is_enabled: bits_voting.is_enabled,
      amount_per_vote: bits_voting.amount_per_vote,
    }
  }
}

impl From<ChannelPointsVoting> for GChannelPointsVoting {
  fn from(channel_points_voting: ChannelPointsVoting) -> GChannelPointsVoting {
    GChannelPointsVoting {
      is_enabled: channel_points_voting.is_enabled,
      amount_per_vote: channel_points_voting.amount_per_vote,
    }
  }
}

impl From<BeginChoices> for GBeginChoices {
  fn from(begin_choices: BeginChoices) -> GBeginChoices {
    GBeginChoices {
      id: begin_choices.id.into(),
      title: begin_choices.title.into(),
    }
  }
}

impl From<Choices> for GChoices {
  fn from(choices: Choices) -> GChoices {
    GChoices {
      id: choices.id.into(),
      title: choices.title.into(),
      votes: choices.votes,
      channel_points_votes: choices.channel_points_votes,
      bits_votes: choices.bits_votes,
    }
  }
}
