# Repository governance

This document defines the maintained change-control contract for the default
branch of `EnjoyableWork/mcp-sync`. It is current-state operational guidance,
not a claim about release signing, vulnerability management, or later
assurance work.

The live GitHub ruleset is authoritative for enforcement. The repository's
credential-free [`verify-public-main-ruleset.sh`](../scripts/verify-public-main-ruleset.sh)
checks the public rule, while the read-only operator-side
[`verify-main-repository-controls.sh`](../scripts/verify-main-repository-controls.sh)
also checks repository merge and signoff settings that GitHub omits from
unauthenticated responses.

## Protected branch contract

The active `Protect main` repository ruleset selects only the default branch
and enforces all of the following:

- commits reach `main` through a pull request rather than a direct push;
- these five status checks must be emitted by the GitHub Actions application
  with integration ID `15368` and must pass against the latest `main`:
  - `Dependency policy`;
  - `Linux x64 — format, Clippy, and test`;
  - `Linux ARM64 — format, Clippy, and test`;
  - `Windows x64 — format, Clippy, and test`; and
  - `Windows ARM64 — format, Clippy, and test`;
- every review conversation is resolved before merge;
- deletion of `main` is blocked; and
- non-fast-forward updates and force pushes are blocked.

The rule has no standing bypass actor. Stable-tag rules and the `release` and
`release-control` environments are separate controls and must not be edited as
part of default-branch administration.

## Contributor-compatible merge policy

The repository currently has one administrator and no independent reviewer.
The required approval count is therefore zero: requiring an approval now would
make every maintainer-authored pull request impossible to merge without using
an exception. Pull requests still require the complete strict CI set and
resolved conversations. Increase the approval count when a second trusted
reviewer can reliably review changes; do not add a nominal reviewer merely to
satisfy a counter.

Merge commits, squash merges, and rebase merges remain available. Each path
must enter through a pull request and satisfy the same rule. Auto-merge remains
disabled, and merged branches continue to be deleted automatically.

Verified commit signatures are not required yet. This avoids excluding
contributors or breaking GitHub-generated merge commits before the external
contributor and web-merge paths have been tested end to end. Cryptographic
commit signing is distinct from contribution sign-off; any DCO or CLA choice
belongs to `MCP-032`.

## Normal change path

1. Branch from current `main` and push only that topic branch.
2. Open a pull request into `main`.
3. Resolve every review conversation.
4. If GitHub reports the branch is behind, update it and let every required
   check rerun against the latest `main`.
5. Merge through one of the three allowed pull-request methods only after all
   five checks pass.
6. Confirm the merged commit is reachable from `main` and rerun both repository
   control verifiers.

A renamed job, replacement GitHub application, or changed check topology is a
governance change. Update and reverify the ruleset deliberately; never remove a
required check merely to make one pull request mergeable.

## Emergency administration

There is deliberately no routine bypass. A repository administrator can edit
the ruleset itself when an emergency makes the normal path impossible, but
that is an explicit administrative event rather than an invisible direct-push
exception.

For an emergency:

1. Record the incident, affected ref, intended change, administrator, and
   restoration condition before the edit when circumstances permit.
2. Preserve protection for `main` whenever the incident can be reproduced or
   recovered with a disposable branch.
3. Constrain any rule edit to the smallest ref, rule, and time window. Never
   alter stable-tag or protected-environment controls as a shortcut.
4. Restore the exact accepted rule immediately after the emergency action.
5. Run both verifiers, inspect the GitHub rule evaluation or audit record, and
   route any resulting source change through a follow-up pull request.

The `MCP-030` drill exercises this path by temporarily adding and then removing
a disposable branch from the rule's scope while `main` remains continuously
protected. A direct update, force push, and deletion of that branch must each
be rejected before the administrator removes only the disposable selector and
deletes the branch normally.

## MCP-030 verification record

The control was activated and exercised on 2026-08-09:

- Public ruleset [`20601003`](https://api.github.com/repos/EnjoyableWork/mcp-sync/rulesets/20601003)
  is active, selects only `~DEFAULT_BRANCH`, contains exactly the four accepted
  rules and five app-bound checks, and reports no bypass actors.
- A direct update to `main` was rejected with `GH013`; administrator-visible
  rule suite `3609610550` records the failed pull-request and required-check
  evaluations, and `main` remained at its original commit.
- On disposable `mcp-030-ruleset-drill`, a fast-forward direct update, a force
  push, and deletion were rejected by administrator-visible rule suites
  `3609600890`, `3609601135`, and `3609601321`. The ref remained unchanged
  after every rejected operation.
- The emergency-administration drill removed only the disposable selector from
  the still-active rule. Both verifiers then passed, the disposable branch was
  deleted normally, and `main` remained unchanged throughout.
- SHA-256 fingerprints of the accepted stable-tag, `release`, and
  `release-control` settings matched exactly before and after the drill:
  `b91e5df98ca37397800c4383ff5e248ba36fc0451ccb851a612ee71a9b219ea5`,
  `fb060808cfa726c38867464b9dd3f903ad70c58478ba130a4871125f8cd3df04`,
  and `938cfe6507e16813470a309c1746b8318dbb3dc128db4122482fcdf5ec9bfb1d`.
- [PR #33](https://github.com/EnjoyableWork/mcp-sync/pull/33) is the normal
  protected path for this change. Its final merged state, app-bound check
  rollup, zero-approval merge, and resolved-conversation state are the durable
  normal-path record.
- The implementation head passed [CI](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31295984170),
  the retained [six-target release preflight](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31295984167),
  and the [source and GNU/Linux preflight](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31295984173)
  before the final evidence-only update.

## Assurance mapping and review triggers

The active pull-request rule and rejected direct-update path provide evidence
for `OSPS-AC-03.01`. The active deletion rule and rejected disposable-branch
deletion path provide evidence for `OSPS-AC-03.02`. This is a scoped
self-assessment input for the later `MCP-035` closeout, not an independent
certification.

Reverify this contract when:

- a required job name or GitHub application identity changes;
- the default branch changes;
- a second trusted reviewer becomes available;
- signed-commit enforcement becomes viable across every accepted merge path;
- GitHub changes repository-ruleset semantics; or
- a normal or emergency path fails.
