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
