# Documentation contract

## Knowledge layers

- `raw/`: immutable USB-monitor evidence and its source catalog.
- Maintained pages: evidence-backed synthesis of current behavior and explicit unknowns.
- `index.md`: content-oriented first read.
- `log.md`: append-only, newest-first record of meaningful documentation and protocol changes.

## Metadata

Maintained pages use YAML frontmatter with `title`, `type`, `status`, `updated`, and repository-relative `sources`. Status is one of `current`, `target`, `mixed`, or `superseded`.

## Rules

- Executable code and raw traces are stronger evidence than prose.
- Mark packet semantics as observed, inferred, or unknown.
- Never silently relabel an old capture after new evidence; add a correction to `log.md`.
- Keep raw traces immutable and add a new dated capture for each experiment.
- Do not publish credentials, serial numbers, installers, or other access-enabling material.
- Keep unfinished implementation work out of Markdown task lists. Use the repository task tracker if one is introduced.
- Validate with `scripts/validate` and `git diff --check`.
