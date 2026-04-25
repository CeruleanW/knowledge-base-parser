use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    ObsidianVault,
    ScrivenerProject,
}

impl std::fmt::Display for SourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceType::ObsidianVault => write!(f, "obsidian_vault"),
            SourceType::ScrivenerProject => write!(f, "scrivener_project"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceInfo {
    #[serde(rename = "type")]
    pub source_type: SourceType,
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub source_type: SourceType,
    pub source_name: String,
    pub title: String,
    pub path: String,
    pub tags: Vec<String>,
    pub generated_tags: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub outbound_links: Vec<String>,
    pub summary: String,
    pub word_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KnowledgeBase {
    pub generated_at: String,
    pub sources: Vec<SourceInfo>,
    pub documents: Vec<Document>,
    pub link_graph: HashMap<String, Vec<String>>,
    pub tag_index: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagNode {
    pub name: String,
    pub full_path: String,
    pub depth: usize,
    pub direct_count: usize,
    pub total_count: usize,
    #[serde(skip_serializing, default)]
    pub document_ids: Vec<String>,
    pub children: Vec<TagNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceTags {
    pub source_name: String,
    #[serde(rename = "type")]
    pub source_type: SourceType,
    pub total_unique_tags: usize,
    pub total_tagged_documents: usize,
    pub tree: Vec<TagNode>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TagsExport {
    pub generated_at: String,
    pub sources: Vec<SourceTags>,
}

pub fn slugify(s: &str) -> String {
    let slug: String = s
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    slug.split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
