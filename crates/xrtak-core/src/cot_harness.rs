use crate::domain::Affiliation;
use crate::geo::{ElevationProvider, ProceduralReliefModel};
use crate::{parse_cot_xml_with_provider, GeoOrigin, TacticalEntity};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CotHarnessError {
    InvalidXml(String),
    MissingPoint,
    MissingUid,
    CoordinateRange(String),
}
