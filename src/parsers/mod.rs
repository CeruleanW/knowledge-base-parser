pub mod obsidian;
pub mod scrivener;

/// Minimum word count for a note to receive auto-generated tags.
const MIN_WORDS_FOR_TAGS: usize = 50;

/// Maximum number of auto-generated tags to produce per document.
const MAX_GENERATED_TAGS: usize = 8;

/// Common English words that carry little topical meaning.
static STOP_WORDS: &[&str] = &[
    "a", "an", "the", "and", "or", "but", "in", "on", "at", "to", "for",
    "of", "with", "by", "from", "up", "as", "is", "are", "was", "were",
    "be", "been", "being", "have", "has", "had", "do", "does", "did",
    "will", "would", "could", "should", "may", "might", "shall", "must",
    "can", "that", "this", "these", "those", "it", "its", "they", "them",
    "their", "there", "here", "we", "you", "he", "she", "not", "no",
    "nor", "so", "yet", "both", "either", "neither", "each", "every",
    "all", "some", "any", "few", "more", "most", "other", "such", "own",
    "than", "too", "very", "just", "also", "about", "into", "through",
    "during", "before", "after", "above", "below", "between", "out",
    "off", "over", "under", "again", "then", "once", "if", "because",
    "while", "although", "though", "since", "unless", "until", "when",
    "where", "who", "which", "what", "how", "why", "one", "two",
    "first", "last", "new", "old", "good", "many", "much", "even",
    "well", "now", "get", "use", "make", "come", "take", "know",
    "think", "see", "look", "want", "give", "find", "tell", "seem",
    "feel", "like", "used", "need", "back", "only", "way", "time",
    "your", "our", "been", "made", "said", "done", "went", "took",
    "left", "keep", "puts", "sets", "just", "when", "some", "that",
    "were", "have", "from", "with", "this", "will", "also", "been",
    "then", "than", "them", "into", "over", "such", "more", "very",
];

/// Generate keyword tags from plain text using word-frequency analysis.
/// Returns an empty list when the document is too short to be meaningful.
pub fn generate_tags(plain: &str, word_count: usize) -> Vec<String> {
    use std::collections::{HashMap, HashSet};

    if word_count < MIN_WORDS_FOR_TAGS {
        return Vec::new();
    }

    let stop: HashSet<&str> = STOP_WORDS.iter().copied().collect();

    let mut freq: HashMap<String, usize> = HashMap::new();
    for token in plain.split(|c: char| !c.is_alphabetic()) {
        let w = token.to_lowercase();
        if w.len() >= 4 && !stop.contains(w.as_str()) {
            *freq.entry(w).or_insert(0) += 1;
        }
    }

    // Require at least 2 occurrences for longer notes to reduce noise.
    let min_freq = if word_count >= 200 { 2 } else { 1 };

    let mut candidates: Vec<(String, usize)> = freq
        .into_iter()
        .filter(|(_, count)| *count >= min_freq)
        .collect();

    // Primary sort: frequency descending; secondary: alphabetical for stability.
    candidates.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));

    candidates
        .into_iter()
        .take(MAX_GENERATED_TAGS)
        .map(|(w, _)| w)
        .collect()
}

/// Strip common markdown formatting to produce plain text suitable for summarization.
/// Uses a line-by-line state machine to handle fenced code blocks correctly.
pub fn strip_markdown(text: &str) -> String {
    use std::sync::OnceLock;
    use regex::Regex;

    static HEADING_RE: OnceLock<Regex> = OnceLock::new();
    static BOLD_ITALIC_RE: OnceLock<Regex> = OnceLock::new();
    static INLINE_CODE_RE: OnceLock<Regex> = OnceLock::new();
    static WIKILINK_RE: OnceLock<Regex> = OnceLock::new();
    static MD_LINK_RE: OnceLock<Regex> = OnceLock::new();
    static IMAGE_RE: OnceLock<Regex> = OnceLock::new();
    static HR_RE: OnceLock<Regex> = OnceLock::new();
    static HTML_TAG_RE: OnceLock<Regex> = OnceLock::new();
    static LIST_RE: OnceLock<Regex> = OnceLock::new();

    let heading_re = HEADING_RE.get_or_init(|| Regex::new(r"^#{1,6}\s+").unwrap());
    let bold_italic_re = BOLD_ITALIC_RE.get_or_init(|| Regex::new(r"\*{1,3}([^*]+)\*{1,3}|_{1,3}([^_]+)_{1,3}").unwrap());
    let inline_code_re = INLINE_CODE_RE.get_or_init(|| Regex::new(r"`[^`]+`").unwrap());
    let wikilink_re = WIKILINK_RE.get_or_init(|| Regex::new(r"\[\[([^\]|#]+)(?:[|#][^\]]*)?\]\]").unwrap());
    let md_link_re = MD_LINK_RE.get_or_init(|| Regex::new(r"\[([^\]]+)\]\([^)]*\)").unwrap());
    let image_re = IMAGE_RE.get_or_init(|| Regex::new(r"!\[[^\]]*\]\([^)]*\)").unwrap());
    let hr_re = HR_RE.get_or_init(|| Regex::new(r"^[-*_]{3,}\s*$").unwrap());
    let html_tag_re = HTML_TAG_RE.get_or_init(|| Regex::new(r"<[^>]+>").unwrap());
    let list_re = LIST_RE.get_or_init(|| Regex::new(r"^\s*[-*+]\s+|^\s*\d+\.\s+").unwrap());

    let mut result = Vec::new();
    let mut in_code_block = false;

    for line in text.lines() {
        let trimmed = line.trim();
        // Toggle fenced code block state
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_code_block = !in_code_block;
            continue;
        }
        if in_code_block {
            continue;
        }
        // Skip horizontal rules
        if hr_re.is_match(trimmed) {
            continue;
        }

        let mut l = line.to_string();
        // Images before links so `![[wikilink-image]]` gets handled
        l = image_re.replace_all(&l, "").to_string();
        // Wikilinks → display title
        l = wikilink_re.replace_all(&l, "$1").to_string();
        // Markdown links → link text
        l = md_link_re.replace_all(&l, "$1").to_string();
        // Remove ATX headings marker
        l = heading_re.replace_all(&l, "").to_string();
        // Remove list markers
        l = list_re.replace_all(&l, "").to_string();
        // Bold/italic → inner text
        l = bold_italic_re.replace_all(&l, |caps: &regex::Captures| {
            caps.get(1).or_else(|| caps.get(2))
                .map(|m| m.as_str().to_string())
                .unwrap_or_default()
        }).to_string();
        // Remove inline code
        l = inline_code_re.replace_all(&l, "").to_string();
        // Remove HTML tags
        l = html_tag_re.replace_all(&l, "").to_string();

        let trimmed_line = l.trim().to_string();
        if !trimmed_line.is_empty() {
            result.push(trimmed_line);
        }
    }

    result.join(" ")
}

/// Take the first `n` words from a plain-text string.
pub fn take_words(text: &str, n: usize) -> String {
    if n == 0 {
        return String::new();
    }
    let words: Vec<&str> = text.split_whitespace().take(n).collect();
    words.join(" ")
}
