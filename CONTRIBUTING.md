# Contributing to mcp-sync

Thank you for helping improve `mcp-sync`. Contributions may be code,
documentation, defect reports, feature proposals, or clear descriptions of a
usage obstacle.

Participation in project spaces is governed by the
[Code of Conduct](CODE_OF_CONDUCT.md). This guide is the public contribution
and support contract for the project.

## Choose the right public path

Search the [existing issues](https://github.com/EnjoyableWork/mcp-sync/issues)
before opening a new one.

- Report a reproducible defect with the
  [bug form](https://github.com/EnjoyableWork/mcp-sync/issues/new?template=01-bug-report.yml).
- Propose a change or describe a usage obstacle with the
  [feature form](https://github.com/EnjoyableWork/mcp-sync/issues/new?template=02-feature-request.yml).
- Submit an implementation through a focused pull request.
- Request private follow-up about a conduct concern with the
  [conduct contact form](https://github.com/EnjoyableWork/mcp-sync/issues/new?template=03-conduct-contact.yml).

GitHub Issues and pull-request conversations are the project's public
discussion mechanisms. They are handled on a best-effort basis and do not
provide a private support channel or guaranteed response time. The latest
public release is the only supported release; `main` and other source snapshots
are development versions. Questions about client behavior outside
`mcp-sync`'s documented ownership boundary should normally go to that client's
project.

## Keep security reports and sensitive data private

Do not report a suspected vulnerability in an issue, pull request, discussion,
or conduct request. Follow [SECURITY.md](SECURITY.md) and use
[GitHub private vulnerability reporting](https://github.com/EnjoyableWork/mcp-sync/security/advisories/new).

Never submit credentials, tokens, headers, environment values, private paths,
user data, complete configuration files, or raw unreviewed logs. Reproduce
problems with generic server names and synthetic, redacted values. Include only
the minimum structure needed to explain a problem; environment key names may
also be sensitive and should be renamed when their names reveal private
context.

## Report a defect

A useful defect report identifies:

- the exact `mcp-sync` version or development commit;
- installation channel, operating system, architecture, client, and affected
  command;
- minimal reproduction steps using a disposable or synthetic configuration;
- expected and actual behavior in redacted terms; and
- whether the result is deterministic and whether recovery was required.

Do not reproduce against real user configuration merely to improve a report.
If the bug concerns data loss, redaction, conflict handling, backup behavior,
or rollback, say so without publishing the affected data. The bug form routes
security concerns to the private path before it requests diagnostic detail.

## Propose a change or usage improvement

Use the feature form to explain the user problem, desired outcome, alternatives,
and any configuration-safety or compatibility consequences. A proposed change
is not accepted merely because an issue exists. Maintainers use the public
thread to clarify scope and decide whether it belongs in the ordered delivery
plan in [PROJECT.md](PROJECT.md).

Small corrections may go directly to a pull request. Material product changes
should have a linked issue or accepted ticket before implementation so the
scope, evidence, and ordering are explicit.

## Prepare a source checkout

Install the current stable Rust toolchain and clone the repository over HTTPS:

```bash
git clone https://github.com/EnjoyableWork/mcp-sync.git
cd mcp-sync
cargo build --locked
cargo run --locked -- --help
```

Tests must use the repository's synthetic-home boundaries. Do not point an
automated test at real MCP client configuration.

## Make and verify a change

Keep a pull request focused on one accepted outcome and preserve unrelated
work. Follow Conventional Commits with an imperative summary, such as
`fix(sync): preserve target-only fields`.

Add the narrowest useful regression test for changed behavior. A defect that
could lose configuration, expose sensitive data, mishandle conflicts, or break
rollback requires a focused regression test.

Run the native quality gate for your platform:

```bash
./scripts/check.sh
cargo deny --all-features --locked check
```

On Windows PowerShell:

```powershell
.\scripts\check.ps1
cargo deny --all-features --locked check
```

The substantive-change handoff checks are:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
cargo deny --all-features --locked check
```

Documentation-only changes must also pass `git diff --check`; verify headings,
tables, relative links, code fences, and current-state claims. Run any focused
schema, release, or repository-control checks affected by the change.

## Submit a pull request

Open the pull request against `main` and complete the repository template. Link
the issue or ticket, describe user-visible and safety effects, list the exact
checks run, and call out anything that remains unverified. Do not put a private
security report or sensitive diagnostic in the pull request.

Every pull request must satisfy the protected-branch checks and resolve review
conversations before merge. Maintainers may request a smaller scope, stronger
evidence, or changes needed to preserve configuration and redaction
invariants. Maintainers perform the final merge.

## Contribution licensing, CLA, and DCO

The project uses the [MIT License](LICENSE). By intentionally submitting a
contribution for inclusion, you agree that it is provided under the same MIT
License and represent that you have the right to provide it. Do not submit code,
documentation, or other material whose terms are incompatible with that
license.

The project does not require a Contributor License Agreement (CLA) and does not
require copyright assignment. It also does not currently require a Developer
Certificate of Origin (DCO) `Signed-off-by` line. Those are separate policy
mechanisms: the no-CLA choice does not silently create a DCO requirement, and a
DCO sign-off would not be treated as a CLA. Adopting either mechanism later
requires a recorded policy decision, updated guidance, and compatible
enforcement before it applies to contributions.
