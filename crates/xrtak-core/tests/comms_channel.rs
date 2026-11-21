use xrtak_core::comms_channel::{CommsChannelPacket, RadioChannel, RadioMode};

#[test]
fn comms_channel_tx_power_preserves_sub_tenth_watt_precision() {
    let ch = RadioChannel::new(1, "PRC-152", 30_000_000, 25000, RadioMode::Fm).with_tx_power(0.04);
    let packet = CommsChannelPacket::new(
        "PLAN-PWR",
        "ALPHA",
        "2026-11-16T10:00:00Z",
        "2026-11-16T11:00:00Z",
    )
    .add_channel(ch);

    let xml = packet.to_cot_xml();
    assert!(
        xml.contains("power=\"0.04\""),
        "expected power=\"0.04\" in xml, got: {}",
        xml
    );

    let parsed = CommsChannelPacket::from_cot_xml(&xml).expect("parsed");
    assert_eq!(parsed.channels[0].tx_power_watts, Some(0.04));
}
