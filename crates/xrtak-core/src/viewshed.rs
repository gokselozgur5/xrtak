use crate::domain::{LinkLosResult, ViewshedResult, ViewshedSample};
use crate::geo::ElevationProvider;

/// Compute 3D Line-of-Sight (Viewshed) point cloud over any `ElevationProvider` (DIP).
pub fn compute_viewshed_with_provider(
    provider: &dyn ElevationProvider,
    observer_east_m: f64,
    observer_north_m: f64,
    observer_height_offset_m: f64,
    radius_m: f64,
    num_azimuth_beams: usize,
    points_per_beam: usize,
) -> ViewshedResult {
    let base_ground_m = provider.sample_relief_m(observer_east_m, observer_north_m);
    let observer_up_m = base_ground_m + observer_height_offset_m.max(2.0);
    let mut samples = Vec::with_capacity(num_azimuth_beams * points_per_beam);
    let mut visible_count = 0usize;

    for b in 0..num_azimuth_beams {
        let az_rad = (b as f64 / num_azimuth_beams as f64) * std::f64::consts::TAU;
        let (dir_e, dir_n) = (az_rad.sin(), az_rad.cos());
        let mut max_slope = f64::NEG_INFINITY;

        for s in 1..=points_per_beam {
            let dist = (s as f64 / points_per_beam as f64) * radius_m;
            let (sample_e, sample_n) = (
                observer_east_m + dir_e * dist,
                observer_north_m + dir_n * dist,
            );
            let terrain_up = provider.sample_relief_m(sample_e, sample_n);
            let slope = (terrain_up - observer_up_m) / dist;
            let visible = slope >= max_slope - 1e-6;
            if slope > max_slope {
                max_slope = slope;
            }
            if visible {
                visible_count += 1;
            }
            samples.push(ViewshedSample {
                east_m: sample_e,
                up_m: terrain_up,
                north_m: sample_n,
                visible,
            });
        }
    }

    let total = samples.len().max(1);
    ViewshedResult {
        observer_east_m,
        observer_up_m,
        observer_north_m,
        radius_m,
        visible_ratio: visible_count as f64 / total as f64,
        samples,
    }
}

/// Compute point-to-point 3D Line-of-Sight (intervisibility) between `from_enu` and `to_enu` (`REQ-TAC-01`).
pub fn compute_link_los(
    provider: &dyn ElevationProvider,
    from_enu: (f64, f64, f64),
    to_enu: (f64, f64, f64),
    num_samples: usize,
) -> LinkLosResult {
    let steps = num_samples.max(4);
    let (de, du, dn) = (
        to_enu.0 - from_enu.0,
        to_enu.1 - from_enu.1,
        to_enu.2 - from_enu.2,
    );
    let slant_range_m = (de * de + du * du + dn * dn).sqrt();
    let mut min_clearance_m = f64::INFINITY;
    let mut obstruction_enu: Option<(f64, f64, f64)> = None;

    for k in 1..steps {
        let t = k as f64 / steps as f64;
        let (ray_e, ray_u, ray_n) = (
            from_enu.0 + t * de,
            from_enu.1 + t * du,
            from_enu.2 + t * dn,
        );
        let terrain_u = provider.sample_relief_m(ray_e, ray_n);
        let clearance = ray_u - terrain_u;
        if clearance < min_clearance_m {
            min_clearance_m = clearance;
        }
        if clearance < 0.0 && obstruction_enu.is_none() {
            obstruction_enu = Some((ray_e, terrain_u, ray_n));
        }
    }

    LinkLosResult {
        from_enu,
        to_enu,
        slant_range_m,
        clear: obstruction_enu.is_none(),
        min_clearance_m: if min_clearance_m.is_finite() {
            min_clearance_m
        } else {
            0.0
        },
        obstruction_enu,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct RidgeTerrain;
    impl ElevationProvider for RidgeTerrain {
        fn sample_relief_m(&self, east_m: f64, _north_m: f64) -> f64 {
            if east_m.abs() < 40.0 {
                120.0
            } else {
                20.0
            }
        }
    }

    #[test]
    fn test_compute_link_los_occluded_and_clear() {
        let blocked = compute_link_los(&RidgeTerrain, (-200.0, 50.0, 0.0), (200.0, 50.0, 0.0), 32);
        assert!(
            !blocked.clear && blocked.min_clearance_m < -60.0 && blocked.obstruction_enu.is_some()
        );

        let clear = compute_link_los(&RidgeTerrain, (-200.0, 180.0, 0.0), (200.0, 180.0, 0.0), 32);
        assert!(
            clear.clear
                && (clear.min_clearance_m - 60.0).abs() < 1e-6
                && clear.obstruction_enu.is_none()
        );
    }
}
