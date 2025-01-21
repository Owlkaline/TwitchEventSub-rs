extends Panel


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	pass # Replace with function body.


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	pass

func _on_twitch_event_node_chat_message(message_data: GMessageData) -> void:
	var bottom : bool = $ChatScrollContainer.scroll_vertical == $ChatScrollContainer.get_v_scroll_bar().max_value - $ChatScrollContainer.get_v_scroll_bar().get_rect().size.y
	var label : RichTextLabel = RichTextLabel.new()
	var time = Time.get_time_dict_from_system()
	label.fit_content = true
	label.selection_enabled = true
	label.push_font_size(12)
	label.push_color(Color.WEB_GRAY)
	label.add_text("%02d:%02d " % [time["hour"], time["minute"]])
	label.pop()
	label.push_font_size(14)
#	
	var badges: Array[GBadgeVersion] = %TwitchEventNode.get_badges_urls(message_data.badges);
	for badge in badges:
		label.add_image(%TwitchEventNode.get_static_texture_from_url(badge.images.url_1x));
	label.push_bold()
	
	if (message_data.colour != ""):
		label.push_color(Color(message_data.colour));
	label.add_text(" %s" % message_data.chatter.name);
	label.push_color(Color.WHITE)
	label.push_normal()
	label.add_text(": ")
	for fragment in message_data.message.fragments:
		if fragment.is_any_emote():
			var url = %TwitchEventNode.get_emote_url_1x(fragment);
			label.add_image(%TwitchEventNode.get_generic_emote_texture_from_url(url));
		else:
			label.add_text(fragment.text);

	$ChatScrollContainer/Messages.add_child(label)
	await(get_tree().process_frame)
	if (bottom):
		$ChatScrollContainer.scroll_vertical = $ChatScrollContainer.get_v_scroll_bar().max_value


func _on_twitch_event_node_custom_point_reward_redeem(reward: GCustomRewardRedeem) -> void:
	print("A channel point redeem was just redeem: {}", reward.title);
