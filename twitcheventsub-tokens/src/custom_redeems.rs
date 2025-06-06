use twitcheventsub_api::TwitchApiError;
use twitcheventsub_structs::prelude::{GetCustomRewards, UpdateCustomReward};

use crate::TokenHandler;

pub struct RedeemsHandler {
  all_redeems: GetCustomRewards,
}

impl RedeemsHandler {
  pub fn new(tokens: &mut TokenHandler) -> Result<RedeemsHandler, TwitchApiError> {
    let twitch_id = tokens.client_twitch_id.clone();
    let all_redeems = tokens.get_custom_rewards(&twitch_id)?;

    Ok(RedeemsHandler { all_redeems })
  }

  pub fn create_or_enable_redeem(
    &mut self,
    mut tokens: &mut TokenHandler,
    redeem: ManageRedeem,
  ) -> bool {
    if let Some(title) = &redeem.update.title {
      for (i, redeem_id) in
        self
          .all_redeems
          .data
          .iter()
          .enumerate()
          .filter_map(|(i, other_redeem)| {
            if other_redeem.title.eq(title) {
              Some((i, other_redeem.id.to_owned()))
            } else {
              None
            }
          })
      {
        // id
        let twitch_id = tokens.client_twitch_id.to_owned();
        tokens
          .update_custom_rewards(&twitch_id, &redeem_id, &redeem.update)
          .and_then(|mut updated_reward| {
            let new_redeem = updated_reward.data.remove(0);

            //  self.all_redeems.data.remove(i);
            //  self.all_redeems.data.push(new_redeem);
            Ok(())
          });
      }
    }

    false
  }
}

impl Default for ManageRedeem {
  fn default() -> Self {
    ManageRedeem {
      update: UpdateCustomReward::default(),
    }
  }
}

pub struct ManageRedeem {
  update: UpdateCustomReward,
}

impl ManageRedeem {
  pub fn get_all() -> ManageRedeem {
    ManageRedeem::default()
  }

  pub fn title(mut self, title: &str) -> ManageRedeem {
    self.update.title = Some(String::from(title));
    self
  }

  pub fn prompt(mut self, prompt: &str) -> ManageRedeem {
    self.update.prompt = Some(String::from(prompt));
    self
  }

  pub fn cost(mut self, cost: i64) -> ManageRedeem {
    self.update.cost = Some(cost);
    self
  }

  pub fn background_colour(mut self, colour: &str) -> ManageRedeem {
    self.update.background_colour = Some(String::from(colour));
    self
  }

  pub fn enable(mut self, enabled: bool) -> ManageRedeem {
    self.update.is_enabled = Some(enabled);
    self
  }

  pub fn pause(mut self, paused: bool) -> ManageRedeem {
    self.update.is_paused = Some(paused);
    self
  }

  pub fn requires_user_input(mut self, required: bool) -> ManageRedeem {
    self.update.is_user_input_required = Some(required);
    self
  }

  pub fn max_redeems_per_stream(mut self, max_redemptions_per_stream: i64) -> ManageRedeem {
    if max_redemptions_per_stream > 0 {
      self.update.is_max_per_stream_enabled = Some(false);
    } else {
      self.update.is_max_per_stream_enabled = Some(true);
      self.update.max_per_stream = Some(max_redemptions_per_stream);
    }
    self
  }

  pub fn max_redeems_per_user_per_stream(
    mut self,
    max_redemptions_per_user_per_stream: i64,
  ) -> ManageRedeem {
    if max_redemptions_per_user_per_stream > 0 {
      self.update.is_max_per_user_per_stream_enabled = Some(false);
    } else {
      self.update.is_max_per_user_per_stream_enabled = Some(true);
      self.update.max_per_user_per_stream = Some(max_redemptions_per_user_per_stream);
    }
    self
  }

  pub fn global_cooldown(mut self, duration_secs: i64) -> ManageRedeem {
    if duration_secs > 0 {
      self.update.is_global_cooldown_enabled = Some(true);
      self.update.global_cooldown_seconds = Some(duration_secs);
    } else {
      self.update.is_global_cooldown_enabled = Some(false);
    }
    self
  }

  pub fn skip_request_queue(mut self, should_skip: bool) -> ManageRedeem {
    self.update.should_redeptions_skip_request_queue = Some(should_skip);
    self
  }

  pub fn new() -> ManageRedeem {
    ManageRedeem::default()
  }
}
