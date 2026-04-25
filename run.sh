#!/usr/bin/env bash
set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
BIN="$PROJECT_DIR/target/release/kbparser"
OUT="/common/artifacts/context"

cargo build --release --manifest-path "$PROJECT_DIR/Cargo.toml"

"$BIN" --vault /knowledge-base/obsidian/it         --output "$OUT/obsidian-it"
"$BIN" --vault /knowledge-base/obsidian/personal    --output "$OUT/obsidian-personal"
"$BIN" --scrivener "/knowledge-base/scrivener/Deland.scriv"   --output "$OUT/scrivener-deland"
"$BIN" --scrivener "/knowledge-base/scrivener/短篇写作.scriv"  --output "$OUT/scrivener-短篇写作"
"$BIN" --scrivener "/knowledge-base/scrivener/超能力者.scriv"  --output "$OUT/scrivener-超能力者"
