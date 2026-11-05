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
| **Phase 4–7** | [`NEXT_GEN_MASTER_PLAN.md`](docs/architecture/NEXT_GEN_MASTER_PLAN.md) | Bilinear DEM Engine (`RasterDemProvider`), 3D Threat Domes & Link LOS, 4D AAR Time-Machine (`AarTimeline`), Collaborative Laser & SAPIENT AI Fusion | **Spec & EARS Baseline Ready** |

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

### 2. Inject a Live CoT XML Packet via UDP (`:4242`)

```bash
printf '<event version="2.0" uid="COT-UDP-01" type="a-f-G-U-C" how="h-g-i-g-o"><point lat="39.9218" lon="32.8522" hae="990.0" ce="10" le="10"/><detail><contact callsign="PARALLAX-HQ"/><track course="90.0" speed="0.0"/><remarks>Live UDP CoT XML Packet</remarks></detail></event>' | nc -u -w 1 127.0.0.1 4242
```

### 3. Run Unit Tests

```bash
cargo test
```
