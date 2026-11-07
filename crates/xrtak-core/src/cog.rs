use crate::geo::RasterDemProvider;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoTiffDsmResult {
    pub metadata: GeoTiffMetadata,
    pub center_lat_lon: Option<(f64, f64)>,
    pub min_asl_m: f64,
    pub max_asl_m: f64,
    pub dem: RasterDemProvider,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoTiffMetadata {
    pub width: usize,
    pub height: usize,
    pub bits_per_sample: u16,
    /// 1 = uint, 2 = int, 3 = IEEE float
    pub sample_format: u16,
    pub data_offsets: Vec<usize>,
    /// Optional geographic top-left origin (`lon, lat`) from `ModelTiepointTag` (33922)
    pub origin_lon_lat: Option<(f64, f64)>,
    /// Optional pixel scale (`scale_x, scale_y`) from `ModelPixelScaleTag` (33550)
    pub pixel_scale_xy: Option<(f64, f64)>,
}

#[derive(Clone, Copy)]
enum Endian {
    Little,
    Big,
}

impl Endian {
    fn u16(self, b: &[u8], off: usize) -> Option<u16> {
        let s = b.get(off..off + 2)?;
        Some(match self {
            Self::Little => u16::from_le_bytes([s[0], s[1]]),
            Self::Big => u16::from_be_bytes([s[0], s[1]]),
        })
    }

    fn u32(self, b: &[u8], off: usize) -> Option<u32> {
        let s = b.get(off..off + 4)?;
        Some(match self {
            Self::Little => u32::from_le_bytes([s[0], s[1], s[2], s[3]]),
            Self::Big => u32::from_be_bytes([s[0], s[1], s[2], s[3]]),
        })
    }

    fn i16(self, b: &[u8], off: usize) -> Option<i16> {
        let s = b.get(off..off + 2)?;
        Some(match self {
            Self::Little => i16::from_le_bytes([s[0], s[1]]),
            Self::Big => i16::from_be_bytes([s[0], s[1]]),
        })
    }

    fn f32(self, b: &[u8], off: usize) -> Option<f32> {
        let s = b.get(off..off + 4)?;
        Some(match self {
            Self::Little => f32::from_le_bytes([s[0], s[1], s[2], s[3]]),
            Self::Big => f32::from_be_bytes([s[0], s[1], s[2], s[3]]),
        })
    }

    fn f64(self, b: &[u8], off: usize) -> Option<f64> {
        let s = b.get(off..off + 8)?;
        Some(match self {
            Self::Little => f64::from_le_bytes([s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7]]),
            Self::Big => f64::from_be_bytes([s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7]]),
        })
    }
}

/// Parses the primary IFD of a GeoTIFF / Cloud-Optimized GeoTIFF (`REQ-COG-01`).
pub fn parse_geotiff_metadata(bytes: &[u8]) -> Result<GeoTiffMetadata, String> {
    if bytes.len() < 8 {
        return Err("GeoTIFF buffer shorter than 8-byte TIFF header".to_string());
    }
    let endian = match (&bytes[0..2], bytes[2], bytes[3]) {
        (b"II", 42, 0) => Endian::Little,
        (b"MM", 0, 42) => Endian::Big,
        _ => return Err("Invalid TIFF byte-order or magic number (expected II/MM + 42)".to_string()),
    };

    let ifd_offset = endian
        .u32(bytes, 4)
        .ok_or_else(|| "Truncated IFD offset".to_string())? as usize;
    let entry_count = endian
        .u16(bytes, ifd_offset)
        .ok_or_else(|| "Truncated IFD entry count".to_string())? as usize;

    let mut width = 0usize;
    let mut height = 0usize;
    let mut bits_per_sample = 16u16;
    let mut sample_format = 1u16;
    let mut data_offsets = Vec::new();
    let mut origin_lon_lat = None;
    let mut pixel_scale_xy = None;

    for i in 0..entry_count {
        let base = ifd_offset + 2 + i * 12;
        let tag = endian.u16(bytes, base).ok_or("Truncated IFD tag")?;
        let field_type = endian.u16(bytes, base + 2).ok_or("Truncated IFD type")?;
        let count = endian.u32(bytes, base + 4).ok_or("Truncated IFD count")? as usize;
        let val_off = base + 8;

        let read_scalar_u32 = || -> Option<u32> {
            match field_type {
                3 => endian.u16(bytes, val_off).map(u32::from),
                4 => endian.u32(bytes, val_off),
                _ => endian.u32(bytes, val_off),
            }
        };

        match tag {
            256 => width = read_scalar_u32().unwrap_or(0) as usize,
            257 => height = read_scalar_u32().unwrap_or(0) as usize,
            258 => bits_per_sample = read_scalar_u32().unwrap_or(16) as u16,
            339 => sample_format = read_scalar_u32().unwrap_or(1) as u16,
            273 | 324 => {
                if count == 1 {
                    if let Some(off) = read_scalar_u32() {
                        data_offsets.push(off as usize);
                    }
                } else if let Some(arr_off) = endian.u32(bytes, val_off) {
                    let arr_base = arr_off as usize;
                    for k in 0..count {
                        let item = if field_type == 3 {
                            endian.u16(bytes, arr_base + k * 2).map(|v| v as usize)
                        } else {
                            endian.u32(bytes, arr_base + k * 4).map(|v| v as usize)
                        };
                        if let Some(off) = item {
                            data_offsets.push(off);
                        }
                    }
                }
            }
            33550 if count >= 2 => {
                if let Some(arr_off) = endian.u32(bytes, val_off) {
                    let b_off = arr_off as usize;
                    if let (Some(sx), Some(sy)) = (endian.f64(bytes, b_off), endian.f64(bytes, b_off + 8)) {
                        pixel_scale_xy = Some((sx, sy));
                    }
                }
            }
            33922 if count >= 6 => {
                if let Some(arr_off) = endian.u32(bytes, val_off) {
                    let b_off = arr_off as usize;
                    if let (Some(lon), Some(lat)) =
                        (endian.f64(bytes, b_off + 24), endian.f64(bytes, b_off + 32))
                    {
                        origin_lon_lat = Some((lon, lat));
                    }
                }
            }
            _ => {}
        }
    }

    if width == 0 || height == 0 || data_offsets.is_empty() {
        return Err("GeoTIFF missing required ImageWidth, ImageLength, or Strip/TileOffsets".to_string());
    }

    Ok(GeoTiffMetadata {
        width,
        height,
        bits_per_sample,
        sample_format,
        data_offsets,
        origin_lon_lat,
        pixel_scale_xy,
    })
}

/// Decodes uncompressed `Float32` (32-bit IEEE) or `Int16` (16-bit signed/unsigned) GeoTIFF DSM
/// elevation rasters, filters void samples (`<= -9999`), and resamples into a `RasterDemProvider` (`REQ-COG-01`).
pub fn ingest_geotiff_dsm(
    name: impl Into<String>,
    extent_m: f64,
    target_res: usize,
    bytes: &[u8],
) -> Result<GeoTiffDsmResult, String> {
    let meta = parse_geotiff_metadata(bytes)?;
    let endian = if &bytes[0..2] == b"II" {
        Endian::Little
    } else {
        Endian::Big
    };

    let bytes_per_sample = match meta.bits_per_sample {
        32 => 4usize,
        16 => 2usize,
        other => {
            return Err(format!(
                "Unsupported GeoTIFF DSM BitsPerSample={other} (expected 16 or 32)"
            ))
        }
    };

    let total_pixels = meta.width * meta.height;
    let first_offset = meta.data_offsets[0];
    let needed_bytes = total_pixels * bytes_per_sample;
    if first_offset + needed_bytes > bytes.len() {
        return Err("GeoTIFF pixel buffer truncated relative to ImageWidth * ImageLength".to_string());
    }

    let mut raw_asl = Vec::with_capacity(total_pixels);
    let mut min_asl_m = f64::INFINITY;
    let mut max_asl_m = f64::NEG_INFINITY;

    for idx in 0..total_pixels {
        let off = first_offset + idx * bytes_per_sample;
        let val = if bytes_per_sample == 4 {
            endian.f32(bytes, off).unwrap_or(f32::NAN) as f64
        } else {
            endian.i16(bytes, off).unwrap_or(-32768) as f64
        };
        if val.is_finite() && val > -9999.0 {
            min_asl_m = min_asl_m.min(val);
            max_asl_m = max_asl_m.max(val);
            raw_asl.push(val);
        } else {
            raw_asl.push(f64::NAN);
        }
    }

    if !min_asl_m.is_finite() {
        min_asl_m = 0.0;
        max_asl_m = 0.0;
    }

    let res = target_res.max(2);
    let mut resampled_relief = Vec::with_capacity(res * res);
    // Row 0 in RasterDemProvider is South (`north = -extent_m`), whereas Row 0 in GeoTIFF is North (top-left).
    for r in 0..res {
        let v_south_to_north = r as f64 / (res - 1) as f64;
        let src_row = (((1.0 - v_south_to_north) * (meta.height - 1) as f64).round() as usize)
            .min(meta.height - 1);
        for c in 0..res {
            let u_west_to_east = c as f64 / (res - 1) as f64;
            let src_col =
                ((u_west_to_east * (meta.width - 1) as f64).round() as usize).min(meta.width - 1);
            let sample = raw_asl[src_row * meta.width + src_col];
            let relief_m = if sample.is_finite() {
                (sample - min_asl_m).max(0.0)
            } else {
                0.0
            };
            resampled_relief.push((relief_m * 100.0).round() / 100.0);
        }
    }

    let center_lat_lon = match (meta.origin_lon_lat, meta.pixel_scale_xy) {
        (Some((lon0, lat0)), Some((sx, sy))) if lon0.abs() <= 180.0 && lat0.abs() <= 90.0 => {
            let center_lon = lon0 + (meta.width as f64 * 0.5) * sx;
            let center_lat = lat0 - (meta.height as f64 * 0.5) * sy;
            Some((center_lat, center_lon))
        }
        (Some((lon0, lat0)), None) if lon0.abs() <= 180.0 && lat0.abs() <= 90.0 => {
            Some((lat0, lon0))
        }
        _ => None,
    };

    let dem = RasterDemProvider::new(name, res, res, extent_m, resampled_relief)?;
    Ok(GeoTiffDsmResult {
        metadata: meta,
        center_lat_lon,
        min_asl_m: (min_asl_m * 100.0).round() / 100.0,
        max_asl_m: (max_asl_m * 100.0).round() / 100.0,
        dem,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geo::ElevationProvider;

    fn build_synthetic_f32_geotiff_dsm() -> Vec<u8> {
        // Little-endian Classic TIFF with 2x2 Float32 pixels + ModelPixelScale + ModelTiepoint
        let mut buf = Vec::new();
        buf.extend_from_slice(b"II");
        buf.extend_from_slice(&42u16.to_le_bytes());
        buf.extend_from_slice(&8u32.to_le_bytes()); // IFD at offset 8

        let num_entries = 7u16;
        buf.extend_from_slice(&num_entries.to_le_bytes());

        // IFD ends at 8 + 2 + 7*12 + 4 = 98
        // Scale (3 * f64 = 24 bytes) at offset 100..124
        // Tiepoint (6 * f64 = 48 bytes) at offset 124..172
        // Pixel data (4 * f32 = 16 bytes) at offset 172..188
        let mut write_entry = |tag: u16, ftype: u16, count: u32, val_or_off: u32| {
            buf.extend_from_slice(&tag.to_le_bytes());
            buf.extend_from_slice(&ftype.to_le_bytes());
            buf.extend_from_slice(&count.to_le_bytes());
            buf.extend_from_slice(&val_or_off.to_le_bytes());
        };

        write_entry(256, 3, 1, 2); // ImageWidth = 2
        write_entry(257, 3, 1, 2); // ImageLength = 2
        write_entry(258, 3, 1, 32); // BitsPerSample = 32
        write_entry(273, 4, 1, 172); // StripOffsets = 172
        write_entry(339, 3, 1, 3); // SampleFormat = 3 (IEEE float)
        write_entry(33550, 12, 3, 100); // ModelPixelScaleTag at 100
        write_entry(33922, 12, 6, 124); // ModelTiepointTag at 124

        buf.extend_from_slice(&0u32.to_le_bytes()); // next IFD = 0
        while buf.len() < 100 {
            buf.push(0);
        }

        // ModelPixelScale: (0.01 deg, 0.01 deg, 0.0)
        for v in [0.01f64, 0.01f64, 0.0f64] {
            buf.extend_from_slice(&v.to_le_bytes());
        }
        // ModelTiepoint: (0, 0, 0, lon=-4.12, lat=52.81, 0)
        for v in [0.0f64, 0.0, 0.0, -4.12, 52.81, 0.0] {
            buf.extend_from_slice(&v.to_le_bytes());
        }
        // 2x2 Float32 pixels: North row [910.0, 930.0], South row [850.0, 870.0]
        for p in [910.0f32, 930.0, 850.0, 870.0] {
            buf.extend_from_slice(&p.to_le_bytes());
        }
        buf
    }

    #[test]
    fn test_parse_geotiff_dsm_f32_with_tiepoint() {
        let tiff = build_synthetic_f32_geotiff_dsm();
        let res = ingest_geotiff_dsm("llanbedr-dsm", 1000.0, 2, &tiff).expect("valid GeoTIFF DSM");
        assert_eq!(res.metadata.width, 2);
        assert_eq!(res.metadata.height, 2);
        assert_eq!(res.metadata.bits_per_sample, 32);
        assert!((res.min_asl_m - 850.0).abs() < 1e-6);
        assert!((res.max_asl_m - 930.0).abs() < 1e-6);

        let (clat, clon) = res.center_lat_lon.expect("tiepoint georeference");
        assert!((clat - 52.80).abs() < 0.02);
        assert!((clon - (-4.11)).abs() < 0.02);

        // South-West corner (-1000, -1000) corresponds to South row col 0 = 850m ASL -> 0m relief
        assert!((res.dem.sample_relief_m(-1000.0, -1000.0) - 0.0).abs() < 1e-6);
        // North-East corner (+1000, +1000) corresponds to North row col 1 = 930m ASL -> 80m relief
        assert!((res.dem.sample_relief_m(1000.0, 1000.0) - 80.0).abs() < 1e-6);
    }
}

