# TwitchEventSub-rs

A simple rust library for dealing with that pesky twitch api, specifically event subs.

## Quick Start

### Get your keys

Go to your [Twitch Console](https://dev.twitch.tv/console)
```
1. Click Register your application, give it an very unique name
2. For now, put http://localhost:3000 exactly the OAuth redirect
3. Save
4. Then get your client ID and Client secret from your registered app.
```

### Setting up authentication keys.

Create a .secrets.env or .example.env file in working directory with the following filled out:

```dotenv
TWITCH_CLIENT_ID = "CLIENT_ID from twitch console app"
TWITCH_CLIENT_SECRET = "CLIENT_SECRET from twitch console app"
TWITCH_BROADCASTER_ID = "Your broadcaster ID as numbers"
```

## Godot Example

1. Grab the compiled binaries and .gdextension file from release or compile your own
2. Paste them into the root directory of your project and click reload project
3. Add a single TwitchEvent Object to your scene
4. Select it, and on the right side of the editor, next to the inspector tab click the node tab
5. In this tab you will see all avaiable twitch event signals
6. Double click on the one you want and add it to a script!

Don't forget to enable/disable the subscriptions you want to use!

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

### Rust Example Usage

You can find the full code here: [chat message example](https://github.com/Owlkaline/TwitchEventSub-rs/blob/main/examples/chat_messages.rs)

Here is a sample match statement of how you recieve events
```rust
// users program main loop simulation
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

## Donations

### Please don't donate unless you truely have the money to do so.
### If you would like to support me to allow me to continue expanding this library and continue streaming you can donate any amount you feel comfortable with via kofi.
### thanks~

[![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/T6T1S1CMK)

## License

The `TwitchEventSub-rs` crate is licensed under the MIT license, see [`LICENSE`](LICENSE) for more
details.
