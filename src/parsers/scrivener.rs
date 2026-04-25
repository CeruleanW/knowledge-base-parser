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

        let binder = xml_doc
            .root_element()
            .children()
            .find(|n| n.has_tag_name("Binder"))
            .ok_or_else(|| anyhow!("No <Binder> element in {:?}", scrivx_path))?;

        let mut items: Vec<(String, String)> = Vec::new();
        for top_item in binder.children().filter(|n| n.has_tag_name("BinderItem")) {
            collect_text_items(top_item, &mut items);
        }

        let mut documents = Vec::new();
        let mut used_ids: HashSet<String> = HashSet::new();

        for (id, title) in items {
            let content_path = match find_content_file(&self.scriv_path, &id) {
                Some(p) => p,
                None => continue,
            };

            let plain_text = read_and_convert(&content_path);
            let summary = take_words(&plain_text, summary_words);
            let word_count = plain_text.split_whitespace().count();

            let base_id = format!("{}-{}", slugify(&project_name), slugify(&title));
            let doc_id = unique_id(base_id, &mut used_ids);

            documents.push(Document {
                id: doc_id,
                source_type: SourceType::ScrivenerProject,
                source_name: project_name.clone(),
                title,
                path: format!("Files/Data/{}", id),
                tags: Vec::new(),
                generated_tags: Vec::new(),
                metadata: HashMap::new(),
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

fn collect_text_items(node: roxmltree::Node, items: &mut Vec<(String, String)>) {
    let is_text = node.attribute("Type") == Some("Text");
    if is_text {
        // Try both "UUID" and "ID" attributes (Scrivener versions differ)
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
            items.push((id, title));
        }
    }
    // Recurse into Children regardless of type
    if let Some(children_node) = node.children().find(|n| n.has_tag_name("Children")) {
        for child in children_node.children().filter(|n| n.has_tag_name("BinderItem")) {
            collect_text_items(child, items);
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
