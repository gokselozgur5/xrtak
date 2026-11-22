use crate::geo::{ElevationProvider, ProceduralReliefModel};
use crate::GeoOrigin;

pub const XRTAK_OK: i32 = 0;
pub const XRTAK_ERR_NULL: i32 = -1;
pub const XRTAK_ERR_INVALID: i32 = -2;

/// C-ABI terrain elevation query for native OpenXR / Godot / JNI consumers.
#[no_mangle]
pub extern "C" fn xrtak_sample_relief_m(east_m: f64, north_m: f64) -> f64 {
    if !east_m.is_finite() || !north_m.is_finite() {
        return f64::NAN;
    }
    ProceduralReliefModel.sample_relief_m(east_m, north_m)
}

/// C-ABI WGS84 to local ENU projection writing `[east_m, up_m, north_m]` into `out_enu`.
///
/// # Safety
/// `out_enu` must be null or point to a valid mutable array of at least 3 `f64` elements.
#[no_mangle]
pub unsafe extern "C" fn xrtak_lla_to_enu(
    origin_lat: f64,
    origin_lon: f64,
    origin_alt_m: f64,
    lat: f64,
    lon: f64,
    hae_m: f64,
    out_enu: *mut f64,
) -> i32 {
    if out_enu.is_null() {
        return XRTAK_ERR_NULL;
    }
    let coords_ok = [origin_lat, origin_lon, origin_alt_m, lat, lon, hae_m]
        .iter()
        .all(|v| v.is_finite())
        && (-90.0..=90.0).contains(&origin_lat)
        && (-90.0..=90.0).contains(&lat)
        && (-180.0..=180.0).contains(&origin_lon)
        && (-180.0..=180.0).contains(&lon);
    if !coords_ok {
        return XRTAK_ERR_INVALID;
    }
    let origin = GeoOrigin {
        lat: origin_lat,
        lon: origin_lon,
        alt_m: origin_alt_m,
        extent_m: 1500.0,
    };
    let (e, u, n) = origin.lla_to_enu(lat, lon, hae_m);
    unsafe {
        *out_enu.add(0) = e;
        *out_enu.add(1) = u;
        *out_enu.add(2) = n;
    }
    XRTAK_OK
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_abi_ffi_projection_and_relief() {
        let h = xrtak_sample_relief_m(120.0, -80.0);
        assert!(h > 10.0);
        let mut enu = [0.0_f64; 3];
        let (olat, olon, oalt) = (39.9200, 32.8500, 850.0);
        let rc = unsafe {
            xrtak_lla_to_enu(olat, olon, oalt, 39.9210, 32.8510, 950.0, enu.as_mut_ptr())
        };
        assert_eq!(rc, XRTAK_OK);
        assert!(enu[0] > 50.0 && enu[2] > 50.0);
    }
}
