use std::process;

use godot::classes::ISprite2D;
use godot::classes::Sprite2D;
use godot::classes::{INode, INode2D, Node, Node2D};
use godot::init::EditorRunBehavior;
use godot::prelude::*;

use twitch_eventsub::*;

struct TwitchApi;

#[gdextension]
unsafe impl ExtensionLibrary for TwitchApi {
    fn editor_run_behavior() -> EditorRunBehavior {
        EditorRunBehavior::ToolClassesOnly
    }
}

#[derive(GodotClass)]
#[class(base=Node)]
struct CustomLoop {
    twitch: Option<TwitchEventSubApi>,
    base: Base<Node>,
}

#[godot_api]
impl CustomLoop {
    #[signal]
    fn jumpscare();
}

#[godot_api]
impl INode for CustomLoop {
    fn init(base: Base<Node>) -> Self {
        Self { twitch: None, base }
    }

    fn ready(&mut self) {
        let keys = TwitchKeys::from_secrets_env();
        let redirect_url = "http://localhost:3000";

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
            .add_subscription(SubscriptionPermission::AdBreakBegin)
            .build()
            .unwrap();
        self.twitch = Some(twitch);
    }

    fn process(&mut self, delta: f64) {
        if let Some(ref mut api) = &mut self.twitch {
            for message in api.receive_messages() {
                match message {
                    MessageType::Message(message_data) => {
                        let username = message_data.username;
                        let message = message_data.message;

                        if message == "godot" {
                            // Do stuff
                            self.base_mut().emit_signal("jumpscare".into(), &[]);
                        }
                    }
                    MessageType::CustomRedeem((username, input, reward)) => {}
                    _ => {}
                }
            }
        }
    }
}

#[derive(GodotClass)]
#[class(base=Sprite2D)]
struct Player {
    speed: f64,
    angular_speed: f64,

    base: Base<Sprite2D>,
}

#[godot_api]
impl ISprite2D for Player {
    fn init(base: Base<Sprite2D>) -> Self {
        godot_print!("Hello, World!");

        Self {
            speed: 400.0,
            angular_speed: std::f64::consts::PI,
            base,
        }
    }

    fn physics_process(&mut self, delta: f64) {
        let radians = (self.angular_speed * delta) as f32;
        self.base_mut().rotate(radians);
    }
}

// pub fn add(left: usize, right: usize) -> usize {
//     left + right
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }
