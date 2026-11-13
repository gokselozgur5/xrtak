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
    kml.push_str(&format!(
        "    <name>XR-TAK Tactical Scene: {}</name>\n",
        scene_name
    ));
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
        kml.push_str(&format!(
            "        <coordinates>{:.6},{:.6},{:.1}</coordinates>\n",
            u.lon, u.lat, u.alt_m
        ));
        kml.push_str("      </Point>\n");
        kml.push_str("    </Placemark>\n");
    }

    kml.push_str("  </Document>\n");
    kml.push_str("</kml>\n");
    kml
}

/// Generates standard 5-Paragraph NATO Field Operation Order (OPORD / SMEAC) Markdown.
pub fn generate_nato_opord_markdown(
    operation_name: &str,
    origin_lla: (f64, f64, f64),
    units: &[OpordUnitSummary],
    zone_count: usize,
    dome_count: usize,
) -> String {
    let mut doc = String::with_capacity(2048);
    doc.push_str(&format!(
        "# NATO OPERATION ORDER (OPORD): {}\n\n",
        operation_name.to_uppercase()
    ));
    doc.push_str(&format!(
        "**Area of Operations (AO Center):** {:.5}N, {:.5}E (Elevation: {:.0}m ASL)\n\n",
        origin_lla.0, origin_lla.1, origin_lla.2
    ));

    doc.push_str("## 1. SITUATION\n");
    doc.push_str(&format!(
        "- **Forces Tracked:** {} active tactical units in theatre.\n",
        units.len()
    ));
    doc.push_str(&format!(
        "- **Threat Profile:** {} air-defense/SAM threat domes active.\n",
        dome_count
    ));
    doc.push_str(&format!(
        "- **Airspace Restrictions:** {} tactical geofence/ROZ zones established.\n\n",
        zone_count
    ));

    doc.push_str("## 2. MISSION\n");
    doc.push_str(&format!("Task Force conducts multi-domain spatial awareness and persistent ISR across {} to ensure aerial deconfliction and datalink integrity.\n\n", operation_name));

    doc.push_str("## 3. EXECUTION\n");
    doc.push_str("- **Commander's Intent:** Maintain 3D holographic common operational picture (COP) via XR-TAK.\n");
    doc.push_str("- **Airspace Coordination:** Strict compliance with 300m horizontal / 100m vertical CPA margins.\n");
    doc.push_str("- **Rules of Engagement (ROE):** Autonomous gimbal tracking permitted; engagement requires positive identification (PID).\n\n");

    doc.push_str("## 4. ADMINISTRATION & LOGISTICS\n");
    doc.push_str("- **Primary Recovery Site:** Base Station ENU (0, 0, Relief Alt).\n");
    doc.push_str("- **Power & Spectrum:** MPU5 / Silvus battery rotation every 120 mins.\n\n");

    doc.push_str("## 5. COMMAND & SIGNAL\n");
    doc.push_str("- **Primary C2 Mesh:** TAK Server TCP/UDP :4242 / :8087.\n");
    doc.push_str(
        "- **Datalink Bands:** UHF 900MHz (Telemetry), ISM 2.4GHz (C2), SHF 5.8GHz (Video).\n",
    );
    doc.push_str("- **Emergency Signal:** Code RED via ATAK emergency beacon broadcast.\n");
    doc
}

/// Synthesizes complete export bundle containing 3D KML Scene and NATO OPORD.
pub fn build_opord_bundle(
    operation_name: &str,
    origin_lla: (f64, f64, f64),
    units: &[OpordUnitSummary],
    zone_count: usize,
    dome_count: usize,
) -> OpordExportBundle {
    let kml_scene = generate_kml_tactical_scene(operation_name, units, origin_lla);
    let opord_markdown = generate_nato_opord_markdown(
        operation_name,
        origin_lla,
        units,
        zone_count,
        dome_count,
    );

    OpordExportBundle {
        opord_markdown,
        kml_scene,
        unit_count: units.len(),
        zone_count,
        dome_count,
        generated_utc: "2026-11-07T11:00:00Z".to_string(),
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_kml_tactical_scene_generation() {
        let units = [
            OpordUnitSummary {
                callsign: "FALCON-1".to_string(),
                cot_type: "a-f-A-M-F-Q".to_string(),
                affiliation: "Friendly".to_string(),
                lat: 39.9218,
                lon: 32.8522,
                alt_m: 1250.0,
            },
        ];
        let kml = generate_kml_tactical_scene("OPERATION-PARALLAX", &units, (39.92, 32.85, 950.0));
        assert!(kml.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(kml.contains("<name>FALCON-1</name>"));
        assert!(kml.contains("<coordinates>32.852200,39.921800,1250.0</coordinates>"));
    }

    #[test]
    fn test_nato_opord_markdown_structure() {
        let units = [
            OpordUnitSummary {
                callsign: "VIPER-2".to_string(),
                cot_type: "a-h-A".to_string(),
                affiliation: "Hostile".to_string(),
                lat: 39.93,
                lon: 32.86,
                alt_m: 1400.0,
            },
        ];
        let bundle = build_opord_bundle("OP-ANATOLIA-SHIELD", (39.92, 32.85, 950.0), &units, 2, 1);
        assert_eq!(bundle.unit_count, 1);
        assert_eq!(bundle.zone_count, 2);
        assert_eq!(bundle.dome_count, 1);
        assert!(bundle.opord_markdown.contains("## 1. SITUATION"));
        assert!(bundle.opord_markdown.contains("## 2. MISSION"));
        assert!(bundle.opord_markdown.contains("## 3. EXECUTION"));
        assert!(bundle.opord_markdown.contains("## 4. ADMINISTRATION & LOGISTICS"));
        assert!(bundle.opord_markdown.contains("## 5. COMMAND & SIGNAL"));
    }
}
