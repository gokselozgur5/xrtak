use super::shape::MarkerShape;
use crate::domain::Affiliation;

pub fn shape_for_affiliation(affil: Affiliation) -> MarkerShape {
    match affil {
        Affiliation::Hostile => MarkerShape::Diamond,
        Affiliation::Friendly => MarkerShape::Rectangle,
        Affiliation::Neutral => MarkerShape::Circle,
        Affiliation::Unknown => MarkerShape::Square,
    }
}
