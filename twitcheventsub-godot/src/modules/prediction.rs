use godot::prelude::*;
use twitcheventsub::prelude::{
  BeginOutcome, Outcome, PredictionBeginData, PredictionEndData, PredictionLockData,
  PredictionProgressData, TopPredictors,
};

use super::GUser;

#[derive(GodotClass, Debug)]
#[class(init)]
pub struct GTopPredictors {
  #[var]
  pub user: Gd<GUser>,
  #[var]
  pub channel_points_won: Array<u32>,
  #[var]
  pub channel_points_used: u32,
}

#[derive(GodotClass, Debug)]
#[class(init)]
pub struct GBeginOutcome {
  #[var]
  id: GString,
  #[var]
  title: GString,
  #[var]
  colour: GString,
}

#[derive(GodotClass, Debug)]
#[class(init)]
pub struct GOutcome {
  #[var]
  pub id: GString,
  #[var]
  pub title: GString,
  #[var]
  pub colour: GString,
  #[var]
  pub users: u32,
  #[var]
  pub channel_points: u32,
  #[var]
  pub top_predictors: Array<Gd<GTopPredictors>>,
}

#[derive(GodotClass, Debug)]
#[class(init)]
pub struct GPredictionBegin {
  #[var]
  pub id: GString,
  #[var]
  pub broadcaster: Gd<GUser>,
  #[var]
  pub title: GString,
  #[var]
  pub outcomes: Array<Gd<GBeginOutcome>>,
  #[var]
  pub started_at: GString,
  #[var]
  pub locks_at: GString,
}

#[derive(GodotClass, Debug)]
#[class(init)]
pub struct GPredictionProgress {
  #[var]
  pub id: GString,
  #[var]
  broadcaster: Gd<GUser>,
  #[var]
  pub title: GString,
  #[var]
  pub outcomes: Array<Gd<GOutcome>>,
  #[var]
  pub started_at: GString,
  #[var]
  pub locks_at: GString,
}

#[derive(GodotClass, Debug)]
#[class(init)]
pub struct GPredictionLock {
  #[var]
  pub id: GString,
  #[var]
  pub broadcaster: Gd<GUser>,
  #[var]
  pub title: GString,
  #[var]
  pub outcomes: Array<Gd<GOutcome>>,
  #[var]
  pub started_at: GString,
  #[var]
  pub locked_at: GString,
}

#[derive(GodotClass, Debug)]
#[class(init)]
pub struct GPredictionEnd {
  #[var]
  pub id: GString,
  #[var]
  pub broadcaster: Gd<GUser>,
  #[var]
  pub title: GString,
  #[var]
  pub winning_outcome_id: GString,
  #[var]
  pub outcomes: Array<Gd<GOutcome>>,
  #[var]
  pub status: GString,
  #[var]
  pub started_at: GString,
  #[var]
  pub ended_at: GString,
}

impl From<TopPredictors> for GTopPredictors {
  fn from(value: TopPredictors) -> Self {
    let mut channel_points_won = Array::new();

    if let Some(points) = value.channel_points_won {
      channel_points_won.push(points);
    }

    GTopPredictors {
      user: Gd::from_object(GUser::from(value.user)),
      channel_points_won,
      channel_points_used: value.channel_points_used,
    }
  }
}

impl From<BeginOutcome> for GBeginOutcome {
  fn from(value: BeginOutcome) -> Self {
    GBeginOutcome {
      id: value.id.to_godot(),
      title: value.title.to_godot(),
      colour: value.colour.to_godot(),
    }
  }
}

impl From<Outcome> for GOutcome {
  fn from(value: Outcome) -> Self {
    let mut top_predictors = Array::new();

    for predictor in value.top_predictors {
      top_predictors.push(&Gd::from_object(GTopPredictors::from(predictor)));
    }

    GOutcome {
      id: value.id.to_godot(),
      title: value.title.to_godot(),
      colour: value.colour.to_godot(),
      users: value.users,
      channel_points: value.channel_points,
      top_predictors,
    }
  }
}

impl From<PredictionBeginData> for GPredictionBegin {
  fn from(value: PredictionBeginData) -> Self {
    let mut outcomes = Array::new();

    for outcome in value.outcomes {
      outcomes.push(&Gd::from_object(GBeginOutcome::from(outcome)));
    }

    GPredictionBegin {
      id: value.id.to_godot(),
      broadcaster: Gd::from_object(GUser::from(value.broadcaster)),
      title: value.title.to_godot(),
      outcomes,
      started_at: value.started_at.to_godot(),
      locks_at: value.locks_at.to_godot(),
    }
  }
}

impl From<PredictionProgressData> for GPredictionProgress {
  fn from(value: PredictionProgressData) -> Self {
    let mut outcomes = Array::new();

    for outcome in value.outcomes {
      outcomes.push(&Gd::from_object(GOutcome::from(outcome)));
    }

    GPredictionProgress {
      id: value.id.to_godot(),
      broadcaster: Gd::from_object(GUser::from(value.broadcaster)),
      title: value.title.to_godot(),
      outcomes,
      started_at: value.started_at.to_godot(),
      locks_at: value.locks_at.to_godot(),
    }
  }
}

impl From<PredictionEndData> for GPredictionEnd {
  fn from(value: PredictionEndData) -> Self {
    let mut outcomes = Array::new();

    for outcome in value.outcomes {
      outcomes.push(&Gd::from_object(GOutcome::from(outcome)));
    }

    GPredictionEnd {
      id: value.id.to_godot(),
      broadcaster: Gd::from_object(GUser::from(value.broadcaster)),
      title: value.title.to_godot(),
      winning_outcome_id: value.winning_outcome_id.to_godot(),
      outcomes,
      status: value.status.to_godot(),
      started_at: value.started_at.to_godot(),
      ended_at: value.ended_at.to_godot(),
    }
  }
}

impl From<PredictionLockData> for GPredictionLock {
  fn from(value: PredictionLockData) -> Self {
    let mut outcomes = Array::new();

    for outcome in value.outcomes {
      outcomes.push(&Gd::from_object(GOutcome::from(outcome)));
    }

    GPredictionLock {
      id: value.id.to_godot(),
      broadcaster: Gd::from_object(GUser::from(value.broadcaster)),
      title: value.title.to_godot(),
      outcomes,
      started_at: value.started_at.to_godot(),
      locked_at: value.locked_at.to_godot(),
    }
  }
}
