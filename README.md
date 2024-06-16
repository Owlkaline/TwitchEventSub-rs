# TwitchEventSub-rs
A simple rust library for dealing with that pesky twitch api, specifically event subs.

## Quick Start
 - Setting up authentication keys.

.secrets.env or .example.env in working directory
```
// Required
TWITCH_OAUTH_TOKEN = ""
TWITCH_CLIENT_ID = "CLIENT_ID from twitch console app"
TWITCH_CLIENT_SECRET = "CLIENT_SECRET from twitch console app"
TWITCH_BROADCASTER_ID = "Your broadcaster ID as numbers"

// Required but are generated once the first time during run time can be reused after first run.
TWITCH_TEMP_APP_TOKEN = "Don't remember what we need this for"
TWITCH_TEMP_SCOPED_TOKEN = "Required as it allows subscriptions based on the scopes of the genericaled oauth key"

// Optional
TWITCH_BOT_ID = "The ID of the account your bot uses"
```

Example Usage
```rust
fn main() {
  let event_sub_api = TwitchEventSubApiBuilder::new(TwitchKeys::from_secrets_env())
    .set_local_authentication_server("localhost:3000")
    .add_subscription(SubscriptionPermission::ChatMessage)
    .add_subscription(SubscriptionPermission::ChannelFollow)
    .add_subscription(SubscriptionPermission::CustomRedeem);

   let mut api = {
     match event_sub_api.build() {
       Ok(api) => api,
       Err(EventSubError::TokenMissingScope) => {
         panic!("Reauthorisation of toke is required for the token to have all the requested subscriptions.");
       }
       Err(EventSubError::NoSubscriptionsRequested) => {
         panic!("No subsciptions passed into builder!");
       }
       Err(EventSubError::NoScopedOuthTokenProvided) => {
         // Provide a Scoped Oauth key or get a new one

         panic!("");
       }
       Err(e) => {
         // some other error
         panic!("{:?}", e);
       }
     }
   };


  // users program main loop simulation
  loop {
    // non blocking for loop of messages
    for msg in api.receive_messages() {
      match msg {
        MessageType::Message((username, message)) => {
          println!("{} said: {}", username, message);
          api.send_chat_message(MessageType::ChannelMessage(format!("Thank you for chatting {}!", username));
        }
        MessageType::Close => println!("Twitch requested socket close."),
        _ => {
          // Events that you don't care about or are not subscribed to, can be ignored.
        }
      }
    }
  }
}
```

To new Scoped Token with the subscriptions you want, you can run the following command
```Rust

if let Ok(new_token) = TwitchEventSubApi::web_browser_authorisation(
  &twitch_keys,
  "localhost:3000",
  &subscriptions,
) {
  // Save new token somewhere safe and Not in a repo
  // Put it in the .secrets.env or directly into Twitchkeys if you are manually creating it.
}
```

## Building

```
cargo run --release
```
## FAQ

* Getting a Parameter+redirect_uri+does+not+match+registered+URI error
  If you are getting this error, you've most likely forgotten http:// part of th oauth_redirect_url, as it has to match EXACTLY with what you have put as the OAuth Redirect URLs in the Twitch Console of your App.

## License

The `TwitchEventSub-rs` crate is licensed under the MIT license, see [`LICENSE`](LICENSE) for more
details.
