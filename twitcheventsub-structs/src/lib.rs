use serde::{Deserialize as Deserialise, Serialize as Serialise};

mod api_structs;
mod eventsub_structs;
mod response_messages;
mod subscriptions;

pub mod prelude {
  pub use crate::api_structs::*;
  pub use crate::eventsub_structs::*;
  pub use crate::response_messages::*;
  pub use crate::subscriptions::*;
}
