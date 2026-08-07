# Suggested commands
- Build: `cargo build --locked`.
- Run the implemented CLI surface: `cargo run --locked -- --help` or
  `cargo run --locked -- --version`.
- Canonical quality gate: `./scripts/check.sh`. It runs formatting, Clippy, unit,
  and integration checks with locked dependencies, a cleared environment, and
  disposable user-configuration roots.
- The underlying commands are `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`,
  and `cargo test --workspace --all-targets --all-features --locked`; use the
  script for normal handoff so they run through the synthetic home.
- Install from the checkout into an isolated root when verifying packaging:
  `cargo install --locked --path . --root <temporary-directory>`.
- README commands other than help/version remain target product examples.
- For documentation-only checks, also use `mem:task_completion`.
