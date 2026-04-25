use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

use crate::models::KnowledgeBase;

pub fn write_markdown(kb: &KnowledgeBase, dir: &Path) -> Result<()> {
    let path = dir.join("context.md");
    let content = render_markdown(kb);
    std::fs::write(&path, content)?;
    println!("Wrote {}", path.display());
    Ok(())
}

fn render_markdown(kb: &KnowledgeBase) -> String {
    let mut out = String::new();

    // Header
    out.push_str("# Knowledge Base Context\n\n");
    out.push_str(&format!(
        "Generated: {} | Documents: {} | Tags: {}\n\n",
        kb.generated_at,
        kb.documents.len(),
        kb.tag_index.len()
    ));

    // Sources summary
    out.push_str("## Sources\n\n");
    for source in &kb.sources {
        let count = kb
            .documents
            .iter()
            .filter(|d| d.source_name == source.name)
            .count();
        out.push_str(&format!(
            "- {}: \"{}\" ({} document{})\n",
            source.source_type,
            source.name,
            count,
            if count == 1 { "" } else { "s" }
        ));
    }
    out.push('\n');

    // Group documents by source
    let mut by_source: HashMap<String, Vec<_>> = HashMap::new();
    for doc in &kb.documents {
        by_source
            .entry(doc.source_name.clone())
            .or_default()
            .push(doc);
    }

    for source in &kb.sources {
        let docs = match by_source.get(&source.name) {
            Some(d) => d,
            None => continue,
        };

        out.push_str("---\n\n");
        out.push_str(&format!(
            "## {} ({})\n\n",
            source.name, source.source_type
        ));

        for doc in docs {
            out.push_str(&format!("### {}\n", doc.title));

            // Info line: tags, generated tags, links, word count
            let mut info_parts = Vec::new();
            if !doc.tags.is_empty() {
                info_parts.push(format!("**Tags**: {}", doc.tags.join(", ")));
            }
            if !doc.generated_tags.is_empty() {
                info_parts.push(format!("**Auto-tags**: {}", doc.generated_tags.join(", ")));
            }
            if !doc.outbound_links.is_empty() {
                let links_str = doc
                    .outbound_links
                    .iter()
                    .take(5)
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ");
                let suffix = if doc.outbound_links.len() > 5 {
                    format!(" (+{})", doc.outbound_links.len() - 5)
                } else {
                    String::new()
                };
                info_parts.push(format!("**Links**: {}{}", links_str, suffix));
            }
            info_parts.push(format!("**Words**: {}", doc.word_count));

            if !info_parts.is_empty() {
                out.push_str(&info_parts.join(" | "));
                out.push('\n');
            }

            // Summary as blockquote
            if !doc.summary.is_empty() {
                out.push_str(&wrap_blockquote(&doc.summary, 78));
                out.push('\n');
            }

            out.push('\n');
        }
    }

    out
}

/// Wrap text into a markdown blockquote with lines at most `width` chars wide.
fn wrap_blockquote(text: &str, width: usize) -> String {
    let prefix = "> ";
    let available = if width > prefix.len() { width - prefix.len() } else { 40 };

    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line.push_str(word);
        } else if current_line.len() + 1 + word.len() <= available {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(format!("{}{}", prefix, current_line));
            current_line = word.to_string();
        }
    }
    if !current_line.is_empty() {
        lines.push(format!("{}{}", prefix, current_line));
    }

    lines.join("\n")
}
