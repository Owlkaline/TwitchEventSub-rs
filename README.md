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
    .set_local_authentication_server("http://localhost:3000")
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
    }
  };

  // users program main loop simulation
  loop {
    // non blocking for loop of messages
    for msg in api.receive_messages() {
      match msg {
        MessageType::Message((username, message)) => {
          println!("{} said: {}", username, message);
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
