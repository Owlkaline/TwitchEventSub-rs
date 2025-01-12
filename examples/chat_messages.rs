use std::time::Duration;

use twitcheventsub::*;

fn main() {
  let keys = TwitchKeys::from_secrets_env().unwrap();

  let twitch = TwitchEventSubApi::builder(keys)
    // Make sure this matches exactly what you have set in twitch console
    .set_redirect_url("http://localhost:3000")
    // These generate help automate regeneration of tokens -- recommended
    .generate_new_token_if_insufficent_scope(true)
    .generate_new_token_if_none(true)
    .generate_access_token_on_expire(true)
    // What and where to store user_token and refresh tokens
    .auto_save_load_created_tokens(".user_token.env", ".refresh_token.env")
    // What you are subscribing to, enables and disables features
    .add_subscriptions(vec![Subscription::ChatMessage]);

  // Check for results or just unwrap if you are spicy!
  let mut api = {
    match twitch.build() {
      Ok(api) => api,
      Err(EventSubError::TokenMissingScope) => {
        panic!("Reauthorisation of token is required for the token to have all the requested subscriptions.");
      }
      Err(EventSubError::NoSubscriptionsRequested) => {
        panic!("No subscriptions passed into builder!");
      }
      Err(e) => {
        // some other error
        panic!("{:?}", e);
      }
    }
  };

  let bot_response = "Thank you for chatting!";

  loop {
    // Set duration to ZERO for non blocking for loop of messages
    // Recommended for most setups
    // If you are not running this inside a game and just byitself
    // Such as a chat bot, setting this to 1 millis seems to be good
    let responses = api.receive_all_messages(Some(Duration::from_millis(1)));
    for response in responses {
      // In this example we don't care about Error Response types so match only events
      if let ResponseType::Event(event) = response {
        match event {
          // Match all chat messages coming in
          TwitchEvent::ChatMessage(message_data) => {
            let message = message_data.message.text;
            let username = message_data.chatter.name;

            if message.eq(&bot_response) {
              // This is the message we just sent, so don't reply to it.
              continue;
            }

            println!("{} said: {}", username, message);
            let _ = api.send_chat_message(bot_response).unwrap();
          }
          _ => {
            // Events that you don't care about or are not subscribed to, can be ignored.
          }
        }
      }
    }
  }
}
