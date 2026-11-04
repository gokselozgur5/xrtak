extends Node3D

@export var gateway_state_url: String = "http://127.0.0.1:8090/api/state"
@export var m_to_table_scale: float = 0.0005

var xr_interface: XRInterface
var http_req: HTTPRequest
var entity_nodes: Dictionary = {}

func _ready() -> void:
	xr_interface = XRServer.find_interface("OpenXR")
	if xr_interface and xr_interface.is_initialized():
		get_viewport().use_xr = true
		if xr_interface.get_supported_environment_blend_modes().has(XRInterface.XR_ENV_BLEND_MODE_ALPHA_BLEND):
			xr_interface.environment_blend_mode = XRInterface.XR_ENV_BLEND_MODE_ALPHA_BLEND
			get_viewport().transparent_bg = true

	http_req = HTTPRequest.new()
	add_child(http_req)
	http_req.request_completed.connect(_on_state_received)

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
