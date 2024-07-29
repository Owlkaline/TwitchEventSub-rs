//use twitch_eventsub_structs::serde::{Deserialize as Deserialise, Serialize as Serialise};
#[macro_use]
use crate::serde::{self, Deserialize as Deserialise, Serialize as Serialise};

use twitch_eventsub_structs::User;

#[derive(Serialise, Deserialise, Debug)]
#[serde(crate = "self::serde")]
pub struct AdSchedule {
  pub data: Vec<AdDetails>,
}

#[derive(Serialise, Deserialise, Debug)]
#[serde(crate = "self::serde")]
pub struct AdDetails {
  pub next_ad_at: u32,
  pub last_ad_at: u32,
  pub duration: u32,
  pub preroll_free_time: u32,
  pub snooze_count: u32,
  pub snooze_refresh_at: u32,
}

#[derive(Serialise, Deserialise, Debug)]
#[serde(crate = "self::serde")]
pub struct Pagination {
  pub cursor: Option<String>,
}

#[derive(Serialise, Deserialise, Debug)]
#[serde(crate = "self::serde")]
pub struct GetChatters {
  pub data: Vec<User>,
  pub pagination: Pagination,
  pub total: i32,
}
