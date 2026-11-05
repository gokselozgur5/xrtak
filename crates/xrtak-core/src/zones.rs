use crate::domain::{TacticalZone, ThreatDome};
use crate::geo::ElevationProvider;
use crate::viewshed::compute_link_los;

/// Evaluate whether a 3D target `(east_m, up_m, north_m)` is inside a `ThreatDome` hemisphere
/// AND has unobstructed terrain line-of-sight from the emitter's radar mast (`REQ-TAC-02`).
pub fn is_engaged_by_threat_dome(
    dome: &ThreatDome,
    target_enu: (f64, f64, f64),
    provider: &dyn ElevationProvider,
) -> bool {
    let (de, du, dn) = (
        target_enu.0 - dome.east_m,
        target_enu.1 - dome.up_m,
        target_enu.2 - dome.north_m,
    );
    if du < -15.0 || du > dome.ceiling_m {
        return false;
    }
    if (de * de + du * du + dn * dn).sqrt() > dome.radius_m {
        return false;
    }
    let radar_mast_enu = (dome.east_m, dome.up_m + 12.0, dome.north_m);
    compute_link_los(provider, radar_mast_enu, target_enu, 28).clear
}

/// Jordan Curve Theorem (even-odd ray-casting) 2D point-in-polygon check in `(east_m, north_m)` (`REQ-TAC-03`).
pub fn point_in_polygon_en(east_m: f64, north_m: f64, vertices: &[(f64, f64)]) -> bool {
    if vertices.len() < 3 {
        return false;
    }
    let mut inside = false;
    let mut j = vertices.len() - 1;
    for i in 0..vertices.len() {
        let (ei, ni) = vertices[i];
        let (ej, nj) = vertices[j];
        if ((ni > north_m) != (nj > north_m))
            && (east_m < (ej - ei) * (north_m - ni) / (nj - ni) + ei)
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// Evaluate whether a 3D coordinate `(east_m, up_m, north_m)` lies inside a 3D `TacticalZone` prism (`REQ-TAC-03`).
pub fn is_inside_tactical_zone(zone: &TacticalZone, point_enu: (f64, f64, f64)) -> bool {
    point_enu.1 >= zone.floor_m
        && point_enu.1 <= zone.ceiling_m
        && point_in_polygon_en(point_enu.0, point_enu.2, &zone.vertices_en)
}
