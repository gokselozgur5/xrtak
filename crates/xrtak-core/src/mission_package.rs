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

fn extract_content_hashes(xml: &str) -> Vec<(String, String)> {
    let mut results = Vec::new();
    let mut current_zip_entry: Option<String> = None;
    for line in xml.lines() {
        if let Some(ze_idx) = line.find("zipEntry=\"") {
            let rest = &line[ze_idx + 10..];
            if let Some(end) = rest.find('"') {
                current_zip_entry = Some(rest[..end].to_string());
            }
        }
        if line.contains("name=\"sha256\"") {
            if let Some(v_idx) = line.find("value=\"") {
                let rest = &line[v_idx + 7..];
                if let Some(end) = rest.find('"') {
                    let sha = rest[..end].to_string();
                    if let Some(ref entry) = current_zip_entry {
                        results.push((entry.clone(), sha));
                    }
                }
            }
        }
    }
    results
}

pub fn compute_crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

pub fn compute_sha256(data: &[u8]) -> [u8; 32] {
    let k: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];

    let bit_len = (data.len() as u64) * 8;
    let mut msg = data.to_vec();
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0x00);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[4 * i],
                chunk[4 * i + 1],
                chunk[4 * i + 2],
                chunk[4 * i + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut h_val = h[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h_val
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(k[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h_val = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(h_val);
    }

    let mut out = [0u8; 32];
    for (i, val) in h.iter().enumerate() {
        out[4 * i..4 * i + 4].copy_from_slice(&val.to_be_bytes());
    }
    out
}

pub fn compute_sha256_hex(data: &[u8]) -> String {
    let digest = compute_sha256(data);
    let mut s = String::with_capacity(64);
    for b in digest {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

pub fn create_zip_archive(files: &[(String, Vec<u8>)]) -> Vec<u8> {
    let mut body = Vec::new();
    let mut cd_records = Vec::new();

    for (name, content) in files {
        let name_bytes = name.as_bytes();
        let local_offset = body.len() as u32;
        let crc = compute_crc32(content);
        let size = content.len() as u32;

        body.extend_from_slice(&0x04034b50u32.to_le_bytes());
        body.extend_from_slice(&20u16.to_le_bytes());
        body.extend_from_slice(&0u16.to_le_bytes());
        body.extend_from_slice(&0u16.to_le_bytes());
        body.extend_from_slice(&0u16.to_le_bytes());
        body.extend_from_slice(&0x21u16.to_le_bytes());
        body.extend_from_slice(&crc.to_le_bytes());
        body.extend_from_slice(&size.to_le_bytes());
        body.extend_from_slice(&size.to_le_bytes());
        body.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
        body.extend_from_slice(&0u16.to_le_bytes());
        body.extend_from_slice(name_bytes);
        body.extend_from_slice(content);

        cd_records.extend_from_slice(&0x02014b50u32.to_le_bytes());
        cd_records.extend_from_slice(&20u16.to_le_bytes());
        cd_records.extend_from_slice(&20u16.to_le_bytes());
        cd_records.extend_from_slice(&0u16.to_le_bytes());
        cd_records.extend_from_slice(&0u16.to_le_bytes());
        cd_records.extend_from_slice(&0u16.to_le_bytes());
        cd_records.extend_from_slice(&0x21u16.to_le_bytes());
        cd_records.extend_from_slice(&crc.to_le_bytes());
        cd_records.extend_from_slice(&size.to_le_bytes());
        cd_records.extend_from_slice(&size.to_le_bytes());
        cd_records.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
        cd_records.extend_from_slice(&0u16.to_le_bytes());
        cd_records.extend_from_slice(&0u16.to_le_bytes());
        cd_records.extend_from_slice(&0u16.to_le_bytes());
        cd_records.extend_from_slice(&0u16.to_le_bytes());
        cd_records.extend_from_slice(&0u32.to_le_bytes());
        cd_records.extend_from_slice(&local_offset.to_le_bytes());
        cd_records.extend_from_slice(name_bytes);
    }

    let cd_offset = body.len() as u32;
    let cd_size = cd_records.len() as u32;
    body.extend_from_slice(&cd_records);

    let count = files.len() as u16;
    body.extend_from_slice(&0x06054b50u32.to_le_bytes());
    body.extend_from_slice(&0u16.to_le_bytes());
    body.extend_from_slice(&0u16.to_le_bytes());
    body.extend_from_slice(&count.to_le_bytes());
    body.extend_from_slice(&count.to_le_bytes());
    body.extend_from_slice(&cd_size.to_le_bytes());
    body.extend_from_slice(&cd_offset.to_le_bytes());
    body.extend_from_slice(&0u16.to_le_bytes());

    body
}

pub fn extract_zip_archive(bytes: &[u8]) -> Result<Vec<(String, Vec<u8>)>, MissionPackageError> {
    if bytes.len() < 22 {
        return Err(MissionPackageError::InvalidZipArchive("file too short".into()));
    }

    let mut eocd_pos = None;
    for i in (0..=bytes.len() - 22).rev() {
        if bytes[i..i + 4] == [0x50, 0x4b, 0x05, 0x06] {
            eocd_pos = Some(i);
            break;
        }
    }
    let eocd = eocd_pos.ok_or_else(|| MissionPackageError::InvalidZipArchive("EOCD missing".into()))?;

    let entry_count = u16::from_le_bytes([bytes[eocd + 10], bytes[eocd + 11]]) as usize;
    let cd_offset = u32::from_le_bytes([
        bytes[eocd + 16],
        bytes[eocd + 17],
        bytes[eocd + 18],
        bytes[eocd + 19],
    ]) as usize;

    let mut files = Vec::with_capacity(entry_count);
    let mut cursor = cd_offset;

    for _ in 0..entry_count {
        if cursor + 46 > bytes.len() || bytes[cursor..cursor + 4] != [0x50, 0x4b, 0x01, 0x02] {
            return Err(MissionPackageError::InvalidZipArchive("corrupt central directory".into()));
        }
        let fn_len = u16::from_le_bytes([bytes[cursor + 28], bytes[cursor + 29]]) as usize;
        let extra_len = u16::from_le_bytes([bytes[cursor + 30], bytes[cursor + 31]]) as usize;
        let comment_len = u16::from_le_bytes([bytes[cursor + 32], bytes[cursor + 33]]) as usize;
        let local_offset = u32::from_le_bytes([
            bytes[cursor + 42],
            bytes[cursor + 43],
            bytes[cursor + 44],
            bytes[cursor + 45],
        ]) as usize;

        let name_start = cursor + 46;
        if name_start + fn_len > bytes.len() {
            return Err(MissionPackageError::InvalidZipArchive("name out of bounds".into()));
        }
        let name_str = std::str::from_utf8(&bytes[name_start..name_start + fn_len])
            .map_err(|e| MissionPackageError::InvalidZipArchive(e.to_string()))?
            .to_string();

        if local_offset + 30 > bytes.len()
            || bytes[local_offset..local_offset + 4] != [0x50, 0x4b, 0x03, 0x04]
        {
            return Err(MissionPackageError::InvalidZipArchive("corrupt local header".into()));
        }
        let l_fn_len = u16::from_le_bytes([bytes[local_offset + 26], bytes[local_offset + 27]]) as usize;
        let l_ex_len = u16::from_le_bytes([bytes[local_offset + 28], bytes[local_offset + 29]]) as usize;
        let comp_size = u32::from_le_bytes([
            bytes[local_offset + 18],
            bytes[local_offset + 19],
            bytes[local_offset + 20],
            bytes[local_offset + 21],
        ]) as usize;

        let data_start = local_offset + 30 + l_fn_len + l_ex_len;
        if data_start + comp_size > bytes.len() {
            return Err(MissionPackageError::InvalidZipArchive("entry data out of bounds".into()));
        }
        let file_data = bytes[data_start..data_start + comp_size].to_vec();
        files.push((name_str, file_data));

        cursor += 46 + fn_len + extra_len + comment_len;
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_test_vectors() {
        assert_eq!(
            compute_sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            compute_sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn test_crc32_calculation() {
        let crc = compute_crc32(b"123456789");
        assert_eq!(crc, 0xcbf43926);
    }

    #[test]
    fn test_package_builder_and_manifest() {
        let mut pkg = MissionPackageArchive::new("MISSION-99", "Alpha Operation")
            .with_on_receive_delete(true);
        pkg.add_cot_file("overlays/point.cot", "<event uid='P1'/>");

        let xml = pkg.generate_manifest_xml();
        assert!(xml.contains("value=\"MISSION-99\""));
        assert!(xml.contains("value=\"Alpha Operation\""));
        assert!(xml.contains("zipEntry=\"overlays/point.cot\""));
        assert!(xml.contains("name=\"sha256\""));
    }

    #[test]
    fn test_zip_build_and_extract_roundtrip() {
        let mut pkg = MissionPackageArchive::new("PKG-01", "Test Package");
        pkg.add_cot_file("routes/alpha.cot", "<event uid='route-1'/>");
        pkg.add_file("notes.txt", b"tactical notes".to_vec(), "text/plain", false);

        let zip_bytes = pkg.build_zip().expect("build zip");
        let parsed = MissionPackageArchive::parse_zip(&zip_bytes).expect("parse zip");

        assert_eq!(parsed.uid, "PKG-01");
        assert_eq!(parsed.name, "Test Package");
        assert_eq!(parsed.entries.len(), 2);
    }

    #[test]
    fn test_zip_integrity_verification_success() {
        let mut pkg = MissionPackageArchive::new("PKG-VERIFY", "Integrity Test");
        pkg.add_cot_file("recon.cot", "<event type='b-m-p-s-p-loc'/>");

        let zip_bytes = pkg.build_zip().expect("build zip");
        let report = MissionPackageArchive::verify_zip(&zip_bytes).expect("verify zip");

        assert!(report.is_valid);
        assert_eq!(report.total_entries, 1);
        assert_eq!(report.verified_entries, vec!["recon.cot"]);
        assert!(report.failed_entries.is_empty());
    }

    #[test]
    fn test_zip_integrity_verification_tamper_detection() {
        let mut pkg = MissionPackageArchive::new("PKG-TAMPER", "Tamper Test");
        pkg.add_cot_file("target.cot", "<event uid='target-orig'/>");

        let zip_bytes = pkg.build_zip().expect("build zip");

        let mut tampered = zip_bytes.clone();
        for i in 0..tampered.len() - 4 {
            if &tampered[i..i + 4] == b"orig" {
                tampered[i..i + 4].copy_from_slice(b"hack");
                break;
            }
        }

        let report = MissionPackageArchive::verify_zip(&tampered).expect("verify tampered zip");
        assert!(!report.is_valid);
        assert_eq!(report.failed_entries.len(), 1);
        assert_eq!(report.failed_entries[0].0, "target.cot");
    }
}
