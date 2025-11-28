extends Panel


# Called when the node enters the scene tree for the first time.
func _ready() -> void:
	
	#tween_in.tween_property(self, "modulate", Color(1.0, 1.0,1.0, 1.0), 0.5);

	
	var tween = self.create_tween();
	tween.tween_property(self, "modulate", Color(1.0, 1.0, 1.0, 0.0), 0.0);
	tween.set_ease(Tween.EASE_IN);
	tween.set_trans(Tween.TRANS_QUINT)
	#tween.interpolate_value(Color(1.0,1.0,1.0, 0.0), 0.1, 0.1, 5.0);
	tween.tween_property(self, "modulate", Color(1.0, 1.0,1.0, 1.0), 0.6);
	tween.tween_interval(2.0);
	tween.set_ease(Tween.EASE_OUT);
	tween.set_trans(Tween.TRANS_QUINT);
	tween.tween_property(self, "modulate", Color(1.0, 1.0,1.0, 0.0), 3.0);
	tween.tween_callback(self.queue_free);
	pass # Replace with function body.


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta: float) -> void:
	pass
