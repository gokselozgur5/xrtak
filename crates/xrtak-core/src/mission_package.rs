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
        )
    }

    pub fn generate_manifest_xml(&self) -> String {
        let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n");
        out.push_str("<MissionPackageManifest version=\"2\">\n");
        out.push_str("  <Configuration>\n");
        let uid = escape_xml_attr(&self.uid);
        let name = escape_xml_attr(&self.name);
        out.push_str(&format!("    <Parameter name=\"uid\" value=\"{uid}\"/>\n"));
        out.push_str(&format!("    <Parameter name=\"name\" value=\"{name}\"/>\n"));
        out.push_str(&format!(
            "    <Parameter name=\"onReceiveDelete\" value=\"{}\"/>\n",
            self.on_receive_delete
        ));
        out.push_str("  </Configuration>\n");
        out.push_str("  <Contents>\n");
        for entry in &self.entries {
            let path = escape_xml_attr(&entry.zip_entry);
            out.push_str(&format!(
                "    <Content zipEntry=\"{path}\" ignore=\"{}\">\n",
                entry.ignore
            ));
            let mime = escape_xml_attr(&entry.content_type);
            out.push_str(&format!("      <Parameter name=\"contentType\" value=\"{mime}\"/>\n"));
            out.push_str(&format!(
                "      <Parameter name=\"sha256\" value=\"{}\"/>\n",
                entry.sha256_hex
            ));
            out.push_str("    </Content>\n");
        }
        out.push_str("  </Contents>\n");
        out.push_str("</MissionPackageManifest>\n");
        out
    }

    pub fn build_zip(&self) -> Result<Vec<u8>, MissionPackageError> {
        if self.entries.is_empty() {
            return Err(MissionPackageError::EmptyPackage);
        }
        let manifest_xml = self.generate_manifest_xml();
        let mut zip_files = Vec::with_capacity(self.entries.len() + 1);
        zip_files.push(("MANIFEST/manifest.xml".to_string(), manifest_xml.into_bytes()));
        for e in &self.entries {
            zip_files.push((e.zip_entry.clone(), e.data.clone()));
        }
        Ok(create_zip_archive(&zip_files))
    }

    pub fn parse_zip(zip_bytes: &[u8]) -> Result<Self, MissionPackageError> {
        let files = extract_zip_archive(zip_bytes)?;
        let manifest_bytes = files
            .iter()
            .find(|(name, _)| name == "MANIFEST/manifest.xml" || name == "manifest.xml")
            .map(|(_, data)| data)
            .ok_or(MissionPackageError::MissingManifest)?;

        let manifest_str = std::str::from_utf8(manifest_bytes)
            .map_err(|e| MissionPackageError::XmlError(e.to_string()))?;

        let uid = extract_manifest_param(manifest_str, "uid").unwrap_or_else(|| "TAK-PKG".to_string());
        let name = extract_manifest_param(manifest_str, "name").unwrap_or_else(|| "Package".to_string());
        let on_receive_delete = extract_manifest_param(manifest_str, "onReceiveDelete")
            .map(|v| v == "true")
            .unwrap_or(false);

        let mut archive = Self::new(uid, name).with_on_receive_delete(on_receive_delete);
        for (fname, data) in files {
            if fname == "MANIFEST/manifest.xml" || fname == "manifest.xml" {
                continue;
            }
            archive.add_file(fname, data, "application/octet-stream", false);
        }
        Ok(archive)
    }

    pub fn verify_integrity(&self) -> Result<VerificationReport, MissionPackageError> {
        let mut report = VerificationReport {
            package_uid: self.uid.clone(),
            package_name: self.name.clone(),
            total_entries: self.entries.len(),
            verified_entries: Vec::new(),
            failed_entries: Vec::new(),
            is_valid: true,
        };

        for entry in &self.entries {
            let actual = compute_sha256_hex(&entry.data);
            if actual == entry.sha256_hex {
                report.verified_entries.push(entry.zip_entry.clone());
            } else {
                report.failed_entries.push((
                    entry.zip_entry.clone(),
                    entry.sha256_hex.clone(),
                    actual,
                ));
                report.is_valid = false;
            }
        }
        Ok(report)
    }

    pub fn verify_zip(zip_bytes: &[u8]) -> Result<VerificationReport, MissionPackageError> {
        let files = extract_zip_archive(zip_bytes)?;
        let manifest_bytes = files
            .iter()
            .find(|(name, _)| name == "MANIFEST/manifest.xml" || name == "manifest.xml")
            .map(|(_, data)| data)
            .ok_or(MissionPackageError::MissingManifest)?;

        let manifest_str = std::str::from_utf8(manifest_bytes)
            .map_err(|e| MissionPackageError::XmlError(e.to_string()))?;

        let uid = extract_manifest_param(manifest_str, "uid").unwrap_or_default();
        let name = extract_manifest_param(manifest_str, "name").unwrap_or_default();
        let expected_hashes = extract_content_hashes(manifest_str);

        let mut verified_entries = Vec::new();
        let mut failed_entries = Vec::new();

        for (fname, expected_hash) in &expected_hashes {
            let file_data = files.iter().find(|(n, _)| n == fname).map(|(_, d)| d);
            match file_data {
                Some(data) => {
                    let actual = compute_sha256_hex(data);
                    if actual.eq_ignore_ascii_case(expected_hash) {
                        verified_entries.push(fname.clone());
                    } else {
                        failed_entries.push((fname.clone(), expected_hash.clone(), actual));
                    }
                }
                None => {
                    failed_entries.push((
                        fname.clone(),
                        expected_hash.clone(),
                        "FILE_MISSING".to_string(),
                    ));
                }
            }
        }

        let is_valid = failed_entries.is_empty();
        Ok(VerificationReport {
            package_uid: uid,
            package_name: name,
            total_entries: expected_hashes.len(),
            verified_entries,
            failed_entries,
            is_valid,
        })
    }
}

fn extract_manifest_param(xml: &str, param_name: &str) -> Option<String> {
    let needle = format!("name=\"{param_name}\"");
    for line in xml.lines() {
        if line.contains(&needle) {
            if let Some(val_idx) = line.find("value=\"") {
                let rest = &line[val_idx + 7..];
                if let Some(end) = rest.find('"') {
                    return Some(rest[..end].to_string());
                }
            }
        }
    }
    None
}

