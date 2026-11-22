use xrtak_core::geotiff_bounds::parse_geotiff_model_transform_4x4;

#[test]
fn test_geotiff_model_transform_4x4_accepts_valid_rotated_matrix() {
    let rotated = [
        0.0, 1.0, 0.0, 10.0, 1.0, 0.0, 0.0, 20.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];
    let tf = parse_geotiff_model_transform_4x4(&rotated)
        .expect("valid rotated matrix should be accepted");
    assert_eq!(tf.a, 0.0);
    assert_eq!(tf.b, 1.0);
    assert_eq!(tf.c, 10.0);
    assert_eq!(tf.d, 1.0);
    assert_eq!(tf.e, 0.0);
    assert_eq!(tf.f, 20.0);
}

#[test]
fn test_geotiff_model_transform_4x4_rejects_singular_matrix() {
    let singular = [
        0.0, 0.0, 0.0, 10.0, 0.0, 1.0, 0.0, 20.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];
    assert!(parse_geotiff_model_transform_4x4(&singular).is_err());
}
