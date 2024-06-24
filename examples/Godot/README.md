# Running TwitchEventSub-rs with Godot!
Intergrating TwitchEventSub-rs Library with your current godot game or project is very simple and non-intrusive.


## Project Setup
The setup is exactly as they explain in the godot-rust book.\
It can be found here: https://godot-rust.github.io/book/intro/hello-world.html

## Setting up the Godot Side

After setting up the .gdextension file linking your rust cdylibs to your Godot Project.

In godot there will be a new Node type called TwitchEvents, adding this to your project will enable Twitch Event singals.

## GDScript

From any GDScript on a object you can tell it to listen for signals like chat_message and point_redeem.

```GDScript
func _on_twitch_event_chat_message(username, message):
	message = message.to_lower();

	if message.contains("godot"):
          # Do stuff
          print("A message sent contained the word godot!")
    

```
