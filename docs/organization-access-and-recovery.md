# Organization access and ownership recovery

This document defines the `MCP-034` access, authentication, automation, and
ownership-recovery contract for the GitHub organization controls that protect
`EnjoyableWork/mcp-sync` and its `EnjoyableWork/homebrew-tap` distribution
repository. It records only non-sensitive policy and aggregate evidence. It
must never contain a person's identity, authentication method, recovery code,
credential, private-key material, secret value, recovery-record location, or
the private instructions used during an exercise.

`MCP-034` is complete. The initial authenticated read-only audit on
2026-08-09 verified required organization-wide 2FA, read-only Actions token
defaults, no organization- or repository-level Actions secrets, and the then
active protected-environment credential backed by one tap-only write deploy key. It
also found one organization owner, no teams, default repository permission of
`read`, and member repository creation enabled. After explicit owner approval,
the same-day operator change set default permission to `none` and disabled all
member repository creation; an immediate authenticated read-back verified both
values. The owner also accepted the single-owner recovery exception and
confirmed completion of the private installed-application review and recovery
exercise. The owner then confirmed that secure-method-only 2FA is enabled and
that the same-day exercise passed. An owner-only aggregate evidence file was
used for the complete non-disclosing verifier and removed immediately after
the verifier succeeded; no identity, authentication method, recovery detail,
application name, or credential entered public evidence.

## Authentication and collaborator access policy

Every organization member, outside collaborator, billing manager, and owner
must use GitHub 2FA. The organization must also enable GitHub's
**Only allow secure two-factor methods** option, which accepts passkeys, security keys,
authenticator applications, and GitHub Mobile while excluding SMS-only access.
The REST API exposes the general 2FA requirement but not this stronger toggle,
so an owner confirms the toggle privately during every access review without
recording which methods any person uses. See GitHub's
[organization 2FA guidance](https://docs.github.com/en/organizations/keeping-your-organization-secure/managing-two-factor-authentication-for-your-organization/requiring-two-factor-authentication-in-your-organization).

Organization base repository permission must be `none`. Public repositories
remain publicly readable, while a new member receives no implicit access to a
private repository. Every elevated repository grant is therefore manual and
must name its repository, smallest sufficient role, business purpose, owner,
and next private review date. Outside collaborators already require a manual
repository-specific grant. GitHub documents the `none` base-permission option
in its [base-permission guidance](https://docs.github.com/en/organizations/managing-user-access-to-your-organizations-repositories/managing-repository-roles/setting-base-permissions-for-an-organization).

Non-owner repository creation must be disabled. A proposed repository needs an
owner-reviewed purpose, visibility, data classification, initial access model,
security settings, and retirement owner before creation. Organization owners
retain the ability to create a repository under GitHub's
[repository-creation policy](https://docs.github.com/en/organizations/managing-organization-settings/restricting-repository-creation-in-your-organization).

Use repository-specific teams when a real second maintainer or contributor
cohort exists. Grant `triage`, `write`, `maintain`, or `admin` only when the
work requires that role; reserve organization owner and repository `admin` for
organization-wide or destructive administration. A one-person organization
must not create a nominal empty team merely to satisfy a checklist. Direct
grants remain exceptional and are reviewed with the same private record.

## Ownership continuity and private recovery record

GitHub recommends at least two organization owners because a sole unavailable
owner can make organization projects inaccessible. See GitHub's
[ownership-continuity guidance](https://docs.github.com/en/organizations/managing-peoples-access-to-your-organization-with-roles/maintaining-ownership-continuity-for-your-organization).
The accepted state must therefore be exactly one of:

- at least two genuinely trusted owners who have independently confirmed
  administrative access; or
- an explicitly owner-accepted single-owner exception with a current private
  recovery plan and a successful recovery exercise.

The single-owner exception is a residual risk, not equivalent redundancy. It
must be replaced with two trusted owners when a suitable second person exists.
Never promote a nominal or untrusted owner solely to satisfy a count.

The owner keeps the operational recovery plan and recovery material outside
the repository in protected private storage. GitHub recommends two or more
authentication methods plus securely stored recovery codes in its
[recovery-method guidance](https://docs.github.com/en/authentication/securing-your-account-with-two-factor-authentication-2fa/configuring-two-factor-authentication-recovery-methods).
The private record confirms, without naming a method or location, that:

1. independent secure authentication and recovery paths are available;
2. recovery material is current and privately protected;
3. an alternate path opened a fresh signed-out session;
4. that session reached organization settings and both in-scope repositories
   with the expected administrative access without changing state;
5. the session was signed out and any consumed recovery material was replaced;
   and
6. the exercise date, generic result, residual risk, and next review date were
   recorded privately.

The operator verifier reads only a separate aggregate attestation file. It
rejects extra fields so that identities, factor types, credential metadata,
recovery contents, and storage locations cannot enter that evidence. The file
must be a non-symlink regular file outside the repository with no group or
world permissions. Generate an identity-free template with
`./scripts/prepare-organization-access-evidence.sh EnjoyableWork/mcp-sync`,
capture its standard output directly into private owner-only storage, and set a
boolean to `true` only after performing that check. The following values are an
illustrative schema, not current evidence:

```json
{
  "schema": 1,
  "reviewed_on": "YYYY-MM-DD",
  "ownership_mode": "single-owner-recovery",
  "owner_choice_explicitly_accepted": true,
  "secure_methods_only_confirmed": true,
  "independent_recovery_paths_confirmed": true,
  "private_recovery_record_current": true,
  "recovery_exercise_passed": true,
  "least_privilege_access_reviewed": true,
  "automation_access_reviewed": true,
  "expected_counts": {
    "organization_members": 0,
    "organization_owners": 0,
    "outside_collaborators": 0,
    "pending_invitations": 0,
    "teams": 0,
    "all_repository_app_installations": 0,
    "write_capable_all_repository_app_installations": 0
  }
}
```

Use `two-trusted-owners` instead of `single-owner-recovery` only when the live
owner count is at least two. Replace every illustrative count with the
privately reviewed aggregate live count. Do not place this file in the source
tree, paste it into an issue or pull request, or add fields describing people,
methods, credentials, applications, or private storage.

## Automation credential boundary

Normal workflows use GitHub's repository-scoped `GITHUB_TOKEN`, whose default
permission is `read`, which cannot approve pull requests, and which expires
when its job ends. GitHub describes that lifetime and repository boundary in
the [`GITHUB_TOKEN` contract](https://docs.github.com/en/actions/concepts/security/github_token).
Committed jobs continue to request only the permissions they use.

GitHub App installations must be reviewed privately for both permissions and
repository selection. Select only the repositories each integration needs;
an all-repository grant needs an explicit operational reason in the private
record. Installation tokens expire after one hour and may be narrowed further
at issuance, but that short lifetime does not excuse an unnecessarily broad
persistent installation grant. See GitHub's
[installation-token guidance](https://docs.github.com/en/apps/creating-github-apps/authenticating-with-a-github-app/generating-an-installation-access-token-for-a-github-app).

Homebrew uses a tap-owned publication workflow, manually dispatched in
`EnjoyableWork/homebrew-tap`. Its validation job independently checks
the immutable upstream release and exact downstream transition without write
permission. Only its protected publish job receives the tap repository's
short-lived, job-scoped `GITHUB_TOKEN`, and that job may stage only
`Formula/mcp-sync.rb`. Its completed credential boundary requires zero source and tap deploy keys
plus no source Homebrew secret or cross-repository write authority. There is no
organization secret, repository secret, Cargo
publication token, general-purpose personal access token, GitHub App
credential, or automatic cross-repository dispatch credential in that path.

Interactive administrative credentials are not automation credentials and
must never be stored in Actions. Prefer a short expiration and the smallest
repository and organization permissions supported for each operator task.
Re-authenticate for a later task rather than turning one broad credential into
a permanent release dependency.

## Non-disclosing verification

After the explicit organization-policy decision, secure-method confirmation,
private access review, ownership choice, and recovery exercise, run:

```sh
MCP_SYNC_PRIVATE_ORGANIZATION_EVIDENCE=/absolute/private/path/evidence.json \
  ./scripts/verify-organization-access-controls.sh EnjoyableWork/mcp-sync
```

The authenticated verifier reads settings, aggregate counts, key state, and
secret names needed to distinguish the one committed workflow boundary. It
discards every identity and never reads a secret value, public key, factor,
recovery item, or private plan. Its only success output is a generic result;
failures name the control class but not the private payload.

Re-run it after an organization member, owner, outside collaborator, team,
repository, installed application, deploy key, Actions secret, workflow-token
default, protected environment, release workflow, 2FA policy, or recovery plan
changes, after an access or credential incident, and before the `MCP-035`
self-assessment.

## OpenSSF OSPS Baseline evidence boundary

For the [OpenSSF OSPS Baseline `v2026.02.19`](https://baseline.openssf.org/versions/2026-02-19.html):

- `OSPS-AC-01.01` requires MFA before a user can read or modify a sensitive
  resource. Organization-required secure-method 2FA plus the private
  confirmation covers the GitHub organization and its sensitive repository,
  workflow, release, and credential settings.
- `OSPS-AC-02.01` requires manual permission assignment for a new collaborator
  or the lowest available default. Base permission `none`, repository-specific
  grants, and the private aggregate access review provide that evidence.

On 2026-08-09, the live verifier passed this exact policy and aggregate private
evidence boundary. The implementation head also passed [CI](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31328986122),
[CodeQL](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31328983549),
the [six-target release preflight](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31328986128),
and the complete [source and GNU/Linux preflight](https://github.com/EnjoyableWork/mcp-sync/actions/runs/31328986137).
This is the durable `MCP-034` evidence for `OSPS-AC-01.01` and
`OSPS-AC-02.01`. It remains an input to the later dated `MCP-035`
self-assessment, not an assurance badge, certification, regulatory claim, or
statement that the complete OSPS Baseline has passed.
