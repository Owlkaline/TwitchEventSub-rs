use godot::prelude::*;
use twitcheventsub::prelude::*;

use crate::modules::GUser;

#[derive(GodotClass, Debug)]
#[class(init)]
pub struct GPagination {
  #[var]
  cursor: Array<GString>,
}

#[derive(GodotClass, Debug)]
#[class(init)]
pub struct GGetChatters {
  #[var]
  data: Array<Gd<GUser>>,
  #[var]
  pagination: Gd<GPagination>,
  #[var]
  total: i32,
}

impl From<GetChatters> for GGetChatters {
  fn from(get_chatters: GetChatters) -> GGetChatters {
    let mut cursor = Array::new();
    let mut data = Array::new();

    if let Some(cursor_data) = get_chatters.pagination.cursor {
      cursor.push(&cursor_data);
    }

    for i in 0..get_chatters.data.len() {
      let user = get_chatters.data[i].to_owned();
      data.push(&Gd::from_object(GUser::from(user)));
    }

    GGetChatters {
      data,
      pagination: Gd::from_object(GPagination { cursor }),
      total: get_chatters.total.into(),
    }
  }
}
