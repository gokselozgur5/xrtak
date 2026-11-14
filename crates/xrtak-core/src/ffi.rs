use crate::geo::{ElevationProvider, ProceduralReliefModel};
use crate::GeoOrigin;

/// C-ABI terrain elevation query for native OpenXR / Godot / JNI consumers.
#[no_mangle]
pub extern "C" fn xrtak_sample_relief_m(east_m: f64, north_m: f64) -> f64 {
    ProceduralReliefModel.sample_relief_m(east_m, north_m)
}

/// C-ABI WGS84 to local ENU projection writing `[east_m, up_m, north_m]` into `out_enu`.
///
/// # Safety
/// `out_enu` must point to a valid mutable array of at least 3 `f64` elements.
#[no_mangle]
pub unsafe extern "C" fn xrtak_lla_to_enu(lat: f64, lon: f64, hae_m: f64, out_enu: *mut f64) {
    if out_enu.is_null() {
        return;
    }
    let (e, u, n) = GeoOrigin::default().lla_to_enu(lat, lon, hae_m);
    unsafe {
        *out_enu.add(0) = e;
        *out_enu.add(1) = u;
        *out_enu.add(2) = n;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_c_abi_ffi_projection_and_relief() {
        let h = xrtak_sample_relief_m(120.0, -80.0);
        assert!(h > 10.0);
        let mut enu = [0.0_f64; 3];
        unsafe { xrtak_lla_to_enu(39.9210, 32.8510, 950.0, enu.as_mut_ptr()) };
        assert!(enu[0] > 50.0 && enu[2] > 50.0);
    }
}
