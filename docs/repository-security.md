# Repository security controls

This document records the current, non-sensitive repository security boundary
for `EnjoyableWork/mcp-sync`. It complements the public [security policy](../SECURITY.md)
without publishing alert contents, vulnerability reports, secrets, contact
credentials, or maintainer identities.

The live GitHub settings are authoritative. The authenticated, read-only
[`verify-repository-security-controls.sh`](../scripts/verify-repository-security-controls.sh)
checks the accepted settings and fails generically when a private baseline
requires remediation; it never prints a finding payload.

## Public reporting contract

Repository security contacts are the `mcp-sync` maintainers reached through the
repository's
[private vulnerability reporting form](https://github.com/EnjoyableWork/mcp-sync/security/advisories/new).
The root `SECURITY.md` records the supported-version policy, the private route,
safe public-issue fallback, requested redacted evidence, response targets, and
coordinated-disclosure expectations. No personal address or contact credential
is required in the public repository.

## Live zero-cost baseline

The organization currently uses GitHub Free and the repository is public. The
accepted repository-level baseline was verified on 2026-08-09:

| Control | Accepted state |
| --- | --- |
| Dependency graph and Dependabot alerts | Enabled; GitHub recognizes the committed Cargo dependency inventory and exposes its SBOM export to authorized verification. |
| Dependabot security updates | Enabled. Security-fix pull requests are in scope; scheduled version-update configuration and `.github/dependabot.yml` remain `MCP-033` scope. |
| CodeQL | [Default setup](https://docs.github.com/en/code-security/how-tos/find-and-fix-code-vulnerabilities/configure-code-scanning/configure-code-scanning?learn=code_security_actions) is configured with the default query suite for Rust and GitHub Actions. GitHub owns the generated workflow and scans protected-branch pushes, eligible pull requests, and the default schedule. |
| Secret scanning | [Provider-pattern scanning](https://docs.github.com/en/code-security/how-tos/secure-your-secrets/detect-secret-leaks/enable-secret-scanning) is enabled. Finding contents remain only in GitHub's restricted security views. |
| Push protection | Enabled for supported provider patterns. A recognized secret is blocked before storage unless a contributor deliberately uses GitHub's recorded bypass path. |
| Private vulnerability reporting | Enabled for the public repository; reporters use the stable GitHub form linked from `SECURITY.md`. |

GitHub Free does not make two supplemental GitHub Secret Protection features
available to this organization-owned repository:

- [non-provider pattern scanning](https://docs.github.com/en/code-security/secret-scanning/using-advanced-secret-scanning-and-push-protection-features/non-provider-patterns/enabling-secret-scanning-for-non-provider-patterns);
  and
- [automatic partner-pattern validity checks](https://docs.github.com/en/code-security/how-tos/secure-your-secrets/customize-leak-detection/enable-validity-checks?learn=secret_scanning&learnProduct=code-security).

GitHub currently limits both settings to organization-owned repositories on
GitHub Team with GitHub Secret Protection enabled. Attempts to enable them
through the repository API leave both settings disabled. Provider-pattern
secret scanning and push protection remain enabled and are the accepted
zero-cost prevention boundary. Revisit this limitation if the organization
plan or GitHub's feature entitlements change; do not describe either
supplemental feature as active before the live verifier passes.

## Non-disclosing baseline verification

The operator verifier checks only configuration and whether each private open
finding collection is empty. It discards dependency, CodeQL, and secret-scanning
payloads and emits one generic remediation failure when any collection is not
empty. Durable public evidence may record only that the baseline passed, the
verification date, and non-sensitive workflow or settings URLs. It must never
copy alert identifiers, rules, locations, dependency findings, secret types or
values, report contents, dismissals, or reporter and maintainer identities.

The baseline must be reverified after a dependency or workflow change, a CodeQL
configuration change, a GitHub security-feature or plan change, a secret or
vulnerability incident, and before the later `MCP-035` self-assessment.

## OpenSSF OSPS mapping

For the [OpenSSF OSPS Baseline `v2026.02.19` Level 1
checklist](https://baseline.openssf.org/versions/2026-02-19-checklist.md):

- `OSPS-BR-07.01` is evidenced by the existing local-environment exclusions,
  enabled provider-pattern secret scanning, enabled push protection, and the
  generic zero-open-finding gate; and
- `OSPS-VM-02.01` is evidenced by the root `SECURITY.md`, its stable private
  contact route, and GitHub-recognized security-policy location.

This mapping is an evidence input for the later complete, dated `MCP-035`
self-assessment. It is not an assurance badge, certification, or claim that the
whole OSPS Baseline is complete.
