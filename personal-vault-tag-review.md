# Personal Obsidian Vault — Tag Cleanup Review

**Date:** 2026-04-25
**Scope:** `/knowledge-base/obsidian/personal/`
**Source:** `/common/artifacts/context/obsidian-personal/tags.json` (kbparser output, regenerated this session)
**Counts at time of review:** 306 unique tags · 1,410 tagged documents · 2,431 notes total

This document is the tag-ontology counterpart to `notes-cleanup-review.md`. That review targets per-file deletion candidates; this one targets the **shape and naming of the tag tree** itself. The two are independent — fixing tags will not delete notes, fixing stub notes will not collapse tags.

---

## Executive summary

The vault has a workable backbone — `media/`, `topic/`, `type/`, `rating/`, `mine/`, `para/`, `爱主你的神/` — and these account for the bulk of tagged content. The main problems are at the leaves:

1. **Cross-language synonyms** under `media/videogame/`: same concept tagged with both an English acronym and a Chinese name (e.g. `JRPG` 11 / `日式RPG` 24).
2. **Facet collision** in `media/videogame/`: platforms, genres, and an index-page marker share one tree level.
3. **Long-tail singletons**: 71 leaves (~27% of leaves) have exactly one document. Most are noise from one-off tagging.
4. **Bare-numeric leaves**: `rating/3`, `rating/4`, `topic/word/C` — incomplete entries that should have descriptors.
5. **Root-level singleton concepts** (`Idea`, `Vocabulary`, `角色原型`, `诗人`, `CE`, `HUMA1010`, …) that don't fit existing top-level namespaces and weren't promoted into one.

Recommended order of work: **§3 Quick wins** first (mechanical, ~10 minutes in Obsidian), then **§4 Synonym merges**, then **§5 Refactor `media/videogame`**, then **§6 Long-tail review** (the biggest time sink).

---

## 1. Confirmed synonym pairs (merge candidates)

These are pairs where two tag names refer to the same concept. Pick one canonical form and rewrite the other across the vault. All counts below are `direct_count` (notes with that exact tag).

| Keep | Merge from | Rationale |
|------|------------|-----------|
| `media/videogame/日式RPG` (24) | `media/videogame/JRPG` (11) | Chinese form is more frequent; Chinese is the dominant vault language. |
| `media/videogame/欧美RPG` (9) | `media/videogame/CRPG` (4) | Same reasoning. |
| `media/videogame/即时战略` (7) | `media/videogame/RTS` (3) | Same reasoning. |
| `media/videogame/FTG` (7) | `media/videogame/格斗类` (1), `genre/格斗` (1) | FTG is the more frequent form here. |
| `media/videogame/战略类` (4) | `media/videogame/SRPG` (3), `media/videogame/SPG` (1) | These are NOT exactly the same — SRPG = strategy RPG, SPG = simulation strategy game. **Do not auto-merge.** Decide per-doc whether each is a strategy game or strategy-RPG, and re-tag. |
| `mine/Deland/Deland故事集` (3) | `mine/Deland/德兰得故事集` (2) | Romanized vs Chinese form of the same series. Pick one. |

Method in Obsidian: open the tag pane, click the source tag, "Search and replace" via the global tag find-and-replace plugin, or do a vault-wide find on the literal `#tag` form (frontmatter and inline both).

## 2. Bare-numeric / incomplete leaves

These look like data-entry slips — someone wrote `rating/3` instead of `rating/3还行`, etc.

| Probably is | Bare form | Count | Fix |
|-------------|-----------|-------|-----|
| `rating/3还行` (50) | `rating/3` | 1 | Re-tag the lone document. |
| `rating/4不错` (188) | `rating/4` | 1 | Re-tag the lone document. |
| `topic/word/C人造物工具` (12) | `topic/word/C` | 1 | Re-tag the lone document. |

After this, the `rating/` tree becomes self-consistent (every leaf has both numeric prefix and descriptor) and is a much cleaner ontology.

## 3. Quick-win fixes (mechanical)

Order these from easiest to least-easy:

1. **`#`-prefix leftovers in source files.** The parser now strips leading `#` from frontmatter tag values (P1 fix), so the tree no longer shows ghost `#media`/`media` duplicate roots. **However, the `#` is still in your `.md` files.** If you ever change tooling, the dupes will return. To fix at source:
   - Find files where YAML tags contain `#` literals: `grep -rEn '^[[:space:]]*-[[:space:]]*"?#' /knowledge-base/obsidian/personal/ | grep -i 'tags\|^[^:]*:.*\.md'` (better: open Obsidian's tag pane and look for any tag whose name starts with `#`).
   - The fix is to open the file and remove the literal `#` from the frontmatter. Inline `#tag` in the note body is fine and should be left alone.
2. **Bare-numeric fixes** from §2 above (3 documents to retag).
3. **Synonym merges** from §1 above (~10 distinct tag values to rewrite).

These three steps cover the highest-confidence cleanup with negligible judgment required.

## 4. Likely-overlapping fiction subtypes

These pairs are *probably* synonyms, but each one is a per-doc judgment call rather than a mechanical rewrite.

| Pair | Counts | Likely call |
|------|--------|-------------|
| `type/fiction/爱情小说` ↔ `type/fiction/言情小说` | 1 ↔ 1 | Both = romance fiction. Merge to `言情小说` (more common in Chinese taxonomy). |
| `type/fiction/色情小说` ↔ `type/fiction/成人小说` | 2 ↔ 1 | Both = adult fiction. Merge to `成人小说`. |
| `type/fiction/网游小说` ↔ `type/fiction/网络小说` | 1 ↔ 7 | NOT synonyms — `网游` = MMO-themed fiction, `网络小说` = web fiction (publishing channel). Keep both. |
| `type/fiction/志怪小说` ↔ `type/fiction/猎奇小说` | 1 ↔ 1 | Different genres (one is classical Chinese supernatural, one is modern grotesque). Keep separate but verify the docs are tagged correctly. |

## 5. Refactor `media/videogame/` — facet collision

This is the highest-impact structural change in the vault. `media/videogame/` currently contains a single hierarchy level mixing **three orthogonal axes**:

- **Platform**: `PC` (103), `NDS` (17), `NS` (12), `GBA` (22), `FC` (10), `iOS` (8), `3DS` (6), `GBC` (6), `PSP` (4), `GB` (4), `RCG` (3), `PS3` (1), `PS` (1), `Web` (2), `ARC` (6), `Gacha` (2), `Galgame` (2)
- **Genre**: `RPG` (42), `ACT` (25), `日式RPG` (24), `ARPG` (21), `Text-based` (13), `JRPG` (11), `武侠RPG` (11), `欧美RPG` (9), `模拟经营` (8), `FTG` (7), `即时战略` (7), `强手棋` (7), `SPT` (14), `模拟类` (7), `恐怖游戏` (5), `生存类` (5), `大富翁` (5), `战略类` (4), `清版动作` (4), `沙盒类` (3), `视觉小说` (3), `RTS` (3), `SRPG` (3), `平台跳跃` (2), `casual` (2), `music` (2), `visual-novel` (2), `独立游戏` (2), `无双类` (2), `Roguelike` (1), `SPG` (1), `detective` (1), `格斗类` (1)
- **Index/list marker**: `游戏列表` (197) — **this is not a genre or platform**, it's a "this note is part of the games index" flag. It dominates the tree by count and pollutes signal.

**Why this matters:** a single PC RPG today must pick one of `PC` or `RPG` as its tag — you lose the other axis. If you tag both, the parent `media/videogame` shows the same doc under two siblings and the count math gets fuzzy. The deep-tree shape implies hierarchy, but the children aren't actually hierarchical with each other.

**Recommended refactor** (do this in stages):

```
# Current shape
media/videogame/PC          (platform leaks into media/)
media/videogame/RPG         (genre leaks into media/)
media/videogame/游戏列表     (content-type leaks into media/)

# Target shape
media/videogame             (just marks "this note is about a videogame")
platform/PC                 (orthogonal facet, applies to any media type)
genre/RPG                   (already exists at root — promote videogame genres into it)
index/videogame             (or just remove the tag — Obsidian's tag search already gives you the list)
```

This means a single Final Fantasy note becomes:
```
media/videogame
platform/NDS
genre/JRPG       (or 日式RPG after §1)
```

Three small tags instead of one deep ambiguous one. Each is searchable independently, and the counts under each parent are meaningful again.

**Order of operations:**

1. First do the §1 synonym merges so you don't refactor the same doc twice.
2. Pick a pilot subset (say, all `media/videogame/PC` notes — 103 docs) and re-tag them with `media/videogame` + `platform/PC` + their appropriate genre. The "genre" axis is already partially populated under root `genre/` (54 docs), so this consolidates the existing splits.
3. Once the platform axis is established, batch-update the rest by platform.
4. **Drop `media/videogame/游戏列表`** entirely. The 197 notes under it are presumably the games index. If you need them as one searchable set, use a saved search in Obsidian instead of a tag — that's what `游戏列表` is doing badly.

Estimated effort: medium. ~600 distinct doc-tag changes, but mostly mechanical once the rule is set. This is the single change that would most improve the LLM-context output.

## 6. Long-tail singletons

71 leaves have `direct_count == 1`. Singletons are not always wrong — sometimes a tag genuinely applies to one doc — but in clusters they indicate that someone tagged ad-hoc instead of using the existing structure.

Top offenders by parent (singletons under each):

| Parent | Singleton count | Singletons |
|--------|-----------------|------------|
| `topic/` | 10 | `anime`, `criticism`, `food`, `meme`, `planning`, `relax`, `translation`, `workflow`, … |
| `genre/` | 9 | `adventure`, `crime`, `kuso`, `sci-fi`, `世界系`, `剑戟片`, `格斗`, `警匪片`, … |
| `type/fiction/` | 7 | `志怪小说`, `成人小说`, `爱情小说`, `猎奇小说`, `科幻小说`, `网游小说`, `言情小说` |
| `type/` | 7 | `autobiography`, `lectures`, `lyrics`, `news`, `non-fiction`, `prose`, `translation` |
| `media/videogame/` | 6 | `PS`, `PS3`, `Roguelike`, `SPG`, `detective`, `格斗类` |
| `para/resource/` | 3 | `avoid`, `use`, `作品的可能性` |

**How to triage:**

- **Keep singletons that are genuine taxonomies you'll grow into.** `genre/sci-fi` (1) is fine if you plan to read more sci-fi. `topic/criticism` (1) is fine if you'll write more criticism notes.
- **Drop singletons that were experimental.** `genre/kuso` (1), `genre/世界系` (1) — if these aren't part of an active taxonomy, just retag the one doc with a more general parent.
- **Promote fiction singletons that overlap.** Apply §4's synonym calls before triaging this list — that removes a few automatically.
- **Watch for typo-shaped singletons.** `topic/translation` and `type/translation` both exist with count 1 — same concept, two different parents. Pick one (`type/translation` is more accurate since translation is a content-type, not a topic).

Optional: ask kbparser to flag these for you on every run — that's the proposed P2 (anomalies report) we discussed.

## 7. Root-level singletons and small bare roots

Roots that exist outside any namespace, in descending order of count:

```
Relax       (41)
只是爱写字   (40)   — "just love to write" 
Template    (27)
作者         (25)   — "author"
Workflow    (18)
Idea        (15)
我的健身     (13)   — "my fitness"  
生活管理     (13)   — "life management"
收藏         (12)   — "collection / saved"
Review       (8)
Vocabulary   (7)
角色原型     (7)    — "character archetype"
HUMA1010     (6)
建立习惯     (5)    — "build habits"
诗人         (5)    — "poet"
CE           (4)
```

Many of these are coherent and should stay roots (`Template`, `Workflow`, `Vocabulary`). But several are concept-singletons that arguably belong inside an existing namespace:

| Tag | Count | Suggested home |
|-----|-------|----------------|
| `只是爱写字` | 40 | `mine/我的小说` or `mine/我的文章` (looks like personal-writing aggregate) |
| `作者` | 25 | `topic/people/author` (you already have `topic/people/family`, `topic/people/friend`) |
| `角色原型` | 7 | `topic/writing/角色原型` or merge into the existing `topic/writing/创作素材/角色` (9 docs) |
| `建立习惯` | 5 | `topic/habit` or `生活管理/习惯` |
| `诗人` | 5 | `topic/people/poet` |
| `HUMA1010`, `CE` | 6, 4 | These look like course codes. Consider `course/HUMA1010`, `course/CE` (mirroring how your `it` vault uses `ITEC2210` etc., but namespaced). |

This consolidation halves the root count and makes the top-level tree easier to scan.

## 8. Naming conventions to prevent recurrence

The vault has clearly grown organically. Three rules would have prevented most of this report:

1. **One language per facet.** Pick English or Chinese for `genre/`, for `media/videogame/<genre>`, for `type/fiction/<subtype>`, etc. Don't allow both forms. Chinese is the dominant content language — recommend Chinese for all topical/genre tags, English only for tech/programming concepts that have no good Chinese form (which mostly live in the `it` vault anyway).
2. **No `#` in tag values.** Frontmatter tags should be bare names: `tags: [media/videogame]`, never `tags: ["#media/videogame"]`. Inline `#tag` in the body is fine — that's just Obsidian syntax.
3. **Facets stay flat.** If something is a *kind* of dimension (platform, genre, rating, status, type), give it its own root and never bury it inside another root. `media/videogame/PC` is wrong; `media/videogame` plus `platform/PC` is right.

## 9. What to leave alone

Some patterns look messy at first glance but are actually fine:

- **`mine/我的*` subtree.** These are personal categorization labels (我的日记 89, 我的思考 18, 我的诗 17, etc.). They look redundant with their parent (`mine` already implies "mine") but the granularity is useful and the counts are high enough to justify each leaf.
- **`topic/word/A人`/`B自然物`/`C人造物工具`/`G空间`.** The letter prefixes (A, B, C, G…) look like a custom vocabulary classification. As long as you remember the scheme, leave it. Just fix the bare-`C` slip from §2.
- **`type/recipe/<protein-or-dish>` (96 total, 95 direct).** The leaves under recipe (Chicken, MainDish, dessert, drink, sides, vegetable, 懵懵吃, 鱼类) look haphazard but are functionally a recipe filter. Keep — most of these are direct-tagged on a single recipe each, which is the expected shape for recipes.
- **`rating/<n>descriptor` scheme.** Once §2 is done, this is a clean ordinal scale.

## Appendix: relevant kbparser invocation

```bash
# Regenerate context + tag exports for this vault
bash /ai-projects/knowledge-base-parser/run.sh

# Outputs land in:
# /common/artifacts/context/obsidian-personal/tags.md   (this human-readable tree)
# /common/artifacts/context/obsidian-personal/tags.json (machine-readable, 86 KB after slim)
```

The parser already strips `#` prefixes from frontmatter tags, sorts by frequency, and excludes `document_ids` from `tags.json`, so the tree above reflects clean post-fix counts.
