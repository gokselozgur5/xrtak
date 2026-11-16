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

/// Individual content file entry within a mission package archive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestContentEntry {
    pub zip_entry: String,
    pub ignore: bool,
    pub parameters: Vec<ManifestParameter>,
}

impl ManifestContentEntry {
    pub fn new(zip_entry: impl Into<String>, ignore: bool) -> Self {
        Self {
            zip_entry: zip_entry.into(),
            ignore,
            parameters: Vec::new(),
        }
    }
}

impl ManifestContentEntry {
    pub fn with_param(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.parameters.push(ManifestParameter::new(name, value));
        self
    }

    pub fn add_param(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.parameters.push(ManifestParameter::new(name, value));
    }
}

/// Top-level TAK Mission Package manifest (`MANIFEST/manifest.xml`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissionPackageManifest {
    pub version: u32,
    pub uid: String,
    pub name: String,
    pub on_receive_delete: bool,
    pub parameters: Vec<ManifestParameter>,
    pub contents: Vec<ManifestContentEntry>,
}

impl MissionPackageManifest {
    pub fn new(uid: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            version: 2,
            uid: uid.into(),
            name: name.into(),
            on_receive_delete: false,
            parameters: Vec::new(),
            contents: Vec::new(),
        }
    }

    pub fn with_on_receive_delete(mut self, on_receive_delete: bool) -> Self {
        self.on_receive_delete = on_receive_delete;
        self
    }
}
