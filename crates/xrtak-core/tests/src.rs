use xrtak_core::affiliation::{AffiliationMapper, StandardIdentity};
use xrtak_core::domain::Affiliation;
use xrtak_core::{classify_cot_type, GeoOrigin, TacticalWorld};

fn as_identity(a: Affiliation) -> StandardIdentity {
    match a {
        Affiliation::Friendly => StandardIdentity::Friendly,
        Affiliation::Hostile => StandardIdentity::Hostile,
        Affiliation::Neutral => StandardIdentity::Neutral,
        Affiliation::Unknown => StandardIdentity::Unknown,
    }
}

#[test]
fn both_affiliation_classifiers_agree() {
    for cot_type in ["a-f-G", "a-h-A", "a-s-G", "b-a-o-tbl", "b-f-t-r", "t-h-x"] {
        let world = as_identity(classify_cot_type(cot_type).0);
        let mapper = AffiliationMapper::parse_affiliation(cot_type).to_standard_identity();
        assert_eq!(world, mapper, "classifiers disagree for {cot_type}");
    }
}

#[test]
fn test_sim_disabled_removes_demo_tactical_zones() {
    let mut world = TacticalWorld::new(GeoOrigin::default());
    assert_eq!(world.tactical_zones.len(), 2);
    world.set_sim_enabled(false);
    assert!(world.tactical_zones.is_empty());
    world.set_sim_enabled(true);
    assert_eq!(world.tactical_zones.len(), 2);
}

#[test]
fn default_build_exposes_active_gateway_surface() {
    let (_, dim) = classify_cot_type("a-f-A");
    let band = xrtak_core::RfBand::ISM2400;
    let _link = xrtak_core::evaluate_rf_link(
        (0.0, 20.0, 0.0),
        (100.0, 20.0, 0.0),
        band,
        16,
        &xrtak_core::ProceduralReliefModel,
    );
    assert_eq!(dim, xrtak_core::Dimension::Air);
    assert_eq!(band.frequency_hz(), 2_400_000_000.0);
}

#[cfg(not(feature = "experimental"))]
mod surface_probe {
    #[allow(non_camel_case_types)]
    pub mod fallback {
        pub struct cas_nine_line;
        pub struct route_nav;
        pub struct tls_pool;
        pub struct MgrsCoord;
        pub struct MissionPackageArchive;
        pub struct TlsConnectionPool;
    }
    pub mod probe {
        pub use super::fallback::*;
        pub use xrtak_core::*;
    }
}

#[cfg(not(feature = "experimental"))]
#[test]
fn default_build_omits_experimental_modules_and_reexports() {
    use std::any::TypeId;
    use surface_probe::{fallback, probe};
    assert_eq!(probe::GeoOrigin::default().extent_m, 1500.0);
    assert_eq!(
        TypeId::of::<probe::cas_nine_line>(),
        TypeId::of::<fallback::cas_nine_line>()
    );
    assert_eq!(
        TypeId::of::<probe::route_nav>(),
        TypeId::of::<fallback::route_nav>()
    );
    assert_eq!(
        TypeId::of::<probe::tls_pool>(),
        TypeId::of::<fallback::tls_pool>()
    );
    assert_eq!(
        TypeId::of::<probe::MgrsCoord>(),
        TypeId::of::<fallback::MgrsCoord>()
    );
    assert_eq!(
        TypeId::of::<probe::MissionPackageArchive>(),
        TypeId::of::<fallback::MissionPackageArchive>()
    );
    assert_eq!(
        TypeId::of::<probe::TlsConnectionPool>(),
        TypeId::of::<fallback::TlsConnectionPool>()
    );
}

#[cfg(feature = "experimental")]
#[test]
fn experimental_feature_exposes_gated_modules_and_reexports() {
    use std::any::TypeId;
    assert_ne!(
        TypeId::of::<xrtak_core::MgrsCoord>(),
        TypeId::of::<xrtak_core::MissionPackageArchive>()
    );
    assert_ne!(
        TypeId::of::<xrtak_core::TlsConnectionPool>(),
        TypeId::of::<xrtak_core::route_nav::RouteNavStatus>()
    );
}
