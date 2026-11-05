# XR-TAK — Virtual Reality (VR) & Mixed-Reality 3D Tactical Sand Table & CoT HUD

![XR-TAK 3D Holographic Sand Table Demo](docs/xrtak-demo.gif)

**XR-TAK** is a lightweight, high-performance **Virtual Reality (VR)** and **Mixed-Reality (OpenXR / WebXR)** 3D tactical situational awareness system, holographic sand table, and Cursor-on-Target (CoT) HUD for **ATAK-CIV** and desktop command posts.

It ingests standard **Cursor-on-Target (CoT)** XML streams (`UDP / TCP`) and projects friendly, hostile, neutral, and unknown air/ground tracks onto a 3D topographic terrain model with volumetric **EO/IR gimbal sensor FOV frustums**.

---

## MBSE & SOLID Architecture (`docs/architecture/MBSE_MODEL.md`)

| Phase | Subsystem | Stack & SOLID Boundary | Status |
| :--- | :--- | :--- | :--- |
| **Phase 1** | [`crates/xrtak-core`](crates/xrtak-core) & [`crates/xrtak-gateway`](crates/xrtak-gateway) | Rust CoT Engine, `ElevationProvider` (DIP), Point-to-Slew SPI (`b-m-p-s-p-i`), 3D Viewshed (`AGame_Actor_Viewshed`), 3D Spatial Video Wall, Three.js WebXR | **Complete (`:8090` / UDP `:4242`)** |
| **Phase 2** | [`atak-plugin/`](atak-plugin) | Bidirectional ATAK-CIV 5.2 Plugin Bridge (`XrTakMapComponent`, `CotBridgeTransport` DIP, `AtakItemSerializer` SRP) | **Complete (`javac` verified against ATAK 5.2 SDK)** |
| **Phase 3** | [`godot-xr/`](godot-xr) & [`xrtak-core::ffi`](crates/xrtak-core/src/ffi.rs) | Meta Quest 3 OpenXR Passthrough Sand Table + C-ABI FFI (`xrtak_lla_to_enu`, `xrtak_sample_relief_m`) + First-Person Tactical HUD (`xrtak-core::hud`) | **Complete (Godot 4.6 OpenXR)** |
| **Phase 4** | [`xrtak-core::geo`](crates/xrtak-core/src/geo.rs) | Bilinear DEM Engine (`RasterDemProvider`), SRTM `.hgt` 16-bit big-endian ingestion, and live WebXR terrain preset/DEM switching (`/api/terrain`) | **Complete (`REQ-DEM-01..04`)** |
| **Phase 5** | [`xrtak-core::zones`](crates/xrtak-core/src/zones.rs) & [`viewshed`](crates/xrtak-core/src/viewshed.rs) | 3D SAM/Radar Threat Domes (`ThreatDome`), Point-to-Point Link LOS (`compute_link_los`), and 3D Polygonal Geofences (`TacticalZone`) | **Complete (`REQ-TAC-01..03`)** |
| **Phase 6** | [`xrtak-core::aar`](crates/xrtak-core/src/aar.rs) | 4D After-Action Review (`AarTimeline`) ring-buffer, $\mathcal{O}(\log N)$ temporal interpolation, and interactive WebXR 4D timeline scrubber (`/api/aar`) | **Complete (`REQ-AAR-01..03`)** |
| **Phase 7** | [`xrtak-gateway`](crates/xrtak-gateway/src/main.rs) | Live TAK Server TCP (`:8087`) & UDP (`:4242`/`:4243`) Bridge, Auto-Recenter on Live CoT, `SIM+LIVE` toggle, and Quest 3 Controller 3D Laser Pointer (`b-m-p-c`) | **Complete (`REQ-COL-01`)** |
| **Phase 8** | [`xrtak-core::sapient`](crates/xrtak-core/src/sapient.rs) | BSI Flex 335 v2.0 SAPIENT Autonomous AI Threat Fusion (`POST /api/sapient`) and Automatic UAV EO/IR Gimbal SPI Slew (`b-m-p-s-p-i`) | **Complete (`ICD-09`)** |
| **Phase 9** | [`xrtak-core::zones`](crates/xrtak-core/src/zones.rs) | Dynamic ATAK `u-d-f` Polygon Zone Ingestion, Dynamic Hostile Emitter Threat Domes (`a-h-G-E-S`), and Interactive 3D Link LOS Chord UI (`POST /api/los`) | **Complete (`REQ-TAC-01..03`)** |
| **Phase 10** | [`xrtak-core::hud`](crates/xrtak-core/src/hud.rs) | 4D AAR Auto-Playback (`Play / Pause / 1x / 2x / 4x`), Replay Threat Dome Evaluation, and Helmet-Mounted First-Person AR Visor HUD Symbology (`GET /api/hud`) | **Complete (`ICD-10`)** |
| **Phase 11** | [`scripts/run_live_demo.sh`](scripts/run_live_demo.sh) | Turnkey Multi-Domain Live Demo Director exercising ATAK `u-d-f` polygons, SAPIENT AI auto-slew, SAM domes, 3D Link LOS, and Quest 3 laser cues | **Complete** |

See [`docs/architecture/MBSE_MODEL.md`](docs/architecture/MBSE_MODEL.md) for the SysML Block Definition Diagram (BDD) and Interface Control Document (`ICD-01`..`ICD-08`), and [`docs/architecture/NEXT_GEN_MASTER_PLAN.md`](docs/architecture/NEXT_GEN_MASTER_PLAN.md) for the formal EARS requirements, mathematical algorithms, and execution roadmap.

---

## Key Capabilities (Stage 1)

- **Shared Rust CoT Core (`crates/xrtak-core`):**
  - Zero-bloat Cursor-on-Target (`<event>`, `<point>`, `<contact>`, `<track>`, `<sensor>`) XML parser and generator.
  - Real-time **WGS84 (`lat, lon, hae`) $\leftrightarrow$ Local Tangent Plane (`East, Up, North` meters)** coordinate projection.
  - Volumetric 3D gimbal FOV pyramid extraction (`azimuth`, `elevation`, `hfov`, `vfov`, `range`).
- **Real-Time Telemetry & UDP CoT Bridge (`crates/xrtak-gateway`):**
  - Listens for raw CoT XML datagrams on `udp://0.0.0.0:4242`.
  - Streams 20 Hz world snapshots over Server-Sent Events (`/api/stream`) to WebXR and desktop clients.
  - Supports interactive 3D terrain pin-drop (`POST /api/spawn`) and raw CoT XML injection (`POST /api/cot`).
- **3D Holographic Sand Table & Gimbal PiP UI:**
  - Hypsometric 3D terrain relief mesh with contour lines and cardinal compass ring.
  - 3D air/ground tactical markers with altitude drop-stems, breadcrumb trails, and volumetric gimbal FOV cones.
  - Interactive **UAV Chase Cam** and simulated **EO/IR Thermal Gimbal Targeting PiP**.
  - One-click **Meta Quest 3 Mixed Reality (`immersive-ar`)** launcher placing the 1.5m sand table directly onto your physical desk.

---

## Quick Start

### 1. Run the Rust CoT Gateway & WebXR Server

```bash
cargo run -p xrtak-gateway
```

Then open **`http://localhost:8090`** in your browser (or `http://<mac-lan-ip>:8090` inside the Meta Quest 3 browser).

### 2. Run the Turnkey Multi-Domain Live Demo Showcase

```bash
./scripts/run_live_demo.sh
```

This script injects an ATAK `u-d-f` polygonal ROZ (`KILLBOX DELTA`), a **BSI Flex 335 v2.0 SAPIENT** autonomous AI detection (`UAV_ROTARY 97%` with automatic `FALCON-1` EO/IR gimbal SPI lock), a mobile SAM/Radar emitter (`SA-15 GAUNTLET` projecting a 3D Threat Dome), a 3D Point-to-Point Link LOS check, and a Meta Quest 3 spatial laser cue (`b-m-p-c`).

### 3. Inject a Live CoT XML Packet via UDP (`:4242`)

```bash
printf '<event version="2.0" uid="COT-UDP-01" type="a-f-G-U-C" how="h-g-i-g-o"><point lat="39.9218" lon="32.8522" hae="990.0" ce="10" le="10"/><detail><contact callsign="PARALLAX-HQ"/><track course="90.0" speed="0.0"/><remarks>Live UDP CoT XML Packet</remarks></detail></event>' | nc -u -w 1 127.0.0.1 4242
```

### 4. Connect Meta Quest 3 & Live TAK Server (`:8087` TCP / `:4242` UDP)

1. **Connect to a Live TAK Server (TCP `:8087` or UDP `:4242`):**
   - In the right-hand **LIVE TAK SERVER & QUEST 3 BRIDGE** panel, enter your TAK Server TCP address (e.g., `192.168.1.50:8087`) and click **Connect**, or stream raw CoT XML directly to `udp://<mac-lan-ip>:4242`.
   - Click **Sim Units: OFF (LIVE)** to hide built-in rehearsal tracks and view only real TAK Server / ATAK-CIV tracks, or click **Auto-Center CoT** so the 3km x 3km sand table automatically centers on the first incoming live CoT coordinate.
2. **Meta Quest 3 WebXR (`immersive-ar` Color Passthrough):**
   - Option A (USB ADB Reverse — zero Wi-Fi/TLS config): Connect Quest 3 via USB-C, run `adb reverse tcp:8090 tcp:8090`, open `http://localhost:8090` in Meta Quest Browser, and click **QUEST 3: ENTER MIXED REALITY (WebXR)**.
   - Option B (Wi-Fi LAN): Open `http://<mac-lan-ip>:8090` (enable `chrome://flags/#unsafely-treat-insecure-origin-as-secure` for `http://<mac-lan-ip>:8090` in Quest Browser) and press **QUEST 3: ENTER MIXED REALITY (WebXR)**.
   - Pull the Quest 3 trigger (`selectstart`) while pointing at any ridge or valley on the holographic table to broadcast a live `b-m-p-c` 3D Spatial Laser Pointer cue back to ATAK (`udp://127.0.0.1:4243`).

### 4. Run Unit Tests

```bash
cargo test
```
