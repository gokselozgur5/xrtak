//! Cursor-on-Target (CoT) mission package ZIP manifest XML generator.

use serde::{Deserialize, Serialize};

/// Key-value configuration parameter within a Mission Package manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestParameter {
    pub name: String,
    pub value: String,
}

impl ManifestParameter {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}
