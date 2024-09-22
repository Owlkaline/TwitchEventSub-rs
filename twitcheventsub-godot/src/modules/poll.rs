use godot::prelude::*;

use twitcheventsub::*;

use crate::modules::GUser;

#[derive(GodotClass, Debug)]
#[class(init)]
pub struct GChoices {
  #[var]
  id: GString,
  #[var]
  title: GString,
  #[var]
  votes: u32,
  #[var]
  channel_points_votes: u32,
  #[var]
  bits_votes: u32,
}

#[derive(GodotClass, Debug)]
#[class(init)]
pub struct GBeginChoices {
  #[var]
  id: GString,
  #[var]
  title: GString,
}

#[derive(GodotClass, Debug, Default)]
#[class(init)]
pub struct GBitsVotingData {
  #[var]
  is_enabled: bool,
  #[var]
  amount_per_vote: u32,
}

#[derive(GodotClass, Debug, Default)]
#[class(init)]
pub struct GChannelPointsVoting {
  #[var]
  is_enabled: bool,
  #[var]
  amount_per_vote: u32,
}

#[derive(GodotClass, Debug)]
#[class(init)]
pub struct GPollBegin {
  #[var]
  id: GString,
  #[var]
  broadcaster: Gd<GUser>,
  #[var]
  title: GString,
  #[var]
  choices: Array<Gd<GBeginChoices>>,
  #[var]
  bits_voting: Gd<GBitsVotingData>,
  #[var]
  channel_points_voting: Gd<GChannelPointsVoting>,
  #[var]
  started_at: GString,
  #[var]
  ends_at: GString,
}

#[derive(GodotClass, Debug)]
#[class(init)]
pub struct GPollProgress {
  #[var]
  id: GString,
  #[var]
  broadcaster: Gd<GUser>,
  #[var]
  title: GString,
  #[var]
  choices: Array<Gd<GChoices>>,
  #[var]
  bits_voting: Gd<GBitsVotingData>,
  #[var]
  channel_points_voting: Gd<GChannelPointsVoting>,
  #[var]
  started_at: GString,
  #[var]
  ends_at: GString,
}

#[derive(GodotClass, Debug)]
#[class(init)]
pub struct GPollEnd {
  #[var]
  id: GString,
  #[var]
  broadcaster: Gd<GUser>,
  #[var]
  title: GString,
  #[var]
  choices: Array<Gd<GChoices>>,
  #[var]
  bits_voting: Gd<GBitsVotingData>,
  #[var]
  channel_points_voting: Gd<GChannelPointsVoting>,
  #[var]
  started_at: GString,
  #[var]
  ended_at: GString,
  #[var]
  status: GString,
}

impl From<PollBeginData> for GPollBegin {
  fn from(poll: PollBeginData) -> Self {
    let mut choices = Array::new();

    for choice in poll.choices {
      choices.push(Gd::from_object(GBeginChoices::from(choice)));
    }

    GPollBegin {
      id: poll.id.into(),
      broadcaster: Gd::from_object(GUser::from(poll.broadcaster)),
      title: poll.title.into(),
      choices,
      bits_voting: Gd::from_object(GBitsVotingData::from(poll.bits_voting)),
      channel_points_voting: Gd::from_object(GChannelPointsVoting::from(
        poll.channel_points_voting,
      )),
      started_at: poll.started_at.into(),
      ends_at: poll.ends_at.into(),
    }
  }
}

impl From<PollProgressData> for GPollProgress {
  fn from(poll: PollProgressData) -> Self {
    let mut choices = Array::new();

    for choice in poll.choices {
      choices.push(Gd::from_object(GChoices::from(choice)));
    }

    GPollProgress {
      id: poll.id.into(),
      broadcaster: Gd::from_object(GUser::from(poll.broadcaster)),
      title: poll.title.into(),
      choices,
      bits_voting: Gd::from_object(GBitsVotingData::from(poll.bits_voting)),
      channel_points_voting: Gd::from_object(GChannelPointsVoting::from(
        poll.channel_points_voting,
      )),
      started_at: poll.started_at.into(),
      ends_at: poll.ends_at.into(),
    }
  }
}

impl From<PollEndData> for GPollEnd {
  fn from(poll: PollEndData) -> Self {
    let mut choices = Array::new();

    for choice in poll.choices {
      choices.push(Gd::from_object(GChoices::from(choice)));
    }

    GPollEnd {
      id: poll.id.into(),
      broadcaster: Gd::from_object(GUser::from(poll.broadcaster)),
      title: poll.title.into(),
      choices,
      bits_voting: Gd::from_object(GBitsVotingData::from(poll.bits_voting)),
      channel_points_voting: Gd::from_object(GChannelPointsVoting::from(
        poll.channel_points_voting,
      )),
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
