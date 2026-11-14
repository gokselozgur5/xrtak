use super::color::MarkerColor;
use crate::domain::Affiliation;

pub fn color_for_affiliation(affil: Affiliation) -> MarkerColor {
    match affil {
        Affiliation::Hostile => MarkerColor::RED,
        Affiliation::Friendly => MarkerColor::BLUE,
        Affiliation::Neutral => MarkerColor::GREEN,
        Affiliation::Unknown => MarkerColor::YELLOW,
    }
}
