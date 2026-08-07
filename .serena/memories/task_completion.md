# Task completion
- Run `cargo fmt --all -- --check`.
- Run `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`.
- Run `cargo test --workspace --all-targets --all-features --locked`.
- For CLI or packaging changes, build with `cargo build --locked` and exercise
  affected behavior through the built or isolated-installed `mcp-sync` binary.
- For documentation changes, also run `git diff --check` and inspect heading
  hierarchy, tables, code fences, local links, claims, and trailing whitespace.
