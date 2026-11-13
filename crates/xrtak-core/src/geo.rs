use serde::{Deserialize, Serialize};

/// Abstraction for digital terrain elevation models (DTED, SRTM, raster DEM, or procedural relief).
/// Satisfies the Dependency Inversion Principle (DIP) and Open/Closed Principle (OCP).
pub trait ElevationProvider: Send + Sync {
    fn sample_relief_m(&self, east_m: f64, north_m: f64) -> f64;

    /// Export a row-major (`north` from `-extent_m` to `+extent_m`, `east` from `-extent_m` to `+extent_m`)
    /// elevation grid of size `resolution * resolution` for 3D sand table mesh synchronization.
    fn export_grid(&self, extent_m: f64, resolution: usize) -> Vec<f64> {
        let res = resolution.max(2);
        let mut out = Vec::with_capacity(res * res);
        for row in 0..res {
            let v = row as f64 / (res - 1) as f64;
            let north_m = -extent_m + v * (2.0 * extent_m);
            for col in 0..res {
                let u = col as f64 / (res - 1) as f64;
                let east_m = -extent_m + u * (2.0 * extent_m);
                out.push((self.sample_relief_m(east_m, north_m) * 100.0).round() / 100.0);
            }
        }
        out
    }

    /// Compute local terrain slope angle in degrees (`[0.0, 90.0)`) using central finite differences.
    fn sample_slope_deg(&self, east_m: f64, north_m: f64) -> f64 {
        let step_m = 10.0;
        let dz_de = (self.sample_relief_m(east_m + step_m, north_m)
            - self.sample_relief_m(east_m - step_m, north_m))
            / (2.0 * step_m);
        let dz_dn = (self.sample_relief_m(east_m, north_m + step_m)
            - self.sample_relief_m(east_m, north_m - step_m))
            / (2.0 * step_m);
        let grad = (dz_de * dz_de + dz_dn * dz_dn).sqrt();
        grad.atan().to_degrees()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElevationProfileSample {
    pub dist_m: f64,
    pub east_m: f64,
    pub north_m: f64,
    pub relief_m: f64,
    pub asl_m: f64,
    pub slope_deg: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElevationProfileResult {
    pub total_distance_m: f64,
    pub total_ascent_m: f64,
    pub total_descent_m: f64,
    pub min_asl_m: f64,
    pub max_asl_m: f64,
    pub max_slope_deg: f64,
    pub samples: Vec<ElevationProfileSample>,
}

/// Compute a terrain elevation cross-section profile along a sequence of `(east_m, north_m)` waypoints.
pub fn compute_elevation_profile(
    provider: &dyn ElevationProvider,
    waypoints_en: &[(f64, f64)],
    base_alt_m: f64,
    samples_per_segment: usize,
) -> ElevationProfileResult {
    let steps = samples_per_segment.max(4);
    let mut samples = Vec::new();
    let mut cum_dist_m = 0.0;
    let mut total_ascent_m = 0.0;
    let mut total_descent_m = 0.0;
    let mut min_asl_m = f64::INFINITY;
    let mut max_asl_m = f64::NEG_INFINITY;
    let mut max_slope_deg = 0.0_f64;

    if waypoints_en.is_empty() {
        return ElevationProfileResult {
            total_distance_m: 0.0,
            total_ascent_m: 0.0,
            total_descent_m: 0.0,
            min_asl_m: base_alt_m,
            max_asl_m: base_alt_m,
            max_slope_deg: 0.0,
            samples,
        };
    }

    if waypoints_en.len() == 1 {
        let (e, n) = waypoints_en[0];
        let rel = provider.sample_relief_m(e, n);
        let asl = base_alt_m + rel;
        let slope = provider.sample_slope_deg(e, n);
        samples.push(ElevationProfileSample {
            dist_m: 0.0,
            east_m: e,
            north_m: n,
            relief_m: rel,
            asl_m: asl,
            slope_deg: slope,
        });
        return ElevationProfileResult {
            total_distance_m: 0.0,
            total_ascent_m: 0.0,
            total_descent_m: 0.0,
            min_asl_m: asl,
            max_asl_m: asl,
            max_slope_deg: slope,
            samples,
        };
    }

    let mut prev_asl: Option<f64> = None;
    for seg_idx in 0..(waypoints_en.len() - 1) {
        let (e0, n0) = waypoints_en[seg_idx];
        let (e1, n1) = waypoints_en[seg_idx + 1];
        let seg_len = ((e1 - e0).powi(2) + (n1 - n0).powi(2)).sqrt();
        let start_s = if seg_idx == 0 { 0 } else { 1 };
        for s in start_s..=steps {
            let frac = s as f64 / steps as f64;
            let e = e0 + (e1 - e0) * frac;
            let n = n0 + (n1 - n0) * frac;
            let d = cum_dist_m + seg_len * frac;
            let rel = provider.sample_relief_m(e, n);
            let asl = base_alt_m + rel;
            let slope = provider.sample_slope_deg(e, n);
            if let Some(p_asl) = prev_asl {
                let dz = asl - p_asl;
                if dz > 0.0 {
                    total_ascent_m += dz;
                } else {
                    total_descent_m += -dz;
                }
            }
            prev_asl = Some(asl);
            min_asl_m = min_asl_m.min(asl);
            max_asl_m = max_asl_m.max(asl);
            max_slope_deg = max_slope_deg.max(slope);
            samples.push(ElevationProfileSample {
                dist_m: (d * 10.0).round() / 10.0,
                east_m: (e * 10.0).round() / 10.0,
                north_m: (n * 10.0).round() / 10.0,
                relief_m: (rel * 100.0).round() / 100.0,
                asl_m: (asl * 100.0).round() / 100.0,
                slope_deg: (slope * 10.0).round() / 10.0,
            });
        }
        cum_dist_m += seg_len;
    }

    ElevationProfileResult {
        total_distance_m: (cum_dist_m * 10.0).round() / 10.0,
        total_ascent_m: (total_ascent_m * 10.0).round() / 10.0,
        total_descent_m: (total_descent_m * 10.0).round() / 10.0,
        min_asl_m: (min_asl_m * 10.0).round() / 10.0,
        max_asl_m: (max_asl_m * 10.0).round() / 10.0,
        max_slope_deg: (max_slope_deg * 10.0).round() / 10.0,
        samples,
    }
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

/// Continuous Bilinear Digital Elevation Model (DEM / SRTM / DTED) heightmap provider (`REQ-DEM-02`, `REQ-DEM-03`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RasterDemProvider {
    pub name: String,
    pub width: usize,
    pub height: usize,
    pub extent_m: f64,
    pub heights_m: Vec<f64>,
}

impl RasterDemProvider {
    /// Construct a validated raster DEM provider from a row-major elevation grid (`width * height`).
    pub fn new(
        name: impl Into<String>,
        width: usize,
        height: usize,
        extent_m: f64,
        heights_m: Vec<f64>,
    ) -> Result<Self, String> {
        if width < 2 || height < 2 {
            return Err("RasterDemProvider width and height must be >= 2".to_string());
        }
        if extent_m <= 0.0 {
            return Err("RasterDemProvider extent_m must be > 0".to_string());
        }
        if heights_m.len() != width * height {
            return Err(format!(
                "Expected {} elevation samples ({}x{}), got {}",
                width * height,
                width,
                height,
                heights_m.len()
            ));
        }
        Ok(Self {
            name: name.into(),
            width,
            height,
            extent_m,
            heights_m,
        })
    }

    /// Parse raw 16-bit signed big-endian SRTM `.hgt` / DTED elevation samples (`i16` meters)
    /// and normalize relief relative to the grid's minimum base elevation.
    pub fn from_srtm_i16_be(
        name: impl Into<String>,
        width: usize,
        height: usize,
        extent_m: f64,
        bytes: &[u8],
    ) -> Result<Self, String> {
        if bytes.len() != width * height * 2 {
            return Err(format!(
                "Expected {} bytes for {}x{} i16 SRTM grid, got {}",
                width * height * 2,
                width,
                height,
                bytes.len()
            ));
        }
        let mut raw = Vec::with_capacity(width * height);
        let mut min_valid = f64::INFINITY;
        for chunk in bytes.chunks_exact(2) {
            let val = i16::from_be_bytes([chunk[0], chunk[1]]);
            // SRTM void value is -32768
            if val > -10000 {
                let f = val as f64;
                if f < min_valid {
                    min_valid = f;
                }
                raw.push(f);
            } else {
                raw.push(f64::NAN);
            }
        }
        if !min_valid.is_finite() {
            min_valid = 0.0;
        }
        let heights_m: Vec<f64> = raw
            .into_iter()
            .map(|v| {
                if v.is_finite() {
                    (v - min_valid).max(0.0)
                } else {
                    0.0
                }
            })
            .collect();
        Self::new(name, width, height, extent_m, heights_m)
    }

    /// Build a named tactical DEM preset (`alpine`, `canyon`, `urban_bowl`, or `valley`) at a given grid resolution.
    pub fn from_preset(preset: &str, extent_m: f64, resolution: usize) -> Self {
        let res = resolution.max(16);
        let mut heights = Vec::with_capacity(res * res);
        let preset_id = preset.trim().to_lowercase();

        for row in 0..res {
            let v = row as f64 / (res - 1) as f64;
            let north_m = -extent_m + v * (2.0 * extent_m);
            for col in 0..res {
                let u = col as f64 / (res - 1) as f64;
                let east_m = -extent_m + u * (2.0 * extent_m);
                let nx = east_m / extent_m; // [-1, 1]
                let nz = north_m / extent_m; // [-1, 1]

                let h = match preset_id.as_str() {
                    "alpine" => {
                        // High twin-peak alpine ridge with steep saddle pass
                        let p1 = (-((nx - 0.35).powi(2) + (nz - 0.25).powi(2)) * 4.2).exp() * 235.0;
                        let p2 = (-((nx + 0.42).powi(2) + (nz + 0.30).powi(2)) * 3.8).exp() * 210.0;
                        let spine = (1.0 - (nx * 0.7 - nz * 0.7).abs().min(1.0)).powi(2) * 95.0;
                        let crag = (nx * 9.0).sin() * (nz * 9.0).cos() * 14.0;
                        (p1 + p2 + spine + crag + 18.0).max(5.0)
                    }
                    "canyon" => {
                        // Winding deep gorge cutting through a high desert plateau
                        let meander = (nz * 2.4).sin() * 0.28;
                        let dist_to_gorge = (nx - meander).abs();
                        let gorge_profile = 1.0 / (1.0 + (-((dist_to_gorge - 0.22) * 18.0)).exp());
                        let plateau = 175.0 * gorge_profile;
                        let terraces = ((plateau / 35.0).floor() * 35.0) * 0.35 + plateau * 0.65;
                        let roughness = (nx * 7.5 + nz * 6.0).sin() * 8.0;
                        (terraces + roughness + 14.0).max(4.0)
                    }
                    "urban_bowl" => {
                        // Surrounded caldera / bowl perimeter with central compound terraces
                        let r = (nx * nx + nz * nz).sqrt();
                        let rim = (-((r - 0.78).powi(2)) * 14.0).exp() * 165.0;
                        let central_mound = (-(r * r * 12.0)).exp() * 48.0;
                        let grid_blocks = if r < 0.48
                            && (nx * 14.0).sin().abs() > 0.35
                            && (nz * 14.0).sin().abs() > 0.35
                        {
                            26.0
                        } else {
                            0.0
                        };
                        (rim + central_mound + grid_blocks + 12.0).max(4.0)
                    }
                    _ => sample_terrain_relief_m(east_m, north_m),
                };
                heights.push((h * 100.0).round() / 100.0);
            }
        }

        Self {
            name: preset_id,
            width: res,
            height: res,
            extent_m,
            heights_m: heights,
        }
    }
}

impl ElevationProvider for RasterDemProvider {
    fn sample_relief_m(&self, east_m: f64, north_m: f64) -> f64 {
        let span = (2.0 * self.extent_m).max(1e-6);
        let u_norm = ((east_m + self.extent_m) / span).clamp(0.0, 1.0);
        let v_norm = ((north_m + self.extent_m) / span).clamp(0.0, 1.0);

        let u = u_norm * (self.width - 1) as f64;
        let v = v_norm * (self.height - 1) as f64;

        let i0 = (u.floor() as usize).min(self.width - 1);
        let j0 = (v.floor() as usize).min(self.height - 1);
        let i1 = (i0 + 1).min(self.width - 1);
        let j1 = (j0 + 1).min(self.height - 1);

        let alpha = u - i0 as f64;
        let beta = v - j0 as f64;

        let z00 = self.heights_m[j0 * self.width + i0];
        let z10 = self.heights_m[j0 * self.width + i1];
        let z01 = self.heights_m[j1 * self.width + i0];
        let z11 = self.heights_m[j1 * self.width + i1];

        (1.0 - alpha) * (1.0 - beta) * z00
            + alpha * (1.0 - beta) * z10
            + (1.0 - alpha) * beta * z01
            + alpha * beta * z11
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raster_dem_bilinear_interpolation_and_clamping() {
        // 2x2 grid over [-100, +100] meters:
        // row 0 (south, north = -100): [10.0, 20.0]
        // row 1 (north, north = +100): [30.0, 50.0]
        let dem = RasterDemProvider::new("test-2x2", 2, 2, 100.0, vec![10.0, 20.0, 30.0, 50.0])
            .expect("valid 2x2 DEM");

        assert!((dem.sample_relief_m(-100.0, -100.0) - 10.0).abs() < 1e-9);
        assert!((dem.sample_relief_m(100.0, -100.0) - 20.0).abs() < 1e-9);
        assert!((dem.sample_relief_m(-100.0, 100.0) - 30.0).abs() < 1e-9);
        assert!((dem.sample_relief_m(100.0, 100.0) - 50.0).abs() < 1e-9);

        // Center (0, 0) -> average of 4 corners = (10 + 20 + 30 + 50) / 4 = 27.5
        assert!((dem.sample_relief_m(0.0, 0.0) - 27.5).abs() < 1e-9);

        // Out-of-bounds clamping (REQ-DEM-03)
        assert!((dem.sample_relief_m(9999.0, 9999.0) - 50.0).abs() < 1e-9);
        assert!((dem.sample_relief_m(-9999.0, -9999.0) - 10.0).abs() < 1e-9);
    }

    #[test]
    fn test_srtm_i16_be_ingestion_and_presets() {
        let samples: [i16; 4] = [850, 900, 950, -32768];
        let mut bytes = Vec::new();
        for s in samples {
            bytes.extend_from_slice(&s.to_be_bytes());
        }
        let dem = RasterDemProvider::from_srtm_i16_be("srtm-tile", 2, 2, 500.0, &bytes)
            .expect("valid SRTM buffer");
        assert!((dem.sample_relief_m(-500.0, -500.0) - 0.0).abs() < 1e-6);
        assert!((dem.sample_relief_m(500.0, -500.0) - 50.0).abs() < 1e-6);
        // Void sample (-32768) replaced with 0.0
        assert!((dem.sample_relief_m(500.0, 500.0) - 0.0).abs() < 1e-6);

        let alpine = RasterDemProvider::from_preset("alpine", 1500.0, 32);
        assert!(alpine.sample_relief_m(0.0, 0.0) > 20.0);
    }

    #[test]
    fn test_elevation_profile_and_slope() {
        let dem = RasterDemProvider::new(
            "ramp-2x2",
            2,
            2,
            100.0,
            vec![0.0, 100.0, 0.0, 100.0],
        )
        .expect("valid ramp DEM");

        // Moving 200m east rises 100m -> grade = 0.5 -> atan(0.5) ~= 26.565 deg
        let slope = dem.sample_slope_deg(0.0, 0.0);
        assert!((slope - 26.565).abs() < 0.2);

        let prof = compute_elevation_profile(
            &dem,
            &[(-100.0, 0.0), (100.0, 0.0), (-100.0, 0.0)],
            850.0,
            10,
        );
        assert!((prof.total_distance_m - 400.0).abs() < 1e-6);
        assert!((prof.total_ascent_m - 100.0).abs() < 1e-6);
        assert!((prof.total_descent_m - 100.0).abs() < 1e-6);
        assert!((prof.min_asl_m - 850.0).abs() < 1e-6);
        assert!((prof.max_asl_m - 950.0).abs() < 1e-6);
        assert!(prof.samples.len() >= 21);
    }
}
