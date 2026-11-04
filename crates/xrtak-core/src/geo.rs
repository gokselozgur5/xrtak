/// Abstraction for digital terrain elevation models (DTED, DSM, or procedural relief).
/// Satisfies the Dependency Inversion Principle (DIP) and Open/Closed Principle (OCP).
pub trait ElevationProvider: Send + Sync {
    fn sample_relief_m(&self, east_m: f64, north_m: f64) -> f64;
}

/// Default deterministic procedural topographic relief model.
#[derive(Debug, Default, Clone, Copy)]
pub struct ProceduralReliefModel;

impl ElevationProvider for ProceduralReliefModel {
    fn sample_relief_m(&self, east_m: f64, north_m: f64) -> f64 {
        sample_terrain_relief_m(east_m, north_m)
    }
}

/// Deterministic procedural topographic height function (shared logic with frontend).
pub fn sample_terrain_relief_m(east_m: f64, north_m: f64) -> f64 {
    let nx = east_m * 0.0018;
    let nz = north_m * 0.0018;

    let ridge1 = ((nx * 1.3 + nz * 0.8).sin() * (nx * 0.7 - nz * 1.1).cos()) * 115.0;
    let ridge2 = ((nx * 2.7 - nz * 2.1).sin() * 0.5 + (nz * 3.1).cos() * 0.5) * 42.0;
    let detail = (nx * 5.5 + nz * 4.8).sin() * 12.0;
    let valley_dist = ((nx - nz * 0.6).abs()).min(1.5);
    let valley_factor = 0.35 + 0.65 * (valley_dist / 1.5);

    (ridge1 + ridge2 + detail + 95.0) * valley_factor
}
