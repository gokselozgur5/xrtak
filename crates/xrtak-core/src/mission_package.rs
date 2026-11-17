//! TAK Mission Package ZIP manifest generator and SHA256 integrity verifier.

use crate::mission_manifest::escape_xml_attr;

/// Error conditions encountered during mission package generation or verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MissionPackageError {
    EmptyPackage,
    MissingManifest,
    InvalidZipArchive(String),
    XmlError(String),
    VerificationMismatch {
        entry_name: String,
        expected_sha256: String,
        actual_sha256: String,
    },
}

impl std::fmt::Display for MissionPackageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyPackage => write!(f, "mission package contains no content files"),
            Self::MissingManifest => write!(f, "archive does not contain MANIFEST/manifest.xml"),
            Self::InvalidZipArchive(msg) => write!(f, "invalid zip archive: {msg}"),
            Self::XmlError(msg) => write!(f, "manifest xml error: {msg}"),
            Self::VerificationMismatch {
                entry_name,
                expected_sha256,
                actual_sha256,
            } => write!(
                f,
                "integrity check failed for '{entry_name}': expected {expected_sha256}, got {actual_sha256}"
            ),
        }
    }
}

impl std::error::Error for MissionPackageError {}

/// File entry staged inside a mission package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissionPackageEntry {
    pub zip_entry: String,
    pub data: Vec<u8>,
    pub content_type: String,
    pub sha256_hex: String,
    pub ignore: bool,
}

impl MissionPackageEntry {
    pub fn new(
        zip_entry: impl Into<String>,
        data: Vec<u8>,
        content_type: impl Into<String>,
        ignore: bool,
    ) -> Self {
        let sha256_hex = compute_sha256_hex(&data);
        Self {
            zip_entry: zip_entry.into(),
            data,
            content_type: content_type.into(),
            sha256_hex,
            ignore,
        }
    }
}

/// Result report from integrity validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationReport {
    pub package_uid: String,
    pub package_name: String,
    pub total_entries: usize,
    pub verified_entries: Vec<String>,
    pub failed_entries: Vec<(String, String, String)>,
    pub is_valid: bool,
}

/// ATAK Mission Package Archive manager.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissionPackageArchive {
    pub uid: String,
    pub name: String,
    pub on_receive_delete: bool,
    pub entries: Vec<MissionPackageEntry>,
}

impl MissionPackageArchive {
    pub fn new(uid: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            uid: uid.into(),
            name: name.into(),
            on_receive_delete: false,
            entries: Vec::new(),
        }
    }

    pub fn with_on_receive_delete(mut self, delete: bool) -> Self {
        self.on_receive_delete = delete;
        self
    }

    pub fn add_file(
        &mut self,
        zip_entry: impl Into<String>,
        data: Vec<u8>,
        content_type: impl Into<String>,
        ignore: bool,
    ) -> &mut Self {
        let entry = MissionPackageEntry::new(zip_entry, data, content_type, ignore);
        self.entries.push(entry);
        self
    }

    pub fn add_cot_file(&mut self, zip_entry: impl Into<String>, cot_xml: &str) -> &mut Self {
        self.add_file(
            zip_entry,
            cot_xml.as_bytes().to_vec(),
            "application/x-cot+xml",
            false,
