#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${XRTAK_URL:-http://127.0.0.1:8090}"
if ! curl -fsS "${BASE_URL}/api/state" >/dev/null 2>&1; then
  if curl -fsS "http://127.0.0.1:8091/api/state" >/dev/null 2>&1; then
    BASE_URL="http://127.0.0.1:8091"
  else
    echo "[ERROR] xrtak-gateway is not reachable on :8090 or :8091. Start it with: cargo run -p xrtak-gateway" >&2
    exit 1
  fi
fi

echo "=================================================================="
echo "  XR-TAK Live Multi-Domain Tactical Demo Director"
echo "  Gateway Endpoint : ${BASE_URL}"
echo "=================================================================="

echo "[1/5] Injecting ATAK u-d-f Polygonal Restricted Operating Zone (KILLBOX DELTA ROZ)..."
POLY_COT='<event version="2.0" uid="ATAK-POLY-DELTA" type="u-d-f" how="h-e"><point lat="39.9210" lon="32.8510" hae="850.0" ce="10" le="10"/><detail><contact callsign="KILLBOX DELTA [ATAK ROZ]"/><link point="39.9195,32.8495,850.0"/><link point="39.9195,32.8535,850.0"/><link point="39.9225,32.8535,850.0"/><link point="39.9225,32.8495,850.0"/><remarks>Live ATAK Freeform Polygon Zone</remarks></detail></event>'
curl -fsS -X POST "${BASE_URL}/api/cot" -d "${POLY_COT}" | head -c 160
echo ""

echo "[2/5] Injecting BSI Flex 335 v2.0 SAPIENT AI Detection (UAV_ROTARY 97% -> Auto-Slew FALCON-1)..."
curl -fsS -X POST "${BASE_URL}/api/sapient" \
  -H "Content-Type: application/json" \
  -d '{"node_id":"ASM-EDGE-01","object_id":"ROTARY-09","classification":"UAV_ROTARY","confidence":0.97,"east_m":-240.0,"north_m":210.0,"alt_offset_m":145.0,"speed_mps":17.5,"course_deg":135.0}' | head -c 160
echo ""

echo "[3/5] Injecting Hostile Mobile SAM/Radar Emitter over UDP :4242 (Dynamic 3D Threat Dome)..."
SAM_COT='<event version="2.0" uid="HST-SAM-SA15" type="a-h-G-E-S" how="m-g"><point lat="39.9175" lon="32.8470" hae="920.0" ce="5" le="5"/><detail><contact callsign="SA-15 GAUNTLET"/><track course="45.0" speed="0.0"/><remarks>Mobile SHORAD Radar Emitter</remarks></detail></event>'
printf '%s' "${SAM_COT}" | nc -u -w 1 127.0.0.1 4242 || curl -fsS -X POST "${BASE_URL}/api/cot" -d "${SAM_COT}" >/dev/null
echo "  -> Dispatched SA-15 GAUNTLET (uid=HST-SAM-SA15)"

echo "[4/5] Computing 3D Point-to-Point Link LOS (OP-BRAVO -> CONTACT-RED-1) & Viewshed..."
curl -fsS -X POST "${BASE_URL}/api/los" \
  -H "Content-Type: application/json" \
  -d '{"from_uid":"GND-OBSERVE-BRAVO","to_uid":"HST-ARMOR-01"}'
echo ""

echo "[5/5] Emitting Meta Quest 3 3D Spatial Laser Pointer (b-m-p-c) & Querying 4D AAR + Visor HUD..."
curl -fsS -X POST "${BASE_URL}/api/pointer" \
  -H "Content-Type: application/json" \
  -d '{"callsign":"QUEST3-LEAD","east_m":125.0,"north_m":-95.0}' | head -c 160
echo ""
curl -fsS "${BASE_URL}/api/hud?observer=GND-SPEAR-ALPHA" | head -c 180
echo ""

echo "=================================================================="
echo "  Live Demo Sequence Complete — Open ${BASE_URL} in Browser/Quest 3"
echo "=================================================================="
