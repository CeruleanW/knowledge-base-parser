use anyhow::Result;
use clap::Parser;
use std::collections::HashMap;
use std::path::PathBuf;

mod models;
mod output;
mod parsers;
mod tags;

use models::{Document, KnowledgeBase};

#[derive(Parser, Debug)]
#[command(
    name = "kbparser",
    about = "Parse knowledge bases (Obsidian, Scrivener) and generate compact context files for LLM consumption"
)]
struct Cli {
    /// Obsidian vault directory (can be specified multiple times)
    #[arg(long, value_name = "PATH")]
    vault: Vec<PathBuf>,

    /// Scrivener project directory (.scriv) (can be specified multiple times)
    #[arg(long, value_name = "PATH")]
    scrivener: Vec<PathBuf>,

    /// Output directory for context.json and context.md
    #[arg(long, value_name = "DIR", default_value = "./output")]
    output: PathBuf,

    /// Number of words to include in each document summary
    #[arg(long, value_name = "N", default_value_t = 150)]
    summary_words: usize,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.vault.is_empty() && cli.scrivener.is_empty() {
        eprintln!("No sources specified. Use --vault or --scrivener to provide input paths.");
        eprintln!("Run with --help for usage information.");
        std::process::exit(1);
    }

    let mut sources = Vec::new();
    let mut all_docs: Vec<Document> = Vec::new();

    for vault_path in &cli.vault {
        let parser = parsers::obsidian::ObsidianParser::new(vault_path.clone());
        match parser.parse(cli.summary_words) {
            Ok((source, docs)) => {
                println!(
                    "Parsed vault \"{}\" — {} notes",
                    source.name,
                    docs.len()
                );
                sources.push(source);
                all_docs.extend(docs);
            }
            Err(e) => eprintln!("Error parsing vault {:?}: {}", vault_path, e),
        }
    }

    for scriv_path in &cli.scrivener {
        let parser = parsers::scrivener::ScrivenerParser::new(scriv_path.clone());
        match parser.parse(cli.summary_words) {
            Ok((source, docs)) => {
                println!(
                    "Parsed Scrivener project \"{}\" — {} documents",
                    source.name,
                    docs.len()
                );
                sources.push(source);
                all_docs.extend(docs);
            }
            Err(e) => eprintln!("Error parsing Scrivener project {:?}: {}", scriv_path, e),
        }
    }

    let link_graph = build_link_graph(&all_docs);
    let tag_index = build_tag_index(&all_docs);
    let tags_export = tags::build_tags_export(&sources, &all_docs);

    let kb = KnowledgeBase {
        generated_at: chrono::Utc::now().to_rfc3339(),
        sources,
        documents: all_docs,
        link_graph,
        tag_index,
    };

    output::write_outputs(&kb, &tags_export, &cli.output)?;
    println!(
        "Done. {} total documents processed.",
        kb.documents.len()
    );

    Ok(())
}

fn build_link_graph(docs: &[Document]) -> HashMap<String, Vec<String>> {
    docs.iter()
        .filter(|d| !d.outbound_links.is_empty())
        .map(|d| (d.title.clone(), d.outbound_links.clone()))
        .collect()
}

fn build_tag_index(docs: &[Document]) -> HashMap<String, Vec<String>> {
    let mut index: HashMap<String, Vec<String>> = HashMap::new();
    for doc in docs {
        for tag in &doc.tags {
            index.entry(tag.clone()).or_default().push(doc.id.clone());
        }
    }
    index
}
