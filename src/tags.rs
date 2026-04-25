use std::collections::{BTreeMap, HashMap, HashSet};

use crate::models::{Document, SourceInfo, SourceTags, TagNode, TagsExport};

pub fn build_tags_export(sources: &[SourceInfo], docs: &[Document]) -> TagsExport {
    let mut by_source: HashMap<&str, Vec<&Document>> = HashMap::new();
    for d in docs {
        by_source.entry(d.source_name.as_str()).or_default().push(d);
    }

    let sources_out = sources
        .iter()
        .map(|s| {
            let docs = by_source.get(s.name.as_str()).cloned().unwrap_or_default();
            build_source_tags(s, &docs)
        })
        .collect();

    TagsExport {
        generated_at: chrono::Utc::now().to_rfc3339(),
        sources: sources_out,
    }
}

fn build_source_tags(source: &SourceInfo, docs: &[&Document]) -> SourceTags {
    let mut tagged_docs: HashSet<&str> = HashSet::new();
    let mut all_tags: HashSet<&str> = HashSet::new();
    let mut roots: BTreeMap<String, Builder> = BTreeMap::new();

    for d in docs {
        if !d.tags.is_empty() {
            tagged_docs.insert(d.id.as_str());
        }
        for tag in &d.tags {
            all_tags.insert(tag.as_str());
            let parts: Vec<&str> = tag.split('/').filter(|s| !s.is_empty()).collect();
            if !parts.is_empty() {
                insert(&mut roots, &parts, 0, String::new(), &d.id);
            }
        }
    }

    let tree = roots.into_values().map(finalize).collect();

    SourceTags {
        source_name: source.name.clone(),
        source_type: source.source_type.clone(),
        total_unique_tags: all_tags.len(),
        total_tagged_documents: tagged_docs.len(),
        tree,
    }
}

struct Builder {
    name: String,
    full_path: String,
    depth: usize,
    direct_doc_ids: Vec<String>,
    children: BTreeMap<String, Builder>,
}

fn insert(
    map: &mut BTreeMap<String, Builder>,
    parts: &[&str],
    depth: usize,
    parent_path: String,
    doc_id: &str,
) {
    let part = parts[0];
    let full_path = if parent_path.is_empty() {
        part.to_string()
    } else {
        format!("{}/{}", parent_path, part)
    };

    let entry = map.entry(part.to_string()).or_insert_with(|| Builder {
        name: part.to_string(),
        full_path: full_path.clone(),
        depth,
        direct_doc_ids: Vec::new(),
        children: BTreeMap::new(),
    });

    if parts.len() == 1 {
        entry.direct_doc_ids.push(doc_id.to_string());
    } else {
        insert(&mut entry.children, &parts[1..], depth + 1, full_path, doc_id);
    }
}

fn finalize(b: Builder) -> TagNode {
    let mut seen = HashSet::new();
    let direct_doc_ids: Vec<String> = b
        .direct_doc_ids
        .into_iter()
        .filter(|id| seen.insert(id.clone()))
        .collect();

    let children: Vec<TagNode> = b.children.into_values().map(finalize).collect();

    let mut all_doc_ids: HashSet<String> = direct_doc_ids.iter().cloned().collect();
    for c in &children {
        collect_doc_ids(c, &mut all_doc_ids);
    }

    TagNode {
        name: b.name,
        full_path: b.full_path,
        depth: b.depth,
        direct_count: direct_doc_ids.len(),
        total_count: all_doc_ids.len(),
        document_ids: direct_doc_ids,
        children,
    }
}

fn collect_doc_ids(node: &TagNode, out: &mut HashSet<String>) {
    for id in &node.document_ids {
        out.insert(id.clone());
    }
    for c in &node.children {
        collect_doc_ids(c, out);
    }
}
