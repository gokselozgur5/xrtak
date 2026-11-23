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
