use crate::modules::generic_message::*;
use crate::TwitchKeys;

use serde_derive::{Deserialize, Serialize};

macro_rules! from_string {
    ($enum_name:ident { $($variant:ident),* }) => {
        pub fn from_string(t: &str) -> Option<$enum_name> {
            $(
                if $enum_name::$variant.tag() == t {
                    return Some($enum_name::$variant);
                }
            )*
            None
        }
    };
}

#[derive(Clone, Debug)]
pub enum SubscriptionPermission {
  UserUpdate,
  ChannelFollow,
  ChannelRaid,
  ChannelUpdate,
  ChannelSubscribe,
  ChannelSubscriptionEnd,
  ChannelSubscriptionGift,
  ChannelSubscriptionMessage,
  ChannelCheer,
  ChannelPointsCustomRewardRedeem,
  ChannelPointsAutoRewardRedeem,
  ChannelPollBegin,
  ChannelPollProgress,
  ChannelPollEnd,
  ChannelPredictionBegin,
  ChannelPredictionProgress,
  ChannelPredictionLock,
  ChannelPredictionEnd,
  ChannelGoalBegin,
  ChannelGoalProgress,
  ChannelGoalEnd,
  ChannelHypeTrainBegin,
  ChannelHypeTrainProgress,
  ChannelHypeTrainEnd,
  ChannelShoutoutCreate,
  ChannelShoutoutReceive,
  ChatMessage,
  BanTimeoutUser,
  DeleteMessage,
  AdBreakBegin,
  Custom((String, String, EventSubscription)),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EventSubscription {
  #[serde(rename = "type")]
  pub kind: String,
  pub version: String,
  pub condition: Condition,
  pub transport: Transport,
}

impl SubscriptionPermission {
  from_string!(SubscriptionPermission {
    UserUpdate,
    ChannelFollow,
    ChannelRaid,
    ChannelUpdate,
    ChannelSubscribe,
    ChannelSubscriptionEnd,
    ChannelSubscriptionGift,
    ChannelSubscriptionMessage,
    ChannelCheer,
    ChannelPointsCustomRewardRedeem,
    ChannelPointsAutoRewardRedeem,
    ChannelPollBegin,
    ChannelPollProgress,
    ChannelPollEnd,
    ChannelPredictionBegin,
    ChannelPredictionProgress,
    ChannelPredictionLock,
    ChannelPredictionEnd,
    ChannelGoalBegin,
    ChannelGoalProgress,
    ChannelGoalEnd,
    ChannelHypeTrainBegin,
    ChannelHypeTrainProgress,
    ChannelHypeTrainEnd,
    ChannelShoutoutCreate,
    ChannelShoutoutReceive,
    ChatMessage,
    BanTimeoutUser,
    DeleteMessage,
    AdBreakBegin
  });

  fn details(&self) -> (String, String, String) {
    let details = match self {
      SubscriptionPermission::UserUpdate => ("user.update", "", "1"),
      SubscriptionPermission::ChannelFollow => ("channel.follow", "moderator:read:followers", "2"),
      SubscriptionPermission::ChannelRaid => ("channel.raid", "", "1"),
      SubscriptionPermission::ChatMessage => (
        "channel.chat.message",
        "user:read:chat+user:write:chat",
        "1",
      ),
      SubscriptionPermission::ChannelPointsCustomRewardRedeem => (
        "channel.channel_points_custom_reward_redemption.add",
        "channel:read:redemptions",
        "1",
      ),
      SubscriptionPermission::AdBreakBegin => ("channel.ad_break.begin", "channel:read:ads", "1"),
      SubscriptionPermission::ChannelUpdate => ("channel.update", "", "2"),
      SubscriptionPermission::BanTimeoutUser => ("", "moderator:manage:banned_users", ""),
      SubscriptionPermission::DeleteMessage => ("", "moderator:manage:chat_messages", ""),
      SubscriptionPermission::ChannelSubscribe => {
        ("channel.subscribe", "channel:read:subscriptions", "1")
      }
      SubscriptionPermission::ChannelSubscriptionEnd => (
        "channel.subscription.end",
        "channel:read:subscriptions",
        "1",
      ),
      SubscriptionPermission::ChannelSubscriptionGift => (
        "channel.subscription.gift",
        "channel:read:subscriptions",
        "1",
      ),
      SubscriptionPermission::ChannelSubscriptionMessage => (
        "channel.subscription.message",
        "channel:read:subscriptions",
        "1",
      ),
      SubscriptionPermission::ChannelCheer => ("channel.cheer", "bits:read", "1"),
      SubscriptionPermission::ChannelPointsAutoRewardRedeem => (
        "channel.channel_points_automatic_reward_redemption.add",
        "channel:read:redemptions",
        "1",
      ),
      SubscriptionPermission::ChannelPollBegin => (
        "channel.poll.begin",
        "channel:read:polls+channel:write:polls",
        "1",
      ),
      SubscriptionPermission::ChannelPollProgress => (
        "channel.poll.progress",
        "channel:read:polls+channel:write:polls",
        "1",
      ),
      SubscriptionPermission::ChannelPollEnd => (
        "channel.poll.end",
        "channel:read:polls+channel:write:polls",
        "1",
      ),
      SubscriptionPermission::ChannelPredictionBegin => (
        "channel.prediction.begin",
        "channel:read:predictions+channel:write:predictions",
        "1",
      ),
      SubscriptionPermission::ChannelPredictionProgress => (
        "channel.prediction.progress",
        "channel:read:predictions+channel:write:predictions",
        "1",
      ),
      SubscriptionPermission::ChannelPredictionLock => (
        "channel.prediction.lock",
        "channel:read:predictions+channel:write:predictions",
        "1",
      ),
      SubscriptionPermission::ChannelPredictionEnd => (
        "channel.prediction.end",
        "channel:read:predictions+channel:write:predictions",
        "1",
      ),
      SubscriptionPermission::ChannelGoalBegin => ("channel.goal.begin", "channel:read:goals", "1"),
      SubscriptionPermission::ChannelGoalProgress => {
        ("channel.goal.progress", "channel:read:goals", "1")
      }
      SubscriptionPermission::ChannelGoalEnd => ("channel.goal.end", "channel:read:goals", "1"),
      SubscriptionPermission::ChannelHypeTrainBegin => {
        ("channel.hype_train.begin", "channel:read:hype_train", "1")
      }
      SubscriptionPermission::ChannelHypeTrainProgress => (
        "channel.hype_train.progress",
        "channel:read:hype_train",
        "1",
      ),
      SubscriptionPermission::ChannelHypeTrainEnd => {
        ("channel.hype_train.end", "channel:read:hype_train", "1")
      }
      SubscriptionPermission::ChannelShoutoutCreate => (
        "channel.shoutout.create",
        "moderator:read:shoutouts+moderator:write:shoutouts",
        "1",
      ),
      SubscriptionPermission::ChannelShoutoutReceive => (
        "channel.shoutout.receive",
        "moderator:read:shoutouts+moderator:write:shoutouts",
        "1",
      ),
      SubscriptionPermission::Custom((tag, scope, ..)) => (tag.as_str(), scope.as_str(), ""),
    };

    (
      details.0.to_owned(),
      details.1.to_owned(),
      details.2.to_owned(),
    )
  }

  pub fn tag(&self) -> String {
    self.details().0
  }

  pub fn required_scope(&self) -> String {
    self.details().1
  }

  pub fn version(&self) -> String {
    self.details().2
  }

  pub fn construct_data(&self, session_id: &str, twitch_keys: &TwitchKeys) -> EventSubscription {
    let transport = Transport::new(session_id);

    let event_subscription = EventSubscription::new(self, transport);
    let condition =
      Condition::new().broadcaster_user_id(twitch_keys.broadcaster_account_id.to_owned());

    match self {
      SubscriptionPermission::UserUpdate => event_subscription
        .condition(Condition::new().user_id(twitch_keys.broadcaster_account_id.to_owned())),
      SubscriptionPermission::ChannelFollow => event_subscription.condition(
        condition
          .moderator_user_id(twitch_keys.broadcaster_account_id.to_owned())
          .user_id(twitch_keys.broadcaster_account_id.to_owned()),
      ),
      SubscriptionPermission::ChatMessage => event_subscription
        .condition(condition.user_id(twitch_keys.broadcaster_account_id.to_owned())),
      SubscriptionPermission::ChannelPointsCustomRewardRedeem => {
        event_subscription.condition(condition)
      }
      SubscriptionPermission::AdBreakBegin => event_subscription.condition(condition),
      SubscriptionPermission::ChannelRaid => event_subscription.condition(condition),
      SubscriptionPermission::ChannelUpdate => event_subscription.condition(condition),
      SubscriptionPermission::ChannelSubscribe => event_subscription.condition(condition),
      SubscriptionPermission::ChannelSubscriptionEnd => event_subscription.condition(condition),
      SubscriptionPermission::ChannelSubscriptionGift => event_subscription.condition(condition),
      SubscriptionPermission::ChannelSubscriptionMessage => event_subscription.condition(condition),
      SubscriptionPermission::Custom((_, _, event)) => {
        let mut event = event.to_owned();
        event = event.transport(Transport::new(session_id));
        event.to_owned()
      }
      _ => event_subscription,
    }
  }
}

impl EventSubscription {
  pub fn new(event: &SubscriptionPermission, transport: Transport) -> EventSubscription {
    EventSubscription {
      kind: event.tag(),
      version: event.version(),
      condition: Condition::new(),
      transport,
    }
  }

  pub fn transport(mut self, transport: Transport) -> EventSubscription {
    self.transport = transport;
    self
  }

  pub fn condition(mut self, condition: Condition) -> EventSubscription {
    self.condition = condition;
    self
  }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Condition {
  pub user_id: Option<String>,
  pub moderator_user_id: Option<String>,
  pub broadcaster_user_id: Option<String>,
  pub reward_id: Option<String>,
  pub from_broadcaster_user_id: Option<String>,
  pub to_broadcaster_user_id: Option<String>,
}

impl Condition {
  pub fn new() -> Condition {
    Condition {
      ..Default::default()
    }
  }

  pub fn user_id<S: Into<String>>(mut self, user_id: S) -> Condition {
    self.user_id = Some(user_id.into());
    self
  }

  pub fn moderator_user_id<S: Into<String>>(mut self, moderator_user_id: S) -> Condition {
    self.moderator_user_id = Some(moderator_user_id.into());
    self
  }

  pub fn broadcaster_user_id<S: Into<String>>(mut self, broadcaster_user_id: S) -> Condition {
    self.broadcaster_user_id = Some(broadcaster_user_id.into());
    self
  }

  pub fn reward_id<S: Into<String>>(mut self, reward_id: S) -> Condition {
    self.reward_id = Some(reward_id.into());
    self
  }

  pub fn from_broadcaster_user_id<S: Into<String>>(
    mut self,
    from_broadcaster_user_id: S,
  ) -> Condition {
    self.from_broadcaster_user_id = Some(from_broadcaster_user_id.into());
    self
  }

  pub fn to_broadcaster_user_id<S: Into<String>>(mut self, to_broadcaster_user_id: S) -> Condition {
    self.to_broadcaster_user_id = Some(to_broadcaster_user_id.into());
    self
  }
}
