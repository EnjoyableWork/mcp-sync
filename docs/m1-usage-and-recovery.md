# Source-checkout usage and recovery guide

This guide describes the currently implemented source-checkout behavior of
`mcp-sync`: the completed M1 foundation, the global Windsurf, native VS Code,
and global Codex adapters added by `MCP-014` through `MCP-016`, and the bounded
STDIO initialize health boundary added by `MCP-017`, the Linux path and
behavior support added by `MCP-018`, and the Windows source-checkout support
completed by `MCP-019`, plus the explicit restore and retention behavior added
by `MCP-020`, the cross-process mutation serialization added by `MCP-036`, and
the bounded global-user Kiro adapter with inherited Kiro Crew coverage added by
`MCP-037`.
It is the operational companion to the
[north-star README](../README.md), not a replacement for that product
specification. Use it when building from source on macOS, GNU/Linux, or Windows
and reconciling the six implemented global targets: Claude Desktop, Cursor,
Windsurf's legacy Cascade configuration, VS Code's native default user profile,
the global Codex host configuration shared by the ChatGPT desktop app, Codex
CLI, and Codex IDE extension, and Kiro's global-user configuration inherited by
Kiro Crew.

## Current supported journey

| Area | Currently implemented behavior |
| --- | --- |
| Platform | macOS plus native x64/ARM64 GNU/Linux and Windows MSVC source-checkout behavior |
| Canonical format | Strict JSON schema version `1` for local STDIO servers |
| Client targets | Global Claude Desktop, global Cursor, global Windsurf legacy Cascade configuration, native VS Code default user profile, global Codex host configuration, and global-user Kiro configuration inherited by Kiro Crew |
| Commands | `init`, `add`, `list`, `test`, `sync --dry-run`, `sync`, `restore <configuration> --dry-run`, and `restore <configuration>` |
| Safety | Structural redaction, bounded health-process execution, fail-fast cross-process mutation serialization, plan-first validation, target-first per-file abrupt-process recovery, one-generation recoverable backups, guarded restore, no-op detection, and reverse-order returned-failure rollback |
| Installation | Build and run from a source checkout |

Only `test` starts the one named canonical server. `init`, `sync`, and
`restore`, including their dry-run forms, remain configuration operations and
never start a configured MCP server. This guide deliberately uses a source
checkout; the verified zero-cost `v0.1.0` Cargo, source-building Homebrew, and
GNU/Linux archive boundaries are documented by the
[README](../README.md) and [PROJECT.md](../PROJECT.md). Project-issued macOS
and Windows binaries, WinGet, and explicit prune behavior remain later work.

## Build and verify the checkout

Use the repository-selected stable Rust toolchain and committed lockfile on
macOS or GNU/Linux:

```bash
cargo build --locked
./target/debug/mcp-sync --help
./scripts/check.sh
cargo deny --all-features --locked check
```

Use the native PowerShell quality gate on Windows:

```powershell
cargo build --locked
.\target\debug\mcp-sync.exe --help
.\scripts\check.ps1
cargo deny --all-features --locked check
```

The examples below use `./target/debug/mcp-sync`; on Windows PowerShell,
substitute `.\target\debug\mcp-sync.exe`. The installed executable is named
`mcp-sync` once a verified distribution channel exists.

## Files managed by platform

Canonical configuration uses a platform-local configuration root:

| Purpose | macOS and GNU/Linux | Windows |
| --- | --- | --- |
| Canonical configuration | `$XDG_CONFIG_HOME/mcp-sync/config.json` when `XDG_CONFIG_HOME` is a non-empty absolute path; otherwise `$HOME/.config/mcp-sync/config.json` | `%LOCALAPPDATA%\mcp-sync\config.json` |
| Persistent empty operation lock | Beside canonical state at `$XDG_CONFIG_HOME/mcp-sync/operation.lock`, or `$HOME/.config/mcp-sync/operation.lock` under the fallback | Beside canonical state at `%LOCALAPPDATA%\mcp-sync\operation.lock` |

Four clients retain home-relative default paths:

| Purpose | macOS and GNU/Linux | Windows |
| --- | --- | --- |
| Cursor global target | `$HOME/.cursor/mcp.json` | `%USERPROFILE%\.cursor\mcp.json` |
| Windsurf global legacy Cascade target | `$HOME/.codeium/windsurf/mcp_config.json` | `%USERPROFILE%\.codeium\windsurf\mcp_config.json` |
| Codex global host target | `$HOME/.codex/config.toml` | `%USERPROFILE%\.codex\config.toml` |
| Kiro global-user target | `$HOME/.kiro/settings/mcp.json` | `%USERPROFILE%\.kiro\settings\mcp.json` |

For Kiro only, a non-empty absolute `KIRO_HOME` without parent traversal
selects `<KIRO_HOME>/settings/mcp.json` on every platform. An unset or empty
value uses the home-relative default above. Relative values and values
containing `..` fail closed before discovery.

Claude Desktop and VS Code use the platform user-data root:

| Purpose | macOS | GNU/Linux | Windows |
| --- | --- | --- | --- |
| Claude Desktop global target | `$HOME/Library/Application Support/Claude/claude_desktop_config.json` | `$XDG_CONFIG_HOME/Claude/claude_desktop_config.json`, or `$HOME/.config/Claude/claude_desktop_config.json` when XDG is unset or empty | `%APPDATA%\Claude\claude_desktop_config.json` |
| VS Code native default user-profile target | `$HOME/Library/Application Support/Code/User/mcp.json` | `$XDG_CONFIG_HOME/Code/User/mcp.json`, or `$HOME/.config/Code/User/mcp.json` when XDG is unset or empty | `%APPDATA%\Code\User\mcp.json` |

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

The Kiro boundary follows the current
[Kiro MCP configuration contract](https://kiro.dev/docs/mcp/configuration/)
and is exactly the selected global-user file above. Kiro accepts quoted-property
JSON with comments and trailing commas under root
`mcpServers`; duplicate object properties and malformed root or `mcpServers`
shapes fail closed. Compatible local STDIO entries contribute only a non-empty
literal string `command`, string-array `args`, and string-valued `env` to
canonical state. Comments, formatting, `disabled`, `autoApprove`,
`disabledTools`, `cwd`, `timeout`, unknown local fields, target-only entries,
and every root field remain unowned by `mcp-sync` and structurally preserved.

Entries with `url`, `headers`, `oauth`, `oauthScopes`, `type`, mixed or opaque
transports, non-string environment values, malformed owned fields, or comments
inside an owned `args` or `env` collection remain unmanaged. So does any entry
whose owned strings contain a valid `${VARIABLE}` reference. Kiro expands those
references recursively at launch time, while canonical v1 stores literal
strings; `mcp-sync` therefore never imports, rewrites, expands, or reports them
as ordinary local values. A colliding canonical local name is refused before
mutation.

Kiro agent definitions and workspace `.kiro/settings/mcp.json` files take
precedence over the global-user file and are never discovered or mutated.
Organization-managed policy, remote-service state, credentials, and alternate
stores are also excluded. Current
[Kiro CLI 2.3 behavior](https://kiro.dev/changelog/cli/2-3/) documents
`KIRO_HOME`; the current
Kiro IDE smoke uses the default `~/.kiro` location because the installed IDE
resolves that default directly. Current Kiro Crew reads the default global file
as an inherited source, lets `~/.kiro/crew/mcp.json` override it, and renders
its own `~/.kiro/agents/kirocrew.json` with `includeMcpJson: false`. Neither
Crew-specific file is a target, and `KIROCREW_HOME` does not relocate the
global Kiro source for this adapter. The Crew behavior is pinned to its current
[MCP architecture at `24a6f8e`](https://github.com/kirodotdev/KiroCrew/blob/24a6f8ee586350879f10a3629782c5cd0b31a8e9/docs/architecture/mcp.md).

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

`operation.lock` is different: it is persistent empty coordination metadata
and never contains a PID, command, environment value, configuration value, or
owner record. Do not write to it or delete it as a stale-lock remedy. The
operating system releases ownership when the holding process closes or exits;
the empty file intentionally remains for the next invocation.

## Safe first import

Quit Claude Desktop, Cursor, Windsurf, VS Code, active Codex hosts, Kiro IDE,
and Kiro Crew before a first import so their native files stay stable while
they are read. `init` reads all six global targets and creates the canonical
file only when it does not already exist:

```bash
./target/debug/mcp-sync init
```

The operation has these outcomes:

- Compatible local definitions are imported deterministically. An identical
  definition shared by multiple clients is imported once.
- Conflicting definitions stop the operation without creating canonical state.
  The diagnostic names the server, clients, and differing field categories,
  but not commands, arguments, or environment values.
- Named unmanaged Cursor, Windsurf, VS Code, Codex, and Kiro entries are preserved
  in their native files and reported as skipped because canonical schema v1
  cannot represent them.
- A local definition that collides with an unmanaged entry is an error.
- An existing canonical path is never overwritten. Move it aside only after
  deciding which copy is authoritative; do not delete it merely to make
  `init` succeed.
- All six native client files are read-only during `init`; excluded project,
  profile, workspace, agent, Crew-only, generated-agent, credential,
  extension-owned, and alternate-product files are never accessed.

The first mutating invocation may create the canonical directory and empty
`operation.lock` even when later import validation refuses to create
`config.json`. The coordination file is not canonical state and does not mean
initialization succeeded.

After a successful import, inspect the redacted catalog:

```bash
./target/debug/mcp-sync list
```

`list` shows escaped server names, argument counts, and escaped environment key
names. It does not show commands, argument contents, or environment values.

## Test one canonical STDIO server

Run the explicit health boundary only after reviewing the selected definition:

```bash
./target/debug/mcp-sync test project-files
```

`test` validates canonical state, resolves exactly one named definition, and
executes its literal command and ordered arguments directly without a shell.
The child environment is cleared, then receives the canonical `env` entries
and only the caller's `PATH` when canonical state does not provide one. Child
stdin and stdout are dedicated protocol pipes; child stderr is discarded and
never treated as protocol or copied to terminal output.

The current handshake contract follows the final session-based
[MCP 2025-11-25 lifecycle](https://modelcontextprotocol.io/specification/2025-11-25/basic/lifecycle)
and [newline-delimited STDIO transport](https://modelcontextprotocol.io/specification/2025-11-25/basic/transports):

- `mcp-sync` sends one JSON-RPC `2.0` `initialize` request with numeric ID `1`,
  empty client capabilities, its package identity, and protocol version
  `2025-11-25`;
- one newline-delimited response is bounded to 1 MiB and five seconds;
- the response must be duplicate-free JSON, match the JSON-RPC version and
  request ID, contain either a structurally valid error or result, and provide
  object `capabilities`, string server identity fields, and a supported
  negotiated protocol version;
- negotiated `2025-11-25`, `2025-06-18`, `2025-03-26`, and `2024-11-05`
  handshake versions are accepted; and
- after a valid result, `mcp-sync` sends `notifications/initialized`, closes
  stdin, allows 500 milliseconds for clean exit, then force-terminates and
  reaps a child that does not stop.

A timeout, malformed or oversized message, mismatched response, server error,
failed notification, or unclean shutdown returns non-zero. Diagnostics report
only the named server, protocol categories, fixed limits, and operating-system
error categories. They never echo commands, arguments, environment values,
raw stdout, raw stderr, JSON-RPC error messages, error data, or unchecked
protocol-version text. Cleanup runs on every success and failure path; a
cleanup failure is combined with the original failure instead of being hidden.

This command deliberately implements the initialize-based compatibility
boundary required by `MCP-017`. It does not claim validation of the stateless
`2026-07-28` protocol, remote HTTP transports, tools, resources, prompts,
OAuth, or server behavior after initialization.

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

Input is validated before the operation lock or canonical state is accessed.
After validation, `add` owns the same mutation lock as every other writer while
it reads and conditionally replaces canonical state. A semantic no-op
preserves the canonical bytes and any existing backup. A changed canonical
regular file uses the target-first journal protocol below: the exact prior
target is staged while the existing `config.json.bak` remains authoritative,
the target becomes the commit point, and only then is the prior target
published as the new backup.

## Preview and apply target changes

Always review the complete plan before applying it:

```bash
./target/debug/mcp-sync sync --dry-run
```

Dry-run validates the canonical file and all six native documents, renders
and reparses every proposed output, and reports every target without changing
a file, creating a backup, or acquiring the mutation lock. The plan can
contain:

- `add` for a canonical server missing from a target;
- `update` for a compatible local entry whose owned fields differ;
- `no-op` for an exact normalized match;
- `drift` for a target-only entry that will be preserved; and
- unmanaged Cursor, Windsurf, VS Code, Codex, and Kiro names that remain
  structurally untouched.

The report exposes names, counts, environment key names, and changed-field
categories only. It never prints process values.

After the plan is understood, quit Claude Desktop, Cursor, Windsurf, VS Code,
active Codex hosts, Kiro IDE, and Kiro Crew and apply it:

```bash
./target/debug/mcp-sync sync
```

The apply invocation acquires the mutation lock before planning and retains it
through every write and any reverse-order rollback. Apply then consumes the
already validated plan; it does not recalculate a different desired state.
Existing changed targets use a same-directory target-first transaction and
receive exact `.bak` files containing their immediately preceding bytes.
Missing changed targets are created without a prior-file backup. Target-only
entries, unowned native fields, unmanaged Cursor, Windsurf, VS Code, Codex,
and Kiro entries, and excluded
project/profile/workspace/agent/Crew/credential/extension files remain
untouched.

Reopen the clients only after `sync` finishes. Then repeat both checks:

```bash
./target/debug/mcp-sync sync --dry-run
./target/debug/mcp-sync sync
```

A settled configuration reports all six targets unchanged. Neither command
rewrites native bytes or replaces existing backups for a no-op.

## Concurrent mutating operations

One operating-system exclusive lock serializes every cooperating writer that
resolves the same canonical configuration root. The complete locked boundary
is:

- `init`, from before canonical and client discovery through create-only
  publication;
- valid `add`, from before canonical read through no-op or guarded replacement;
- non-dry-run `sync`, from before canonical and native planning through apply
  and any rollback; and
- non-dry-run `restore`, from before target and backup planning through no-op
  or guarded restoration.

If one of those commands already owns the root, another mutating invocation
returns non-zero immediately with
`another mutating mcp-sync operation is already in progress`; it does not read
managed configuration, wait, join the older plan, or write any target or
backup. Different canonical roots remain independent. `list`, `test`,
`sync --dry-run`, and `restore --dry-run` do not take the mutation lock. They
can run while a writer is active. Once an existing-file transaction journal is
published, a lock-free command that reads that affected file refuses it as
incomplete rather than planning from an intermediate generation. A lock-free
command that does not read the affected file can still run, so treat any view
produced during a writer as transient and repeat it after the writer finishes.

The lock coordinates `mcp-sync` processes. Native clients and editors do not
participate, so the existing exact-byte guards still refuse a target or backup
that an external process changes after planning.

## Backup and transaction behavior

The current implementation uses one adjacent backup slot per existing file:

| Changed file | Backup path |
| --- | --- |
| Canonical configuration changed by `add` | The resolved canonical path above with `.bak` appended |
| Any existing client target changed by `sync` | That platform's resolved target path above with `.bak` appended |
| Any existing managed file changed by `restore` | The selected target path with `.bak` appended; after success it contains the exact target bytes from immediately before restore |

Retention is exactly one adjacent generation per managed file. A successful
changed `add`, `sync`, or existing-target `restore` replaces that slot with the
exact target bytes observed immediately before the operation. There is no
time-based expiration, age policy, or automatic multi-version history. A
creation has no prior-file backup; recreating a missing target through
`restore` preserves its existing backup unchanged. A no-op, validation refusal,
failed mutation with successful compensation, or rolled-back `sync`
transaction leaves the pre-operation retention state intact. If longer history
is important, copy the current file and its backup to a separate,
access-controlled location before making another change.

Changed existing files share one target-first transaction protocol across
`add`, `sync`, and `restore`:

1. mcp-sync validates and snapshots the exact regular target and adjacent
   backup state.
2. It creates two private same-directory stages and publishes an adjacent
   `<target>.mcp-sync-transaction.json` journal before writing either stage.
   The versioned journal contains only byte counts, SHA-256 fingerprints, the
   transaction phase, and two traversal-free stage basenames; it never contains
   configuration bytes.
3. It writes and synchronizes the exact replacement and original-target stages,
   then durably advances the journal to `prepared`.
4. After rechecking the target, backup, journal, and stages, it atomically
   publishes the replacement target. This is the per-file commit point; the
   previous backup remains unchanged until then.
5. It publishes the exact original target as `.bak`, then removes only the
   journal-named regular stages whose fingerprints still match and cleans up the
   journal.

The stages contain private configuration bytes even though the journal does
not. Do not print, upload, or casually inspect either kind of recovery artifact.
This protocol covers abrupt mcp-sync process termination. It does not promise
whole-machine or power-loss durability, nor does it make all six target commits
atomic across termination.

During a running invocation, `sync` treats returned failures as one six-target
transaction. Claude Desktop is applied first, Cursor second, Windsurf third,
VS Code fourth, Codex fifth, and Kiro sixth. If a later target returns a
failure, earlier changes are rolled back in reverse order:

- an updated file and any backup that existed before the transaction are
  restored exactly;
- a newly created target is removed only if its bytes are still the bytes
  written by that transaction; and
- a concurrent change is never silently overwritten.

The command remains unsuccessful after any apply or rollback failure. Read the
per-target outcomes before retrying.

## Failure and recovery playbook

### Another mutating operation is in progress

Let the current `mcp-sync` invocation finish, then rerun the command from a new
dry-run where applicable. Do not delete `operation.lock`: an empty persistent
file is normal, and deleting a locked file can let two processes coordinate on
different file identities. If the earlier process exited or was terminated,
the operating system has already released its ownership even though the file
remains, so a retry can acquire it. A non-empty, symbolic-link, directory, or
otherwise non-regular lock artifact fails closed; replace such an artifact
only after confirming no `mcp-sync` operation is running and preserving it for
inspection when tampering or corruption is possible.

### An earlier process stopped during existing-file replacement

An abrupt exit can leave an adjacent
`<target>.mcp-sync-transaction.json` and one or two private
`.mcp-sync-XXXXXX.tmp` stages. Do not delete or rename them. `list`, `test`,
`sync --dry-run`, or `restore --dry-run` returns non-zero if it reaches the
affected file because lock-free commands never perform recovery.

Keep the clients closed and rerun a valid mutating command: `init`, `add`,
non-dry-run `sync`, or non-dry-run `restore`. After acquiring the canonical-root
operation lock and before planning, mcp-sync inspects all seven resolved managed
paths. If the target still has its original bytes, recovery aborts the pending
replacement and preserves the prior backup. If the target has the exact
replacement bytes, recovery publishes the exact original target as `.bak`. An
already complete target/backup pair needs cleanup only. Recovery is idempotent,
so retrying after another abrupt exit is safe.

Malformed metadata, a symbolic link or non-regular artifact, a missing or
fingerprint-mismatched required stage, or an external target or backup edit
fails closed without overwriting or deleting the ambiguous state. In that case:

1. Do not rerun `sync` repeatedly or remove a journal or stage by filename.
2. Preserve the target, `.bak`, journal, and journal-named stages in separate,
   access-controlled storage.
3. Resolve which generation is authoritative without printing configuration
   values or fingerprints into a ticket, terminal transcript, or chat.
4. Restore only after checking file types, syntax, exact intent, and ownership;
   then start again with a dry-run where applicable.

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

## Restore one managed configuration from an adjacent backup

Quit the affected client and preserve both files in access-controlled storage
first when there is any uncertainty about which state is authoritative. Select
one fixed global configuration and preview the operation:

```bash
./target/debug/mcp-sync restore cursor --dry-run
```

The accepted selections are:

| Selection | Managed file |
| --- | --- |
| `canonical` | Canonical `config.json` at the resolved platform path |
| `claude-desktop` | Global Claude Desktop JSON |
| `cursor` | Global Cursor JSON |
| `windsurf` | Global Windsurf legacy Cascade JSON |
| `vscode` | Native VS Code default user-profile JSON |
| `codex` | Global Codex TOML |
| `kiro` | Global-user Kiro comment-preserving JSON |

Arbitrary paths, project files, named profiles, alternate products, and OAuth
credential stores cannot be selected. Dry-run requires the adjacent `.bak` to
be a regular file and validates its exact bytes with the same strict canonical,
native JSON, comment-preserving Kiro JSON, or Codex TOML parser used elsewhere.
It reports only the selected
path and structural outcome; configuration and backup values remain redacted.

Apply only after the preview is understood:

```bash
./target/debug/mcp-sync restore cursor
```

For an existing regular target whose bytes differ, guarded apply verifies that
both files still match the preview, stages both generations, atomically
publishes the selected backup bytes to the target as the commit point, and then
rotates the exact previous target into `.bak`. Running the command again can
therefore undo a restore when the rotated bytes also form a valid document. If
the target is missing, restore recreates it without consuming or rewriting the
backup. Equal target and backup bytes are a no-op. A missing or invalid backup,
symbolic link, directory, other non-regular path, permission failure, or
concurrent change returns non-zero without silently overwriting either file.

Restoring a client target without restoring canonical state normally produces
an expected `sync --dry-run` update that would reapply canonical definitions.
Restoring canonical state can similarly produce planned changes for multiple
clients. Review that new plan before any subsequent `sync`.

### Manual JSON fallback on macOS and Linux

The built-in command is the supported cross-platform path. If the binary is
temporarily unavailable, the following `bash` fallback performs a guarded,
same-directory replacement for an existing JSON target on macOS or Linux. It
requires Python 3 for syntax and duplicate-key validation and does not rotate
the current target into `.bak`; preserve both files separately first. Use it
for the canonical file or one of the five JSON clients only when the retained
document is strict JSON, not for Codex TOML. Kiro permits comments and trailing
commas, so use the built-in `restore kiro` path for those documents rather than
flattening or stripping their syntax.
Set `target` to exactly one applicable path from the tables above.

```bash
(
set -euo pipefail

target="$HOME/.cursor/mcp.json"
backup="${target}.bak"
json_python=python3

if [[ ! -f "$target" || -L "$target" || ! -f "$backup" || -L "$backup" ]]; then
  printf '%s\n' 'target and backup must both be regular files' >&2
  exit 1
fi
if ! "$json_python" -c 'import json' >/dev/null 2>&1; then
  printf '%s\n' 'JSON restoration requires Python 3' >&2
  exit 1
fi

stage=$(mktemp "${target}.restore.XXXXXX")
trap 'rm -f -- "$stage"' EXIT

cp -p -- "$backup" "$stage"
"$json_python" - "$stage" <<'PY'
import json
import sys

def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("duplicate JSON object key")
        result[key] = value
    return result

def reject_nonstandard_number(value):
    raise ValueError(f"non-standard JSON number: {value}")

with open(sys.argv[1], encoding="utf-8") as document:
    json.load(
        document,
        object_pairs_hook=unique_object,
        parse_constant=reject_nonstandard_number,
    )
PY
cmp -s -- "$backup" "$stage"
mv -f -- "$stage" "$target"

trap - EXIT
)
```

For Claude Desktop, replace the `target=` line with:

```bash
target="$HOME/Library/Application Support/Claude/claude_desktop_config.json"
```

On Linux with the default XDG root, use:

```bash
target="$HOME/.config/Claude/claude_desktop_config.json"
```

For Windsurf, replace the `target=` line with:

```bash
target="$HOME/.codeium/windsurf/mcp_config.json"
```

For VS Code's native default user profile, replace the `target=` line with:

```bash
target="$HOME/Library/Application Support/Code/User/mcp.json"
```

On Linux with the default XDG root, use:

```bash
target="$HOME/.config/Code/User/mcp.json"
```

For the default canonical path, replace the `target=` line with:

```bash
target="$HOME/.config/mcp-sync/config.json"
```

For a strict-JSON Kiro file with no comments or trailing commas, replace the
`target=` line with:

```bash
target="${KIRO_HOME:-$HOME/.kiro}/settings/mcp.json"
```

Use that fallback only after confirming `KIRO_HOME`, when set, is the exact
absolute user directory intended for this invocation. The built-in
`./target/debug/mcp-sync restore kiro` command remains the supported path for
all valid Kiro documents.

If a non-empty absolute `XDG_CONFIG_HOME` selected the canonical root or a
Linux user-data root, replace `$HOME/.config` with that exact value. Validate
canonical state afterward with
`./target/debug/mcp-sync list`; validate target state and see the resulting
difference with `./target/debug/mcp-sync sync --dry-run`.
Restoring a target without restoring the canonical definition normally creates
an expected update plan that would reapply the canonical state.

### Manual Codex TOML fallback on macOS and Linux

For `$HOME/.codex/config.toml`, use the following fallback variant. It
requires Python 3.11 or newer for its standard-library `tomllib` validator. If
that module is unavailable, stop and obtain an equivalent trusted TOML parser;
do not skip validation.

```bash
(
set -euo pipefail

target="$HOME/.codex/config.toml"
backup="${target}.bak"
toml_python=python3 # set to an installed Python 3.11+ executable when needed

if [[ ! -f "$target" || -L "$target" || ! -f "$backup" || -L "$backup" ]]; then
  printf '%s\n' 'target and backup must both be regular files' >&2
  exit 1
fi
if ! "$toml_python" -c 'import tomllib' >/dev/null 2>&1; then
  printf '%s\n' 'TOML restoration requires Python 3.11+ with tomllib' >&2
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

## Current-client verification

On 2026-08-11, controlled macOS smokes exercised the installed, signed and
notarized Kiro IDE `1.0.288` and Kiro Crew `0.1.3` against one synthetic local
STDIO definition rendered by the built `mcp-sync` binary.

For Kiro IDE, the real global-user file and its `settings` directory were
absent before the smoke. A synthetic canonical root plus an explicit
`KIRO_HOME` pointing at the default real Kiro directory let `sync --dry-run`
prove the six-target plan and `sync` create only that real Kiro target while
the other five targets remained under the synthetic home. The IDE opened an
empty synthetic workspace in restricted mode and started the rendered server;
the marker recorded `initialize`, `notifications/initialized`, and
`tools/list`. After the app closed, the child was reaped, the exact rendered
file hash and type were rechecked, and only that created file and its now-empty
directory were removed. The prior absence was restored, while the excluded
real `agents` and `crew` directories retained their exact baseline inode,
mode, and modification time and the Crew override remained absent.

For Kiro Crew, both `HOME` and `KIROCREW_HOME` were isolated. The current Crew
gateway listed the mcp-sync-rendered global definition, its MCP probe completed
the same initialize and tools-list exchange, and its generated
`kirocrew.json` contained an exact inherited server entry with
`includeMcpJson: false`. The Crew override stayed absent, the global source
hash remained unchanged, and every gateway and server process was reaped. This
is evidence for the current Crew global-inheritance and MCP-probe path; it is
not a claim about a complete chat session or a standalone `kiro-cli`, which was
not installed on the smoke host.

## Current operational limitations

The following are delivery facts, not changes to the README's intended product
promise:

- The six-target source state is verified through complete native ARM64 and
  Rosetta x64 macOS suites plus native x64/ARM64 GNU/Linux and Windows MSVC
  pull-request CI on [MCP-037 PR #58](https://github.com/EnjoyableWork/mcp-sync/pull/58).
  The earlier platform introductions remain recorded by the
  [GNU/Linux](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31240608728)
  and [Windows MSVC](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31244565101)
  gates for `MCP-018` and `MCP-019`. The current Windows gate includes path,
  replacement, rollback, copied-binary, and PowerShell health behavior. Usage
  remains from a Rust source checkout.
- Only global Claude Desktop, global Cursor, Windsurf's global legacy Cascade
  JSON, VS Code's native default user-profile JSON, global Codex TOML, and the
  selected global-user Kiro JSON document are managed. Linux has deterministic
  path, fixture, and built-binary behavior evidence, but no Linux
  current-client smoke claim. Windows has no current-client smoke claim.
  Windsurf, VS Code, and Codex have no current-client smoke claim on any
  implemented platform. Kiro's current-client evidence is the bounded macOS
  IDE and inherited Crew smoke above; workspace, agent, Crew-only,
  generated-agent, organization-managed, remote-service, and credential stores
  remain unsupported.
- Canonical schema v1 represents local STDIO definitions with `command`,
  ordered `args`, and literal `env` only. Remote transports, OAuth, working
  directories, and secret references are not canonical capabilities yet.
- `mcp-sync test <name>` validates one bounded initialize exchange for a
  canonical local STDIO server. It does not test remote transports, later
  protocol operations, or the stateless `2026-07-28` protocol. `init` and
  `sync` never execute server commands.
- Target-only definitions are drift and are never deleted. There is no prune
  command.
- Restore intentionally retains one adjacent generation rather than automatic
  backup history. Copy target and backup bytes to separate access-controlled
  storage when more than one previous state is required.
- Abrupt-process recovery is target-first and per existing file. It does not
  make the six target commits atomic across termination and does not claim
  whole-machine or power-loss durability.
- The immutable `v0.1.0` release verifies GNU/Linux archives, Cargo across the
  six supported native hosts, and source-building Homebrew on supported macOS
  and GNU/Linux hosts. It intentionally has no project-issued macOS or Windows
  binary and no WinGet package; those remain funding-dependent later work.

See [PROJECT.md](../PROJECT.md) for the ordered implementation record and the
evidence required before any later capability is described as delivered.
