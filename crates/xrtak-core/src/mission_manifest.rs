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

impl MissionPackageManifest {
    pub fn add_parameter(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.parameters.push(ManifestParameter::new(name, value));
    }

    pub fn add_content(&mut self, zip_entry: impl Into<String>, ignore: bool) {
        self.contents.push(ManifestContentEntry::new(zip_entry, ignore));
    }

    pub fn find_entry(&self, zip_entry: &str) -> Option<&ManifestContentEntry> {
        self.contents.iter().find(|c| c.zip_entry == zip_entry)
    }
}

impl MissionPackageManifest {
    pub fn active_entries(&self) -> Vec<&ManifestContentEntry> {
        self.contents.iter().filter(|c| !c.ignore).collect()
    }

    pub fn content_count(&self) -> usize {
        self.contents.len()
    }
}

pub(crate) fn escape_xml_attr(val: &str) -> String {
    val.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

impl MissionPackageManifest {
    /// Generate a compliant TAK `manifest.xml` file.
    pub fn generate_xml(&self) -> String {
        let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n");
        out.push_str(&format!("<MissionPackageManifest version=\"{}\">\n", self.version));
        out.push_str("  <Configuration>\n");
        let uid = escape_xml_attr(&self.uid);
        let name = escape_xml_attr(&self.name);
        out.push_str(&format!("    <Parameter name=\"uid\" value=\"{uid}\"/>\n"));
        out.push_str(&format!("    <Parameter name=\"name\" value=\"{name}\"/>\n"));
        out.push_str(&format!("    <Parameter name=\"onReceiveDelete\" value=\"{}\"/>\n", self.on_receive_delete));
        for p in &self.parameters {
            let k = escape_xml_attr(&p.name);
            let v = escape_xml_attr(&p.value);
            out.push_str(&format!("    <Parameter name=\"{k}\" value=\"{v}\"/>\n"));
        }
        out.push_str("  </Configuration>\n");
        self.append_contents_xml(&mut out);
        out.push_str("</MissionPackageManifest>\n");
        out
    }
}

impl MissionPackageManifest {
    fn append_contents_xml(&self, out: &mut String) {
        out.push_str("  <Contents>\n");
        for c in &self.contents {
            let ze = escape_xml_attr(&c.zip_entry);
            if c.parameters.is_empty() {
                out.push_str(&format!("    <Content zipEntry=\"{ze}\" ignore=\"{}\"/>\n", c.ignore));
            } else {
                out.push_str(&format!("    <Content zipEntry=\"{ze}\" ignore=\"{}\">\n", c.ignore));
                for p in &c.parameters {
                    let k = escape_xml_attr(&p.name);
                    let v = escape_xml_attr(&p.value);
                    out.push_str(&format!("      <Parameter name=\"{k}\" value=\"{v}\"/>\n"));
                }
                out.push_str("    </Content>\n");
            }
        }
        out.push_str("  </Contents>\n");
    }
}

pub(crate) fn extract_attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let key = format!("{name}=\"");
    let start = tag.find(&key)? + key.len();
    let end = tag[start..].find('"')? + start;
    Some(&tag[start..end])
}

impl MissionPackageManifest {
    /// Parse a TAK Mission Package `manifest.xml` document.
    pub fn parse_xml(xml: &str) -> Result<Self, String> {
        let version = extract_attr(xml, "version")
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(2);
        let mut manifest = Self {
            version,
            uid: String::new(),
            name: String::new(),
            on_receive_delete: false,
            parameters: Vec::new(),
            contents: Vec::new(),
        };
        manifest.parse_config_block(xml);
        manifest.parse_contents_block(xml);
        Ok(manifest)
    }
}
