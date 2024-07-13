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
