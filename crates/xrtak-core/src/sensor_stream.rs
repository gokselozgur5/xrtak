//! TAK sensor video stream descriptor XML generator and parser.
//!
//! Provides Cursor-on-Target (CoT) `b-i-v` XML event generation and
//! TAK video connection `<feed>` descriptor XML serialization.

use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Errors arising during sensor video stream descriptor XML operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SensorStreamError {
    InvalidXml(String),
    MissingEventTag,
    MissingPointTag,
    MissingDetailTag,
    MissingVideoTag,
    MissingFeedTag,
    MissingAttribute(&'static str),
}

impl fmt::Display for SensorStreamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidXml(e) => write!(f, "Invalid XML: {}", e),
            Self::MissingEventTag => write!(f, "Missing event tag"),
            Self::MissingPointTag => write!(f, "Missing point tag"),
            Self::MissingDetailTag => write!(f, "Missing detail tag"),
            Self::MissingVideoTag => write!(f, "Missing video stream tag"),
            Self::MissingFeedTag => write!(f, "Missing feed tag"),
            Self::MissingAttribute(a) => write!(f, "Missing attribute: {}", a),
        }
    }
}

impl std::error::Error for SensorStreamError {}

/// Type of optical or imaging sensor payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SensorType {
    #[default]
    ElectroOptical,
    Infrared,
    ShortWaveInfrared,
    MidWaveInfrared,
    LongWaveInfrared,
    DualEoIr,
    SyntheticApertureRadar,
    Other(String),
}

impl SensorType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::ElectroOptical => "EO",
            Self::Infrared => "IR",
            Self::ShortWaveInfrared => "SWIR",
            Self::MidWaveInfrared => "MWIR",
            Self::LongWaveInfrared => "LWIR",
            Self::DualEoIr => "EO/IR",
            Self::SyntheticApertureRadar => "SAR",
            Self::Other(s) => s.as_str(),
        }
    }
}

impl SensorType {
    pub fn from_str_name(s: &str) -> Self {
        match s.trim().to_uppercase().as_str() {
            "EO" => Self::ElectroOptical,
            "IR" => Self::Infrared,
            "SWIR" => Self::ShortWaveInfrared,
            "MWIR" => Self::MidWaveInfrared,
            "LWIR" => Self::LongWaveInfrared,
            "EO/IR" | "EOIR" => Self::DualEoIr,
            "SAR" => Self::SyntheticApertureRadar,
            other => Self::Other(other.to_string()),
        }
    }
}

/// Optics and gimbal orientation telemetry of a sensor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SensorGimbal {
    pub sensor_type: SensorType,
    pub model: String,
    pub fov_horizontal_deg: f64,
    pub fov_vertical_deg: f64,
    pub slant_range_m: f64,
    pub azimuth_deg: f64,
    pub elevation_deg: f64,
    pub roll_deg: f64,
    pub zoom_ratio: f64,
}

impl Default for SensorGimbal {
    fn default() -> Self {
        Self {
            sensor_type: SensorType::ElectroOptical,
            model: "GENERIC-EO".to_string(),
            fov_horizontal_deg: 45.0,
            fov_vertical_deg: 30.0,
            slant_range_m: 1000.0,
            azimuth_deg: 0.0,
            elevation_deg: -45.0,
            roll_deg: 0.0,
            zoom_ratio: 1.0,
        }
    }
}

/// Video stream connection and transport parameters for TAK.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VideoStreamConnection {
    pub url: String,
    pub stream_uid: String,
    pub alias: String,
    pub rover_port: Option<u16>,
    pub rtsp_reliable: bool,
    pub ignore_embedded_klv: bool,
    pub buffer_ms: u32,
    pub preferred_uid: Option<String>,
}

impl VideoStreamConnection {
    pub fn new(url: impl Into<String>, alias: impl Into<String>) -> Self {
        let u = url.into();
        let a = alias.into();
        let su = format!("feed-{a}");
        Self {
            url: u,
            stream_uid: su,
            alias: a,
            rover_port: None,
            rtsp_reliable: true,
            ignore_embedded_klv: false,
            buffer_ms: 500,
            preferred_uid: None,
        }
    }
}

/// Complete TAK sensor video stream descriptor packet.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SensorStreamDescriptor {
    pub uid: String,
    pub callsign: String,
    pub cot_type: String,
    pub lat: f64,
    pub lon: f64,
    pub hae: f64,
    pub time_iso: String,
    pub stale_iso: String,
    pub video: VideoStreamConnection,
    pub sensor: SensorGimbal,
    pub parent_uid: Option<String>,
}

impl SensorStreamDescriptor {
    pub fn new(
        uid: impl Into<String>,
        callsign: impl Into<String>,
        lat: f64,
        lon: f64,
        hae: f64,
        video: VideoStreamConnection,
    ) -> Self {
        Self {
            uid: uid.into(),
            callsign: callsign.into(),
            cot_type: "b-i-v".to_string(),
            lat,
            lon,
            hae,
            time_iso: "2026-11-16T12:00:00Z".to_string(),
            stale_iso: "2026-11-16T12:05:00Z".to_string(),
            video,
            sensor: SensorGimbal::default(),
            parent_uid: None,
        }
    }
}

impl SensorStreamDescriptor {
    pub fn with_sensor(mut self, sensor: SensorGimbal) -> Self {
        self.sensor = sensor;
        self
    }

    pub fn with_parent_uid(mut self, parent_uid: impl Into<String>) -> Self {
        self.parent_uid = Some(parent_uid.into());
        self
    }

    pub fn with_timestamps(
        mut self,
        time_iso: impl Into<String>,
        stale_iso: impl Into<String>,
    ) -> Self {
        self.time_iso = time_iso.into();
        self.stale_iso = stale_iso.into();
        self
    }
}

impl SensorStreamDescriptor {
    /// Generates ATAK/WinTAK `<feed>` configuration XML entry.
    pub fn to_feed_entry_xml(&self) -> String {
        let rover = self
            .video
            .rover_port
            .map(|p| format!("  <entry key=\"roverPort\" value=\"{p}\"/>\n"))
            .unwrap_or_default();
        let pref = self
            .video
            .preferred_uid
            .as_deref()
            .or(self.parent_uid.as_deref())
            .map(|u| format!("  <entry key=\"preferredUid\" value=\"{}\"/>\n", escape_xml_attr(u)))
            .unwrap_or_default();
        format!(
            "<feed>\n  <entry key=\"uid\" value=\"{}\"/>\n  <entry key=\"alias\" value=\"{}\"/>\n  <entry key=\"url\" value=\"{}\"/>\n  <entry key=\"rtspReliable\" value=\"{}\"/>\n  <entry key=\"ignoreEmbeddedKLV\" value=\"{}\"/>\n  <entry key=\"bufferTime\" value=\"{}\"/>\n  <entry key=\"sensor\" value=\"{}\"/>\n{rover}{pref}</feed>",
            escape_xml_attr(&self.video.stream_uid),
            escape_xml_attr(&self.video.alias),
            escape_xml_attr(&self.video.url),
            if self.video.rtsp_reliable { 1 } else { 0 },
            self.video.ignore_embedded_klv,
            self.video.buffer_ms,
            escape_xml_attr(self.sensor.sensor_type.as_str()),
        )
    }
}

impl SensorStreamDescriptor {
    fn format_video_tag(&self) -> String {
        let rover = self
            .video
            .rover_port
            .map(|p| format!(" roverPort=\"{p}\""))
            .unwrap_or_default();
        let pref = self
            .video
            .preferred_uid
            .as_deref()
            .or(self.parent_uid.as_deref())
            .map(|u| format!(" preferredUid=\"{}\"", escape_xml_attr(u)))
            .unwrap_or_default();
        format!(
            "<__video url=\"{}\" uid=\"{}\" sensor=\"{}\" alias=\"{}\" rtspReliable=\"{}\" ignoreEmbeddedKLV=\"{}\"{rover}{pref}/>",
            escape_xml_attr(&self.video.url),
            escape_xml_attr(&self.video.stream_uid),
            escape_xml_attr(self.sensor.sensor_type.as_str()),
            escape_xml_attr(&self.video.alias),
            if self.video.rtsp_reliable { 1 } else { 0 },
            self.video.ignore_embedded_klv,
        )
    }
}

impl SensorStreamDescriptor {
    fn format_sensor_tag(&self) -> String {
        format!(
            "<sensor model=\"{}\" type=\"{}\" fov=\"{:.1}\" vfov=\"{:.1}\" range=\"{:.1}\" azimuth=\"{:.1}\" elevation=\"{:.1}\" roll=\"{:.1}\" zoom=\"{:.2}\"/>",
            escape_xml_attr(&self.sensor.model),
            escape_xml_attr(self.sensor.sensor_type.as_str()),
            self.sensor.fov_horizontal_deg,
            self.sensor.fov_vertical_deg,
            self.sensor.slant_range_m,
            self.sensor.azimuth_deg,
            self.sensor.elevation_deg,
            self.sensor.roll_deg,
            self.sensor.zoom_ratio,
        )
    }

    fn format_link_tag(&self) -> String {
        self.parent_uid
            .as_deref()
            .map(|p| format!("<link relation=\"p-p\" uid=\"{}\" type=\"a-f-G-U-C\"/>", escape_xml_attr(p)))
            .unwrap_or_default()
    }
}

impl SensorStreamDescriptor {
    /// Generates full TAK Cursor-on-Target `b-i-v` XML event string.
    pub fn to_cot_xml(&self) -> String {
        let video_elem = self.format_video_tag();
        let sensor_elem = self.format_sensor_tag();
        let link_elem = self.format_link_tag();
        let contact = format!("<contact callsign=\"{}\"/>", escape_xml_attr(&self.callsign));
        format!(
            "<event version=\"2.0\" uid=\"{}\" type=\"{}\" time=\"{}\" start=\"{}\" stale=\"{}\" how=\"m-g\">\n  <point lat=\"{:.6}\" lon=\"{:.6}\" hae=\"{:.1}\" ce=\"10.0\" le=\"10.0\"/>\n  <detail>\n    {contact}\n    {video_elem}\n    {sensor_elem}\n    {link_elem}\n  </detail>\n</event>",
            escape_xml_attr(&self.uid),
            escape_xml_attr(&self.cot_type),
            escape_xml_attr(&self.time_iso),
            escape_xml_attr(&self.time_iso),
            escape_xml_attr(&self.stale_iso),
            self.lat,
            self.lon,
            self.hae,
        )
    }
}

/// Serializes a slice of sensor stream descriptors to TAK `<videoConnections>` document.
pub fn to_video_connections_xml(descriptors: &[SensorStreamDescriptor]) -> String {
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<videoConnections>\n");
    for desc in descriptors {
        let entry = desc.to_feed_entry_xml();
        for line in entry.lines() {
            out.push_str("  ");
            out.push_str(line);
            out.push('\n');
        }
    }
    out.push_str("</videoConnections>");
    out
}

/// Parses a TAK `<feed>` entry XML string into a VideoStreamConnection.
pub fn parse_feed_entry_xml(xml: &str) -> Result<VideoStreamConnection, SensorStreamError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut in_feed = false;
    let mut uid = None;
    let mut alias = None;
    let mut url = None;
    let mut rover_port = None;
    let mut rtsp_reliable = true;
    let mut ignore_klv = false;
    let mut buffer_ms = 500u32;
    let mut preferred_uid = None;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"feed" => in_feed = true,
            Ok(Event::Empty(ref e)) if in_feed && e.name().as_ref() == b"entry" => {
                let mut k = None;
                let mut v = None;
                for attr in e.attributes().flatten() {
                    match attr.key.as_ref() {
                        b"key" => k = Some(unescape_attr(&attr)),
                        b"value" => v = Some(unescape_attr(&attr)),
                        _ => {}
                    }
                }
                if let (Some(k), Some(v)) = (k, v) {
                    match k.as_str() {
                        "uid" => uid = Some(v),
                        "alias" => alias = Some(v),
                        "url" => url = Some(v),
                        "roverPort" => rover_port = v.parse::<u16>().ok(),
                        "rtspReliable" => rtsp_reliable = v == "1" || v.eq_ignore_ascii_case("true"),
                        "ignoreEmbeddedKLV" => ignore_klv = v.eq_ignore_ascii_case("true"),
                        "bufferTime" => buffer_ms = v.parse::<u32>().unwrap_or(500),
                        "preferredUid" => preferred_uid = Some(v),
                        _ => {}
                    }
                }
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"feed" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(SensorStreamError::InvalidXml(e.to_string())),
            _ => {}
        }
        buf.clear();
    }
    if !in_feed {
        return Err(SensorStreamError::MissingFeedTag);
    }
    let u = url.ok_or(SensorStreamError::MissingAttribute("url"))?;
    let a = alias.unwrap_or_else(|| "Video Stream".to_string());
    let su = uid.unwrap_or_else(|| format!("feed-{a}"));
    Ok(VideoStreamConnection {
        url: u,
        stream_uid: su,
        alias: a,
        rover_port,
        rtsp_reliable,
        ignore_embedded_klv: ignore_klv,
        buffer_ms,
        preferred_uid,
    })
}

/// Parses a TAK CoT `b-i-v` XML event into a SensorStreamDescriptor.
pub fn parse_sensor_stream_cot_xml(xml: &str) -> Result<SensorStreamDescriptor, SensorStreamError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut uid = None;
    let mut cot_type = "b-i-v".to_string();
    let mut time_iso = String::new();
    let mut stale_iso = String::new();
    let mut lat = None;
    let mut lon = None;
    let mut hae = 0.0;
    let mut callsign = None;
    let mut parent_uid = None;
    let mut video_conn: Option<VideoStreamConnection> = None;
    let mut sensor_gimbal = SensorGimbal::default();
    let mut in_detail = false;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"event" => {
                for a in e.attributes().flatten() {
                    match a.key.as_ref() {
                        b"uid" => uid = Some(unescape_attr(&a)),
                        b"type" => cot_type = unescape_attr(&a),
                        b"time" => time_iso = unescape_attr(&a),
                        b"stale" => stale_iso = unescape_attr(&a),
                        _ => {}
                    }
                }
            }
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) if e.name().as_ref() == b"point" => {
                for a in e.attributes().flatten() {
                    match a.key.as_ref() {
                        b"lat" => lat = String::from_utf8_lossy(&a.value).parse::<f64>().ok(),
                        b"lon" => lon = String::from_utf8_lossy(&a.value).parse::<f64>().ok(),
                        b"hae" => hae = String::from_utf8_lossy(&a.value).parse::<f64>().unwrap_or(0.0),
                        _ => {}
                    }
                }
            }
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"detail" => in_detail = true,
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e))
                if in_detail && (e.name().as_ref() == b"__video" || e.name().as_ref() == b"video") =>
            {
                let mut url = None;
                let mut stream_uid = None;
                let mut alias = None;
                let mut rover_port = None;
                let mut rtsp_reliable = true;
                let mut ignore_klv = false;
                let mut pref = None;
                for a in e.attributes().flatten() {
                    match a.key.as_ref() {
                        b"url" => url = Some(unescape_attr(&a)),
                        b"uid" => stream_uid = Some(unescape_attr(&a)),
                        b"alias" => alias = Some(unescape_attr(&a)),
                        b"roverPort" => rover_port = String::from_utf8_lossy(&a.value).parse::<u16>().ok(),
                        b"rtspReliable" => rtsp_reliable = a.value.as_ref() == b"1" || a.value.as_ref() == b"true",
                        b"ignoreEmbeddedKLV" => ignore_klv = a.value.as_ref() == b"true",
                        b"preferredUid" => pref = Some(unescape_attr(&a)),
                        _ => {}
                    }
                }
                if let Some(u) = url {
                    let a = alias.unwrap_or_else(|| "Video Stream".to_string());
                    let su = stream_uid.unwrap_or_else(|| format!("feed-{a}"));
                    video_conn = Some(VideoStreamConnection {
                        url: u, stream_uid: su, alias: a, rover_port, rtsp_reliable,
                        ignore_embedded_klv: ignore_klv, buffer_ms: 500, preferred_uid: pref,
                    });
                }
            }
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e))
                if in_detail && e.name().as_ref() == b"contact" =>
            {
                for a in e.attributes().flatten() {
                    if a.key.as_ref() == b"callsign" {
                        callsign = Some(unescape_attr(&a));
                    }
                }
            }
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e))
                if in_detail && e.name().as_ref() == b"link" =>
            {
                for a in e.attributes().flatten() {
                    if a.key.as_ref() == b"uid" {
                        parent_uid = Some(unescape_attr(&a));
                    }
                }
            }
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e))
                if in_detail && e.name().as_ref() == b"sensor" =>
            {
                for a in e.attributes().flatten() {
                    let val = String::from_utf8_lossy(&a.value);
                    match a.key.as_ref() {
                        b"model" => sensor_gimbal.model = val.into_owned(),
                        b"type" => sensor_gimbal.sensor_type = SensorType::from_str_name(&val),
                        b"fov" => sensor_gimbal.fov_horizontal_deg = val.parse().unwrap_or(45.0),
                        b"vfov" => sensor_gimbal.fov_vertical_deg = val.parse().unwrap_or(30.0),
                        b"range" => sensor_gimbal.slant_range_m = val.parse().unwrap_or(1000.0),
                        b"azimuth" => sensor_gimbal.azimuth_deg = val.parse().unwrap_or(0.0),
                        b"elevation" => sensor_gimbal.elevation_deg = val.parse().unwrap_or(-45.0),
                        b"roll" => sensor_gimbal.roll_deg = val.parse().unwrap_or(0.0),
                        b"zoom" => sensor_gimbal.zoom_ratio = val.parse().unwrap_or(1.0),
                        _ => {}
                    }
                }
            }
            Ok(Event::End(ref e)) if e.name().as_ref() == b"event" => break,
            Ok(Event::Eof) => break,
            Err(e) => return Err(SensorStreamError::InvalidXml(e.to_string())),
            _ => {}
        }
        buf.clear();
    }
    let u = uid.ok_or(SensorStreamError::MissingEventTag)?;
    let la = lat.ok_or(SensorStreamError::MissingPointTag)?;
    let lo = lon.ok_or(SensorStreamError::MissingPointTag)?;
    let v = video_conn.ok_or(SensorStreamError::MissingVideoTag)?;
    let cs = callsign.unwrap_or_else(|| u.clone());
    Ok(SensorStreamDescriptor {
        uid: u,
        callsign: cs,
        cot_type,
        lat: la,
        lon: lo,
        hae,
        time_iso,
        stale_iso,
        video: v,
        sensor: sensor_gimbal,
        parent_uid,
    })
}

fn unescape_attr(a: &quick_xml::events::attributes::Attribute) -> String {
    a.unescape_value()
        .map(|v| v.into_owned())
        .unwrap_or_else(|_| String::from_utf8_lossy(&a.value).into_owned())
}

fn escape_xml_attr(val: &str) -> String {
    val.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sensor_stream_defaults_and_builder() {
        let conn = VideoStreamConnection::new("rtsp://10.0.0.1/live", "UAV-Cam");
        let desc = SensorStreamDescriptor::new("DESC-1", "RECON-1", 39.9, 32.8, 1200.0, conn)
            .with_parent_uid("UAV-LEAD");
        assert_eq!(desc.callsign, "RECON-1");
        assert_eq!(desc.parent_uid.as_deref(), Some("UAV-LEAD"));
        assert_eq!(desc.video.alias, "UAV-Cam");
        assert_eq!(desc.sensor.model, "GENERIC-EO");
    }

    #[test]
    fn test_feed_entry_xml_generation_and_roundtrip() {
        let mut conn = VideoStreamConnection::new("rtsp://192.168.1.100:8554/live", "Thermal-1");
        conn.rover_port = Some(49152);
        conn.preferred_uid = Some("UAV-DRONE-99".to_string());
        let desc = SensorStreamDescriptor::new("DESC-2", "TH-1", 40.0, 30.0, 500.0, conn);
        let xml = desc.to_feed_entry_xml();
        let parsed = parse_feed_entry_xml(&xml).expect("feed parse succeeds");
        assert_eq!(parsed.url, "rtsp://192.168.1.100:8554/live");
        assert_eq!(parsed.alias, "Thermal-1");
        assert_eq!(parsed.rover_port, Some(49152));
        assert_eq!(parsed.preferred_uid.as_deref(), Some("UAV-DRONE-99"));
    }

    #[test]
    fn test_cot_xml_generation_and_roundtrip() {
        let conn = VideoStreamConnection::new("rtsp://192.168.2.50:554/ch0", "Cam-EO");
        let gimbal = SensorGimbal {
            sensor_type: SensorType::DualEoIr,
            model: "TRILLIUM-HD80".to_string(),
            fov_horizontal_deg: 35.5,
            fov_vertical_deg: 22.0,
            slant_range_m: 2500.0,
            azimuth_deg: 185.0,
            elevation_deg: -30.0,
            roll_deg: 2.5,
            zoom_ratio: 4.0,
        };
        let desc = SensorStreamDescriptor::new("DESC-COT-1", "DRONE-ALPHA", 41.01, 28.97, 850.0, conn)
            .with_sensor(gimbal)
            .with_parent_uid("UAV-DRONE-ALPHA");
        let xml = desc.to_cot_xml();
        let parsed = parse_sensor_stream_cot_xml(&xml).expect("cot parse succeeds");
        assert_eq!(parsed.uid, "DESC-COT-1");
        assert_eq!(parsed.callsign, "DRONE-ALPHA");
        assert_eq!(parsed.lat, 41.01);
        assert_eq!(parsed.sensor.sensor_type, SensorType::DualEoIr);
        assert_eq!(parsed.sensor.model, "TRILLIUM-HD80");
        assert_eq!(parsed.parent_uid.as_deref(), Some("UAV-DRONE-ALPHA"));
    }

    #[test]
    fn test_video_connections_catalog() {
        let conn1 = VideoStreamConnection::new("rtsp://10.0.0.1/live", "Feed-1");
        let desc1 = SensorStreamDescriptor::new("D1", "C1", 10.0, 20.0, 100.0, conn1);
        let conn2 = VideoStreamConnection::new("rtsp://10.0.0.2/live", "Feed-2");
        let desc2 = SensorStreamDescriptor::new("D2", "C2", 11.0, 21.0, 200.0, conn2);
        let catalog = to_video_connections_xml(&[desc1, desc2]);
        assert!(catalog.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(catalog.contains("<videoConnections>"));
        assert!(catalog.contains("Feed-1"));
        assert!(catalog.contains("Feed-2"));
    }

    #[test]
    fn test_error_handling_on_invalid_xml_or_missing_tags() {
        let empty_feed = "<feed></feed>";
        assert!(matches!(
            parse_feed_entry_xml(empty_feed),
            Err(SensorStreamError::MissingAttribute("url"))
        ));

        let non_feed = "<root><child/></root>";
        assert!(matches!(
            parse_feed_entry_xml(non_feed),
            Err(SensorStreamError::MissingFeedTag)
        ));

        let no_video_cot = r#"<event uid="U1" type="b-i-v"><point lat="1" lon="2"/><detail></detail></event>"#;
        assert!(matches!(
            parse_sensor_stream_cot_xml(no_video_cot),
            Err(SensorStreamError::MissingVideoTag)
        ));
    }
}
