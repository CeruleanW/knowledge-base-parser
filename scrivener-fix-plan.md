# Scrivener Parser — Fix Plan

Status: planned, awaiting implementation
Scope: six quality gaps in the existing Scrivener parser. All changes confined to `src/parsers/scrivener.rs` plus one `Cargo.toml` dependency. No changes required to `models.rs`, `main.rs`, or the `output/` modules.

## Background

Scrivener support already exists and runs in production via `run.sh:12-14`, parsing three projects (`Deland`, `短篇写作`, `超能力者`) into `/common/artifacts/context/scrivener-*`. The parser (`src/parsers/scrivener.rs`) reads the `.scrivx` XML binder tree, finds each binder item's `content.rtf` / `.md` / `.txt`, strips RTF to plain text, and emits `Document`s.

Inspecting `/common/artifacts/context/scrivener-deland/context.md` revealed six concrete quality gaps. The XML-derived tags and labels are correct (the `.scrivx` is UTF-8); only the RTF body handling is broken.

## Confirmed root cause (Gap 1)

RTF files declare `\ansicpg1252` but store Chinese as consecutive `\'XX` hex escapes under a font with `\fcharset134` (GB2312/GBK). `strip_rtf` (`src/parsers/scrivener.rs:331`) runs each `\'XX` through `windows1252_to_char` (`scrivener.rs:466`), producing `Ó¦¸ÃÊÇ` instead of `应该是`. The `.scrivx` XML is UTF-8, so tags/labels look fine; only RTF body text is garbled.

Sample file inspected:
`/knowledge-base/scrivener/Deland.scriv/Files/Data/0002AC32-FAEC-4442-A2BB-6C61558F73AF/content.rtf`

Header excerpt:
```
{\rtf1\ansi\ansicpg1252\cocoartf2759
{\fonttbl\f0\fnil\fcharset134 PingFangSC-Regular;\f1\fswiss\fcharset0 ArialMT;
\f3\fnil\fcharset134 PingFangSC-Semibold; ...}
```
Body text uses `\'b6\'c5\'bf\'c9\'b7\'f2` — GBK bytes for `杜可夫`.

## Decisions (locked in with user)

1. **Synopsis strategy** — `Document.summary = synopsis + "\n\n" + take_words(body, summary_words)` when `synopsis.txt` is present and non-empty; fall back to body-only when absent. Synopsis also stored in `metadata["synopsis"]`.
2. **Link target form** — `outbound_links` holds resolved **target titles** (from `scrivlnk://UUID` via the binder UUID→Title table), matching how Obsidian wikilinks key the existing `link_graph` (`src/main.rs:101`).
3. **Decoding dependency** — add `encoding_rs = "1"` to `Cargo.toml`. Pure Rust, ~300 KB, covers GBK/GB18030, Big5, Shift-JIS, EUC-KR, plus all Windows code pages.

## Gaps and fixes

### Gap 1 — CJK mojibake (critical)

Rewrite `strip_rtf` (`src/parsers/scrivener.rs:331`):
1. Parse `{\fonttbl ...}` into a `font_num → charset` map. Recognized `\fcharset` values:
   - `0`  → Windows-1252 (existing `windows1252_to_char` path)
   - `128` → Shift-JIS
   - `129` → EUC-KR
   - `134` → GBK / GB2312 (`encoding_rs::GBK`)
   - `136` → Big5
   - `161` → Windows-1253 (Greek)
   - `162` → Windows-1254 (Turkish)
   - `163` → Windows-1255 (Hebrew)
   - `177` → Windows-1255 (Hebrew, alternate)
   - `178` → Windows-1256 (Arabic)
   - `186` → Windows-1257 (Baltic)
   - `204` → Windows-1251 (Cyrillic)
   - `222` → Windows-874 (Thai)
   - `238` → Windows-1250 (Central European)
2. Track the active font number as `\fN` control words are encountered in the body.
3. Accumulate consecutive `\'XX` escapes into a byte buffer; on any non-escape token, flush the buffer by decoding it under the *current font's* charset via `encoding_rs::Decoder`.
4. The existing `\uN` Unicode-escape path (`scrivener.rs:387-407`) stays as a fast path; only `\'XX` handling changes.
5. Add `encoding_rs = "1"` under `[dependencies]` in `Cargo.toml`.

### Gap 2 — Strip RTF destination groups

Today `{\fonttbl...}`, `{\colortbl...}`, `{\*\expandedcolortbl...}` leak into output, and the `\*` branch only skips the `\*` token itself (`scrivener.rs:374-377`).

Fix:
- When `strip_rtf` opens a `{` group, peek ahead to detect destination control words: `\fonttbl`, `\colortbl`, `\stylesheet`, `\info`, `\pict`, `\*\generator`, `\*\expandedcolortbl`, `\*\cocoartextscaling`, `\*\cocoaplatform`, `\fldinst` ( Gap 3 handles `\field` specially). If a destination is found, skip the entire group by tracking brace depth until the matching `}`.
- The current `\*` branch should consume its enclosing group instead of just skipping `\*`.
- Delete the `windows1252_to_char` leaks by no longer calling it on CJK bytes (Gap 1 reuses it only for `\fcharset0`).

### Gap 3 — Strip / capture Scrivener field syntax

Scrivener writes inter-doc links as RTF fields:
```
{\field{\*\fldinst HYPERLINK "scrivlnk://UUID"}{\fldrslt <display text>}}
```
Currently this collapses to `HYPERLINK "scrivlnk://UUID" <display text>` leaking into summaries. Also visible in output: `<$Scr_H::1>`, `<!$Scr_H::1>`, `<$Scr_Ps::0>`, `<$Scr_Ns::..>`, `<$Scr_Rn::..>` auto-numbering placeholders.

Fix:
- Recognize `{\field ...}` groups during RTF parsing (not as a generic destination skip). Capture:
  - the URL from `{\*\fldinst HYPERLINK "..."}` (may have multiple `HYPERLINK` segments)
  - the display text from `{\fldrslt ...}`
- Emit only the display text to the body; route the URL to the outbound-link pipeline (Gap 5).
- After RTF stripping, regex-strip Scrivener placeholders:
  - `<\$Scr_[A-Za-z]+::(\d+|-)>`
  - `<!\$Scr_[A-Za-z]+::(\d+|-)>`

### Gap 4 — Preserve binder hierarchy

`collect_text_items` (`scrivener.rs:142`) flattens Scrivener's manuscript → chapter → scene tree.

Fix:
- Change the signature to thread `parent_path: String` (slash-joined parent titles).
- Add fields to the `Item` struct (`scrivener.rs:126`):
  - `binder_path: String` — full path like `Deland/设定/核心目录速查表`
  - `uuid: String` — the binder item UUID (already in `id`, but rename for clarity; keep `id` as the UUID string for backward compat with `find_content_file`)
- Store `binder_path` in `Document.metadata["binder_path"]` (no `models.rs` change needed — `metadata` is an open `HashMap`).
- Optional: also capture `<IncludeInCompile>` into `metadata["include_in_compile"]` for users who want to filter compile-only documents.

### Gap 5 — Extract Scrivener outbound links

`outbound_links` is hardcoded to `Vec::new()` (`scrivener.rs:108`) despite `scrivlnk://UUID` references being genuine inter-document links (analogous to Obsidian wikilinks).

Fix:
- While walking the binder, build a `HashMap<UUID, Title>` lookup table from `<BinderItem UUID="..." ...>` + `<Title>...</Title>`.
- During RTF field parsing (Gap 3), collect `scrivlnk://UUID` URLs from `\fldinst HYPERLINK`.
- Resolve each UUID to its target title. Drop unresolved UUIDs (e.g., links to research/trash or invalid refs). Deduplicate, preserving first-seen order.
- Push resolved titles into `outbound_links`.
- No change needed to `build_link_graph` in `src/main.rs:101` — it already keys the graph by title for all sources.

### Gap 6 — Use `synopsis.txt` as the summary

Every `<UUID>/` folder contains `content.rtf` and a plain-UTF-8 `synopsis.txt`. Examples:
```
0002AC32-.../synopsis.txt → "神秘博物馆主人"
A3E1D639-.../synopsis.txt → "古迹专家，宝藏猎人，寻宝者\n冒险者公会最厉害的寻宝者之一"
```

Fix:
- In `ScrivenerParser::parse` (`scrivener.rs:64`), after reading `content_path`, also check `<UUID>/synopsis.txt`.
- If it exists and is non-empty (after trimming), read it as UTF-8 and build summary as `synopsis + "\n\n" + take_words(body, summary_words)`. If synopsis is absent or empty, fall back to `take_words(body, summary_words)` (current behavior).
- Store the synopsis string in `metadata["synopsis"]` for stable downstream access regardless of summary truncation length.

## Files changed

| File | Change |
|------|--------|
| `Cargo.toml` | Add `encoding_rs = "1"` under `[dependencies]`. |
| `src/parsers/scrivener.rs` | Rewrite `strip_rtf` (Gaps 1, 2, 3); add `Item.binder_path` + refactor `collect_text_items` (Gap 4); add binder UUID→Title table + populate `outbound_links` (Gap 5); read `synopsis.txt` into summary + metadata (Gap 6). |

No changes to: `src/models.rs`, `src/main.rs`, `src/parsers/mod.rs`, `src/output/**`, `src/tags.rs`, `run.sh`.

## Verification (post-implementation)

1. `cargo build --release` in this project — build succeeds with no warnings beyond what currently exists.
2. Run `./run.sh` (or just the three `--scrivener` lines for faster iteration):
   ```
   ./target/release/kbparser --scrivener "/knowledge-base/scrivener/Deland.scriv"      --output /common/artifacts/context/scrivener-deland
   ./target/release/kbparser --scrivener "/knowledge-base/scrivener/短篇写作.scriv"    --output "/common/artifacts/context/scrivener-短篇写作"
   ./target/release/kbparser --scrivener "/knowledge-base/scrivener/超能力者.scriv"    --output "/common/artifacts/context/scrivener-超能力者"
   ```
3. Inspect `/common/artifacts/context/scrivener-deland/context.md`:
   - Chinese characters render correctly (no mojibake). Sample target: the doc whose UUID is `0002AC32-FAEC-4442-A2BB-6C61558F73AF` should read `神秘博物馆主人 / 杜可夫 ...` not `Ó¦¸ÃÊÇ ...`.
   - No `fonttbl`/`colortbl` text at the top of summaries.
   - No `HYPERLINK "scrivlnk://..."` or `<$Scr_H::..>` tokens in any summary.
   - Document entries with cross-references now show `**Links**: <title>, <title>` lines.
   - Docs with `synopsis.txt` show the synopsis at the top of the summary blockquote.
4. Inspect `/common/artifacts/context/scrivener-deland/tags.json` — no regression expected (tags come from UTF-8 XML).
5. Refresh the `kb` reference by running the `kbparse` tool after the new artifacts land.
6. Optional, additive: unit tests in `src/parsers/scrivener.rs` for `strip_rtf` with a small GBK sample, a `scrivlnk://` field, and Scrivener placeholder syntax. The repo has no test harness today; `cargo test` is the entry point if added.

## Risks / Tradeoffs

- **`encoding_rs` dependency** — pure Rust, well-maintained, ~300 KB added to the binary. Cheaper and much more reliable than a hand-rolled GBK decoder. Alternative considered: a tiny inline GBK table (smaller binary, incomplete coverage for non-Chinese CJK); rejected because the user has Chinese projects and a proper decode path future-proofs other CJK scripts.
- **Non-CJK regression risk** — low. The new byte-buffer logic only triggers on consecutive `\'XX` runs whose active font charset is CJK; Latin scripts via `\fcharset0` go through the existing Windows-1252 path unchanged.
- **Backward compatibility** — populating `outbound_links`, `metadata["binder_path"]`, and `metadata["synopsis"]` for Scrivener docs is purely additive. Downstream readers consuming `context.json` will see more data, never less.
- **Throwaway test scaffolding** — if unit tests are added, they require no new test framework (Rust's built-in `#[test]` + `cargo test` works out of the box).