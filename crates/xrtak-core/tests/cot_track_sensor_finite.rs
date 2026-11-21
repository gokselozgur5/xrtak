use xrtak_core::{parse_cot_xml, GeoOrigin};

fn track_event(course: &str, speed: &str) -> String {
    format!(
        r#"<event version="2.0" uid="U-TRACK" type="a-f-G">
            <point lat="39.92" lon="32.85" hae="900"/>
            <detail>
                <track course="{course}" speed="{speed}"/>
            </detail>
        </event>"#
    )
}

fn sensor_event(attr: &str, val: &str) -> String {
    format!(
        r#"<event version="2.0" uid="U-SENSOR" type="a-f-G">
            <point lat="39.92" lon="32.85" hae="900"/>
            <detail>
                <sensor {attr}="{val}"/>
            </detail>
        </event>"#
    )
}

#[test]
fn parse_cot_xml_replaces_non_finite_track_and_sensor_with_defaults() {
    let origin = GeoOrigin::default();

    for bad in ["NaN", "nan", "inf", "-inf", "Infinity", "-Infinity"] {
        let ent = parse_cot_xml(&track_event(bad, "10.0"), &origin, 0)
            .expect("track course NaN/inf handled");
        assert_eq!(ent.course_deg, 0.0);
        assert!(ent.course_deg.is_finite());

        let ent = parse_cot_xml(&track_event("180.0", bad), &origin, 0)
            .expect("track speed NaN/inf handled");
        assert_eq!(ent.speed_mps, 0.0);
        assert!(ent.speed_mps.is_finite());

        // Sensor azimuth default: 0.0
        let ent = parse_cot_xml(&sensor_event("azimuth", bad), &origin, 0)
            .expect("sensor azimuth NaN/inf handled");
        let s = ent.sensor.expect("sensor present");
        assert_eq!(s.azimuth_deg, 0.0);
        assert!(s.azimuth_deg.is_finite());

        // Sensor elevation default: -35.0
        let ent = parse_cot_xml(&sensor_event("elevation", bad), &origin, 0)
            .expect("sensor elevation NaN/inf handled");
        let s = ent.sensor.expect("sensor present");
        assert_eq!(s.elevation_deg, -35.0);
        assert!(s.elevation_deg.is_finite());

        // Sensor fov / hfov default: 36.0
        let ent = parse_cot_xml(&sensor_event("fov", bad), &origin, 0)
            .expect("sensor fov NaN/inf handled");
        let s = ent.sensor.expect("sensor present");
        assert_eq!(s.hfov_deg, 36.0);
        assert!(s.hfov_deg.is_finite());

        let ent = parse_cot_xml(&sensor_event("hfov", bad), &origin, 0)
            .expect("sensor hfov NaN/inf handled");
        let s = ent.sensor.expect("sensor present");
        assert_eq!(s.hfov_deg, 36.0);
        assert!(s.hfov_deg.is_finite());

        // Sensor vfov default: 22.0
        let ent = parse_cot_xml(&sensor_event("vfov", bad), &origin, 0)
            .expect("sensor vfov NaN/inf handled");
        let s = ent.sensor.expect("sensor present");
        assert_eq!(s.vfov_deg, 22.0);
        assert!(s.vfov_deg.is_finite());

        // Sensor range default: 650.0
        let ent = parse_cot_xml(&sensor_event("range", bad), &origin, 0)
            .expect("sensor range NaN/inf handled");
        let s = ent.sensor.expect("sensor present");
        assert_eq!(s.range_m, 650.0);
        assert!(s.range_m.is_finite());
    }
}
