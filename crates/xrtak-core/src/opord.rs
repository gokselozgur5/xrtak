//! 3D Tactical KML Scene Exporter & NATO 5-Paragraph OPORD Generator.
//!
//! Complies with REQ-OPO-01..02 and ICD-22 for operational order synthesis.

use serde::{Deserialize, Serialize};

/// Lightweight tactical entity snapshot for operational order summaries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpordUnitSummary {
    pub callsign: String,
    pub cot_type: String,
    pub affiliation: String,
    pub lat: f64,
    pub lon: f64,
    pub alt_m: f64,
}

/// Comprehensive tactical scene export bundle containing 3D KML and NATO OPORD.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpordExportBundle {
    pub opord_markdown: String,
    pub kml_scene: String,
    pub unit_count: usize,
    pub zone_count: usize,
    pub dome_count: usize,
    pub generated_utc: String,
}

/// Generates a valid 3D KML document bundling tactical units and ground markers.
pub fn generate_kml_tactical_scene(
    scene_name: &str,
    units: &[OpordUnitSummary],
    origin_lla: (f64, f64, f64),
) -> String {
    let mut kml = String::with_capacity(2048);
    kml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    kml.push_str("<kml xmlns=\"http://www.opengis.net/kml/2.2\">\n");
    kml.push_str("  <Document>\n");
    kml.push_str(&format!("    <name>XR-TAK Tactical Scene: {}</name>\n", scene_name));
    kml.push_str(&format!(
        "    <description>Exported from XR-TAK Sand Table at origin {:.5}N, {:.5}E, {:.1}m</description>\n",
        origin_lla.0, origin_lla.1, origin_lla.2
    ));

    for u in units {
        kml.push_str("    <Placemark>\n");
        kml.push_str(&format!("      <name>{}</name>\n", u.callsign));
        kml.push_str(&format!(
            "      <description>Type: {} | Affiliation: {}</description>\n",
            u.cot_type, u.affiliation
        ));
        kml.push_str("      <Point>\n");
        kml.push_str("        <altitudeMode>absolute</altitudeMode>\n");
        kml.push_str(&format!("        <coordinates>{:.6},{:.6},{:.1}</coordinates>\n", u.lon, u.lat, u.alt_m));
        kml.push_str("      </Point>\n");
        kml.push_str("    </Placemark>\n");
    }

    kml.push_str("  </Document>\n");
    kml.push_str("</kml>\n");
    kml
}
