use crate::domain::Affiliation;
use super::color::MarkerColor;

pub fn color_for_affiliation(affil: Affiliation) -> MarkerColor {
    match affil {
        Affiliation::Hostile => MarkerColor::RED,
        Affiliation::Friendly => MarkerColor::BLUE,
        Affiliation::Neutral => MarkerColor::GREEN,
        Affiliation::Unknown => MarkerColor::YELLOW,
    }
}
