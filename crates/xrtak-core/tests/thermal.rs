use xrtak_core::thermal::{ColorRgb, ThermalPalette, ThermalShaderConfig};

#[test]
fn test_glsl_shader_applies_gain_consistently() {
    let cfg = ThermalShaderConfig {
        gain: 2.0,
        ..Default::default()
    };
    let glsl = cfg.generate_glsl_fragment_shader();
    assert!(
        glsl.contains("float t = clamp(norm * (2.0000), 0.0, 1.0);"),
        "GLSL shader should apply gain directly without uninitialized u_gain multiplication: {glsl}"
    );
    assert!(
        !glsl.contains("u_gain"),
        "GLSL shader should not have redundant u_gain uniform"
    );
}

#[test]
fn test_godot_shader_applies_gain_consistently() {
    let cfg = ThermalShaderConfig {
        gain: 2.5,
        ..Default::default()
    };
    let godot = cfg.generate_godot_shader();
    assert!(
        godot.contains("uniform float gain : hint_range(0.1, 5.0) = 2.5000;"),
        "Godot shader should expose gain uniform with default matching config: {godot}"
    );
    assert!(
        godot.contains("float t = clamp(norm * gain, 0.0, 1.0);"),
        "Godot shader should compute normalized t using gain: {godot}"
    );
}

#[test]
fn test_shader_output_aligns_with_palette_sampling() {
    let cfg = ThermalShaderConfig {
        palette: ThermalPalette::WhiteHot,
        gain: 1.0,
        ..Default::default()
    };
    let glsl = cfg.generate_glsl_fragment_shader();
    assert!(glsl.contains("vec3 col = vec3(t);"));
    let sampled = cfg.palette.sample_color(0.5);
    assert_eq!(sampled, ColorRgb::new(0.5, 0.5, 0.5));
}
