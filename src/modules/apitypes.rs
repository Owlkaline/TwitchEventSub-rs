use crate::{Deserialise, Serialise};

#[derive(Serialise, Deserialise, Debug)]
pub struct AdSchedule {
  pub data: Vec<AdDetails>,
}

#[derive(Serialise, Deserialise, Debug)]
pub struct AdDetails {
  pub next_ad_at: u32,
  pub last_ad_at: u32,
  pub duration: u32,
  pub preroll_free_time: u32,
  pub snooze_count: u32,
  pub snooze_refresh_at: u32,
}
