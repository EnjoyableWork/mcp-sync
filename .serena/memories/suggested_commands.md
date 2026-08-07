# Suggested commands
- Build: `cargo build --locked`.
- Run the implemented CLI surface: `cargo run --locked -- --help` or
  `cargo run --locked -- --version`.
- Format check: `cargo fmt --all -- --check`.
- Lint: `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`.
- Test: `cargo test --workspace --all-targets --all-features --locked`.
- Install from the checkout into an isolated root when verifying packaging:
  `cargo install --locked --path . --root <temporary-directory>`.
- README commands other than help/version remain target product examples.
- For documentation-only checks, also use `mem:task_completion`.
