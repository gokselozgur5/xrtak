use quick_xml::events::Event;
use quick_xml::Reader;
use xrtak_core::sensor_stream::{SensorStreamDescriptor, VideoStreamConnection};

fn video_url(xml: &str) -> Option<String> {
    let mut reader = Reader::from_str(xml);
    loop {
        match reader.read_event() {
            Ok(Event::Empty(e)) if e.name().as_ref() == b"__video" => {
                let attr = e.try_get_attribute("url").ok()??;
                return attr.unescape_value().ok().map(|v| v.into_owned());
            }
            Ok(Event::Eof) | Err(_) => return None,
            _ => {}
        }
    }
}

#[test]
fn url_with_query_string_is_emitted_as_well_formed_xml() {
    let url = "rtsp://10.0.0.5:554/live?user=op&token=abc";
    let conn = VideoStreamConnection::new(url, "Cam");
    let xml = SensorStreamDescriptor::new("D1", "C1", 1.0, 2.0, 3.0, conn).to_cot_xml();
    assert_eq!(video_url(&xml).as_deref(), Some(url));
}
