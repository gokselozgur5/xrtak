use xrtak_core::hud::*;

#[test]
fn test_hud_cue_four_cardinal_bearings() {
    let obs = (0.0, 0.0, 0.0);

    let north = compute_hud_cue(obs, (0.0, 0.0, 100.0));
    assert!((north.bearing_deg - 0.0).abs() < 1e-9);

    let east = compute_hud_cue(obs, (100.0, 0.0, 0.0));
    assert!((east.bearing_deg - 90.0).abs() < 1e-9);

    let south = compute_hud_cue(obs, (0.0, 0.0, -100.0));
    assert!((south.bearing_deg - 180.0).abs() < 1e-9);

    let west = compute_hud_cue(obs, (-100.0, 0.0, 0.0));
    assert!((west.bearing_deg - 270.0).abs() < 1e-9);
}
