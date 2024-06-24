extends Sprite2D


# Called when the node enters the scene tree for the first time.
func _ready():
	pass # Replace with function body.


# Called every frame. 'delta' is the elapsed time since the previous frame.
func _process(delta):
	pass


func _on_custom_loop_jumpscare():
	#visible = !visible;
	var shader = (material as ShaderMaterial);
	var opacity = shader.get_shader_parameter("opacity");
	shader.set_shader_parameter("opacity", 1.0 - opacity);
