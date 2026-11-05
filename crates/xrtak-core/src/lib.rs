pub mod aar;
pub mod domain;
pub mod ffi;
pub mod geo;
pub mod hud;
pub mod viewshed;
pub mod zones;

pub use aar::{AarFrame, AarSummary, AarTimeline};
pub use domain::{
    Affiliation, Dimension, LinkLosResult, SensorCone, TacticalZone, ThreatDome, ViewshedResult,
    ViewshedSample,
};
pub use geo::{ElevationProvider, ProceduralReliefModel, RasterDemProvider};
pub use hud::{compute_hud_cue, HudCue};
pub use viewshed::{compute_link_los, compute_viewshed_with_provider};
pub use zones::{is_engaged_by_threat_dome, is_inside_tactical_zone, point_in_polygon_en};

use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TacticalEntity {
    pub uid: String,
    pub callsign: String,
    pub cot_type: String,
    pub affiliation: Affiliation,
    pub dimension: Dimension,
    pub lat: f64,
    pub lon: f64,
    pub hae_m: f64,
    pub course_deg: f64,
    pub speed_mps: f64,
    pub remarks: String,
    pub sensor: Option<SensorCone>,
    /// Optional TAK `<__video url="..."/>` stream endpoint (`VH_COTDetail_Video`)
    pub video_url: Option<String>,
    /// Parent entity UID when this entity is a linked SPI (`b-m-p-s-p-i` with `<link relation="p-p"/>`)
    pub parent_uid: Option<String>,
    /// Local ENU coordinates relative to sand table center (meters)
    pub east_m: f64,
    pub up_m: f64,
    pub north_m: f64,
    pub updated_unix_ms: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GeoOrigin {
    pub lat: f64,
    pub lon: f64,
    pub alt_m: f64,
    /// Radius of the tactical area in meters (e.g., 1500m = 3km x 3km box)
    pub extent_m: f64,
}

impl Default for GeoOrigin {
    fn default() -> Self {
        Self {
            lat: 39.9200,
            lon: 32.8500,
            alt_m: 850.0,
            extent_m: 1500.0,
        }
    }
}

impl GeoOrigin {
    /// Convert WGS84 (lat, lon, hae) into local tangent plane (East, Up, North) in meters.
    pub fn lla_to_enu(&self, lat: f64, lon: f64, hae_m: f64) -> (f64, f64, f64) {
        let lat_rad = self.lat.to_radians();
        let meters_per_deg_lat = 111_132.92 - 559.82 * (2.0 * lat_rad).cos();
        let meters_per_deg_lon = 111_412.84 * lat_rad.cos();

        let north_m = (lat - self.lat) * meters_per_deg_lat;
        let east_m = (lon - self.lon) * meters_per_deg_lon;
        let up_m = hae_m - self.alt_m;
        (east_m, up_m, north_m)
    }

    /// Convert local (East, Up, North) meters back to WGS84 (lat, lon, hae).
    pub fn enu_to_lla(&self, east_m: f64, up_m: f64, north_m: f64) -> (f64, f64, f64) {
        let lat_rad = self.lat.to_radians();
        let meters_per_deg_lat = 111_132.92 - 559.82 * (2.0 * lat_rad).cos();
        let meters_per_deg_lon = 111_412.84 * lat_rad.cos();

        let lat = self.lat + north_m / meters_per_deg_lat;
        let lon = self.lon + east_m / meters_per_deg_lon;
        let hae_m = self.alt_m + up_m;
        (lat, lon, hae_m)
    }
}

/// Deterministic procedural topographic height function (shared logic with frontend).
pub fn sample_terrain_relief_m(east_m: f64, north_m: f64) -> f64 {
    let nx = east_m * 0.0018;
    let nz = north_m * 0.0018;

    let ridge1 = ((nx * 1.3 + nz * 0.8).sin() * (nx * 0.7 - nz * 1.1).cos()) * 115.0;
    let ridge2 = ((nx * 2.7 - nz * 2.1).sin() * 0.5 + (nz * 3.1).cos() * 0.5) * 42.0;
    let detail = (nx * 5.5 + nz * 4.8).sin() * 12.0;
    let valley_dist = ((nx - nz * 0.6).abs()).min(1.5);
    let valley_factor = 0.35 + 0.65 * (valley_dist / 1.5);

    (ridge1 + ridge2 + detail + 95.0) * valley_factor
}

/// Compute 3D Line-of-Sight (Viewshed) point cloud matching VR-TAK `AGame_Actor_Viewshed`.
pub fn compute_viewshed(
    observer_east_m: f64,
    observer_north_m: f64,
    observer_height_offset_m: f64,
    radius_m: f64,
    num_azimuth_beams: usize,
    points_per_beam: usize,
) -> ViewshedResult {
    compute_viewshed_with_provider(
        &ProceduralReliefModel,
        observer_east_m,
        observer_north_m,
        observer_height_offset_m,
        radius_m,
        num_azimuth_beams,
        points_per_beam,
    )
}

pub fn classify_cot_type(cot_type: &str) -> (Affiliation, Dimension) {
    if cot_type.starts_with("b-m-p-s-p-i") {
        return (Affiliation::Neutral, Dimension::Ground);
    }
    let parts: Vec<&str> = cot_type.split('-').collect();
    let affiliation = match parts.get(1).copied() {
        Some("f") | Some("a") => Affiliation::Friendly,
        Some("h") | Some("j") | Some("k") => Affiliation::Hostile,
        Some("n") => Affiliation::Neutral,
        _ => Affiliation::Unknown,
    };
    let dimension = match parts.get(2).copied() {
        Some("A") => Dimension::Air,
        Some("G") => Dimension::Ground,
        Some("S") => Dimension::Surface,
        _ => Dimension::Unknown,
    };
    (affiliation, dimension)
}

/// Build a VR-TAK / ATAK compliant `b-m-p-s-p-i` (Sensor Point of Interest) CoT XML packet.
/// Matches `AGame_Point_SPI::UpdateCOT()` from `TAK-Product-Center/VR`.
pub fn build_spi_cot_xml(
    parent_uid: &str,
    parent_callsign: &str,
    parent_type: &str,
    lat: f64,
    lon: f64,
    hae_m: f64,
) -> String {
    let spi_uid = format!("{parent_uid}.SPI1");
    let spi_callsign = format!("{parent_callsign}.SPI1");
    format!(
        r#"<event version="2.0" uid="{spi_uid}" type="b-m-p-s-p-i" how="h-e"><point lat="{lat:.6}" lon="{lon:.6}" hae="{hae_m:.1}" ce="5.0" le="5.0"/><detail><contact callsign="{spi_callsign}"/><link uid="{parent_uid}" relation="p-p" type="{parent_type}"/><remarks>SPI Gimbal Lock Target</remarks></detail></event>"#
    )
}

/// Build a collaborative 3D Laser Pointer / Spot Map cue (`b-m-p-c`) CoT XML packet (`REQ-COL-01`).
pub fn build_laser_pointer_cot_xml(
    operator_callsign: &str,
    lat: f64,
    lon: f64,
    hae_m: f64,
) -> String {
    let clean = operator_callsign.replace(' ', "-");
    let uid = format!("XR-LASER-{clean}");
    let cs = format!("LASER-{clean}");
    format!(
        r#"<event version="2.0" uid="{uid}" type="b-m-p-c" how="h-e"><point lat="{lat:.6}" lon="{lon:.6}" hae="{hae_m:.1}" ce="3.0" le="3.0"/><detail><contact callsign="{cs}"/><remarks>Quest 3 XR Spatial Laser Pointer</remarks></detail></event>"#
    )
}

/// Parse a standard Cursor-on-Target (CoT) XML string into a `TacticalEntity`.
pub fn parse_cot_xml(xml: &str, origin: &GeoOrigin, now_ms: u64) -> Result<TacticalEntity, String> {
    parse_cot_xml_with_provider(xml, origin, &ProceduralReliefModel, now_ms)
}

/// Parse a standard Cursor-on-Target (CoT) XML string using any `ElevationProvider` (DIP).
pub fn parse_cot_xml_with_provider(
    xml: &str,
    origin: &GeoOrigin,
    terrain: &dyn ElevationProvider,
    now_ms: u64,
) -> Result<TacticalEntity, String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut uid = String::new();
    let mut cot_type = String::from("a-u-G");
    let mut lat: Option<f64> = None;
    let mut lon: Option<f64> = None;
    let mut hae_m: f64 = origin.alt_m;
    let mut callsign = String::new();
    let mut course_deg = 0.0;
    let mut speed_mps = 0.0;
    let mut remarks = String::new();
    let mut sensor: Option<SensorCone> = None;
    let mut video_url: Option<String> = None;
    let mut parent_uid: Option<String> = None;
    let mut in_remarks = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let tag_name = e.name();
                match tag_name.as_ref() {
                    b"event" => {
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"uid" => uid = String::from_utf8_lossy(&attr.value).into_owned(),
                                b"type" => {
                                    cot_type = String::from_utf8_lossy(&attr.value).into_owned()
                                }
                                _ => {}
                            }
                        }
                    }
                    b"point" => {
                        for attr in e.attributes().flatten() {
                            let val_str = String::from_utf8_lossy(&attr.value);
                            match attr.key.as_ref() {
                                b"lat" => lat = val_str.parse::<f64>().ok(),
                                b"lon" => lon = val_str.parse::<f64>().ok(),
                                b"hae" => {
                                    if let Ok(v) = val_str.parse::<f64>() {
                                        if v < 900_000.0 {
                                            hae_m = v;
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    b"contact" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"callsign" {
                                callsign = String::from_utf8_lossy(&attr.value).into_owned();
                            }
                        }
                    }
                    b"link" => {
                        let mut link_uid = String::new();
                        let mut relation = String::new();
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"uid" => {
                                    link_uid = String::from_utf8_lossy(&attr.value).into_owned()
                                }
                                b"relation" => {
                                    relation = String::from_utf8_lossy(&attr.value).into_owned()
                                }
                                _ => {}
                            }
                        }
                        if relation == "p-p" && !link_uid.is_empty() && link_uid != uid {
                            parent_uid = Some(link_uid);
                        }
                    }
                    b"track" => {
                        for attr in e.attributes().flatten() {
                            let val_str = String::from_utf8_lossy(&attr.value);
                            match attr.key.as_ref() {
                                b"course" => course_deg = val_str.parse::<f64>().unwrap_or(0.0),
                                b"speed" => speed_mps = val_str.parse::<f64>().unwrap_or(0.0),
                                _ => {}
                            }
                        }
                    }
                    b"sensor" => {
                        let mut az = 0.0;
                        let mut el = -35.0;
                        let mut hfov = 36.0;
                        let mut vfov = 22.0;
                        let mut range = 650.0;
                        for attr in e.attributes().flatten() {
                            let val_str = String::from_utf8_lossy(&attr.value);
                            match attr.key.as_ref() {
                                b"azimuth" => az = val_str.parse::<f64>().unwrap_or(0.0),
                                b"elevation" => el = val_str.parse::<f64>().unwrap_or(-35.0),
                                b"fov" | b"hfov" => hfov = val_str.parse::<f64>().unwrap_or(36.0),
                                b"vfov" => vfov = val_str.parse::<f64>().unwrap_or(22.0),
                                b"range" => range = val_str.parse::<f64>().unwrap_or(650.0),
                                _ => {}
                            }
                        }
                        sensor = Some(SensorCone {
                            azimuth_deg: az,
                            elevation_deg: el,
                            hfov_deg: hfov,
                            vfov_deg: vfov,
                            range_m: range,
                        });
                    }
                    b"__video" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"url" {
                                video_url =
                                    Some(String::from_utf8_lossy(&attr.value).into_owned());
                            }
                        }
                    }
                    b"remarks" => {
                        in_remarks = true;
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(e)) => {
                if in_remarks {
                    remarks = e.unescape().unwrap_or_default().into_owned();
                }
            }
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"remarks" {
                    in_remarks = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(err) => return Err(format!("XML parse error: {err}")),
            _ => {}
        }
    }

    if uid.is_empty() {
        return Err("Missing CoT event uid".to_string());
    }
    let lat = lat.ok_or_else(|| "Missing point lat".to_string())?;
    let lon = lon.ok_or_else(|| "Missing point lon".to_string())?;

    if callsign.is_empty() {
        callsign = uid.clone();
    }

    let (affiliation, dimension) = classify_cot_type(&cot_type);
    let (east_m, mut up_m, north_m) = origin.lla_to_enu(lat, lon, hae_m);

    if (hae_m - 0.0).abs() < 1.0 || (hae_m - origin.alt_m).abs() < 1.0 {
        let ground_relief = terrain.sample_relief_m(east_m, north_m);
        up_m = match dimension {
            Dimension::Air => ground_relief + 260.0,
            _ => ground_relief,
        };
    }

    let (_, _, resolved_hae) = origin.enu_to_lla(east_m, up_m, north_m);

    Ok(TacticalEntity {
        uid,
        callsign,
        cot_type,
        affiliation,
        dimension,
        lat,
        lon,
        hae_m: resolved_hae,
        course_deg,
        speed_mps,
        remarks,
        sensor,
        video_url,
        parent_uid,
        east_m,
        up_m,
        north_m,
        updated_unix_ms: now_ms,
    })
}

/// State manager holding simulated tactical units, SPI gimbal locks, and live external CoT tracks.
pub struct TacticalWorld {
    pub origin: GeoOrigin,
    pub terrain_name: String,
    pub terrain: Box<dyn ElevationProvider>,
    pub sim_enabled: bool,
    pub auto_recenter_on_next_live_cot: bool,
    pub entities: HashMap<String, TacticalEntity>,
    pub threat_domes: Vec<ThreatDome>,
    pub tactical_zones: Vec<TacticalZone>,
    pub aar: AarTimeline,
    /// Map from UAV UID (e.g. `UAV-FALCON-01`) to locked SPI ENU target `(east_m, up_m, north_m)`
    pub spi_locks: HashMap<String, (f64, f64, f64)>,
    pub sim_time_sec: f64,
}

impl TacticalWorld {
    pub fn new(origin: GeoOrigin) -> Self {
        let tactical_zones = vec![
            TacticalZone {
                uid: "ZONE-ROZ-KILLBOX-1".to_string(),
                name: "KILLBOX RED-ALPHA [ROZ]".to_string(),
                affiliation: Affiliation::Hostile,
                vertices_en: vec![(-380.0, -280.0), (60.0, -280.0), (90.0, 40.0), (-340.0, 60.0)],
                floor_m: 0.0,
                ceiling_m: 380.0,
                inside_uids: Vec::new(),
            },
            TacticalZone {
                uid: "ZONE-FOB-SAFE".to_string(),
                name: "OP-BRAVO OVERWATCH SECTOR".to_string(),
                affiliation: Affiliation::Friendly,
                vertices_en: vec![(280.0, -480.0), (640.0, -480.0), (640.0, -140.0), (280.0, -140.0)],
                floor_m: 0.0,
                ceiling_m: 340.0,
                inside_uids: Vec::new(),
            },
        ];
        Self {
            origin,
            terrain_name: "valley".to_string(),
            terrain: Box::new(ProceduralReliefModel),
            sim_enabled: true,
            auto_recenter_on_next_live_cot: false,
            entities: HashMap::new(),
            threat_domes: Vec::new(),
            tactical_zones,
            aar: AarTimeline::default(),
            spi_locks: HashMap::new(),
            sim_time_sec: 0.0,
        }
    }

    /// Re-center the sand table's WGS84 `GeoOrigin` and re-project all existing tracks.
    pub fn recenter_origin(&mut self, new_origin: GeoOrigin) {
        self.origin = new_origin;
        for ent in self.entities.values_mut() {
            let (e, u, n) = self.origin.lla_to_enu(ent.lat, ent.lon, ent.hae_m);
            ent.east_m = e;
            ent.up_m = u.max(self.terrain.sample_relief_m(e, n));
            ent.north_m = n;
        }
    }

    /// Enable or disable the built-in synthetic demo units (`LIVE TAK ONLY` vs `SIM + LIVE`).
    pub fn set_sim_enabled(&mut self, enabled: bool) {
        self.sim_enabled = enabled;
        if !enabled {
            for sim_uid in [
                "UAV-FALCON-01",
                "UAV-FALCON-01.SPI1",
                "UAV-VALKYRIE-02",
                "GND-SPEAR-ALPHA",
                "GND-OBSERVE-BRAVO",
                "HST-ARMOR-01",
                "HST-EMITTER-02",
                "UNK-BOGEY-07",
            ] {
                self.entities.remove(sim_uid);
            }
            self.refresh_threat_domes_and_zones();
        }
    }

    /// Sample active terrain elevation at `(east_m, north_m)` via `ElevationProvider` (DIP).
    pub fn sample_relief_m(&self, east_m: f64, north_m: f64) -> f64 {
        self.terrain.sample_relief_m(east_m, north_m)
    }

    /// Switch the active terrain provider and re-clamp ground entities to the new relief surface (`REQ-DEM-04`).
    pub fn set_terrain_provider(&mut self, name: impl Into<String>, provider: Box<dyn ElevationProvider>) {
        self.terrain_name = name.into();
        self.terrain = provider;
        for ent in self.entities.values_mut() {
            let gnd = self.terrain.sample_relief_m(ent.east_m, ent.north_m);
            if ent.dimension != Dimension::Air {
                ent.up_m = gnd;
                let (_, _, hae) = self.origin.enu_to_lla(ent.east_m, ent.up_m, ent.north_m);
                ent.hae_m = hae;
            }
        }
    }

    pub fn ingest_cot_xml(&mut self, xml: &str, now_ms: u64) -> Result<TacticalEntity, String> {
        let mut entity = parse_cot_xml_with_provider(xml, &self.origin, self.terrain.as_ref(), now_ms)?;
        if self.auto_recenter_on_next_live_cot && !entity.uid.starts_with("XR-") {
            self.auto_recenter_on_next_live_cot = false;
            self.recenter_origin(GeoOrigin {
                lat: entity.lat,
                lon: entity.lon,
                alt_m: (entity.hae_m - 120.0).max(0.0),
                extent_m: self.origin.extent_m,
            });
            entity = parse_cot_xml_with_provider(xml, &self.origin, self.terrain.as_ref(), now_ms)?;
        }
        // If this incoming CoT is an SPI (`b-m-p-s-p-i`) with a parent UAV link, lock that UAV's gimbal!
        if entity.cot_type.starts_with("b-m-p-s-p-i") {
            if let Some(ref parent) = entity.parent_uid {
                self.spi_locks.insert(
                    parent.clone(),
                    (entity.east_m, entity.up_m, entity.north_m),
                );
            }
        }
        self.entities.insert(entity.uid.clone(), entity.clone());
        Ok(entity)
    }

    /// Lock a UAV's gimbal sensor onto a 3D terrain coordinate and emit a `b-m-p-s-p-i` SPI CoT entity.
    pub fn slew_sensor_to_spi(
        &mut self,
        sensor_uid: &str,
        east_m: f64,
        north_m: f64,
        now_ms: u64,
    ) -> Result<TacticalEntity, String> {
        let ground_up_m = self.terrain.sample_relief_m(east_m, north_m);
        let (lat, lon, hae_m) = self.origin.enu_to_lla(east_m, ground_up_m, north_m);

        let (parent_callsign, parent_type) = self
            .entities
            .get(sensor_uid)
            .map(|e| (e.callsign.clone(), e.cot_type.clone()))
            .unwrap_or_else(|| ("FALCON-1".to_string(), "a-f-A-M-F-Q".to_string()));

        let xml = build_spi_cot_xml(
            sensor_uid,
            &parent_callsign,
            &parent_type,
            lat,
            lon,
            hae_m,
        );
        self.ingest_cot_xml(&xml, now_ms)
    }

    /// Clear SPI gimbal lock for a UAV and remove its `.SPI1` marker.
    pub fn clear_spi_lock(&mut self, sensor_uid: &str) {
        self.spi_locks.remove(sensor_uid);
        let spi_uid = format!("{sensor_uid}.SPI1");
        self.entities.remove(&spi_uid);
    }

    /// Advance built-in tactical rehearsal scenario by `dt_sec`.
    pub fn step_simulation(&mut self, dt_sec: f64, now_ms: u64) {
        self.sim_time_sec += dt_sec;
        if !self.sim_enabled {
            self.refresh_threat_domes_and_zones();
            self.aar.maybe_record(self.sim_time_sec, now_ms, &self.entities);
            return;
        }
        let t = self.sim_time_sec;

        // 1. Friendly ISR UAV #1 ("FALCON-1") orbiting North-East sector with sweeping or SPI-locked gimbal FOV
        {
            let orbit_r = 520.0;
            let omega = 0.14;
            let angle = t * omega;
            let east_m = 220.0 + orbit_r * angle.cos();
            let north_m = 180.0 + orbit_r * angle.sin();
            let ground_h = self.terrain.sample_relief_m(east_m, north_m);
            let up_m = ground_h + 290.0;
            let course_deg = (90.0 - angle.to_degrees() - 90.0).rem_euclid(360.0);

            // Check if operator locked FALCON-1's gimbal onto an SPI point (`b-m-p-s-p-i`);
            // otherwise track the hostile armor in the valley.
            let (target_e, target_up, target_n, lock_label) =
                if let Some(&(se, sup, sn)) = self.spi_locks.get("UAV-FALCON-01") {
                    (
                        se,
                        sup,
                        sn,
                        format!("SPI LOCK [E:{:.0}m N:{:.0}m]", se, sn),
                    )
                } else {
                    let te = -180.0 + (t * 0.08).sin() * 260.0;
                    let tn = -120.0 + (t * 0.08).cos() * 190.0;
                    let tup = self.terrain.sample_relief_m(te, tn);
                    (te, tup, tn, "Auto-Track: Valley Sector".to_string())
                };

            let de = target_e - east_m;
            let dn = target_n - north_m;
            let dup = target_up - up_m;
            let horiz_dist = (de * de + dn * dn).sqrt().max(1.0);
            let slant_range = (horiz_dist * horiz_dist + dup * dup).sqrt().clamp(150.0, 1400.0);
            let gimbal_az = de.atan2(dn).to_degrees().rem_euclid(360.0);
            let gimbal_el = dup.atan2(horiz_dist).to_degrees();

            self.upsert_sim_entity(
                "UAV-FALCON-01",
                "FALCON-1 [EO/IR]",
                "a-f-A-M-F-Q",
                east_m,
                up_m,
                north_m,
                course_deg,
                28.5,
                &format!("Dual EO/IR Gimbal Payload | {lock_label}"),
                Some(SensorCone {
                    azimuth_deg: gimbal_az,
                    elevation_deg: gimbal_el,
                    hfov_deg: 32.0,
                    vfov_deg: 20.0,
                    range_m: slant_range,
                }),
                Some("rtsp://10.8.0.11:8554/falcon1-eoir".to_string()),
                None,
                now_ms,
            );
        }

        // 2. Friendly Relay / Recon UAV #2 ("VALKYRIE-2") figure-8 patrol over Western ridge
        {
            let omega = 0.11;
            let a = t * omega;
            let east_m = -480.0 + 380.0 * a.sin();
            let north_m = 90.0 + 320.0 * (2.0 * a).sin() * 0.5;
            let ground_h = self.terrain.sample_relief_m(east_m, north_m);
            let up_m = ground_h + 240.0;
            let course_deg = (a.cos().atan2((2.0 * a).cos())).to_degrees().rem_euclid(360.0);
            let gimbal_az = (course_deg + (t * 0.6).sin() * 35.0).rem_euclid(360.0);

            self.upsert_sim_entity(
                "UAV-VALKYRIE-02",
                "VALKYRIE-2 [SAR/LINK]",
                "a-f-A-M-F-Q",
                east_m,
                up_m,
                north_m,
                course_deg,
                24.0,
                "Tactical Mesh Relay & Wide-Area Overwatch",
                Some(SensorCone {
                    azimuth_deg: gimbal_az,
                    elevation_deg: -42.0,
                    hfov_deg: 44.0,
                    vfov_deg: 26.0,
                    range_m: 560.0,
                }),
                Some("rtsp://10.8.0.12:8554/valkyrie2-sar".to_string()),
                None,
                now_ms,
            );
        }

        // 3. Friendly Ground Team Alpha ("SPEAR-ALPHA")
        {
            let east_m = -120.0 + (t * 0.04).cos() * 140.0;
            let north_m = -420.0 + (t * 0.04).sin() * 90.0;
            let up_m = self.terrain.sample_relief_m(east_m, north_m);
            self.upsert_sim_entity(
                "GND-SPEAR-ALPHA",
                "SPEAR-ALPHA",
                "a-f-G-U-C-I",
                east_m,
                up_m,
                north_m,
                42.0,
                2.4,
                "Recon Squad | Visual on ridge",
                None,
                None,
                None,
                now_ms,
            );
        }

        // 4. Friendly Ground Team Bravo ("OBSERVE-BRAVO") on high peak
        {
            let east_m = 460.0;
            let north_m = -310.0;
            let up_m = self.terrain.sample_relief_m(east_m, north_m);
            self.upsert_sim_entity(
                "GND-OBSERVE-BRAVO",
                "OP-BRAVO [TOC]",
                "a-f-G-U-C",
                east_m,
                up_m,
                north_m,
                315.0,
                0.0,
                "Forward Observation Post & TAK Gateway",
                None,
                Some("rtsp://10.8.0.14:8554/op-bravo-ptz".to_string()),
                None,
                now_ms,
            );
        }

        // 5. Hostile Mobile Track #1 ("RED-ARMOR-01") moving through valley
        {
            let east_m = -180.0 + (t * 0.08).sin() * 260.0;
            let north_m = -120.0 + (t * 0.08).cos() * 190.0;
            let up_m = self.terrain.sample_relief_m(east_m, north_m);
            self.upsert_sim_entity(
                "HST-ARMOR-01",
                "CONTACT-RED-1",
                "a-h-G-E-V-A",
                east_m,
                up_m,
                north_m,
                125.0,
                8.2,
                "Tracked thermal signature confirmed by FALCON-1",
                None,
                None,
                None,
                now_ms,
            );
        }

        // 6. Hostile Radar / Jammer Emitter ("RED-EMITTER-02") on Northern ridge
        {
            let east_m = 140.0 + (t * 0.025).cos() * 60.0;
            let north_m = 540.0;
            let up_m = self.terrain.sample_relief_m(east_m, north_m);
            self.upsert_sim_entity(
                "HST-EMITTER-02",
                "EMITTER-NORTH",
                "a-h-G-E-S",
                east_m,
                up_m,
                north_m,
                180.0,
                0.5,
                "RF Band-X Jammer Activity Detected",
                None,
                None,
                None,
                now_ms,
            );
        }

        // 7. Unknown Low-Slow Aerial Contact ("BOGEY-07")
        {
            let east_m = 580.0 * (t * 0.09).cos();
            let north_m = 480.0 + 180.0 * (t * 0.18).sin();
            let up_m = self.terrain.sample_relief_m(east_m, north_m) + 145.0;
            self.upsert_sim_entity(
                "UNK-BOGEY-07",
                "TRACK-704 [UNK]",
                "a-u-A-M",
                east_m,
                up_m,
                north_m,
                260.0,
                16.5,
                "Unidentified low-altitude rotary/UAS track",
                None,
                None,
                None,
                now_ms,
            );
        }

        self.refresh_threat_domes_and_zones();
        self.aar.maybe_record(self.sim_time_sec, now_ms, &self.entities);
    }

    /// Re-evaluate 3D SAM/Radar threat domes (`REQ-TAC-02`) and polygonal geofences (`REQ-TAC-03`).
    pub fn refresh_threat_domes_and_zones(&mut self) {
        let mut domes = Vec::new();
        if let Some(emitter) = self.entities.get("HST-EMITTER-02") {
            let mut dome = ThreatDome {
                uid: emitter.uid.clone(),
                callsign: "SA-15 SHORAD / JAMMER DOME".to_string(),
                affiliation: Affiliation::Hostile,
                east_m: emitter.east_m,
                up_m: emitter.up_m,
                north_m: emitter.north_m,
                radius_m: 620.0,
                ceiling_m: 440.0,
                engaged_uids: Vec::new(),
            };
            for ent in self.entities.values() {
                if ent.uid != dome.uid
                    && ent.affiliation != Affiliation::Hostile
                    && !ent.cot_type.starts_with("b-m-p-s-p-i")
                    && is_engaged_by_threat_dome(&dome, (ent.east_m, ent.up_m, ent.north_m), self.terrain.as_ref())
                {
                    dome.engaged_uids.push(ent.uid.clone());
                }
            }
            dome.engaged_uids.sort();
            domes.push(dome);
        }
        self.threat_domes = domes;

        for zone in &mut self.tactical_zones {
            zone.inside_uids.clear();
            for ent in self.entities.values() {
                if !ent.cot_type.starts_with("b-m-p-s-p-i")
                    && is_inside_tactical_zone(zone, (ent.east_m, ent.up_m, ent.north_m))
                {
                    zone.inside_uids.push(ent.uid.clone());
                }
            }
            zone.inside_uids.sort();
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn upsert_sim_entity(
        &mut self,
        uid: &str,
        callsign: &str,
        cot_type: &str,
        east_m: f64,
        up_m: f64,
        north_m: f64,
        course_deg: f64,
        speed_mps: f64,
        remarks: &str,
        sensor: Option<SensorCone>,
        video_url: Option<String>,
        parent_uid: Option<String>,
        now_ms: u64,
    ) {
        let (lat, lon, hae_m) = self.origin.enu_to_lla(east_m, up_m, north_m);
        let (affiliation, dimension) = classify_cot_type(cot_type);
        self.entities.insert(
            uid.to_string(),
            TacticalEntity {
                uid: uid.to_string(),
                callsign: callsign.to_string(),
                cot_type: cot_type.to_string(),
                affiliation,
                dimension,
                lat,
                lon,
                hae_m,
                course_deg,
                speed_mps,
                remarks: remarks.to_string(),
                sensor,
                video_url,
                parent_uid,
                east_m,
                up_m,
                north_m,
                updated_unix_ms: now_ms,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cot_xml_and_enu_projection() {
        let origin = GeoOrigin::default();
        let xml = r#"<event version="2.0" uid="TAK-TEST-01" type="a-f-G-U-C" time="2026-11-03T15:00:00Z" start="2026-11-03T15:00:00Z" stale="2026-11-03T15:05:00Z" how="h-g-i-g-o"><point lat="39.9225" lon="32.8540" hae="980.0" ce="10" le="10"/><detail><contact callsign="GOKSEL-1"/><track course="145.0" speed="12.5"/><remarks>Test unit on ridge</remarks></detail></event>"#;

        let entity = parse_cot_xml(xml, &origin, 1_700_000_000).expect("valid CoT XML");
        assert_eq!(entity.uid, "TAK-TEST-01");
        assert_eq!(entity.callsign, "GOKSEL-1");
        assert_eq!(entity.affiliation, Affiliation::Friendly);
        assert_eq!(entity.dimension, Dimension::Ground);
        assert!(entity.north_m > 250.0 && entity.north_m < 300.0);
        assert!(entity.east_m > 300.0 && entity.east_m < 380.0);
        assert_eq!(entity.remarks, "Test unit on ridge");
    }

    #[test]
    fn test_spi_gimbal_slew_and_link_parsing() {
        let mut world = TacticalWorld::new(GeoOrigin::default());
        world.step_simulation(0.1, 1000);

        let spi = world
            .slew_sensor_to_spi("UAV-FALCON-01", 320.0, -240.0, 2000)
            .expect("SPI creation succeeds");
        assert_eq!(spi.uid, "UAV-FALCON-01.SPI1");
        assert_eq!(spi.cot_type, "b-m-p-s-p-i");
        assert_eq!(spi.parent_uid.as_deref(), Some("UAV-FALCON-01"));

        world.step_simulation(0.1, 2100);
        let uav = world.entities.get("UAV-FALCON-01").expect("UAV exists");
        assert!(uav.remarks.contains("SPI LOCK"));
        assert!(uav.sensor.is_some());
    }

    #[test]
    fn test_viewshed_computation() {
        let vs = compute_viewshed(0.0, 0.0, 15.0, 600.0, 36, 15);
        assert_eq!(vs.samples.len(), 36 * 15);
        assert!(vs.visible_ratio > 0.05 && vs.visible_ratio <= 1.0);
    }

    #[test]
    fn test_viewshed_dip_with_custom_elevation_provider() {
        struct FlatTerrain;
        impl ElevationProvider for FlatTerrain {
            fn sample_relief_m(&self, _east_m: f64, _north_m: f64) -> f64 {
                100.0
            }
        }
        let vs = compute_viewshed_with_provider(&FlatTerrain, 0.0, 0.0, 10.0, 500.0, 16, 10);
        assert_eq!(vs.samples.len(), 160);
        // On a perfectly flat plane with observer +10m above ground, 100% of samples are visible
        assert!((vs.visible_ratio - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_auto_recenter_sim_toggle_and_laser_pointer_cot() {
        let mut world = TacticalWorld::new(GeoOrigin::default());
        world.step_simulation(0.1, 1000);
        assert!(world.entities.contains_key("UAV-FALCON-01"));

        world.set_sim_enabled(false);
        assert!(!world.entities.contains_key("UAV-FALCON-01"));

        world.auto_recenter_on_next_live_cot = true;
        let live_xml = r#"<event version="2.0" uid="ATAK-LIVE-99" type="a-f-G-U-C"><point lat="41.0082" lon="28.9784" hae="120.0" ce="5" le="5"/><detail><contact callsign="ISTANBUL-1"/></detail></event>"#;
        let ent = world.ingest_cot_xml(live_xml, 2000).expect("live CoT ingested");
        assert!((world.origin.lat - 41.0082).abs() < 1e-5);
        assert!(ent.east_m.abs() < 1.0 && ent.north_m.abs() < 1.0);

        let laser_xml = build_laser_pointer_cot_xml("QUEST-GOKSEL", 41.0085, 28.9790, 85.0);
        let laser_ent = world.ingest_cot_xml(&laser_xml, 2100).expect("laser CoT valid");
        assert_eq!(laser_ent.cot_type, "b-m-p-c");
    }
}
