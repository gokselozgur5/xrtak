# XR-TAK v1.0 — Open-Source Viral Launch Kit (`Show HN` / Reddit / X)

Use these ready-to-post templates to launch **XR-TAK** across developer, GIS, Rust, and Mixed Reality communities.

---

## 1. Hacker News (`Show HN`)

- **Title:**
  `Show HN: XR-TAK – Free, open-source 3D Mixed-Reality geospatial sand table in Rust and WebXR`
- **URL:**
  `https://github.com/gokselozgur5/xrtak`
- **First Comment (Paste immediately after submitting):**
  > Hi HN!
  >
  > Enterprise 3D geospatial and Mixed-Reality situational awareness tables usually cost $25k–$50k+ per seat, require heavy workstation installs, and lock you into proprietary ecosystems.
  >
  > I built **XR-TAK** (`MIT` licensed) as a free, open-source alternative powered by a pure **Rust** backend (`xrtak-core` + `xrtak-gateway`) and a zero-install **Three.js WebXR** frontend (plus a native Godot 4.6 OpenXR C-ABI FFI bridge).
  >
  > **You can try it live in your browser (or inside Meta Quest 3 Mixed Reality Passthrough) with zero install:**
  > https://gokselozgur5.github.io/xrtak/
  >
  > **What it does out of the box:**
  > - **Search Any City / Mountain on Earth:** Queries OpenStreetMap Nominatim + AWS Terrarium 24-bit SRTM DEM elevation tiles + Esri World Imagery satellite tiles + Overpass API to extrude real-world 3D city buildings onto a 3D topographic sand table in seconds.
  > - **Meta Quest 3 Mixed Reality Passthrough (`immersive-ar`):** Places the 3D holographic terrain table directly inside your physical room with 6DoF grab-and-move floating 3D video wall screens (`✋ GRAB & MOVE`) and an upright 3D holographic command dock.
  > - **Rust CoT & Spatial Analysis Core:** Ingests standard Cursor-on-Target (CoT) XML over UDP/TCP (`:4242` / `:8087`), computes real-time 3D radial Viewshed/LOS matrices across the SRTM elevation grid, evaluates 3D geodesic domes/polygonal zones, and records a 4D After-Action Review (`O(log N)` timeline scrubber).
  > - **Persistent Real-World Pinning & Export:** Drop and drag pins directly on the 3D terrain in VR or desktop and export standard GeoJSON or CoT XML bundles.
  >
  > Would love feedback on the Rust DEM/LOS pipeline and WebXR ergonomics!

---

## 2. Reddit Posts

### A. `r/OculusQuest` & `r/virtualreality`
- **Title:**
  `I made a free, open-source Mixed Reality 3D Satellite & City Sand Table for Quest 3 (WebXR Passthrough, no install needed)`
- **Body:**
  > Hey everyone! I wanted a holographic 3D map table in Quest 3 Mixed Reality Passthrough where you can type any city or mountain on Earth, see real satellite terrain + real 3D OpenStreetMap buildings pop up in your living room, and grab/move floating 3D screens anywhere in space.
  >
  > It's 100% free and open-source (MIT):
  > - **GitHub Repo (with Quest 3 Passthrough GIFs):** https://github.com/gokselozgur5/xrtak
  > - **Live Quest 3 Browser Demo (Just click "ENTER AR"):** https://gokselozgur5.github.io/xrtak/
  >
  > Features:
  > - Grab and move any of the floating 3D screens or the 3D button dock anywhere in your room (`✋ GRAB & MOVE`).
  > - Search any city (`Istanbul`, `Manhattan`, `Matterhorn`, etc.) to load real AWS SRTM elevation + satellite imagery + extruded 3D buildings.
  > - 1:1 Walk-In Valley mode, Free 6DoF Spectate flight, and persistent 3D pin dropping.

### B. `r/rust`
- **Title:**
  `XR-TAK: Open-source 3D Mixed-Reality Geospatial Sand Table & CoT Engine in Rust + WebXR (MIT)`
- **Body:**
  > Built a zero-bloat Rust geospatial & Cursor-on-Target (CoT) engine (`xrtak-core` + `xrtak-gateway` with `axum` / `tokio`) paired with a Three.js WebXR Passthrough frontend and Godot 4 C-ABI FFI bindings.
  >
  > - **GitHub:** https://github.com/gokselozgur5/xrtak
  > - **Live WebXR Demo:** https://gokselozgur5.github.io/xrtak/

---

## 3. X / Twitter Launch Thread

**Tweet 1 (Attach `docs/media/quest3_passthrough_overview.gif`):**
> Enterprise 3D Mixed-Reality geospatial tables cost $50,000+ per seat.
>
> So we built a better one in pure Rust + WebXR — and made it 100% FREE & Open-Source (MIT).
>
> Meet **XR-TAK**: Real-world satellite + SRTM elevation + 3D OpenStreetMap buildings floating in Meta Quest 3 Passthrough.
>
> GitHub: https://github.com/gokselozgur5/xrtak 🧵👇

**Tweet 2 (Attach `docs/media/quest3_3d_video_wall.gif`):**
> Grab & place anything in 6DoF space:
> - Floating 3D Video Wall screens (`✋ GRAB & MOVE`) with live camera/stream binding
> - Search any city on Earth -> instant 3D OSM buildings + AWS Terrarium SRTM terrain
> - Real-time 3D Viewshed / Line-of-Sight raycasting in Rust
>
> Try it live in browser / Quest 3 (zero install):
> https://gokselozgur5.github.io/xrtak/
