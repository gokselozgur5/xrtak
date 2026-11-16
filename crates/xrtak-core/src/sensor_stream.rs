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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SensorType {
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

impl Default for SensorType {
    fn default() -> Self {
        Self::ElectroOptical
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
        let su = format!("feed-{}", &a);
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
            .map(|u| format!("  <entry key=\"preferredUid\" value=\"{u}\"/>\n"))
            .unwrap_or_default();
        format!(
            "<feed>\n  <entry key=\"uid\" value=\"{}\"/>\n  <entry key=\"alias\" value=\"{}\"/>\n  <entry key=\"url\" value=\"{}\"/>\n  <entry key=\"rtspReliable\" value=\"{}\"/>\n  <entry key=\"ignoreEmbeddedKLV\" value=\"{}\"/>\n  <entry key=\"bufferTime\" value=\"{}\"/>\n  <entry key=\"sensor\" value=\"{}\"/>\n{rover}{pref}</feed>",
            self.video.stream_uid,
            self.video.alias,
            self.video.url,
            if self.video.rtsp_reliable { 1 } else { 0 },
            self.video.ignore_embedded_klv,
            self.video.buffer_ms,
            self.sensor.sensor_type.as_str(),
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
            .map(|u| format!(" preferredUid=\"{u}\""))
            .unwrap_or_default();
        format!(
            "<__video url=\"{}\" uid=\"{}\" sensor=\"{}\" alias=\"{}\" rtspReliable=\"{}\" ignoreEmbeddedKLV=\"{}\"{rover}{pref}/>",
            self.video.url,
            self.video.stream_uid,
            self.sensor.sensor_type.as_str(),
            self.video.alias,
            if self.video.rtsp_reliable { 1 } else { 0 },
            self.video.ignore_embedded_klv,
        )
    }
}

impl SensorStreamDescriptor {
    fn format_sensor_tag(&self) -> String {
        format!(
            "<sensor model=\"{}\" type=\"{}\" fov=\"{:.1}\" vfov=\"{:.1}\" range=\"{:.1}\" azimuth=\"{:.1}\" elevation=\"{:.1}\" roll=\"{:.1}\" zoom=\"{:.2}\"/>",
            self.sensor.model,
            self.sensor.sensor_type.as_str(),
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
            .map(|p| format!("<link relation=\"p-p\" uid=\"{p}\" type=\"a-f-G-U-C\"/>"))
            .unwrap_or_default()
    }
}

impl SensorStreamDescriptor {
    /// Generates full TAK Cursor-on-Target `b-i-v` XML event string.
    pub fn to_cot_xml(&self) -> String {
        let video_elem = self.format_video_tag();
        let sensor_elem = self.format_sensor_tag();
        let link_elem = self.format_link_tag();
        let contact = format!("<contact callsign=\"{}\"/>", self.callsign);
        format!(
            "<event version=\"2.0\" uid=\"{}\" type=\"{}\" time=\"{}\" start=\"{}\" stale=\"{}\" how=\"m-g\">\n  <point lat=\"{:.6}\" lon=\"{:.6}\" hae=\"{:.1}\" ce=\"10.0\" le=\"10.0\"/>\n  <detail>\n    {contact}\n    {video_elem}\n    {sensor_elem}\n    {link_elem}\n  </detail>\n</event>",
            self.uid,
            self.cot_type,
            self.time_iso,
            self.time_iso,
            self.stale_iso,
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
                        b"key" => k = Some(String::from_utf8_lossy(&attr.value).into_owned()),
                        b"value" => v = Some(String::from_utf8_lossy(&attr.value).into_owned()),
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
    let su = uid.unwrap_or_else(|| format!("feed-{}", &a));
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
                        b"uid" => uid = Some(String::from_utf8_lossy(&a.value).into_owned()),
                        b"type" => cot_type = String::from_utf8_lossy(&a.value).into_owned(),
                        b"time" => time_iso = String::from_utf8_lossy(&a.value).into_owned(),
                        b"stale" => stale_iso = String::from_utf8_lossy(&a.value).into_owned(),
                        _ => {}
                    }
                }
            }
            Ok(Event::Empty(ref e)) if e.name().as_ref() == b"point" => {
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
