use anyhow::Result;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use walkdir::WalkDir;

use crate::models::{Document, SourceInfo, SourceType, slugify};
use super::{strip_markdown, take_words};

pub struct ObsidianParser {
    pub vault_path: PathBuf,
}

impl ObsidianParser {
    pub fn new(path: PathBuf) -> Self {
        Self { vault_path: path }
    }

    pub fn parse(&self, summary_words: usize) -> Result<(SourceInfo, Vec<Document>)> {
        anyhow::ensure!(
            self.vault_path.exists() && self.vault_path.is_dir(),
            "Vault path does not exist or is not a directory: {:?}",
            self.vault_path
        );

        let vault_name = self
            .vault_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let source = SourceInfo {
            source_type: SourceType::ObsidianVault,
            name: vault_name.clone(),
            path: self.vault_path.to_string_lossy().to_string(),
        };

        let mut documents = Vec::new();
        let mut used_ids: HashSet<String> = HashSet::new();

        for entry in WalkDir::new(&self.vault_path)
            .into_iter()
            .filter_entry(|e| {
                !e.file_name()
                    .to_str()
                    .map(|s| s.starts_with('.'))
                    .unwrap_or(false)
            })
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_type().is_file()
                    && e.path().extension().map_or(false, |ext| ext == "md")
            })
        {
            match self.parse_note(entry.path(), &vault_name, summary_words, &mut used_ids) {
                Ok(doc) => documents.push(doc),
                Err(e) => eprintln!(
                    "Warning: failed to parse {:?}: {}",
                    entry.path(),
                    e
                ),
            }
        }

        Ok((source, documents))
    }

    fn parse_note(
        &self,
        path: &Path,
        source_name: &str,
        summary_words: usize,
        used_ids: &mut HashSet<String>,
    ) -> Result<Document> {
        let raw = std::fs::read(&path)?;
        let content = String::from_utf8_lossy(&raw).into_owned();
        // Strip BOM
        let content = content.trim_start_matches('\u{feff}');

        let relative_path = path
            .strip_prefix(&self.vault_path)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        let (yaml_str, body) = split_frontmatter(content);

        let (fm_title, mut tags, mut metadata) = yaml_str
            .map(|y| extract_frontmatter_data(y))
            .unwrap_or_default();

        // Remove known fields from metadata (they're stored separately)
        metadata.remove("title");
        metadata.remove("tags");

        let title = fm_title
            .or_else(|| find_first_heading(body))
            .unwrap_or_else(|| {
                path.file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            });

        // Inline tags (skip fenced code blocks)
        let inline_tags = extract_inline_tags(body);
        for tag in inline_tags {
            if !tags.contains(&tag) {
                tags.push(tag);
            }
        }

        let outbound_links = extract_wikilinks(body);

        let plain = strip_markdown(body);
        let word_count = plain.split_whitespace().count();
        let summary = take_words(&plain, summary_words);
        let generated_tags = super::generate_tags(&plain, word_count);

        let base_id = format!("{}-{}", slugify(source_name), slugify(&title));
        let id = unique_id(base_id, used_ids);

        Ok(Document {
            id,
            source_type: SourceType::ObsidianVault,
            source_name: source_name.to_string(),
            title,
            path: relative_path,
            tags,
            generated_tags,
            metadata,
            outbound_links,
            summary,
            word_count,
        })
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

/// Split content into optional YAML frontmatter and body.
fn split_frontmatter(content: &str) -> (Option<&str>, &str) {
    if !content.starts_with("---") {
        return (None, content);
    }
    // Find end of opening --- line
    let after_open = match content.find('\n') {
        Some(i) => &content[i + 1..],
        None => return (None, content),
    };
    // Find closing ---
    if let Some(pos) = after_open.find("\n---") {
        let yaml_str = &after_open[..pos];
        let rest_start = pos + "\n---".len();
        // Skip rest of closing --- line
        let body_start = after_open[rest_start..]
            .find('\n')
            .map(|i| rest_start + i + 1)
            .unwrap_or(rest_start);
        (Some(yaml_str), &after_open[body_start..])
    } else {
        (None, content)
    }
}

fn extract_frontmatter_data(
    yaml_str: &str,
) -> (Option<String>, Vec<String>, HashMap<String, serde_json::Value>) {
    let value: serde_yaml::Value =
        match serde_yaml::from_str(yaml_str) {
            Ok(v) => v,
            Err(_) => return (None, Vec::new(), HashMap::new()),
        };

    let mapping = match &value {
        serde_yaml::Value::Mapping(m) => m,
        _ => return (None, Vec::new(), HashMap::new()),
    };

    let mut title = None;
    let mut tags = Vec::new();
    let mut metadata = HashMap::new();

    for (k, v) in mapping {
        let key = match k.as_str() {
            Some(s) => s,
            None => continue,
        };

        match key {
            "title" => {
                if let Some(s) = v.as_str() {
                    title = Some(s.to_string());
                }
            }
            "tags" | "tag" => {
                tags = extract_yaml_tags(v);
            }
            _ => {
                metadata.insert(key.to_string(), yaml_to_json(v.clone()));
            }
        }
    }

    (title, tags, metadata)
}

fn extract_yaml_tags(v: &serde_yaml::Value) -> Vec<String> {
    match v {
        serde_yaml::Value::Sequence(seq) => seq
            .iter()
            .map(|item| match item {
                serde_yaml::Value::String(s) => s.clone(),
                other => format!("{:?}", other),
            })
            .filter(|s| !s.is_empty())
            .collect(),
        serde_yaml::Value::String(s) => s
            .split(',')
            .map(|t| t.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        _ => Vec::new(),
    }
}

fn yaml_to_json(v: serde_yaml::Value) -> serde_json::Value {
    match v {
        serde_yaml::Value::Null => serde_json::Value::Null,
        serde_yaml::Value::Bool(b) => serde_json::Value::Bool(b),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                serde_json::json!(i)
            } else if let Some(f) = n.as_f64() {
                serde_json::json!(f)
            } else {
                serde_json::Value::Null
            }
        }
        serde_yaml::Value::String(s) => serde_json::Value::String(s),
        serde_yaml::Value::Sequence(seq) => {
            serde_json::Value::Array(seq.into_iter().map(yaml_to_json).collect())
        }
        serde_yaml::Value::Mapping(map) => serde_json::Value::Object(
            map.into_iter()
                .filter_map(|(k, v)| k.as_str().map(|s| (s.to_owned(), yaml_to_json(v))))
                .collect(),
        ),
        // Tagged values and anything else — convert to string representation
        _ => serde_json::Value::String(format!("{:?}", v)),
    }
}

fn find_first_heading(body: &str) -> Option<String> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"^#{1,6}\s+(.+)$").unwrap());
    body.lines()
        .find_map(|line| re.captures(line).map(|c| c[1].trim().to_string()))
}

fn extract_inline_tags(body: &str) -> Vec<String> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| {
        Regex::new(r"(?:^|\s)#([a-zA-Z][a-zA-Z0-9_/\-]*)").unwrap()
    });

    let mut tags = Vec::new();
    let mut in_code_block = false;

    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }
        for cap in re.captures_iter(line) {
            tags.push(cap[1].to_string());
        }
    }

    tags
}

fn extract_wikilinks(body: &str) -> Vec<String> {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"\[\[([^\]|#\n]+)").unwrap());

    let mut links: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut in_code_block = false;

    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }
        for cap in re.captures_iter(line) {
            let target = cap[1].trim().to_string();
            if !target.is_empty() && seen.insert(target.clone()) {
                links.push(target);
            }
        }
    }

    links
}
