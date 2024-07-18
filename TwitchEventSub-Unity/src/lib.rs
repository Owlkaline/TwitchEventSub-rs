extern crate rand;

use rand::Rng;
use std::ffi::CString;
use std::mem::transmute;

use twitch_eventsub::*;

pub struct TwitchEvents {
  api: TwitchEventSubApi,
}

#[no_mangle]
pub extern "C" fn create_twitch_events() -> *mut TwitchEventSubApiBuilder {
  let keys = TwitchKeys::from_secrets_env().unwrap();

  let mut twitch = TwitchEventSubApi::builder(keys.clone())
    .set_redirect_url(
      "http://localhost:3000", // CString::from("http://localhost:3000".as_bytes())
                               //   .to_str()
                               //   .unwrap()
                               //   .to_string(),
    )
    .generate_new_token_if_insufficent_scope(true)
    .generate_new_token_if_none(true)
    .generate_access_token_on_expire(true)
    .auto_save_load_created_tokens(".user_token.env", ".refresh_token.env")
    .add_subscriptions(vec![
      //Subscription::UserUpdate,
      Subscription::ChannelFollow,
      Subscription::ChannelRaid,
      //Subscription::ChannelUpdate,
      Subscription::ChannelNewSubscription,
      //Subscription::ChannelSubscriptionEnd,
      Subscription::ChannelGiftSubscription,
      Subscription::ChannelResubscription,
      Subscription::ChannelCheer,
      Subscription::ChannelPointsCustomRewardRedeem,
      Subscription::ChannelPointsAutoRewardRedeem,
      //Subscription::ChannelPollBegin,
      //Subscription::ChannelPollProgress,
      //Subscription::ChannelPollEnd,
      //Subscription::ChannelPredictionBegin,
      //Subscription::ChannelPredictionProgress,
      //Subscription::ChannelPredictionLock,
      //Subscription::ChannelPredictionEnd,
      //Subscription::ChannelGoalBegin,
      //Subscription::ChannelGoalProgress,
      //Subscription::ChannelGoalEnd,
      //Subscription::ChannelHypeTrainBegin,
      //Subscription::ChannelHypeTrainProgress,
      //Subscription::ChannelHypeTrainEnd,
      //Subscription::ChannelShoutoutCreate,
      //Subscription::ChannelShoutoutReceive,
      Subscription::ChatMessage,
      //Subscription::BanTimeoutUser,
      Subscription::DeleteMessage,
      Subscription::AdBreakBegin,
    ]);

  let twitch = twitch.build().unwrap();

  unsafe { transmute(Box::new(twitch)) }
}

#[no_mangle]
pub extern "C" fn get_event(twitch: *mut TwitchEvents) -> CString {
  let twitch = unsafe { &mut *twitch };

  for response in twitch.api.receive_all_messages(None) {
    match response {
      ResponseType::Event(event) => match event {
        Event::ChatMessage(message_data) => {
          println!("chat message recieved");
          return CString::new(message_data.message.text).unwrap();
        }
        _ => {}
      },
      ResponseType::RawResponse(raw_string) => {
        return CString::new(raw_string).unwrap();
      }
      _ => {}
    }
  }
  CString::new("").unwrap()
}

#[no_mangle]
pub extern "C" fn destroy_twitch_events(twitch_events: *mut TwitchEvents) {
  let twitch: Box<TwitchEvents> = unsafe { transmute(twitch_events) };
}
