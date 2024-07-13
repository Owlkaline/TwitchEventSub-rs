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
```

### Example Usage
```Rust
use twitch_eventsub::*;

fn main() {
  let mut twitch = TwitchEventSubApi::builder(keys)
    .set_redirect_url(redirect_url)
    .generate_new_token_if_insufficent_scope(true)
    .generate_new_token_if_none(true)
    .generate_access_token_on_expire(true)
    .auto_save_load_created_tokens(".user_token.env", ".refresh_token.env")
    .add_subscription(Subscription::ChannelFollow)
    .add_subscriptions(vec![
      Subscription::ChannelRaid,
      Subscription::ChannelNewSubscription,
      Subscription::ChannelGiftSubscription,
      Subscription::ChannelResubscription,
      Subscription::ChannelCheer,
      Subscription::ChannelPointsCustomRewardRedeem,
      Subscription::ChannelPointsAutoRewardRedeem,
      Subscription::ChatMessage,
      Subscription::DeleteMessage,
      Subscription::AdBreakBegin
      ]);

   // Check for results or just unwrap if you are spicy!
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
    // Set duration to ZERO for non blocking for loop of messages
    // Recommended for most setups
    // If you are not running this inside a game and just byitself
    // Such as a chat bot, setting this to 1 millis seems to be good
    if let Some(response) in api.receive_single_message(Duration::ZERO) {
      match response {
        ResponseType::Event(event) => {
          Event::ChatMessage(message_data) => {
            let message = message_data.message;
            let username = message_data.username;
            println!("{} said: {}", username, message);
            api.send_chat_message(MessageType::ChannelMessage(format!("Thank you for chatting {}!", username)));
          }
          Event::PointsCustomRewardRedeem(rewaard) => {
            println!(
              "{} redeemed {} with {} Channel Points: {}",
              reward.chatter.name, reward.reward.title, reward.reward.cost, reward.user_input,
            );
          }
        }
        ResponseType::Close => println!("Twitch requested socket close."),
        _ => {
          // Events that you don't care about or are not subscribed to, can be ignored.
        }
      }
    }
  }
}
```
## Godot Example
1. Grab the compiled binaries and .gdextension file from release or compile your own
2. Paste them into the root directory of your project and click reload project
3. Add a single TwitchEvent Object  to your scene
4. Select it, and on the right side of the editor, next to the inspector tab click the node tab
5. In this tab you will see all avaiable twitch event signals
6. Double click on the one you want and add it to a script!

The result should look similar to the following:
```GDScript
extends Sprite2D

# Called when the node enters the scene tree for the first time.
func _ready():
	pass # Replace with function body.

# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	pass

func _on_twitch_event_chat_message(message_data: GMessageData):
	print("user: {} Message: {}", message_data.chatter.name, message_data.message);
	# Do stuff when message comes in or if message contains specific text

func _on_twitch_event_custom_point_reward_redeem(reward: GReward):
	print("A channel point redeem was just redeem: {}", reward.title);
	# Do stuff when a reward is redeemed!
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
