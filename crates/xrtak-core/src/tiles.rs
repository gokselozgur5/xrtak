use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageryProvider {
    EsriWorldImagery,
    BingAerial,
    Sentinel2Cloudless,
    OpenTopoMap,
}

impl ImageryProvider {
    pub fn from_id(id: &str) -> Self {
        match id.trim().to_ascii_lowercase().as_str() {
            "bing" | "bing-aerial" => Self::BingAerial,
            "sentinel2" | "sentinel-2" | "s2" => Self::Sentinel2Cloudless,
            "topo" | "opentopo" => Self::OpenTopoMap,
            _ => Self::EsriWorldImagery,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::EsriWorldImagery => "esri",
            Self::BingAerial => "bing",
            Self::Sentinel2Cloudless => "sentinel2",
            Self::OpenTopoMap => "topo",
        }
    }

    pub fn tile_url(self, tile_x: u32, tile_y: u32, zoom: u8) -> String {
        match self {
            Self::EsriWorldImagery => format!(
                "https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{zoom}/{tile_y}/{tile_x}"
            ),
            Self::BingAerial => {
                let qk = tile_xy_to_quadkey(tile_x, tile_y, zoom);
                let sub = bing_subdomain(tile_x, tile_y);
                format!("https://ecn.t{sub}.tiles.virtualearth.net/tiles/a{qk}.jpeg?g=1")
            }
            Self::Sentinel2Cloudless => format!(
                "https://tiles.maps.eox.at/wmts/1.0.0/s2cloudless-2024_3857/default/g/{zoom}/{tile_y}/{tile_x}.jpg"
            ),
            Self::OpenTopoMap => format!("https://tile.opentopomap.org/{zoom}/{tile_x}/{tile_y}.png"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedTileSpec {
    pub provider: String,
    pub zoom: u8,
    pub tile_x: u32,
    pub tile_y: u32,
    pub quadkey: String,
    pub primary_url: String,
    /// Lower-zoom (`z=13`) ocean/offshore fallback tile URL when `X-VE-Tile-Info: no-tile` or placeholder is detected (`REQ-MAP-02`)
    pub fallback_zoom: u8,
    pub fallback_tile_x: u32,
    pub fallback_tile_y: u32,
    pub fallback_url: String,
}

/// Converts WGS84 `(lat_deg, lon_deg)` at `zoom` into Web Mercator slippy tile indices `(tile_x, tile_y)` (`REQ-MAP-01`).
pub fn latlon_to_tile_xy(lat_deg: f64, lon_deg: f64, zoom: u8) -> (u32, u32) {
    let lat = lat_deg.clamp(-85.051_128_78, 85.051_128_78);
    let lon = lon_deg.clamp(-180.0, 180.0);
    let n = (1u32 << zoom.min(23)) as f64;
    let x = ((lon + 180.0) / 360.0 * n).floor().clamp(0.0, n - 1.0) as u32;
    let lat_rad = lat.to_radians();
    let y = ((1.0 - (lat_rad.tan() + 1.0 / lat_rad.cos()).ln() / PI) * 0.5 * n)
        .floor()
        .clamp(0.0, n - 1.0) as u32;
    (x, y)
}

/// Encodes Web Mercator `(tile_x, tile_y, zoom)` into a Bing Maps Quadkey string (`0`..`3`).
pub fn tile_xy_to_quadkey(tile_x: u32, tile_y: u32, zoom: u8) -> String {
    let z = zoom.min(23);
    let mut qk = String::with_capacity(z as usize);
    for i in (1..=z).rev() {
        let mask = 1u32 << (i - 1);
        let mut digit = b'0';
        if (tile_x & mask) != 0 {
            digit += 1;
        }
        if (tile_y & mask) != 0 {
            digit += 2;
        }
        qk.push(char::from(digit));
    }
    qk
}

pub const fn bing_subdomain(tile_x: u32, tile_y: u32) -> u32 {
    (tile_x.wrapping_add(tile_y)) & 3
}

/// Computes parent tile coordinates at `target_zoom <= zoom` by bit-shifting (`REQ-MAP-02`).
pub const fn parent_tile_xy(tile_x: u32, tile_y: u32, zoom: u8, target_zoom: u8) -> (u32, u32, u8) {
    if target_zoom >= zoom {
        return (tile_x, tile_y, zoom);
    }
    let shift = zoom - target_zoom;
    (tile_x >> shift, tile_y >> shift, target_zoom)
}

/// Resolves both the primary high-zoom tile and the offshore `z=13` parent fallback tile.
pub fn resolve_tile_with_fallback(
    provider: ImageryProvider,
    lat_deg: f64,
    lon_deg: f64,
    zoom: u8,
) -> ResolvedTileSpec {
    let z = zoom.clamp(1, 20);
    let (tx, ty) = latlon_to_tile_xy(lat_deg, lon_deg, z);
    let qk = tile_xy_to_quadkey(tx, ty, z);
    let primary_url = provider.tile_url(tx, ty, z);

    let fb_z = z.min(13);
    let (fb_x, fb_y, _) = parent_tile_xy(tx, ty, z, fb_z);
    let fallback_url = provider.tile_url(fb_x, fb_y, fb_z);

    ResolvedTileSpec {
        provider: provider.as_str().to_string(),
        zoom: z,
        tile_x: tx,
        tile_y: ty,
        quadkey: qk,
        primary_url,
        fallback_zoom: fb_z,
        fallback_tile_x: fb_x,
        fallback_tile_y: fb_y,
        fallback_url,
    }
}

/// Detects uniform light-beige / low-variance `no-tile` placeholder imagery returned by
/// aerial tile servers over open sea at high zoom levels (`REQ-MAP-02`).
pub fn is_placeholder_rgb_tile(rgb_bytes: &[u8]) -> bool {
    let pixels = rgb_bytes.len() / 3;
    if pixels == 0 {
        return true;
    }
    let mut min_r = 255u8;
    let mut max_r = 0u8;
    let mut min_g = 255u8;
    let mut max_g = 0u8;
    let mut min_b = 255u8;
    let mut max_b = 0u8;
    let mut sum_r = 0u64;
    let mut sum_g = 0u64;
    let mut sum_b = 0u64;
    let mut sumsq_r = 0u64;
    let mut sumsq_g = 0u64;
    let mut sumsq_b = 0u64;

    for px in rgb_bytes.chunks_exact(3) {
        let (r, g, b) = (px[0], px[1], px[2]);
        min_r = min_r.min(r);
        max_r = max_r.max(r);
        min_g = min_g.min(g);
        max_g = max_g.max(g);
        min_b = min_b.min(b);
        max_b = max_b.max(b);
        let (r64, g64, b64) = (u64::from(r), u64::from(g), u64::from(b));
        sum_r += r64;
        sum_g += g64;
        sum_b += b64;
        sumsq_r += r64 * r64;
        sumsq_g += g64 * g64;
        sumsq_b += b64 * b64;
    }

    let n = pixels as f64;
    let (mr, mg, mb) = (sum_r as f64 / n, sum_g as f64 / n, sum_b as f64 / n);
    let vr = (sumsq_r as f64 / n) - mr * mr;
    let vg = (sumsq_g as f64 / n) - mg * mg;
    let vb = (sumsq_b as f64 / n) - mb * mb;

    let low_var = vr <= 36.0 && vg <= 36.0 && vb <= 36.0;
    let low_range =
        max_r.saturating_sub(min_r) <= 50 && max_g.saturating_sub(min_g) <= 50 && max_b.saturating_sub(min_b) <= 50;
    let is_beige_light = mr >= 220.0 && mg >= 220.0 && mb >= 220.0;

    low_var && low_range && is_beige_light
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latlon_to_tile_and_bing_quadkey() {
        // Cardigan Bay / Llanbedr offshore test point (52.725N, -4.216W) at zoom 18
        let spec = resolve_tile_with_fallback(ImageryProvider::BingAerial, 52.725, -4.216, 18);
        assert_eq!(spec.tile_x, 128002);
        assert_eq!(spec.tile_y, 85725);
        assert_eq!(spec.quadkey, "031311232022022212");
        assert!(spec.primary_url.contains("a031311232022022212.jpeg"));

        // Parent fallback at zoom 13 where global ocean imagery exists
        assert_eq!(spec.fallback_zoom, 13);
        assert_eq!(spec.fallback_tile_x, 4000);
        assert_eq!(spec.fallback_tile_y, 2678);
        assert!(spec.fallback_url.contains("a0313112320220.jpeg"));
    }

    #[test]
    fn test_offshore_placeholder_detection() {
        // Uniform beige (#F5F3EF = 245, 243, 239) placeholder tile
        let mut beige = Vec::with_capacity(64 * 3);
        for _ in 0..64 {
            beige.extend_from_slice(&[245, 243, 239]);
        }
        assert!(is_placeholder_rgb_rgb(&beige));

        // Real deep blue sea / coastal variation tile
        let mut sea = Vec::with_capacity(64 * 3);
        for i in 0..64u8 {
            sea.extend_from_slice(&[12, 45 + (i % 20), 95 + (i % 30)]);
        }
        assert!(!is_placeholder_rgb_tile(&sea));
    }

    fn is_placeholder_rgb_rgb(b: &[u8]) -> bool {
        is_placeholder_rgb_tile(b)
    }
}
