#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
BIN="$PROJECT_DIR/target/release/kbparser"

# Base directory holding the Obsidian vaults and Scrivener projects.
SOURCE_ROOT="${KB_SOURCE_ROOT:-/knowledge-base}"

# Output directory for the generated context files.
OUTPUT_DIR="${KB_OUTPUT_DIR:-/common/artifacts/context}"

# Individual source overrides (take precedence over KB_SOURCE_ROOT).
IT_VAULT="${KB_OBSIDIAN_IT:-$SOURCE_ROOT/obsidian/it}"
PERSONAL_VAULT="${KB_OBSIDIAN_PERSONAL:-$SOURCE_ROOT/obsidian/personal}"
DELAND="${KB_SCRIVENER_DELAND:-$SOURCE_ROOT/scrivener/Deland.scriv}"
SHORT_STORIES="${KB_SCRIVENER_SHORT:-$SOURCE_ROOT/scrivener/短篇写作.scriv}"
SUPERPOWERS="${KB_SCRIVENER_SUPERPOWERS:-$SOURCE_ROOT/scrivener/超能力者.scriv}"

# Validate every source before writing anything, so a missing mount can't
# overwrite existing context files with empty output.
missing=0
for path in "$IT_VAULT" "$PERSONAL_VAULT"; do
    if [ ! -d "$path" ]; then
        echo "error: Obsidian vault not found: $path" >&2
        missing=1
    fi
done
for path in "$DELAND" "$SHORT_STORIES" "$SUPERPOWERS"; do
    if [ ! -d "$path" ]; then
        echo "error: Scrivener project not found: $path" >&2
        missing=1
    fi
done

if [ "$missing" -ne 0 ]; then
    echo "Set KB_SOURCE_ROOT, or override individual KB_* variables, to point at your knowledge base." >&2
    exit 1
fi

cargo build --release --manifest-path "$PROJECT_DIR/Cargo.toml"

"$BIN" --vault "$IT_VAULT" --output "$OUTPUT_DIR/obsidian-it"
"$BIN" --vault "$PERSONAL_VAULT" --output "$OUTPUT_DIR/obsidian-personal"
"$BIN" --scrivener "$DELAND" --output "$OUTPUT_DIR/scrivener-deland"
"$BIN" --scrivener "$SHORT_STORIES" --output "$OUTPUT_DIR/scrivener-短篇写作"
"$BIN" --scrivener "$SUPERPOWERS" --output "$OUTPUT_DIR/scrivener-超能力者"
