extends Node3D

@export var gateway_base_url: String = "http://127.0.0.1:8090"
@export var m_to_table_scale: float = 0.0005

var xr_interface: XRInterface
var http_state_req: HTTPRequest
var http_cmd_req: HTTPRequest
var entity_nodes: Dictionary = {}

var left_ctrl: XRController3D
var right_ctrl: XRController3D
var active_vr_tool: String = "spi" # "spi" | "laser" | "viewshed"
var is_walk_in_scale: bool = false

var left_gripping: bool = false
var right_gripping: bool = false
var grab_offset: Vector3 = Vector3.ZERO
var initial_pinch_dist: float = 0.0
var initial_table_scale: Vector3 = Vector3.ONE

func _ready() -> void:
	xr_interface = XRServer.find_interface("OpenXR")
	if xr_interface and xr_interface.is_initialized():
		get_viewport().use_xr = true
		if xr_interface.get_supported_environment_blend_modes().has(XRInterface.XR_ENV_BLEND_MODE_ALPHA_BLEND):
			xr_interface.environment_blend_mode = XRInterface.XR_ENV_BLEND_MODE_ALPHA_BLEND
			get_viewport().transparent_bg = true

	left_ctrl = get_node_or_null("XROrigin3D/LeftHandController")
	right_ctrl = get_node_or_null("XROrigin3D/RightHandController")
	if right_ctrl:
		right_ctrl.button_pressed.connect(_on_right_button_pressed)
		right_ctrl.button_released.connect(_on_right_button_released)
	if left_ctrl:
		left_ctrl.button_pressed.connect(_on_left_button_pressed)
		left_ctrl.button_released.connect(_on_left_button_released)

	http_state_req = HTTPRequest.new()
	add_child(http_state_req)
	http_state_req.request_completed.connect(_on_state_received)

	http_cmd_req = HTTPRequest.new()
	add_child(http_cmd_req)

func _process(_delta: float) -> void:
	var table_anchor: Node3D = get_node_or_null("SandTableAnchor")
	if table_anchor == null:
		return
	if left_gripping and right_gripping and left_ctrl and right_ctrl:
		var dist: float = left_ctrl.global_position.distance_to(right_ctrl.global_position)
		if initial_pinch_dist > 0.05:
			var factor: float = clampf(dist / initial_pinch_dist, 0.4, 12.0)
			table_anchor.scale = initial_table_scale * factor
	elif right_gripping and right_ctrl:
		table_anchor.global_position = right_ctrl.global_position + grab_offset

func _on_right_button_pressed(btn_name: String) -> void:
	var table_anchor: Node3D = get_node_or_null("SandTableAnchor")
	if btn_name == "grip_click" and table_anchor and right_ctrl:
		right_gripping = true
		grab_offset = table_anchor.global_position - right_ctrl.global_position
		if left_gripping and left_ctrl:
			initial_pinch_dist = left_ctrl.global_position.distance_to(right_ctrl.global_position)
			initial_table_scale = table_anchor.scale
	elif btn_name == "ax_button" and table_anchor:
		is_walk_in_scale = not is_walk_in_scale
		table_anchor.scale = Vector3.ONE * (10.0 if is_walk_in_scale else 1.0)
	elif btn_name == "by_button":
		active_vr_tool = "laser" if active_vr_tool == "spi" else "spi"
	elif btn_name == "trigger_click" and right_ctrl and table_anchor:
		_fire_controller_ray(right_ctrl, table_anchor)

func _on_right_button_released(btn_name: String) -> void:
	if btn_name == "grip_click":
		right_gripping = false

func _on_left_button_pressed(btn_name: String) -> void:
	var table_anchor: Node3D = get_node_or_null("SandTableAnchor")
	if btn_name == "grip_click" and table_anchor and left_ctrl:
		left_gripping = true
		if right_gripping and right_ctrl:
			initial_pinch_dist = left_ctrl.global_position.distance_to(right_ctrl.global_position)
			initial_table_scale = table_anchor.scale

func _on_left_button_released(btn_name: String) -> void:
	if btn_name == "grip_click":
		left_gripping = false

func _fire_controller_ray(ctrl: XRController3D, table_anchor: Node3D) -> void:
	var ray_origin: Vector3 = ctrl.global_position
	var ray_dir: Vector3 = -ctrl.global_transform.basis.z
	var plane := Plane(Vector3.UP, table_anchor.global_position.y)
	var hit = plane.intersects_ray(ray_origin, ray_dir)
	if hit == null:
		return
	var local_pt: Vector3 = table_anchor.to_local(hit)
	var east_m: float = local_pt.x / m_to_table_scale
	var north_m: float = -local_pt.z / m_to_table_scale
	var headers := PackedStringArray(["Content-Type: application/json"])
	if active_vr_tool == "spi":
		var body := JSON.stringify({"sensor_uid": "UAV-FALCON-01", "east_m": east_m, "north_m": north_m})
		http_cmd_req.request(gateway_base_url + "/api/spi", headers, HTTPClient.METHOD_POST, body)
	else:
		var body := JSON.stringify({"callsign": "QUEST3-OPENXR", "east_m": east_m, "north_m": north_m})
		http_cmd_req.request(gateway_base_url + "/api/pointer", headers, HTTPClient.METHOD_POST, body)

func _on_state_received(_result: int, response_code: int, _headers: PackedStringArray, body: PackedByteArray) -> void:
	if response_code != 200:
		return
	var parsed = JSON.parse_string(body.get_string_from_utf8())
	if typeof(parsed) != TYPE_DICTIONARY or not parsed.has("entities"):
		return
	var table_anchor: Node3D = get_node_or_null("SandTableAnchor")
	if table_anchor == null:
		return
	for ent in parsed["entities"]:
		var uid: String = ent["uid"]
		var marker: MeshInstance3D = entity_nodes.get(uid)
		if marker == null:
			marker = MeshInstance3D.new()
			marker.mesh = BoxMesh.new()
			marker.scale = Vector3(0.02, 0.02, 0.02)
			table_anchor.add_child(marker)
			entity_nodes[uid] = marker
		marker.position = Vector3(
			float(ent["east_m"]) * m_to_table_scale,
			float(ent["up_m"]) * m_to_table_scale * 1.65,
			-float(ent["north_m"]) * m_to_table_scale
		)
