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
        self.contents
            .push(ManifestContentEntry::new(zip_entry, ignore));
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

pub(crate) use crate::escape_xml as escape_xml_attr;
pub(crate) use crate::unescape_xml as unescape_xml_attr;

impl MissionPackageManifest {
    /// Generate a compliant TAK `manifest.xml` file.
    pub fn generate_xml(&self) -> String {
        let mut out =
            String::from("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n");
        out.push_str(&format!(
            "<MissionPackageManifest version=\"{}\">\n",
            self.version
        ));
        out.push_str("  <Configuration>\n");
        let uid = escape_xml_attr(&self.uid);
        let name = escape_xml_attr(&self.name);
        out.push_str(&format!("    <Parameter name=\"uid\" value=\"{uid}\"/>\n"));
        out.push_str(&format!(
            "    <Parameter name=\"name\" value=\"{name}\"/>\n"
        ));
        out.push_str(&format!(
            "    <Parameter name=\"onReceiveDelete\" value=\"{}\"/>\n",
            self.on_receive_delete
        ));
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
                out.push_str(&format!(
                    "    <Content zipEntry=\"{ze}\" ignore=\"{}\"/>\n",
                    c.ignore
                ));
            } else {
                out.push_str(&format!(
                    "    <Content zipEntry=\"{ze}\" ignore=\"{}\">\n",
                    c.ignore
                ));
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

fn attr_val(e: &quick_xml::events::BytesStart<'_>, key: &[u8]) -> Option<String> {
    e.attributes()
        .flatten()
        .find(|a| a.key.as_ref() == key)
        .and_then(|a| a.unescape_value().ok().map(|v| v.into_owned()))
}

impl MissionPackageManifest {
    /// Parse a TAK Mission Package `manifest.xml` document.
    pub fn parse_xml(xml: &str) -> Result<Self, String> {
        use quick_xml::events::Event;
        use quick_xml::Reader;

        let mut reader = Reader::from_str(xml);
        reader.config_mut().trim_text(true);
        let mut found_root = false;
        let mut manifest = Self::new("", "");
        let (mut in_cfg, mut cur_content) = (false, None);

        loop {
            match reader.read_event() {
                Ok(Event::Start(e)) => match e.name().as_ref() {
                    b"MissionPackageManifest" => {
                        found_root = true;
                        if let Some(v) = attr_val(&e, b"version").and_then(|s| s.parse().ok()) {
                            manifest.version = v;
                        }
                    }
                    b"Configuration" => in_cfg = true,
                    b"Content" => {
                        let ze = attr_val(&e, b"zipEntry").unwrap_or_default();
                        let ign = attr_val(&e, b"ignore").as_deref() == Some("true");
                        cur_content = Some(ManifestContentEntry::new(ze, ign));
                    }
                    _ => {}
                },
                Ok(Event::Empty(e)) => match e.name().as_ref() {
                    b"MissionPackageManifest" => {
                        found_root = true;
                        if let Some(v) = attr_val(&e, b"version").and_then(|s| s.parse().ok()) {
                            manifest.version = v;
                        }
                    }
                    b"Parameter" => {
                        let k = attr_val(&e, b"name").unwrap_or_default();
                        let v = attr_val(&e, b"value").unwrap_or_default();
                        if let Some(c) = cur_content.as_mut() {
                            c.add_param(k, v);
                        } else if in_cfg {
                            match k.as_str() {
                                "uid" => manifest.uid = v,
                                "name" => manifest.name = v,
                                "onReceiveDelete" => manifest.on_receive_delete = v == "true",
                                _ => manifest.add_parameter(k, v),
                            }
                        }
                    }
                    b"Content" => {
                        let ze = attr_val(&e, b"zipEntry").unwrap_or_default();
                        let ign = attr_val(&e, b"ignore").as_deref() == Some("true");
                        manifest.contents.push(ManifestContentEntry::new(ze, ign));
                    }
                    _ => {}
                },
                Ok(Event::End(e)) => match e.name().as_ref() {
                    b"Configuration" => in_cfg = false,
                    b"Content" => {
                        if let Some(c) = cur_content.take() {
                            manifest.contents.push(c);
                        }
                    }
                    _ => {}
                },
                Ok(Event::Eof) => break,
                Err(e) => return Err(e.to_string()),
                _ => {}
            }
        }
        if !found_root {
            return Err("missing MissionPackageManifest root element".into());
        }
        Ok(manifest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_parameter_and_entry_construction() {
        let p = ManifestParameter::new("tool", "public");
        assert_eq!(p.name, "tool");
        assert_eq!(p.value, "public");

        let entry =
            ManifestContentEntry::new("overlays/perimeter.kml", false).with_param("uid", "PERIM-1");
        assert_eq!(entry.zip_entry, "overlays/perimeter.kml");
        assert!(!entry.ignore);
        assert_eq!(entry.parameters.len(), 1);
        assert_eq!(entry.parameters[0].name, "uid");
    }

    #[test]
    fn test_manifest_defaults_and_builders() {
        let mut manifest =
            MissionPackageManifest::new("PKG-001", "Op Spear").with_on_receive_delete(true);
        manifest.add_parameter("tool", "public");
        assert_eq!(manifest.version, 2);
        assert_eq!(manifest.uid, "PKG-001");
        assert_eq!(manifest.name, "Op Spear");
        assert!(manifest.on_receive_delete);
        assert_eq!(manifest.parameters.len(), 1);
    }

    #[test]
    fn test_generate_xml_basic_structure() {
        let mut manifest = MissionPackageManifest::new("UID-99", "Alpha Recon");
        manifest.add_content("MANIFEST/manifest.xml", false);
        manifest.add_content("routes/primary.cot", false);
        let xml = manifest.generate_xml();
        assert!(xml.contains("<MissionPackageManifest version=\"2\">"));
        assert!(xml.contains("<Parameter name=\"uid\" value=\"UID-99\"/>"));
        assert!(xml.contains("<Content zipEntry=\"routes/primary.cot\" ignore=\"false\"/>"));
    }

    #[test]
    fn test_generate_xml_with_nested_content_parameters() {
        let mut manifest = MissionPackageManifest::new("UID-1", "Package");
        let mut entry = ManifestContentEntry::new("certs/client.p12", false);
        entry.add_param("cert_alias", "UserCert");
        manifest.contents.push(entry);
        let xml = manifest.generate_xml();
        assert!(xml.contains("<Content zipEntry=\"certs/client.p12\" ignore=\"false\">"));
        assert!(xml.contains("<Parameter name=\"cert_alias\" value=\"UserCert\"/>"));
    }

    #[test]
    fn test_parse_xml_roundtrip() {
        let mut manifest = MissionPackageManifest::new("UID-ROUNDTRIP", "Test Mission")
            .with_on_receive_delete(true);
        manifest.add_parameter("description", "Recon boundary");
        manifest.add_content("layers/overlay.kml", false);
        manifest.add_content("temp/scratch.txt", true);

        let xml = manifest.generate_xml();
        let parsed = MissionPackageManifest::parse_xml(&xml).expect("parse manifest");

        assert_eq!(parsed.uid, "UID-ROUNDTRIP");
        assert_eq!(parsed.name, "Test Mission");
        assert!(parsed.on_receive_delete);
        assert_eq!(parsed.contents.len(), 2);
        assert_eq!(parsed.active_entries().len(), 1);
    }

    #[test]
    fn test_active_entries_and_find_entry() {
        let mut manifest = MissionPackageManifest::new("UID-SEARCH", "Search test");
        manifest.add_content("data/track.cot", false);
        manifest.add_content("data/notes.txt", true);

        assert_eq!(manifest.content_count(), 2);
        assert_eq!(manifest.active_entries().len(), 1);
        assert_eq!(manifest.active_entries()[0].zip_entry, "data/track.cot");
        assert!(manifest.find_entry("data/track.cot").is_some());
        assert!(manifest.find_entry("nonexistent.xml").is_none());
    }

    #[test]
    fn test_xml_attribute_escaping() {
        let manifest = MissionPackageManifest::new("UID<>&\"", "Op 'Spear' & Shield");
        let xml = manifest.generate_xml();
        assert!(xml.contains("value=\"UID&lt;&gt;&amp;&quot;\""));
        assert!(xml.contains("value=\"Op &apos;Spear&apos; &amp; Shield\""));
        let parsed = MissionPackageManifest::parse_xml(&xml).expect("parse manifest");
        assert_eq!(parsed.uid, "UID<>&\"");
        assert_eq!(parsed.name, "Op 'Spear' & Shield");
    }
}
