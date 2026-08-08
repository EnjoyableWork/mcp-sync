# Source-checkout usage and recovery guide

This guide describes the currently implemented source-checkout behavior of
`mcp-sync`: the completed M1 foundation plus the global Windsurf, native VS
Code, and global Codex adapters added by `MCP-014` through `MCP-016`. It is the
operational companion to the
[north-star README](../README.md), not a replacement for that product
specification. Use it when building from source on macOS and reconciling the
five implemented global targets: Claude Desktop, Cursor, Windsurf's legacy
Cascade configuration, VS Code's native default user profile, and the global
Codex host configuration shared by the ChatGPT desktop app, Codex CLI, and
Codex IDE extension.

## Current supported journey

| Area | Currently implemented behavior |
| --- | --- |
| Platform | macOS |
| Canonical format | Strict JSON schema version `1` for local STDIO servers |
| Client targets | Global Claude Desktop, global Cursor, global Windsurf legacy Cascade configuration, native VS Code default user profile, and global Codex host configuration |
| Commands | `init`, `add`, `list`, `sync --dry-run`, and `sync` |
| Safety | Structural redaction, plan-first validation, atomic replacement, recoverable backups, no-op detection, and reverse-order transaction rollback |
| Installation | Build and run from a source checkout |

`init`, `sync --dry-run`, and `sync` are configuration operations. They never
start a configured MCP server. Health testing, additional platforms, packaged
installation, explicit prune behavior, and a built-in restore command remain
later work tracked in
[PROJECT.md](../PROJECT.md).

## Build and verify the checkout

Use the repository-selected stable Rust toolchain and committed lockfile:

```bash
cargo build --locked
./target/debug/mcp-sync --help
./scripts/check.sh
cargo deny --all-features --locked check
```

The examples below use `./target/debug/mcp-sync`. The installed executable is
named `mcp-sync` once a verified distribution channel exists.

## Files managed on macOS

| Purpose | Path |
| --- | --- |
| Canonical configuration | `$XDG_CONFIG_HOME/mcp-sync/config.json` when `XDG_CONFIG_HOME` is a non-empty absolute path; otherwise `$HOME/.config/mcp-sync/config.json` |
| Claude Desktop global target | `$HOME/Library/Application Support/Claude/claude_desktop_config.json` |
| Cursor global target | `$HOME/.cursor/mcp.json` |
| Windsurf global legacy Cascade target | `$HOME/.codeium/windsurf/mcp_config.json` |
| VS Code native default user-profile target | `$HOME/Library/Application Support/Code/User/mcp.json` |
| Codex global host target | `$HOME/.codex/config.toml` |

Project-level `.cursor/mcp.json` files are outside the ownership boundary and
remain untouched. `mcp-sync` also preserves unknown top-level data and fields
outside `command`, `args`, and `env` in compatible local server entries.
Commandless Cursor and Windsurf entries remain unmanaged, including native
remote URLs, headers, authentication options, and tool settings. VS Code root
`servers` entries are managed only when they are compatible local STDIO
definitions with a string `command`, string-array `args`, and string-valued
`env`. Remote transports, opaque entries, and otherwise valid local entries
with number or null environment values remain unmanaged; root `inputs`,
`sandbox`, and unknown fields are preserved.

The Codex boundary is exactly the global TOML file above. Compatible local
STDIO entries under `mcp_servers` contribute only `command`, `args`, and `env`
to canonical state. Comments, formatting, unrelated root settings, unowned
local fields, remote HTTP/OAuth entries, headers, authentication settings,
unknown or mixed transports, and target-only entries remain structural TOML
owned by Codex. A colliding canonical local name is refused before mutation.
Trusted-project `.codex/config.toml` layers have higher precedence and are
neither discovered nor mutated, so a global sync does not claim to override
them. OAuth credential stores are never accessed. This boundary follows the
[Codex configuration precedence](https://developers.openai.com/codex/config-basic/#configuration-precedence)
and [Codex MCP configuration](https://developers.openai.com/codex/mcp/)
contracts; fixture and built-binary evidence is not a current-client smoke
claim.

The VS Code boundary is exactly the native default user-profile file above. It
does not discover or mutate workspace `.vscode/mcp.json` or `.mcp.json`, named
profiles under `Code/User/profiles`, remote profiles, VS Code Insiders,
portable installations, Cline or Roo Code extension stores, Cline's shared
settings, or Agent Host/Copilot CLI configuration. The native user-profile
contract follows the [VS Code MCP configuration
reference](https://code.visualstudio.com/docs/agents/reference/mcp-configuration);
it is not a claim that extension-owned formats are interchangeable.

[Current vendor documentation](https://docs.windsurf.com/windsurf/cascade/mcp)
identifies Windsurf's `mcp_config.json` as the legacy Cascade agent
configuration. This adapter does not discover or claim support for the
separate Devin Local agent configuration.

Canonical and native configuration can contain literal credentials. Their
`.bak` files contain the same private material. Protect all of these files with
the same care as the original secret source, and do not paste their complete
contents into terminal output, bug reports, or chat.

## Safe first import

Quit Claude Desktop, Cursor, Windsurf, VS Code, and active Codex hosts before a
first import so their native files stay stable while they are read. `init`
reads all five global targets and creates the canonical file only when it does
not already exist:

```bash
./target/debug/mcp-sync init
```

The operation has these outcomes:

- Compatible local definitions are imported deterministically. An identical
  definition shared by multiple clients is imported once.
- Conflicting definitions stop the operation without creating canonical state.
  The diagnostic names the server, clients, and differing field categories,
  but not commands, arguments, or environment values.
- Named unmanaged Cursor, Windsurf, VS Code, and Codex entries are preserved
  in their native files and reported as skipped because canonical schema v1
  cannot represent them.
- A local definition that collides with an unmanaged entry is an error.
- An existing canonical path is never overwritten. Move it aside only after
  deciding which copy is authoritative; do not delete it merely to make
  `init` succeed.
- All five native client files are read-only during `init`; excluded project,
  profile, credential, extension-owned, and alternate-product files are never
  accessed.

After a successful import, inspect the redacted catalog:

```bash
./target/debug/mcp-sync list
```

`list` shows escaped server names, argument counts, and escaped environment key
names. It does not show commands, argument contents, or environment values.

## Add or replace one canonical definition

One `add` invocation supplies a complete definition. Repeating `--arg`
preserves literal argument order, and each `--env` value splits at only its
first `=`:

```bash
./target/debug/mcp-sync add project-files \
  --command example-mcp-server \
  --arg --transport \
  --arg stdio \
  --env ACCESS_MODE=read-only
```

`--cmd` is an alias for `--command`. There is no shell parsing, interpolation,
or environment expansion. Updating an existing name replaces its complete
`command`, `args`, and `env` definition; omitted arguments or environment
assignments therefore become empty collections.

Input is validated before canonical state is read. A semantic no-op preserves
the canonical bytes and any existing backup. A changed canonical regular file
is replaced atomically after its exact prior bytes are written to
`config.json.bak`.

## Preview and apply target changes

Always review the complete plan before applying it:

```bash
./target/debug/mcp-sync sync --dry-run
```

Dry-run validates the canonical file and all five native documents, renders
and reparses every proposed output, and reports every target without changing
a file or creating a backup. The plan can contain:

- `add` for a canonical server missing from a target;
- `update` for a compatible local entry whose owned fields differ;
- `no-op` for an exact normalized match;
- `drift` for a target-only entry that will be preserved; and
- unmanaged Cursor, Windsurf, VS Code, and Codex names that remain
  structurally untouched.

The report exposes names, counts, environment key names, and changed-field
categories only. It never prints process values.

After the plan is understood, quit Claude Desktop, Cursor, Windsurf, VS Code,
and active Codex hosts and apply it:

```bash
./target/debug/mcp-sync sync
```

Apply consumes the already validated plan; it does not recalculate a different
desired state. Existing changed targets receive exact `.bak` files before
same-directory atomic replacement. Missing changed targets are created without
a prior-file backup. Target-only entries, unowned native fields, unmanaged
Cursor, Windsurf, VS Code, and Codex entries, and excluded
project/profile/credential/extension files remain untouched.

Reopen the clients only after `sync` finishes. Then repeat both checks:

```bash
./target/debug/mcp-sync sync --dry-run
./target/debug/mcp-sync sync
```

A settled configuration reports all five targets unchanged. Neither command
rewrites native bytes or replaces existing backups for a no-op.

## Backup and transaction behavior

The current implementation uses one adjacent backup slot per existing file:

| Changed file | Backup path |
| --- | --- |
| Canonical configuration changed by `add` | `<canonical-config>.bak` |
| Claude Desktop changed by `sync` | `$HOME/Library/Application Support/Claude/claude_desktop_config.json.bak` |
| Cursor changed by `sync` | `$HOME/.cursor/mcp.json.bak` |
| Windsurf changed by `sync` | `$HOME/.codeium/windsurf/mcp_config.json.bak` |
| VS Code changed by `sync` | `$HOME/Library/Application Support/Code/User/mcp.json.bak` |
| Codex changed by `sync` | `$HOME/.codex/config.toml.bak` |

Each changed write replaces the regular `.bak` with the bytes observed
immediately before that write. A no-op leaves the backup untouched. If a longer
history is important, copy the current file and its backup to a separate,
access-controlled location before making another change.

`sync` is one five-target transaction. Claude Desktop is applied first, Cursor
second, Windsurf third, VS Code fourth, and Codex fifth. If a later target
fails, earlier changes are rolled back in reverse order:

- an updated file and any backup that existed before the transaction are
  restored exactly;
- a newly created target is removed only if its bytes are still the bytes
  written by that transaction; and
- a concurrent change is never silently overwritten.

The command remains unsuccessful after any apply or rollback failure. Read the
per-target outcomes before retrying.

## Failure and recovery playbook

### Import conflict or malformed input

`init` and `sync --dry-run` finish all validation before mutation. On a
conflict, malformed JSON or TOML, unsupported canonical schema version, or
unreadable path:

1. Keep all client applications closed.
2. Preserve the files involved before editing them.
3. Correct the named file or make the conflicting definitions identical,
   rename one, or remove one intentionally.
4. Rerun `init` or `sync --dry-run` before any apply.

Do not diagnose the problem by printing complete files. Compare the named
server and field categories with an editor that does not publish their values.

### Permission, file-type, backup, or concurrent-change refusal

`mcp-sync` refuses symbolic links, directories, other non-regular managed
paths, changed observed bytes, and an unsafe backup path. It also returns
non-zero when a required directory or file cannot be read or written.

Inspect the exact path named by the diagnostic. Do not work around the guard by
loosening broad directory permissions or replacing a symlink target. Close the
clients, resolve ownership or the specific collision, and generate a new
dry-run plan. A stale plan is deliberately not force-applied.

### Apply failure with successful rollback

When every earlier mutation reports that its pre-transaction target and backup
state was restored, the command has already recovered the transaction. Keep
the clients closed, correct the later target failure, and start again with
`sync --dry-run`.

### Rollback failure

Stop immediately when an outcome says rollback failed. Do not rerun `sync`,
because another changed write may replace the most useful `.bak` evidence.

1. Keep all clients closed.
2. Preserve the current target and every adjacent `.bak` in a separate,
   access-controlled location.
3. Use the target and recovery-backup path named in the diagnostic to decide
   which prior state is authoritative.
4. Restore only after checking file type, syntax, and the intended bytes.
5. Run `sync --dry-run` after recovery; apply only when the new plan is
   understood.

## Manual restoration from an adjacent backup

There is no built-in restore command. The following macOS `zsh` procedure
performs a guarded, same-directory replacement for an existing JSON target.
Use it for the canonical file or one of the four JSON clients, not for Codex
TOML. Set `target` to exactly one applicable path from the table above. Keep
the clients closed and first preserve both current files somewhere
access-controlled if there is any uncertainty about which state to keep.

```zsh
(
set -euo pipefail

target="$HOME/.cursor/mcp.json"
backup="${target}.bak"

if [[ ! -f "$target" || -L "$target" || ! -f "$backup" || -L "$backup" ]]; then
  print -u2 'target and backup must both be regular files'
  exit 1
fi

stage=$(mktemp "${target}.restore.XXXXXX")
trap 'rm -f -- "$stage"' EXIT

cp -p -- "$backup" "$stage"
plutil -convert json -o /dev/null -- "$stage"
cmp -s -- "$backup" "$stage"
mv -f -- "$stage" "$target"

trap - EXIT
)
```

For Claude Desktop, replace the `target=` line with:

```zsh
target="$HOME/Library/Application Support/Claude/claude_desktop_config.json"
```

For Windsurf, replace the `target=` line with:

```zsh
target="$HOME/.codeium/windsurf/mcp_config.json"
```

For VS Code's native default user profile, replace the `target=` line with:

```zsh
target="$HOME/Library/Application Support/Code/User/mcp.json"
```

For the default canonical path, replace the `target=` line with:

```zsh
target="$HOME/.config/mcp-sync/config.json"
```

If `XDG_CONFIG_HOME` selected the canonical root, use that absolute path
instead. Validate canonical state afterward with
`./target/debug/mcp-sync list`; validate target state and see the resulting
difference with `./target/debug/mcp-sync sync --dry-run`.
Restoring a target without restoring the canonical definition normally creates
an expected update plan that would reapply the canonical state.

### Restore Codex TOML

For `$HOME/.codex/config.toml`, use the following TOML-specific variant. It
requires Python 3.11 or newer for its standard-library `tomllib` validator. If
that module is unavailable, stop and obtain an equivalent trusted TOML parser;
do not skip validation.

```zsh
(
set -euo pipefail

target="$HOME/.codex/config.toml"
backup="${target}.bak"
toml_python=python3 # set to an installed Python 3.11+ executable when needed

if [[ ! -f "$target" || -L "$target" || ! -f "$backup" || -L "$backup" ]]; then
  print -u2 'target and backup must both be regular files'
  exit 1
fi
if ! "$toml_python" -c 'import tomllib' >/dev/null 2>&1; then
  print -u2 'TOML restoration requires Python 3.11+ with tomllib'
  exit 1
fi

stage=$(mktemp "${target}.restore.XXXXXX")
trap 'rm -f -- "$stage"' EXIT

cp -p -- "$backup" "$stage"
"$toml_python" - "$stage" <<'PY'
import sys
import tomllib

with open(sys.argv[1], "rb") as document:
    tomllib.load(document)
PY
cmp -s -- "$backup" "$stage"
mv -f -- "$stage" "$target"

trap - EXIT
)
```

Afterward, `./target/debug/mcp-sync sync --dry-run` reparses the restored
global document through the same Codex adapter and reports any intended
canonical difference. It still does not inspect a trusted-project layer or an
OAuth credential store.

Absence of a `.bak` does not authorize deleting a target. It may mean the file
was newly created, was never changed, or its backup was moved. Restore from a
known external copy or repair the exact native document; do not infer ownership
and prune entries from backup absence.

## Current operational limitations

The following are delivery facts, not changes to the README's intended product
promise:

- The current implementation is verified on macOS only and runs from a Rust
  source checkout.
- Only global Claude Desktop, global Cursor, Windsurf's global legacy Cascade
  JSON, VS Code's native default user-profile JSON, and global Codex TOML are
  managed. Windsurf, VS Code, and Codex have fixture and built-binary evidence
  but no current-client smoke claim. Linux and Windows remain later main-story
  work.
- Canonical schema v1 represents local STDIO definitions with `command`,
  ordered `args`, and literal `env` only. Remote transports, OAuth, working
  directories, and secret references are not canonical capabilities yet.
- There is no `mcp-sync test` command. `init` and `sync` never execute server
  commands; bounded MCP health testing is later work.
- Target-only definitions are drift and are never deleted. There is no prune
  command.
- Backups use one adjacent slot. There is no retention policy, backup history,
  or built-in restore workflow yet.
- No GitHub Release, Homebrew, WinGet, or Cargo publication has been verified
  for this repository yet.

See [PROJECT.md](../PROJECT.md) for the ordered implementation record and the
evidence required before any later capability is described as delivered.
