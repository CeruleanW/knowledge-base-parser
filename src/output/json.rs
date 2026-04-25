use anyhow::Result;
use std::path::Path;

use crate::models::KnowledgeBase;

pub fn write_json(kb: &KnowledgeBase, dir: &Path) -> Result<()> {
    let path = dir.join("context.json");
    let file = std::fs::File::create(&path)?;
    serde_json::to_writer_pretty(file, kb)?;
    println!("Wrote {}", path.display());
    Ok(())
}
