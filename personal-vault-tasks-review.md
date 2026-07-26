## Personal Obsidian Vault — Tasks & Actions Review

**Date:** 2026-05-11
**Scope:** `/knowledge-base/obsidian/personal/`
**Counts at time of review:** 150 checkboxes across 48 files · 40 open / 110 done · 17 lines use Tasks-plugin metadata

Companion to `personal-vault-tag-review.md` and `notes-cleanup-review.md`. The tag review targets the tag tree; the notes review targets stub files; this review targets the **action layer** — the checkboxes you treat as a to-do list.

---

## Executive summary

- The tooling is fine — `obsidian-tasks-plugin`, `dataview`, `obsidian-kanban` are all installed. The problem is conventions, not capability.
- `inbox/Main Kanban.md` exists but contains 1 task. It is not the central hub.
- Tasks live as embedded checkboxes inside per-topic `pages/` (Driver's License 13, Minecraft 12, Permanent Residency 9, …). That co-location is actually fine — what's missing is a **query-driven central view**.
- Of 40 open tasks, only 17 lines vault-wide carry Tasks-plugin metadata (`📅 due`, `⏫🔼🔽⏬` priority, `✅ done`). Most opens are unprioritized, undated, and indistinguishable from each other.
- A real portion of "open tasks" aren't actions at all — they're bookmarks (`- [ ] [URL]`), wikilink stubs (`- [ ] [[圣女贞德]]`), or empty checkboxes. Removing these makes the actionable set trustworthy.

**Recommended order:** §3 quick wins → §4 separate tracking-checkboxes from actions → §5 adopt the convention → §6 install central view → §7 migrate hot pages.

---

## 1. Where tasks live (open only)

| Bucket | Open count | Notes |
|---|---|---|
| `pages/` topic-projects | 34 / 40 | Spread over 22 pages, max 5 in `父母的签证.md` |
| `journals/` daily notes | 3 / 40 | All from `2026-04-14.md` — captured but never triaged out |
| `inbox/Main Kanban.md` | 1 / 40 | Single backlog item; column structure unused |
| `.obe/undo/...` (recoverable trash) | 1 / 40 | Parser noise — not real. See §8. |
| `pages/` hot-spots | — | `父母的签证` 5, `中世纪传说故事` 3, `Mary Flannery O'Connor` 3, `English Listening` 3 |

The shape is **action items distributed at the point of context**. That's a defensible pattern — but it requires a query view to see them all at once, which doesn't currently exist.

## 2. Stale opens (highest-confidence triage)

Real overdue items, dates in the file:

| File | Task | Signal |
|---|---|---|
| `pages/Mary Flannery O'Connor.md` | `Read 上升的一切必将汇合 📅 2025-09-01` | 8 months overdue |
| `pages/遗书.md` | `Two people to witness 📅 2025-10-02` | 7 months overdue. Likely resolved IRL, never checked off. |
| `pages/团聚移民.md` | `Apply ⏳ 2026-12-31` | Future scheduled — fine, leave alone |
| `journals/2026-04-14.md` | 3 open spiritual-life items | Captured in daily note ~1 month ago, never moved to a page or kanban |

Likely-dead (no date, but reads as abandoned):

- `pages/新年问候.md` → `Reading🔽` — single-word task, no context
- `pages/Visa & Permit.md` → `申请美国签证 / 美签/⏬` — lowest priority, low-effort marker
- `pages/Reading.md` → `规范我的阅读习惯和计划 🔽` — meta-aspiration, not an action
- `pages/阿佛洛狄忒.md` → `- [ ]` (empty checkbox, literally no text) — a bug, delete the line

## 3. Quick-win fixes (mechanical, ~10 minutes)

In order:

1. Delete the empty checkbox in `pages/阿佛洛狄忒.md`.
2. Close the overdue items in §2 row 1–2 — verify with yourself whether the underlying thing actually happened, then check `[x]` (or delete if abandoned).
3. Move the 3 open items from `journals/2026-04-14.md` to their natural home pages (or delete if they're not real intents) — daily journals are for capture, not storage.
4. Either fill the empty Kanban columns with current WIP, or delete `inbox/Main Kanban.md`. An empty hub is worse than no hub.

## 4. Separate "actions" from "tracking checkboxes"

A meaningful portion of `- [x]` in this vault are not completed actions — they are **completion logs** where checking the box meant "I read this / I have this / I cited this." Examples:

- [x] `pages/Fairy Tale.md`: 8 boxes are a bibliography (Tolkien books cited).
- [x] `pages/Permanent Residency.md`: sub-checkboxes are a document-gathering log (`hukou`, `passport`, `fingerprints`, …).
- `pages/Mark 9 14 to 29.md`: `- [ ] read` — Bible-reading marker, not a project task.

**Why it matters:** these inflate the "done" count, they pollute kbparser output if it surfaces tasks, and they make the Tasks-plugin query view useless (a "completed today" query will hit a Tolkien citation).

**Recommendation — pick one of these per case:**

- Convert to plain bullet list (`-` not `- [x]`). Best for bibliographies and reference logs.
- Move to a `log:` block (Dataview-friendly) or a frontmatter field. Best for tracked-but-recurring lists.
- Keep as checkbox only if there's a genuine *future* action embedded ("read X by Y").

This is a one-pass judgment call, ~6 files. Worth the time — it fixes both the open-set trustworthiness and the parser hand-off in §8.

## 5. Recommended convention

Adopt Tasks-plugin syntax **uniformly** for anything you call an action:

```
- [ ] <action verb + object> 📅 YYYY-MM-DD ⏫|🔼|🔽|⏬ #context
```

Rules:

- **Verb-led.** "Read X", "Apply for Y", "Email Z" — not "X" or "Reading".
- **Either dated or prioritized.** A task with neither is a candidate for deletion or for a `someday/` page.
- **`#context` tags are optional but useful.** `#errand`, `#phone`, `#computer`, `#waiting-on` — they let the query view group by where/how, which is what makes GTD-style review work.
- **One language per task is fine.** Mixing is fine. Don't worry about it.
- **Bookmarks are not tasks.** A bare URL behind `- [ ]` should be either a list bullet, a wikilink to a real "read later" page, or a real action ("Read <URL> 📅 …").
- **Wikilink stubs are not tasks.** `- [ ] [[圣女贞德]]` means "write this page someday" — put it on a `someday/Notes to write.md` list as a bullet, not a checkbox.

## 6. Central view: `inbox/Next Actions.md`

Replace the empty `Main Kanban.md` with a single Tasks-plugin query file:

```markdown
# Next Actions

## Due / overdue
` ` `tasks
not done
(due before in 14 days) OR (due before today)
sort by due
` ` `

## Active, prioritized
` ` `tasks
not done
priority is above none
no due date
sort by priority
` ` `

## Capture from journals
` ` `tasks
not done
path includes journals/
` ` `

## Everything else (review weekly)
` ` `tasks
not done
no due date
priority is none
path does not include journals/
` ` `
```

(Remove the backtick spaces — they're escaped here so this doc renders cleanly.)

This gives you one file to open in the morning. Tasks stay physically at the point of context; the view is computed.

Optional: keep `Main Kanban.md` for **current week's WIP only** — populate it Monday, drain it Friday. If you won't, delete it.

## 7. Migration plan

Two passes — ordered easiest-to-hardest:

**Pass A — clean-up (one sitting, ~30 min):**
1. §3 quick wins (4 mechanical fixes).
2. §4 tracking-vs-action separation on the 5–6 files identified.
3. Walk §2 stale opens and either close, delete, or re-date.

**Pass B — convention adoption (top pages first, expand opportunistically):**
1. Pick the 5 hot pages with open work: `父母的签证`, `Mary Flannery O'Connor`, `English Listening`, `Home Purchase`, `遗书`.
2. Rewrite each open task in §5 form (verb-led, dated or prioritized, optional context).
3. Create `inbox/Next Actions.md` with the §6 queries.
4. Decide on `Main Kanban.md` fate (use or delete).
5. Going forward, apply the convention only when you touch a page. Don't sweep the long tail.

Estimated total effort: 60–90 minutes including Pass B.

## 8. Hand-off to kbparser (preview for next spec)

This review's conventions are what makes the kbparser feature tractable. When we spec it, the parser should:

- Extract checkbox lines from `*.md` files under each vault root.
- Parse the Tasks-plugin grammar — `📅 due`, `⏫🔼🔽⏬` priority, `✅ done date`, `🛫 scheduled`, `➕ created`.
- Distinguish **action** vs **tracking** by heuristic: an action has a verb-leading description or any Tasks-plugin metadata; a tracking checkbox has neither and lives in a list under a heading like `## References`, `## Books`, etc. We can refine after Pass A reduces the noise.
- Skip `.obe/undo/`, `.trash/`, `attachments/`, `assets/` — these surface non-vault content. The current parser likely already does this; verify before adding the task extractor.
- Emit `tasks.json` (machine, for LLM context) and `tasks.md` (human, mirror of `Next Actions.md` so you can sanity-check what the parser saw).

We'll do that spec separately once Pass A is done — if you fix the noise first, the parser doesn't need elaborate heuristics.

---

## Appendix: signals used in this review

```bash
# All checkbox lines (open + closed)
grep -rE '^\s*-\s*\[[ x]\]' /knowledge-base/obsidian/personal --include='*.md'

# Open only, grouped by file
grep -rEc '^\s*-\s*\[ \]' /knowledge-base/obsidian/personal --include='*.md' | grep -v ':0$' | sort -t: -k2 -nr

# Tasks-plugin metadata usage
grep -rE '(📅|⏫|🔼|🔽|🔺|⏬|✅|❌|🛫|➕)' /knowledge-base/obsidian/personal --include='*.md'
```
