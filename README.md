# Knowledge Base Parser

Parse Obsidian vaults and Scrivener projects into compact context files for
LLM consumption.

## Usage

```bash
./run.sh
```

By default, sources are read from `/knowledge-base` and output is written to
`/common/artifacts/context`. Override the base directory with `KB_SOURCE_ROOT`:

```bash
KB_SOURCE_ROOT=~/Documents/knowledge-base ./run.sh
```

Every path can also be overridden individually, and the output directory can be
changed with `KB_OUTPUT_DIR`:

| Variable | Default |
| --- | --- |
| `KB_SOURCE_ROOT` | `/knowledge-base` |
| `KB_OBSIDIAN_IT` | `$KB_SOURCE_ROOT/obsidian/it` |
| `KB_OBSIDIAN_PERSONAL` | `$KB_SOURCE_ROOT/obsidian/personal` |
| `KB_SCRIVENER_DELAND` | `$KB_SOURCE_ROOT/scrivener/Deland.scriv` |
| `KB_SCRIVENER_SHORT` | `$KB_SOURCE_ROOT/scrivener/短篇写作.scriv` |
| `KB_SCRIVENER_SUPERPOWERS` | `$KB_SOURCE_ROOT/scrivener/超能力者.scriv` |
| `KB_OUTPUT_DIR` | `/common/artifacts/context` |

The script validates every source before writing anything, so a missing mount
will fail instead of replacing existing context files with empty output.

## Direct CLI

```bash
./target/debug/kbparser --vault /path/to/vault --scrivener /path/to/project.scriv --output ./out
```
