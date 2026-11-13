use xrtak_core::fires::*;
use xrtak_core::geo::ElevationProvider;

struct FlatTerrain;
impl ElevationProvider for FlatTerrain {
    fn sample_relief_m(&self, _east: f64, _north: f64) -> f64 {
        0.0
    }
}

struct HighRidge {
    elevation: f64,
}
impl ElevationProvider for HighRidge {
    fn sample_relief_m(&self, _east: f64, _north: f64) -> f64 {
        self.elevation
    }
}
