/* XRTAK Core C-ABI header. Matches crates/xrtak-core/src/ffi.rs. */
#ifndef XRTAK_H
#define XRTAK_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define XRTAK_OK 0
#define XRTAK_ERR_NULL (-1)
#define XRTAK_ERR_INVALID (-2)

/* Sample procedural terrain relief height (meters) at local ENU (east_m, north_m).
 * Returns NaN if east_m or north_m is non-finite. */
double xrtak_sample_relief_m(double east_m, double north_m);

/* Project WGS84 (lat, lon, hae_m) relative to origin (origin_lat, origin_lon, origin_alt_m)
 * into local ENU meters, writing [east_m, up_m, north_m] into out_enu[0..3].
 * Returns XRTAK_OK (0), XRTAK_ERR_NULL (-1), or XRTAK_ERR_INVALID (-2). */
int32_t xrtak_lla_to_enu(double origin_lat,
                         double origin_lon,
                         double origin_alt_m,
                         double lat,
                         double lon,
                         double hae_m,
                         double *out_enu);

#ifdef __cplusplus
}
#endif

#endif /* XRTAK_H */
