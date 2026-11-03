# XR-TAK — Mixed-Reality 3D Tactical Sand Table & CoT HUD (Meta Quest 3)

![XR-TAK 3D Holographic Sand Table Demo](docs/xrtak-demo.gif)

**XR-TAK** is a lightweight, high-performance Mixed-Reality (OpenXR / WebXR) tactical situational awareness system and 3D holographic sand table designed for **Meta Quest 3** and desktop command posts.

It ingests standard **Cursor-on-Target (CoT)** XML streams (`UDP / TCP`) and projects friendly, hostile, neutral, and unknown air/ground tracks onto a 3D topographic terrain model with volumetric **EO/IR gimbal sensor FOV frustums**.

---

## Architecture & Roadmap

| Stage | Stack | Target | Status |
| :--- | :--- | :--- | :--- |
| **Stage 1** | `xrtak-core` (Rust) + `xrtak-gateway` + Three.js WebXR | Desktop Browser & Quest 3 Browser (`immersive-ar` / `immersive-vr`) | **Active (`:8090` / UDP `:4242`)** |
| **Stage 2** | `xrtak-core` (Rust `gdext`) + Godot 4.6 OpenXR | Standalone Quest 3 Native `.apk` (Hand Tracking + Passthrough Table) | Planned next |
| **Stage 3** | Pure Rust (`openxr` + `wgpu` / Vulkan) | Ultra-low-latency First-Person Tactical Passthrough HUD | Planned |

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
