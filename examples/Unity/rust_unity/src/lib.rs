extern crate rand;

use rand::Rng;
use std::ffi::CString;
use std::mem::transmute;

use twitch_eventsub::*;

pub struct TwitchEvents {
    api: TwitchEventSubApi,
}

#[no_mangle]
pub extern "C" fn create_twitch_event_sub_builder(redirect_url: CString, subscriptions: Vec<i32>) -> *mut TwitchEventSubApiBuilder {
     let keys = TwitchKeys::from_secrets_env().unwrap();
    let mut twitch = TwitchEventSubApi::builder(keys)
        .set_redirect_url(redirect_url.to_str().unwrap().to_string())
        .generate_new_token_if_insufficent_scope(true)
        .generate_new_token_if_none(true)
        .generate_access_token_on_expire(true)
        .auto_save_load_created_tokens(".user_token.env", ".refresh_token.env");
    for subscription in subscriptions {
        twitch = twitch.add_subscription(SubscriptionPermission::ChatMessage);
    }

    let twitch = twitch.build().unwrap();

    unsafe { transmute(Box::new(twitch)) }
}

#[no_mangle]
pub extern "C" fn get_event(twitch: *mut TwitchEvents) -> CString {
    let twitch = unsafe { &mut *twitch };
    for message in twitch.api.receive_messages() {
        match message {
            MessageType::ChatMessage(message_data) => {
                return CString::new(message_data.message).unwrap();
            },
            MessageType::RawResponse(raw_string) => {
                return CString::new(raw_string).unwrap(); }
            _ => {
            }
        }
    }

    CString::new("").unwrap()
}

#[no_mangle]
pub extern "C" fn destroy_twitch_events(twitch_events: *mut TwitchEvents) {
    let twitch: Box<TwitchEvents> = unsafe { transmute(twitch_events) };
}
