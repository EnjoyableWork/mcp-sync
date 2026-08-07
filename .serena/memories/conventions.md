# Conventions
- Write the product name as lowercase `mcp-sync`.
- Keep user-facing paths, commands, configuration keys, filenames, and protocol values in Markdown code formatting.
- Use fenced `bash` blocks for shell examples and fenced `text` blocks for ASCII architecture diagrams.
- Preserve `README.md` as the north-star public product and marketing page; it
  intentionally describes the finished experience. Use `PROJECT.md`, code,
  tests, and releases for current implementation truth.
- Follow the strict main-story ticket sequence in `PROJECT.md`. `SIDE-NNN`
  tickets must remain optional and cannot gate main-story correctness, safety,
  milestones, or releases.
- Each main-story ticket has an exact canonical goal objective in `PROJECT.md`.
  Goal-capable agents reconcile the active thread goal before implementation;
  agents without Goal mode use the same objective as their task contract.
