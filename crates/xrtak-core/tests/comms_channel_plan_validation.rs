use xrtak_core::comms_channel::{CommsChannelError, CommsChannelPacket};

#[test]
fn malformed_channel_plan_is_rejected() {
    let xml = concat!(
        r#"<event version="2.0" uid="PLAN-1" type="a-f-G-U-C"><detail>"#,
        r#"<radioChannels primary="9">"#,
        r#"<channel id="one" name="TAC-1" frequency="225.5MHz"/>"#,
        r#"</radioChannels></detail></event>"#
    );
    // No contact, a primary id that names no channel, a non-numeric id and frequency.
    let result = CommsChannelPacket::from_cot_xml(xml);
    assert!(
        result.is_err(),
        "accepted as {:?}",
        result.unwrap().channels
    );

    let empty =
        r#"<event uid="PLAN-2"><detail><contact callsign="TOC"/><radioChannels/></detail></event>"#;
    assert_eq!(
        CommsChannelPacket::from_cot_xml(empty),
        Err(CommsChannelError::EmptyChannelList)
    );
}
