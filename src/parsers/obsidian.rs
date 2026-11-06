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
    let value: serde_yaml_ng::Value =
        match serde_yaml_ng::from_str(yaml_str) {
            Ok(v) => v,
            Err(_) => return (None, Vec::new(), HashMap::new()),
        };

    let mapping = match &value {
        serde_yaml_ng::Value::Mapping(m) => m,
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

fn extract_yaml_tags(v: &serde_yaml_ng::Value) -> Vec<String> {
    match v {
        serde_yaml_ng::Value::Sequence(seq) => seq
            .iter()
            .map(|item| match item {
                serde_yaml_ng::Value::String(s) => s.clone(),
                other => format!("{:?}", other),
            })
            .filter_map(normalize_tag)
            .collect(),
        serde_yaml_ng::Value::String(s) => s
            .split(',')
            .map(|t| t.to_string())
            .filter_map(normalize_tag)
            .collect(),
        _ => Vec::new(),
    }
}

fn normalize_tag(s: String) -> Option<String> {
    let t = s.trim().trim_start_matches('#').trim();
    if t.is_empty() { None } else { Some(t.to_string()) }
}

fn yaml_to_json(v: serde_yaml_ng::Value) -> serde_json::Value {
    match v {
        serde_yaml_ng::Value::Null => serde_json::Value::Null,
        serde_yaml_ng::Value::Bool(b) => serde_json::Value::Bool(b),
        serde_yaml_ng::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                serde_json::json!(i)
            } else if let Some(f) = n.as_f64() {
                serde_json::json!(f)
            } else {
                serde_json::Value::Null
            }
        }
        serde_yaml_ng::Value::String(s) => serde_json::Value::String(s),
        serde_yaml_ng::Value::Sequence(seq) => {
            serde_json::Value::Array(seq.into_iter().map(yaml_to_json).collect())
        }
        serde_yaml_ng::Value::Mapping(map) => serde_json::Value::Object(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_frontmatter_from_body() {
        let input = "---\ntitle: Hello\ntags: [rust, yaml]\n---\n# Body\n";
        let (yaml, body) = split_frontmatter(input);
        assert_eq!(yaml, Some("title: Hello\ntags: [rust, yaml]"));
        assert_eq!(body, "# Body\n");
    }

    #[test]
    fn content_without_frontmatter_is_all_body() {
        let input = "# Just a note\n";
        let (yaml, body) = split_frontmatter(input);
        assert_eq!(yaml, None);
        assert_eq!(body, input);
    }

    #[test]
    fn unclosed_frontmatter_is_treated_as_body() {
        let input = "---\ntitle: Broken\n";
        let (yaml, body) = split_frontmatter(input);
        assert_eq!(yaml, None);
        assert_eq!(body, input);
    }

    #[test]
    fn extracts_title_tags_and_metadata() {
        let yaml = r##"
title: My Note
tags: [rust, "#yaml", " spaced "]
author: Jane
draft: false
count: 3
nested:
  key: value
list: [1, 2]
empty: null
"##;
        let (title, tags, metadata) = extract_frontmatter_data(yaml);

        assert_eq!(title.as_deref(), Some("My Note"));
        assert_eq!(tags, vec!["rust", "yaml", "spaced"]);
        assert_eq!(metadata["author"], serde_json::json!("Jane"));
        assert_eq!(metadata["draft"], serde_json::json!(false));
        assert_eq!(metadata["count"], serde_json::json!(3));
        assert_eq!(metadata["nested"], serde_json::json!({"key": "value"}));
        assert_eq!(metadata["list"], serde_json::json!([1, 2]));
        assert!(metadata.contains_key("empty"));
        assert_eq!(metadata["empty"], serde_json::Value::Null);
        assert!(!metadata.contains_key("title"));
        assert!(!metadata.contains_key("tags"));
    }

    #[test]
    fn accepts_comma_separated_tags_string() {
        let (_, tags, _) = extract_frontmatter_data("tags: \"rust, #yaml, cli\"\n");
        assert_eq!(tags, vec!["rust", "yaml", "cli"]);
    }

    #[test]
    fn invalid_yaml_yields_defaults() {
        let (title, tags, metadata) = extract_frontmatter_data("title: [unclosed\n");
        assert_eq!(title, None);
        assert!(tags.is_empty());
        assert!(metadata.is_empty());
    }

    #[test]
    fn non_mapping_yaml_yields_defaults() {
        let (title, tags, metadata) = extract_frontmatter_data("- just\n- a list\n");
        assert_eq!(title, None);
        assert!(tags.is_empty());
        assert!(metadata.is_empty());
    }

    #[test]
    fn inline_tags_skip_fenced_code_blocks() {
        let body = "#tag one\n```\n#not-a-tag\n```\n##heading #two\n";
        assert_eq!(extract_inline_tags(body), vec!["tag", "two"]);
    }
}
