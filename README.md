# TwitchEventSub-rs

A simple rust library for dealing with that pesky twitch api, specifically event subs.

## Quick Start

### Setting up authentication keys.

Create a .secrets.env or .example.env file in working directory with the following filled out:

```dotenv
TWITCH_CLIENT_ID = "CLIENT_ID from twitch console app"
TWITCH_CLIENT_SECRET = "CLIENT_SECRET from twitch console app"
TWITCH_BROADCASTER_ID = "Your broadcaster ID as numbers"
```

### Example Usage

```rust
use std::time::Duration;
use twitch_eventsub::*;

fn main() {
    let keys = TwitchKeys::from_secrets_env().unwrap();

    let twitch = TwitchEventSubApi::builder(keys)
        // sockets are used to read data from the request so a port
        // must be specified
        .set_redirect_url("https://your_redirect_url:port_is_necessary")
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
            Subscription::AdBreakBegin,
        ]);

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

    // users program main loop simulation
    loop {
        // Set duration to ZERO for non blocking for loop of messages
        // Recommended for most setups
        // If you are not running this inside a game and just byitself
        // Such as a chat bot, setting this to 1 millis seems to be good
        let responses = api.receive_all_messages(Some(Duration::from_millis(1)));
        for response in responses {
            match response {
                ResponseType::Event(event) => {
                    match event {
                        Event::ChatMessage(message_data) => {
                            let message = message_data.message.text;
                            let username = message_data.chatter.name;
                            println!("{} said: {}", username, message);
                            let _ = api
                                .send_chat_message(format!("Thank you for chatting {}!", username))
                                .unwrap();
                        }
                        Event::PointsCustomRewardRedeem(reward) => {
                            println!(
                                "{} redeemed {} with {} Channel Points: {}",
                                reward.user.name,
                                reward.reward.title,
                                reward.reward.cost,
                                reward.user_input,
                            );
                        }
                        _ => {
                            // Events that you don't care about or are not subscribed to, can be ignored.
                        }
                    }
                }
                ResponseType::Close => println!("Twitch requested socket close."),
                _ => {}
            }
        }
    }
}
```

## Godot Example

1. Grab the compiled binaries and .gdextension file from release or compile your own
2. Paste them into the root directory of your project and click reload project
3. Add a single TwitchEvent Object to your scene
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

```doc
cargo build --release
```

## FAQ

- Error redirect url does not match!

```doc
 Parameter+redirect_uri+does+not+match+registered+URI error
```

If you are recieving this error, you havve most likely forgotten to include the http:// prefix of your app redirect_url, as it has to match EXACTLY with what you have put as the OAuth Redirect URLs in the Twitch Console of your App.

## License

The `TwitchEventSub-rs` crate is licensed under the MIT license, see [`LICENSE`](LICENSE) for more
details.
