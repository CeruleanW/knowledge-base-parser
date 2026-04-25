use anyhow::{anyhow, Result};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::models::{Document, SourceInfo, SourceType, slugify};
use super::{strip_markdown, take_words};

pub struct ScrivenerParser {
    pub scriv_path: PathBuf,
}

impl ScrivenerParser {
    pub fn new(path: PathBuf) -> Self {
        Self { scriv_path: path }
    }

    pub fn parse(&self, summary_words: usize) -> Result<(SourceInfo, Vec<Document>)> {
        anyhow::ensure!(
            self.scriv_path.exists(),
            "Scrivener path does not exist: {:?}",
            self.scriv_path
        );

        let project_name = self
            .scriv_path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let source = SourceInfo {
            source_type: SourceType::ScrivenerProject,
            name: project_name.clone(),
            path: self.scriv_path.to_string_lossy().to_string(),
        };

        let scrivx_path = find_scrivx(&self.scriv_path)?;
        let xml_content = std::fs::read_to_string(&scrivx_path)?;
        let xml_doc = roxmltree::Document::parse(&xml_content)
            .map_err(|e| anyhow!("Failed to parse .scrivx XML: {}", e))?;

        let root = xml_doc.root_element();
        let binder = root
            .children()
            .find(|n| n.has_tag_name("Binder"))
            .ok_or_else(|| anyhow!("No <Binder> element in {:?}", scrivx_path))?;

        let tables = build_project_tables(root);

        let mut items: Vec<Item> = Vec::new();
        for top_item in binder.children().filter(|n| n.has_tag_name("BinderItem")) {
            collect_text_items(top_item, &mut items);
        }

        let mut documents = Vec::new();
        let mut used_ids: HashSet<String> = HashSet::new();

        for item in items {
            let content_path = match find_content_file(&self.scriv_path, &item.id) {
                Some(p) => p,
                None => continue,
            };

            let plain_text = read_and_convert(&content_path);
            let summary = take_words(&plain_text, summary_words);
            let word_count = plain_text.split_whitespace().count();

            let base_id = format!("{}-{}", slugify(&project_name), slugify(&item.title));
            let doc_id = unique_id(base_id, &mut used_ids);

            let mut tags: Vec<String> = Vec::new();
            let mut seen_tags: HashSet<String> = HashSet::new();
            for kid in &item.keyword_ids {
                if let Some(path) = tables.keyword_paths.get(kid) {
                    if seen_tags.insert(path.clone()) {
                        tags.push(path.clone());
                    }
                }
            }
            if let Some(lid) = &item.label_id {
                if let Some(name) = tables.labels.get(lid) {
                    let t = format!("label/{}", name);
                    if seen_tags.insert(t.clone()) {
                        tags.push(t);
                    }
                }
            }

            let mut metadata: HashMap<String, serde_json::Value> = HashMap::new();
            if let Some(sid) = &item.status_id {
                if let Some(name) = tables.statuses.get(sid) {
                    metadata.insert("status".to_string(), serde_json::Value::String(name.clone()));
                }
            }
            for (k, v) in &item.custom {
                metadata.insert(k.clone(), serde_json::Value::String(v.clone()));
            }

            documents.push(Document {
                id: doc_id,
                source_type: SourceType::ScrivenerProject,
                source_name: project_name.clone(),
                title: item.title,
                path: format!("Files/Data/{}", item.id),
                tags,
                generated_tags: Vec::new(),
                metadata,
                outbound_links: Vec::new(),
                summary,
                word_count,
            });
        }

        Ok((source, documents))
    }
}

fn find_scrivx(scriv_dir: &Path) -> Result<PathBuf> {
    std::fs::read_dir(scriv_dir)?
        .filter_map(|e| e.ok())
        .find(|e| e.path().extension().map(|x| x == "scrivx").unwrap_or(false))
        .map(|e| e.path())
        .ok_or_else(|| anyhow!("No .scrivx file found in {:?}", scriv_dir))
}

struct Item {
    id: String,
    title: String,
    keyword_ids: Vec<String>,
    label_id: Option<String>,
    status_id: Option<String>,
    custom: Vec<(String, String)>,
}

#[derive(Default)]
struct ProjectTables {
    keyword_paths: HashMap<String, String>,
    labels: HashMap<String, String>,
    statuses: HashMap<String, String>,
}

fn collect_text_items(node: roxmltree::Node, items: &mut Vec<Item>) {
    let is_text = node.attribute("Type") == Some("Text");
    if is_text {
        let id = node
            .attribute("ID")
            .or_else(|| node.attribute("UUID"))
            .unwrap_or("")
            .to_string();
        let title = node
            .children()
            .find(|n| n.has_tag_name("Title"))
            .and_then(|n| n.text())
            .unwrap_or("Untitled")
            .to_string();
        if !id.is_empty() {
            items.push(Item {
                id,
                title,
                keyword_ids: extract_keyword_ids(node),
                label_id: read_metadata_text(node, "LabelID"),
                status_id: read_metadata_text(node, "StatusID"),
                custom: extract_custom_metadata(node),
            });
        }
    }
    if let Some(children_node) = node.children().find(|n| n.has_tag_name("Children")) {
        for child in children_node.children().filter(|n| n.has_tag_name("BinderItem")) {
            collect_text_items(child, items);
        }
    }
}

fn extract_keyword_ids(item: roxmltree::Node) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(kws) = item.children().find(|n| n.has_tag_name("Keywords")) {
        for kid in kws.children().filter(|n| n.has_tag_name("KeywordID")) {
            if let Some(t) = kid.text() {
                let trimmed = t.trim();
                if !trimmed.is_empty() {
                    out.push(trimmed.to_string());
                }
            }
        }
    }
    out
}

fn read_metadata_text(item: roxmltree::Node, tag: &str) -> Option<String> {
    let md = item.children().find(|n| n.has_tag_name("MetaData"))?;
    md.children()
        .find(|n| n.has_tag_name(tag))
        .and_then(|n| n.text())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn extract_custom_metadata(item: roxmltree::Node) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let md = match item.children().find(|n| n.has_tag_name("MetaData")) {
        Some(n) => n,
        None => return out,
    };
    let cmd = match md.children().find(|n| n.has_tag_name("CustomMetaData")) {
        Some(n) => n,
        None => return out,
    };
    for entry in cmd.children().filter(|n| n.has_tag_name("MetaDataItem")) {
        let field_id = entry
            .children()
            .find(|n| n.has_tag_name("FieldID"))
            .and_then(|n| n.text())
            .map(|s| s.trim().to_string());
        let value = entry
            .children()
            .find(|n| n.has_tag_name("Value"))
            .and_then(|n| n.text())
            .map(|s| s.trim().to_string());
        if let (Some(k), Some(v)) = (field_id, value) {
            if !k.is_empty() && !v.is_empty() {
                out.push((k, v));
            }
        }
    }
    out
}

fn build_project_tables(root: roxmltree::Node) -> ProjectTables {
    let mut tables = ProjectTables::default();

    if let Some(kw_root) = root.children().find(|n| n.has_tag_name("Keywords")) {
        collect_keyword_paths(kw_root, "", &mut tables.keyword_paths);
    }

    if let Some(ls) = root.children().find(|n| n.has_tag_name("LabelSettings")) {
        if let Some(labels) = ls.children().find(|n| n.has_tag_name("Labels")) {
            for label in labels.children().filter(|n| n.has_tag_name("Label")) {
                if let (Some(id), Some(text)) = (label.attribute("ID"), label.text()) {
                    let title = text.trim();
                    if !title.is_empty() {
                        tables.labels.insert(id.to_string(), title.to_string());
                    }
                }
            }
        }
    }

    if let Some(ss) = root.children().find(|n| n.has_tag_name("StatusSettings")) {
        if let Some(items) = ss.children().find(|n| n.has_tag_name("StatusItems")) {
            for st in items.children().filter(|n| n.has_tag_name("Status")) {
                if let (Some(id), Some(text)) = (st.attribute("ID"), st.text()) {
                    let title = text.trim();
                    if !title.is_empty() {
                        tables.statuses.insert(id.to_string(), title.to_string());
                    }
                }
            }
        }
    }

    tables
}

fn collect_keyword_paths(
    node: roxmltree::Node,
    parent_path: &str,
    out: &mut HashMap<String, String>,
) {
    for kw in node.children().filter(|n| n.has_tag_name("Keyword")) {
        let id = kw.attribute("ID").unwrap_or("").to_string();
        let title = kw
            .children()
            .find(|n| n.has_tag_name("Title"))
            .and_then(|n| n.text())
            .map(|s| s.trim().replace('/', "-"))
            .unwrap_or_default();
        let path = if parent_path.is_empty() {
            title.clone()
        } else if title.is_empty() {
            parent_path.to_string()
        } else {
            format!("{}/{}", parent_path, title)
        };
        if !id.is_empty() && !title.is_empty() {
            out.insert(id, path.clone());
        }
        if let Some(children_node) = kw.children().find(|n| n.has_tag_name("Children")) {
            collect_keyword_paths(children_node, &path, out);
        }
    }
}

fn find_content_file(scriv_dir: &Path, id: &str) -> Option<PathBuf> {
    let base = scriv_dir.join("Files").join("Data").join(id);
    let rtf = base.join("content.rtf");
    let md = base.join("content.md");
    let txt = base.join("content.txt");
    if rtf.exists() {
        Some(rtf)
    } else if md.exists() {
        Some(md)
    } else if txt.exists() {
        Some(txt)
    } else {
        None
    }
}

fn read_and_convert(path: &Path) -> String {
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(_) => return String::new(),
    };

    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    match ext {
        "rtf" => {
            let lossy = String::from_utf8_lossy(&bytes).into_owned();
            strip_rtf(&lossy)
        }
        "md" => {
            let text = String::from_utf8_lossy(&bytes).into_owned();
            strip_markdown(&text)
        }
        _ => String::from_utf8_lossy(&bytes).into_owned(),
    }
}

/// Strip RTF control sequences to extract plain text.
/// Handles: group delimiters, control words, hex escapes, Unicode escapes.
pub fn strip_rtf(rtf: &str) -> String {
    let bytes = rtf.as_bytes();
    let len = bytes.len();
    let mut output = String::with_capacity(len / 2);
    let mut i = 0;

    while i < len {
        match bytes[i] {
            b'{' | b'}' => {
                i += 1;
            }
            b'\\' if i + 1 < len => {
                i += 1; // skip backslash
                match bytes[i] {
                    // Hex-encoded character \'XX
                    b'\'' if i + 2 < len => {
                        let hi = hex_digit(bytes[i + 1]);
                        let lo = hex_digit(bytes[i + 2]);
                        let code = hi * 16 + lo;
                        // Map Windows-1252 common punctuation
                        let ch = windows1252_to_char(code);
                        if let Some(c) = ch {
                            output.push(c);
                        }
                        i += 3;
                    }
                    // Escaped special chars
                    b'\\' => {
                        output.push('\\');
                        i += 1;
                    }
                    b'{' => {
                        output.push('{');
                        i += 1;
                    }
                    b'}' => {
                        output.push('}');
                        i += 1;
                    }
                    b'\n' | b'\r' => {
                        // Escaped newline — line continuation
                        i += 1;
                    }
                    b'*' => {
                        // \* marks an ignorable destination — skip it
                        i += 1;
                    }
                    _ => {
                        // Control word: read alphabetic name
                        let start = i;
                        while i < len && bytes[i].is_ascii_alphabetic() {
                            i += 1;
                        }
                        let ctrl = std::str::from_utf8(&bytes[start..i]).unwrap_or("");

                        // Check for Unicode escape \uN
                        if ctrl == "u" && i < len && (bytes[i].is_ascii_digit() || bytes[i] == b'-') {
                            let neg = bytes[i] == b'-';
                            if neg { i += 1; }
                            let num_start = i;
                            while i < len && bytes[i].is_ascii_digit() {
                                i += 1;
                            }
                            let num_str = std::str::from_utf8(&bytes[num_start..i]).unwrap_or("0");
                            if let Ok(mut n) = num_str.parse::<i32>() {
                                if neg { n = -n; }
                                if n < 0 { n += 65536; }
                                if let Some(c) = char::from_u32(n as u32) {
                                    output.push(c);
                                }
                            }
                            // Skip optional trailing space delimiter
                            if i < len && bytes[i] == b' ' { i += 1; }
                            // Skip the ASCII fallback character
                            if i < len && bytes[i] != b'\\' { i += 1; }
                            continue;
                        }

                        // Skip optional numeric parameter
                        if i < len && bytes[i] == b'-' { i += 1; }
                        while i < len && bytes[i].is_ascii_digit() {
                            i += 1;
                        }
                        // Skip optional trailing space delimiter
                        if i < len && bytes[i] == b' ' { i += 1; }

                        match ctrl {
                            "par" | "pard" if ctrl == "par" => output.push('\n'),
                            "line" => output.push('\n'),
                            "tab" => output.push('\t'),
                            "emdash" => output.push('\u{2014}'),
                            "endash" => output.push('\u{2013}'),
                            "lquote" => output.push('\u{2018}'),
                            "rquote" => output.push('\u{2019}'),
                            "ldblquote" => output.push('\u{201C}'),
                            "rdblquote" => output.push('\u{201D}'),
                            _ => {}
                        }
                    }
                }
            }
            b => {
                output.push(b as char);
                i += 1;
            }
        }
    }

    // Clean up: collapse multiple newlines, trim
    let mut clean = String::new();
    let mut prev_newline = false;
    for ch in output.chars() {
        if ch == '\n' {
            if !prev_newline {
                clean.push(' ');
            }
            prev_newline = true;
        } else {
            prev_newline = false;
            clean.push(ch);
        }
    }
    clean.trim().to_string()
}

fn hex_digit(b: u8) -> u8 {
    match b {
        b'0'..=b'9' => b - b'0',
        b'a'..=b'f' => b - b'a' + 10,
        b'A'..=b'F' => b - b'A' + 10,
        _ => 0,
    }
}

/// Map Windows-1252 byte codes to Unicode chars for common cases.
fn windows1252_to_char(code: u8) -> Option<char> {
    match code {
        0x00..=0x7F => {
            let c = code as char;
            if c.is_ascii_control() && c != '\t' && c != '\n' && c != '\r' {
                None
            } else {
                Some(c)
            }
        }
        0x80 => Some('\u{20AC}'), // Euro sign
        0x82 => Some('\u{201A}'),
        0x83 => Some('\u{0192}'),
        0x84 => Some('\u{201E}'),
        0x85 => Some('\u{2026}'), // Ellipsis
        0x86 => Some('\u{2020}'),
        0x87 => Some('\u{2021}'),
        0x88 => Some('\u{02C6}'),
        0x89 => Some('\u{2030}'),
        0x8A => Some('\u{0160}'),
        0x8B => Some('\u{2039}'),
        0x8C => Some('\u{0152}'),
        0x8E => Some('\u{017D}'),
        0x91 => Some('\u{2018}'), // Left single quote
        0x92 => Some('\u{2019}'), // Right single quote
        0x93 => Some('\u{201C}'), // Left double quote
        0x94 => Some('\u{201D}'), // Right double quote
        0x95 => Some('\u{2022}'), // Bullet
        0x96 => Some('\u{2013}'), // En dash
        0x97 => Some('\u{2014}'), // Em dash
        0x98 => Some('\u{02DC}'),
        0x99 => Some('\u{2122}'), // Trademark
        0x9A => Some('\u{0161}'),
        0x9B => Some('\u{203A}'),
        0x9C => Some('\u{0153}'),
        0x9E => Some('\u{017E}'),
        0x9F => Some('\u{0178}'),
        _ => char::from_u32(code as u32),
    }
}

fn unique_id(base: String, used: &mut HashSet<String>) -> String {
    if used.insert(base.clone()) {
        return base;
    }
    let mut counter = 2;
    loop {
        let candidate = format!("{}-{}", base, counter);
        if used.insert(candidate.clone()) {
            return candidate;
        }
        counter += 1;
    }
}
