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
struct TwitchEvent {
    twitch: Option<TwitchEventSubApi>,
    base: Base<Node>,
}

#[godot_api]
impl TwitchEvent {
    #[signal]
    fn chat_message(username: GString, message: GString);

    #[signal]
    fn point_redeem(
        username: GString,
        input: GString,
        redeem_cost: u32,
        redeem_id: GString,
        redeem_prompt: GString,
        title: GString,
    );
}

#[godot_api]
impl INode for TwitchEvent {
    fn init(base: Base<Node>) -> Self {
        Self { twitch: None, base }
    }

    fn ready(&mut self) {
        let keys = TwitchKeys::from_secrets_env();
        let redirect_url = "http://localhost:3000";

        let twitch = TwitchEventSubApi::builder(keys)
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
                        let message = message_data.message.to_ascii_lowercase();

                        self.base_mut().emit_signal(
                            "chat_message".into(),
                            &[username.to_variant(), message.to_variant()],
                        );
                    }
                    MessageType::CustomRedeem((username, input, reward)) => {
                        self.base_mut().emit_signal(
                            "point_redeem".into(),
                            &[
                                username.to_variant(),
                                input.to_variant(),
                                reward.cost.to_variant(),
                                reward.id.to_variant(),
                                reward.prompt.to_variant(),
                                reward.title.to_variant(),
                            ],
                        );
                    }
                    _ => {}
                }
            }
        }
    }
}
