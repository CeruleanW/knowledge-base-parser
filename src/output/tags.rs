use anyhow::Result;
use std::path::Path;

use crate::models::{TagNode, TagsExport};

pub fn write_tags(tags: &TagsExport, dir: &Path) -> Result<()> {
    write_json(tags, dir)?;
    write_markdown(tags, dir)?;
    Ok(())
}

fn write_json(tags: &TagsExport, dir: &Path) -> Result<()> {
    let path = dir.join("tags.json");
    let file = std::fs::File::create(&path)?;
    serde_json::to_writer_pretty(file, tags)?;
    println!("Wrote {}", path.display());
    Ok(())
}

fn write_markdown(tags: &TagsExport, dir: &Path) -> Result<()> {
    let path = dir.join("tags.md");
    let mut out = String::new();

    out.push_str("# Tags\n\n");
    out.push_str(&format!("Generated: {}\n\n", tags.generated_at));

    for src in &tags.sources {
        out.push_str(&format!(
            "## {} ({})\n\n",
            src.source_name, src.source_type
        ));
        out.push_str(&format!(
            "{} unique tags across {} document{}\n\n",
            src.total_unique_tags,
            src.total_tagged_documents,
            if src.total_tagged_documents == 1 { "" } else { "s" }
        ));

        if src.tree.is_empty() {
            out.push_str("_No tags._\n\n");
            continue;
        }

        for node in &src.tree {
            render_node(node, &mut out);
        }
        out.push('\n');
    }

    std::fs::write(&path, out)?;
    println!("Wrote {}", path.display());
    Ok(())
}

fn render_node(node: &TagNode, out: &mut String) {
    let indent = "  ".repeat(node.depth);
    let count_str = if node.direct_count == node.total_count {
        format!("{}", node.total_count)
    } else {
        format!("{} total, {} direct", node.total_count, node.direct_count)
    };
    out.push_str(&format!(
        "{}- `{}` ({})\n",
        indent, node.full_path, count_str
    ));
    for c in &node.children {
        render_node(c, out);
    }
}
