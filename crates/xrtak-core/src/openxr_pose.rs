use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PosePosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl PosePosition {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn to_tuple(self) -> (f64, f64, f64) {
        (self.x, self.y, self.z)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PoseOrientation {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

impl PoseOrientation {
    pub const IDENTITY: Self = Self {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 1.0,
    };

    pub const fn new(x: f64, y: f64, z: f64, w: f64) -> Self {
        Self { x, y, z, w }
    }

    pub fn norm(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w).sqrt()
    }
}

    pub fn normalized(&self) -> Self {
        let n = self.norm();
        if n < 1e-12 {
            Self::IDENTITY
        } else {
            Self {
                x: self.x / n,
                y: self.y / n,
                z: self.z / n,
                w: self.w / n,
            }
        }
    }

    pub fn to_euler_deg(&self) -> (f64, f64, f64) {
        let q = self.normalized();
        let sinr_cosp = 2.0 * (q.w * q.x + q.y * q.z);
        let cosr_cosp = 1.0 - 2.0 * (q.x * q.x + q.y * q.y);
        let roll_deg = sinr_cosp.atan2(cosr_cosp).to_degrees();

        let sinp = 2.0 * (q.w * q.y - q.z * q.x);
        let pitch_deg = if sinp.abs() >= 1.0 {
            sinp.signum() * 90.0
        } else {
            sinp.asin().to_degrees()
        };

        let siny_cosp = 2.0 * (q.w * q.z + q.x * q.y);
        let cosy_cosp = 1.0 - 2.0 * (q.y * q.y + q.z * q.z);
        let yaw_deg = siny_cosp.atan2(cosy_cosp).to_degrees().rem_euclid(360.0);

        (yaw_deg, pitch_deg, roll_deg)
    }

    pub fn rotate_vector(&self, v: (f64, f64, f64)) -> (f64, f64, f64) {
        let q = self.normalized();
        let u = (q.x, q.y, q.z);
        let s = q.w;
        let u_cross_v = (
            u.1 * v.2 - u.2 * v.1,
            u.2 * v.0 - u.0 * v.2,
            u.0 * v.1 - u.1 * v.0,
        );
        let u_cross_uv = (
            u.1 * u_cross_v.2 - u.2 * u_cross_v.1,
            u.2 * u_cross_v.0 - u.0 * u_cross_v.2,
            u.0 * u_cross_v.1 - u.1 * u_cross_v.0,
        );
        (
            v.0 + 2.0 * (s * u_cross_v.0 + u_cross_uv.0),
            v.1 + 2.0 * (s * u_cross_v.1 + u_cross_uv.1),
            v.2 + 2.0 * (s * u_cross_v.2 + u_cross_uv.2),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PoseTrackingFlags {
    pub orientation_valid: bool,
    pub position_valid: bool,
    pub orientation_tracked: bool,
    pub position_tracked: bool,
}

impl PoseTrackingFlags {
    pub const FULLY_TRACKED: Self = Self {
        orientation_valid: true,
        position_valid: true,
        orientation_tracked: true,
        position_tracked: true,
    };

    pub fn from_openxr_bits(bits: u64) -> Self {
        Self {
            orientation_valid: (bits & 0x01) != 0,
            position_valid: (bits & 0x02) != 0,
            orientation_tracked: (bits & 0x04) != 0,
            position_tracked: (bits & 0x08) != 0,
        }
    }

    pub fn is_usable(&self) -> bool {
        self.orientation_valid && self.position_valid
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OpenXrPoseSample {
    pub position: PosePosition,
    pub orientation: PoseOrientation,
    pub tracking_flags: PoseTrackingFlags,
    pub timestamp_ns: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TacticalHeadPose {
    pub head_enu: (f64, f64, f64),
    pub forward_enu: (f64, f64, f64),
    pub yaw_deg: f64,
    pub pitch_deg: f64,
    pub roll_deg: f64,
    pub is_valid: bool,
    pub timestamp_unix_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenXrPoseAdapter {
    pub operator_id: String,
    pub callsign: String,
    pub device_mode: String,
    pub table_anchor_enu: (f64, f64, f64),
    pub table_yaw_deg: f64,
    pub table_scale: f64,
}

impl OpenXrPoseAdapter {
    pub fn new(operator_id: &str, callsign: &str, device_mode: &str) -> Self {
        Self {
            operator_id: operator_id.to_string(),
            callsign: callsign.to_string(),
            device_mode: device_mode.to_string(),
            table_anchor_enu: (0.0, 0.0, 0.0),
            table_yaw_deg: 0.0,
            table_scale: 1.0,
        }
    }
}

    pub fn with_table_transform(
        mut self,
        anchor_enu: (f64, f64, f64),
        yaw_deg: f64,
        scale: f64,
    ) -> Self {
        self.table_anchor_enu = anchor_enu;
        self.table_yaw_deg = yaw_deg.rem_euclid(360.0);
        self.table_scale = scale.clamp(0.01, 100.0);
        self
    }

    pub fn transform_pose(&self, sample: &OpenXrPoseSample) -> TacticalHeadPose {
        let is_valid = sample.tracking_flags.is_usable();
        let ts_ms = (sample.timestamp_ns / 1_000_000).max(0) as u64;

        if !is_valid {
            return TacticalHeadPose {
                head_enu: self.table_anchor_enu,
                forward_enu: (0.0, 0.0, 1.0),
                yaw_deg: self.table_yaw_deg,
                pitch_deg: 0.0,
                roll_deg: 0.0,
                is_valid: false,
                timestamp_unix_ms: ts_ms,
            };
        }

        let local_fwd = (0.0, 0.0, -1.0);
        let rotated_fwd = sample.orientation.rotate_vector(local_fwd);
        let (raw_e, raw_u, raw_n) = (
            sample.position.x * self.table_scale,
            sample.position.y * self.table_scale,
            -sample.position.z * self.table_scale,
        );
        let (fwd_e, fwd_u, fwd_n) = (rotated_fwd.0, rotated_fwd.1, -rotated_fwd.2);

        let yaw_rad = self.table_yaw_deg.to_radians();
        let cos_y = yaw_rad.cos();
        let sin_y = yaw_rad.sin();
        let rotated_e = raw_e * cos_y + raw_n * sin_y;
        let rotated_n = -raw_e * sin_y + raw_n * cos_y;

        let final_e = self.table_anchor_enu.0 + rotated_e;
        let final_u = self.table_anchor_enu.1 + raw_u;
        let final_n = self.table_anchor_enu.2 + rotated_n;

        let rot_fwd_e = fwd_e * cos_y + fwd_n * sin_y;
        let rot_fwd_n = -fwd_e * sin_y + fwd_n * cos_y;

        let fwd_len = (rot_fwd_e * rot_fwd_e + fwd_u * fwd_u + rot_fwd_n * rot_fwd_n).sqrt();
        let normalized_fwd = if fwd_len < 1e-12 {
            (0.0, 0.0, 1.0)
        } else {
            (rot_fwd_e / fwd_len, fwd_u / fwd_len, rot_fwd_n / fwd_len)
        };

        let compass_yaw_deg = rot_fwd_e.atan2(rot_fwd_n).to_degrees().rem_euclid(360.0);
        let horiz_range = (rot_fwd_e * rot_fwd_e + rot_fwd_n * rot_fwd_n).sqrt();
        let pitch_deg = fwd_u.atan2(horiz_range.max(1e-6)).to_degrees();
        let (_euler_yaw, _euler_pitch, roll_deg) = sample.orientation.to_euler_deg();

        TacticalHeadPose {
            head_enu: (final_e, final_u, final_n),
            forward_enu: normalized_fwd,
            yaw_deg: compass_yaw_deg,
            pitch_deg,
            roll_deg,
            is_valid: true,
            timestamp_unix_ms: ts_ms,
        }
    }

    pub fn project_gaze_to_ground(
        &self,
        pose: &TacticalHeadPose,
        ground_up_m: f64,
    ) -> Option<(f64, f64, f64)> {
        if !pose.is_valid || pose.forward_enu.1 >= -1e-6 {
            return None;
        }
        let delta_y = ground_up_m - pose.head_enu.1;
        let t = delta_y / pose.forward_enu.1;
        if t <= 0.0 {
            return None;
        }
        Some((
            pose.head_enu.0 + pose.forward_enu.0 * t,
            ground_up_m,
            pose.head_enu.2 + pose.forward_enu.2 * t,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quaternion_identity_and_euler() {
        let q = PoseOrientation::IDENTITY;
        assert_eq!(q.norm(), 1.0);
        let (yaw, pitch, roll) = q.to_euler_deg();
        assert!(yaw.abs() < 1e-4);
        assert!(pitch.abs() < 1e-4);
        assert!(roll.abs() < 1e-4);
        let v = (1.0, 2.0, 3.0);
        assert_eq!(q.rotate_vector(v), v);
    }

    #[test]
    fn test_tracking_flags_usable() {
        let f = PoseTrackingFlags::from_openxr_bits(0x0F);
        assert!(f.is_usable());
        assert!(f.orientation_tracked);
        assert!(f.position_tracked);

        let f_invalid = PoseTrackingFlags::from_openxr_bits(0x00);
        assert!(!f_invalid.is_usable());
    }
