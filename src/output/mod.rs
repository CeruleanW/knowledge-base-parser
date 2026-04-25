pub mod json;
pub mod markdown;
pub mod tags;

use anyhow::Result;
use std::path::Path;

use crate::models::{KnowledgeBase, TagsExport};

pub fn write_outputs(kb: &KnowledgeBase, tags_export: &TagsExport, output_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(output_dir)?;
    json::write_json(kb, output_dir)?;
    markdown::write_markdown(kb, output_dir)?;
    tags::write_tags(tags_export, output_dir)?;
    Ok(())
}
