# TwitchEventSub-rs
A simple rust library for dealing with that pesky twitch api, specifically event subs.

## Quick Start
### Setting up authentication keys.

Create a .secrets.env or .example.env file in working directory with the following filled out:
```dotenv
// Required
TWITCH_CLIENT_ID = "CLIENT_ID from twitch console app"
TWITCH_CLIENT_SECRET = "CLIENT_SECRET from twitch console app"
TWITCH_BROADCASTER_ID = "Your broadcaster ID as numbers"
TWITCH_BOT_ID = "Your broadcaster ID as numbers"
```

### Example Usage
```Rust
fn main() {
  let mut twitch = TwitchEventSubApi::builder(keys)
    .set_redirect_url(redirect_url)
    .generate_new_token_if_insufficent_scope(true)
    .generate_new_token_if_none(true)
    .generate_access_token_on_expire(true)
    .auto_save_load_created_tokens(".user_token.env", ".refresh_token.env")
    .add_subscription(SubscriptionPermission::ChatMessage)
    .add_subscription(SubscriptionPermission::CustomRedeem)
    .add_subscription(SubscriptionPermission::BanTimeoutUser)
    .add_subscription(SubscriptionPermission::DeleteMessage)
    .add_subscription(SubscriptionPermission::AdBreakBegin);

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
        MessageType::Message(message_data) => {
          let message = message_data.message;
          let username = message_data.username;
          println!("{} said: {}", username, message);
          api.send_chat_message(MessageType::ChannelMessage(format!("Thank you for chatting {}!", username));
        }
        MessageType::CustomRedeem((username, input, reward)) => {
            println!(
              "{} redeemed {} with {} Channel Points: {}",
              username, reward.title, reward.cost, input,
            );
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
cargo build --release
```
## FAQ

* Error redirect url does not match!
```
 Parameter+redirect_uri+does+not+match+registered+URI error
```
If you are recieving this error, you havve most likely forgotten to include the http:// prefix of your app redirect_url, as it has to match EXACTLY with what you have put as the OAuth Redirect URLs in the Twitch Console of your App.

## License

The `TwitchEventSub-rs` crate is licensed under the MIT license, see [`LICENSE`](LICENSE) for more
details.
