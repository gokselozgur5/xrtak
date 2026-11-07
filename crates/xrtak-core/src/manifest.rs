//! Copernicus GLO-30 DEM Cell Planner & Offline Mapset Manifest Generator (`REQ-MAN-01..02`, `ICD-18`).
//!
//! Computes 1°×1° Copernicus 30m Global Digital Elevation Model (GLO-30 COG) tile identifiers
//! and AWS Open Data S3 URLs intersecting any WGS84 bounding box, estimates multi-zoom Web Mercator
//! imagery tile pyramids and storage budgets, and produces a deterministic content-addressed
//! JSON manifest (`manifest_hash`) for air-gapped tactical field deployment.

use crate::tiles::{latlon_to_tile_xy, ImageryProvider};
use serde::{Deserialize, Serialize};

/// A single 1°×1° Copernicus GLO-30 (30m resolution) Cloud-Optimized GeoTIFF DEM cell (`REQ-MAN-01`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CopernicusDemCell {
    /// Official Copernicus GLO-30 COG identifier, e.g. `Copernicus_DSM_COG_10_N39_00_E032_00_DEM`
    pub cell_id: String,
    /// Southwest corner latitude floor (degrees, -90..=89)
    pub lat_floor_deg: i32,
    /// Southwest corner longitude floor (degrees, -180..=179)
    pub lon_floor_deg: i32,
    /// Public AWS Open Data Registry S3 HTTPS endpoint for direct COG download
    pub s3_url: String,
    /// Typical uncompressed/LZW COG file size estimate in bytes (~28 MB per 1°×1° 30m tile)
    pub estimated_bytes: u64,
}

/// Per-zoom Web Mercator tile pyramid count and byte storage estimate (`REQ-MAN-02`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoomTileEstimate {
    pub zoom: u8,
    pub min_tile_x: u32,
    pub max_tile_x: u32,
    pub min_tile_y: u32,
    pub max_tile_y: u32,
    pub tiles_x: u32,
    pub tiles_y: u32,
    pub total_tiles: u64,
    pub estimated_bytes: u64,
}

/// Complete offline mapset deployment manifest combining imagery pyramid estimates and Copernicus DEM cells (`ICD-18`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapsetManifest {
    pub name: String,
    pub provider: ImageryProvider,
    /// Deterministic 64-bit FNV-1a hex content hash of the canonical bounds, provider, and zoom range
    pub manifest_hash: String,
    /// `[west, south, east, north]` in WGS84 degrees
    pub bounds_wsen: [f64; 4],
    pub min_zoom: u8,
    pub max_zoom: u8,
    pub total_tiles: u64,
    pub total_imagery_bytes: u64,
    pub total_dem_bytes: u64,
    pub total_package_bytes: u64,
    pub zoom_levels: Vec<ZoomTileEstimate>,
    pub copernicus_dem_cells: Vec<CopernicusDemCell>,
}

/// Format a 1°×1° Southwest integer coordinate `(lat_floor, lon_floor)` into the official
/// Copernicus GLO-30 30m COG naming convention (`Copernicus_DSM_COG_10_{N|S}{lat:02}_00_{E|W}{lon:03}_00_DEM`).
pub fn format_copernicus_glo30_cell_id(lat_floor_deg: i32, lon_floor_deg: i32) -> String {
    let ns = if lat_floor_deg >= 0 { 'N' } else { 'S' };
    let ew = if lon_floor_deg >= 0 { 'E' } else { 'W' };
    let lat_abs = lat_floor_deg.unsigned_abs();
    let lon_abs = lon_floor_deg.unsigned_abs();
    format!("Copernicus_DSM_COG_10_{ns}{lat_abs:02}_00_{ew}{lon_abs:03}_00_DEM")
}

/// Compute all 1°×1° Copernicus GLO-30 DEM cells intersecting the WGS84 bounding box `(south, west, north, east)`.
pub fn copernicus_glo30_cells_for_bounds(
    south_deg: f64,
    west_deg: f64,
    north_deg: f64,
    east_deg: f64,
) -> Vec<CopernicusDemCell> {
    let s = south_deg.min(north_deg).clamp(-89.999, 89.999);
    let n = south_deg.max(north_deg).clamp(-89.999, 89.999);
    let w = west_deg.min(east_deg).clamp(-179.999, 179.999);
    let e = west_deg.max(east_deg).clamp(-179.999, 179.999);

    let min_lat = s.floor() as i32;
    let max_lat = (n - 1e-9).max(s).floor() as i32;
    let min_lon = w.floor() as i32;
    let max_lon = (e - 1e-9).max(w).floor() as i32;

    let mut cells = Vec::new();
    for lat_floor in min_lat..=max_lat {
        for lon_floor in min_lon..=max_lon {
            let cell_id = format_copernicus_glo30_cell_id(lat_floor, lon_floor);
            let s3_url = format!(
                "https://copernicus-dem-30m.s3.amazonaws.com/{cell_id}/{cell_id}.tif"
            );
            cells.push(CopernicusDemCell {
                cell_id,
                lat_floor_deg: lat_floor,
                lon_floor_deg: lon_floor,
                s3_url,
                estimated_bytes: 28_500_000,
            });
        }
    }
    cells
}

/// Compute Web Mercator tile range and estimated byte size for a single zoom level (`REQ-MAN-02`).
pub fn estimate_tiles_for_bounds(
    south_deg: f64,
    west_deg: f64,
    north_deg: f64,
    east_deg: f64,
    zoom: u8,
) -> ZoomTileEstimate {
    let s = south_deg.min(north_deg);
    let n = south_deg.max(north_deg);
    let w = west_deg.min(east_deg);
    let e = west_deg.max(east_deg);

    // Note: in XYZ/Slippy tiles, north has smaller Y index than south
    let (x0, y_north) = latlon_to_tile_xy(n, w, zoom);
    let (x1, y_south) = latlon_to_tile_xy(s, e, zoom);

    let min_tile_x = x0.min(x1);
    let max_tile_x = x0.max(x1);
    let min_tile_y = y_north.min(y_south);
    let max_tile_y = y_north.max(y_south);

    let tiles_x = max_tile_x - min_tile_x + 1;
    let tiles_y = max_tile_y - min_tile_y + 1;
    let total_tiles = (tiles_x as u64) * (tiles_y as u64);
    // Average compressed 256x256 JPEG/WebP satellite tile ~ 24 KB
    let estimated_bytes = total_tiles * 24_576;

    ZoomTileEstimate {
        zoom,
        min_tile_x,
        max_tile_x,
        min_tile_y,
        max_tile_y,
        tiles_x,
        tiles_y,
        total_tiles,
        estimated_bytes,
    }
}

/// Compute a deterministic 64-bit FNV-1a hex digest over canonical manifest parameters.
fn compute_manifest_fnv1a_hex(
    provider: ImageryProvider,
    south: f64,
    west: f64,
    north: f64,
    east: f64,
    min_zoom: u8,
    max_zoom: u8,
) -> String {
    let canonical = format!(
        "xrtak-mapset-v1|{:?}|{:.6}|{:.6}|{:.6}|{:.6}|z{}-{}",
        provider, south, west, north, east, min_zoom, max_zoom
    );
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in canonical.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

/// Build an offline mapset manifest (`MapsetManifest`) for the given WGS84 bounding box and zoom range (`ICD-18`).
pub fn build_mapset_manifest(
    name: &str,
    provider: ImageryProvider,
    south_deg: f64,
    west_deg: f64,
    north_deg: f64,
    east_deg: f64,
    min_zoom: u8,
    max_zoom: u8,
) -> MapsetManifest {
    let s = south_deg.min(north_deg);
    let n = south_deg.max(north_deg);
    let w = west_deg.min(east_deg);
    let e = west_deg.max(east_deg);
    let z_min = min_zoom.min(max_zoom).clamp(1, 20);
    let z_max = min_zoom.max(max_zoom).clamp(1, 20);

    let mut zoom_levels = Vec::with_capacity((z_max - z_min + 1) as usize);
    let mut total_tiles = 0u64;
    let mut total_imagery_bytes = 0u64;

    for z in z_min..=z_max {
        let est = estimate_tiles_for_bounds(s, w, n, e, z);
        total_tiles += est.total_tiles;
        total_imagery_bytes += est.estimated_bytes;
        zoom_levels.push(est);
    }

    let copernicus_dem_cells = copernicus_glo30_cells_for_bounds(s, w, n, e);
    let total_dem_bytes: u64 = copernicus_dem_cells.iter().map(|c| c.estimated_bytes).sum();
    let total_package_bytes = total_imagery_bytes + total_dem_bytes;
    let manifest_hash = compute_manifest_fnv1a_hex(provider, s, w, n, e, z_min, z_max);

    MapsetManifest {
        name: if name.trim().is_empty() {
            format!("XRTAK-MAPSET-{}", &manifest_hash[..8].to_uppercase())
        } else {
            name.trim().to_string()
        },
        provider,
        manifest_hash,
        bounds_wsen: [w, s, e, n],
        min_zoom: z_min,
        max_zoom: z_max,
        total_tiles,
        total_imagery_bytes,
        total_dem_bytes,
        total_package_bytes,
        zoom_levels,
        copernicus_dem_cells,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_copernicus_glo30_cell_naming_and_s3_urls() {
        // Check Northern/Eastern hemisphere (Ankara 39.92N, 32.85E)
        let cells_ne = copernicus_glo30_cells_for_bounds(39.85, 32.75, 40.05, 33.05);
        // Spans lat 39 and 40, lon 32 and 33 -> 2x2 = 4 cells
        assert_eq!(cells_ne.len(), 4);
        assert_eq!(
            cells_ne[0].cell_id,
            "Copernicus_DSM_COG_10_N39_00_E032_00_DEM"
        );
        assert_eq!(
            cells_ne[0].s3_url,
            "https://copernicus-dem-30m.s3.amazonaws.com/Copernicus_DSM_COG_10_N39_00_E032_00_DEM/Copernicus_DSM_COG_10_N39_00_E032_00_DEM.tif"
        );

        // Check Southern/Western hemisphere (e.g. -12.4S, -77.1W -> S13, W078 floor)
        let cells_sw = copernicus_glo30_cells_for_bounds(-12.4, -77.1, -12.2, -77.05);
        assert_eq!(cells_sw.len(), 1);
        assert_eq!(cells_sw[0].lat_floor_deg, -13);
        assert_eq!(cells_sw[0].lon_floor_deg, -78);
        assert_eq!(
            cells_sw[0].cell_id,
            "Copernicus_DSM_COG_10_S13_00_W078_00_DEM"
        );
    }

    #[test]
    fn test_mapset_manifest_tile_counts_and_hash() {
        let m1 = build_mapset_manifest(
            "ANKARA-AO-SECTOR",
            ImageryProvider::BingAerial,
            39.90,
            32.82,
            39.94,
            32.88,
            12,
            15,
        );
        let m2 = build_mapset_manifest(
            "ANKARA-AO-SECTOR",
            ImageryProvider::BingAerial,
            39.90,
            32.82,
            39.94,
            32.88,
            12,
            15,
        );
        assert_eq!(m1.manifest_hash, m2.manifest_hash);
        assert_eq!(m1.zoom_levels.len(), 4);
        assert!(m1.total_tiles >= 4);
        assert_eq!(m1.copernicus_dem_cells.len(), 1);
        assert!(m1.total_package_bytes > m1.total_imagery_bytes);
    }
}
