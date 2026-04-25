# Obsidian Notes Cleanup Review - Instructions

Use this document to run a periodic review of an Obsidian vault and identify notes that should be deleted.

## Usage

Provide this file along with a prompt like:

> Follow the instructions in `notes-cleanup-instructions.md` to review the notes in `/knowledge-base/obsidian/personal` and generate a cleanup report.

## Target Directory

The default vault path is `/knowledge-base/obsidian/personal`. Adjust as needed.

## What to Flag for Deletion

### 1. Truly Empty Files
- 0 bytes, no content at all.

### 2. Frontmatter-Only Stubs
- Files that contain only YAML frontmatter (tags, aliases, metadata fields) but no note body below the `---` closing delimiter.
- These are typically auto-created when tagging or linking and never filled in.

### 3. Query-Only Stubs
- Files whose entire body is a single embedded query block (e.g., `` ```query tag:SomeTag ``` ``).
- These queries can be run from Obsidian's search bar and don't need a dedicated file.

### 4. Title-Repeat / Near-Empty Stubs
- Files where the body merely restates the filename, contains a single wikilink with no context, or has only one or two trivial words plus a tag.
- Includes abandoned fragments marked with tags like `#暂时放弃`.

## What to Keep

- **Journal entries** — even short ones are personal records with date-based value.
- **Template files** — they serve a structural purpose for the vault.
- **Short but meaningful content** — a file with a quote, a creative writing fragment, a reference link, a theological reflection, or a literary observation is not useless just because it's short. Read the file before judging.
- **Index/MOC pages** — files that link to other notes as a map of content, even if they have little prose.

## Review Process

1. **Count files** per directory to understand the vault structure.
2. **Find the smallest files** by line count (0-5 lines) as primary candidates.
3. **Read every candidate file** — do not judge by line count alone. A 1-line file with a meaningful quote should be kept; a 10-line file with only frontmatter should be flagged.
4. **Categorize** each flagged file into one of the four categories above.
5. **Skip** the `.trash/` directory — Obsidian manages its own trash.

## Output Format

Write the results as a Markdown file named `notes-cleanup-review.md` with:

- A date and scope header.
- One table per category, with columns: `#`, `File` (path relative to vault root), `Content Summary` or `Reason`.
- A summary table with counts per category and a total.
- A notes section explaining what was excluded and why.
