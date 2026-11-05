use crate::geo::ElevationProvider;
use crate::GeoOrigin;
use serde::{Deserialize, Serialize};

/// BSI Flex 335 v2.0 SAPIENT Autonomous Edge AI Detection Report (`REQ-COL`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SapientDetectionReport {
    pub node_id: String,
    pub object_id: String,
    pub classification: String,
    pub confidence: f64,
    pub east_m: f64,
    pub north_m: f64,
    pub alt_offset_m: Option<f64>,
    pub speed_mps: Option<f64>,
    pub course_deg: Option<f64>,
    pub auto_slew_sensor_uid: Option<String>,
}

/// Map BSI Flex 335 SAPIENT object classes to MIL-STD-2525 / CoT types.
pub fn sapient_class_to_cot_type(classification: &str) -> &'static str {
    match classification.trim().to_ascii_uppercase().as_str() {
        "UAV_ROTARY" | "UAV" | "DRONE" | "UAS" => "a-h-A-M-F-Q",
        "HOSTILE_ARMOR" | "ARMOR" | "TANK" | "VEHICLE" => "a-h-G-E-V-A",
        "RF_JAMMER" | "EMITTER" | "RADAR" | "SAM" => "a-h-G-E-S",
        "DISMOUNT_HOSTILE" | "INFANTRY" | "HUMAN" => "a-h-G-U-C-I",
        _ => "a-u-G",
    }
}

/// Convert a SAPIENT AI Detection Report into a standard CoT 2.0 XML `<event>` string.
pub fn build_sapient_cot_xml(
    report: &SapientDetectionReport,
    origin: &GeoOrigin,
    terrain: &dyn ElevationProvider,
) -> String {
    let cot_type = sapient_class_to_cot_type(&report.classification);
    let ground_m = terrain.sample_relief_m(report.east_m, report.north_m);
    let default_offset = if cot_type.contains("-A-") { 155.0 } else { 0.0 };
    let up_m = ground_m + report.alt_offset_m.unwrap_or(default_offset);
    let (lat, lon, hae_m) = origin.enu_to_lla(report.east_m, up_m, report.north_m);

    let clean_obj = report.object_id.replace(' ', "-");
    let uid = format!("SAPIENT-{clean_obj}");
    let pct = (report.confidence.clamp(0.0, 1.0) * 100.0).round() as u32;
    let callsign = format!("AI-{} ({pct}%)", report.classification.to_ascii_uppercase());
    let course = report.course_deg.unwrap_or(0.0);
    let speed = report.speed_mps.unwrap_or(0.0);
    let remarks = format!(
        "[BSI Flex 335 SAPIENT] node={} class={} conf={pct}%",
        report.node_id, report.classification
    );

    format!(
        r#"<event version="2.0" uid="{uid}" type="{cot_type}" how="m-g"><point lat="{lat:.6}" lon="{lon:.6}" hae="{hae_m:.1}" ce="4.0" le="4.0"/><detail><contact callsign="{callsign}"/><track course="{course:.1}" speed="{speed:.1}"/><remarks>{remarks}</remarks></detail></event>"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geo::ProceduralReliefModel;

    #[test]
    fn test_sapient_classification_and_cot_xml_generation() {
        let origin = GeoOrigin::default();
        let report = SapientDetectionReport {
            node_id: "ASM-EDGE-01".to_string(),
            object_id: "DET-901".to_string(),
            classification: "UAV_ROTARY".to_string(),
            confidence: 0.94,
            east_m: -210.0,
            north_m: 315.0,
            alt_offset_m: Some(140.0),
            speed_mps: Some(14.5),
            course_deg: Some(135.0),
            auto_slew_sensor_uid: Some("UAV-FALCON-01".to_string()),
        };
        let xml = build_sapient_cot_xml(&report, &origin, &ProceduralReliefModel);
        assert!(xml.contains(r#"uid="SAPIENT-DET-901""#));
        assert!(xml.contains(r#"type="a-h-A-M-F-Q""#));
        assert!(xml.contains("AI-UAV_ROTARY (94%)"));
        assert!(xml.contains("[BSI Flex 335 SAPIENT] node=ASM-EDGE-01"));
    }
}
