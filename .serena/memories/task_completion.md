# Task completion
- Run `./scripts/check.sh`; it is the canonical wrapper for formatting, Clippy,
  and the full locked test suite through disposable user-configuration roots.
- For CLI or packaging changes, build with `cargo build --locked` and exercise
  affected behavior through the built or isolated-installed `mcp-sync` binary.
- For documentation changes, also run `git diff --check` and inspect heading
  hierarchy, tables, code fences, local links, claims, and trailing whitespace.
