use std::fs;
use std::path::PathBuf;

fn repository_file(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fs::read_to_string(root.join(path)).expect("organization access file should be readable")
}

#[test]
fn organization_access_contract_is_current_non_disclosing_and_not_an_assurance_claim() {
    let contract = repository_file("docs/organization-access-and-recovery.md");

    for required in [
        "`MCP-034` is complete",
        "Only allow secure two-factor methods",
        "base repository permission must be `none`",
        "Non-owner repository creation must be disabled",
        "at least two genuinely trusted owners",
        "explicitly owner-accepted single-owner exception",
        "private recovery record",
        "successful recovery exercise",
        "nominal empty team",
        "repository-scoped `GITHUB_TOKEN`",
        "one write deploy key",
        "GitHub App installations",
        "OSPS-AC-01.01",
        "OSPS-AC-02.01",
        "not an assurance badge, certification",
    ] {
        assert!(
            contract.contains(required),
            "organization access contract should document {required}"
        );
    }

    for forbidden in [
        "OSPS compliant",
        "OSPS certified",
        "independently certified",
        "recovery code is",
        "private key is",
        "token is",
    ] {
        assert!(
            !contract.contains(forbidden),
            "organization access contract must not publish {forbidden}"
        );
    }
}

#[test]
fn operator_verifier_enforces_live_least_privilege_without_printing_private_data() {
    let verifier = repository_file("scripts/verify-organization-access-controls.sh");

    for required in [
        "gh auth status --hostname github.com",
        ".two_factor_requirement_enabled == true",
        ".default_repository_permission == \"none\"",
        ".members_can_create_repositories == false",
        ".members_can_create_public_repositories == false",
        ".members_can_create_private_repositories == false",
        ".members_can_create_internal_repositories == false",
        ".default_workflow_permissions == \"read\"",
        ".can_approve_pull_request_reviews == false",
        "verify-release-repository-controls.sh",
        "orgs/$organization_access_organization/installations?per_page=100",
        "repository_selection == \"all\"",
        "repos/$organization_access_tap_repository/keys?per_page=100",
        ".read_only == false",
        ".verified == true",
        ".enabled == true",
        "orgs/$organization_access_organization/actions/secrets",
        "environments/release/secrets",
        "environments/release-control/secrets",
        "HOMEBREW_TAP_DEPLOY_KEY",
        "MCP_SYNC_PRIVATE_ORGANIZATION_EVIDENCE",
        "private organization evidence must remain outside the repository",
        "private organization evidence must be owned by the current operator",
        "owner_choice_explicitly_accepted",
        "secure_methods_only_confirmed",
        "recovery_exercise_passed",
        "details remain outside the repository",
    ] {
        assert!(
            verifier.contains(required),
            "organization access verifier should enforce {required}"
        );
    }

    for forbidden in [
        "set -x",
        ".login",
        ".email",
        ".key |",
        ".key,",
        ".key)",
        ".title",
        ".fingerprint",
        "recovery_codes",
        "cat \"$organization_access_evidence",
    ] {
        assert!(
            !verifier.contains(forbidden),
            "organization access verifier must not inspect or print {forbidden}"
        );
    }
}

#[test]
fn synthetic_policy_exercise_covers_acceptance_and_important_rejections() {
    let exercise = repository_file("scripts/test-organization-access-policy.sh");
    let workflow = repository_file(".github/workflows/ci.yml");

    for rejection in [
        "missing MFA",
        "implicit repository read access",
        "member repository creation",
        "write-default workflow token",
        "broad second deploy key",
        "extra environment secret",
        "unaccepted owner choice",
        "untested recovery",
        "stale aggregate access count",
        "identity-bearing extra field",
        "single-owner evidence for two owners",
    ] {
        assert!(
            exercise.contains(rejection),
            "synthetic policy exercise should reject {rejection}"
        );
    }
    assert!(exercise.contains("two-trusted-owners"));
    assert!(workflow.contains("./scripts/test-organization-access-policy.sh"));
}

#[test]
fn private_evidence_preparation_outputs_only_aggregate_false_by_default_fields() {
    let preparation = repository_file("scripts/prepare-organization-access-evidence.sh");

    for required in [
        "organization_access_count_array_pages",
        "members?filter=all&role=all&per_page=100",
        "members?filter=all&role=admin&per_page=100",
        "outside_collaborators?per_page=100",
        "invitations?per_page=100",
        "teams?per_page=100",
        "installations?per_page=100",
        "owner_choice_explicitly_accepted: false",
        "secure_methods_only_confirmed: false",
        "recovery_exercise_passed: false",
        "least_privilege_access_reviewed: false",
        "automation_access_reviewed: false",
        "Template only:",
    ] {
        assert!(
            preparation.contains(required),
            "private evidence preparation should contain {required}"
        );
    }

    for forbidden in [
        ".login",
        ".email",
        ".app_slug",
        ".name",
        ".key",
        ".title",
        ".fingerprint",
        "secrets.",
    ] {
        assert!(
            !preparation.contains(forbidden),
            "private evidence preparation must not inspect or print {forbidden}"
        );
    }
}

#[test]
fn trusted_publishing_is_complete_and_kiro_is_the_active_successor() {
    let project = repository_file("PROJECT.md");
    let readme = repository_file("README.md");

    assert!(project.contains("The owner accepted the single-owner residual risk"));
    assert!(project.contains("| MCP-034 | Establish organization access, credential, and ownership recovery policy | M3 | P1 | Codex | Done |"));
    assert!(project.contains("| MCP-035 | Self-assess, publish, and showcase the zero-cost enterprise assurance baseline | M3 | P1 | Codex | Done |"));
    assert!(project.contains("| Done — `MCP-030` through `MCP-035` Done |"));
    assert!(project.contains("M0 through M3, `MCP-001` through `MCP-036`"));
    assert!(
        project.contains("`MCP-037` and D-18 are `In progress` under their exact canonical Goal")
    );
    assert!(project.contains("no side quest is active"));
    assert!(project.contains("| MCP-036 | Serialize every mutating operation per canonical configuration root | Post-M3 corrective maintenance | P0 | Codex | Done |"));
    assert!(project.contains("| MCP-039 | Replace Cargo publication tokens with crates.io Trusted Publishing | Post-M3 release maintenance | P1 | Codex | Done | `MCP-036` |"));
    assert!(project.contains("| MCP-039 | Complete MCP-039: replace reusable crates.io credentials for every Cargo version after 0.1.0"));
    assert!(project.contains("Require trusted publishing for all new versions"));
    assert!(project.contains("`MCP-039` completed the release-maintenance"));
    assert!(project.contains("| SIDE-007 | Redesign the README as a polished, terminal-first public product page | P2 | Codex | Done |"));
    assert!(project.contains("| SIDE-008 | Eliminate duplicate pull-request CI and stabilize Windows health-process fixture timing | P2 | Codex | Done |"));
    assert!(
        project.contains(
            "| D-15 | Deterministic pull-request CI and stable Windows health fixtures |"
        )
    );
    assert!(project.contains("| D-16 | Cross-process serialization for every mutating configuration operation | Post-M3 corrective maintenance | Codex | 2026-08-10 | Done |"));
    assert!(project.contains("| D-17 | Tokenless crates.io publication for every version after `0.1.0` | Post-M3 release maintenance | Codex | 2026-08-11 | Done |"));
    assert!(project.contains("| D-18 | Global-user Kiro adapter with inherited Kiro Crew coverage | M4 | Codex | Current ticket | In progress |"));
    assert!(project.contains("Complete SIDE-008: make pull-request CI deterministic"));
    assert!(project.contains("| MCP-037 | Add a global-user Kiro MCP adapter with inherited Kiro Crew coverage | M4 | P1 | Codex | In progress | `MCP-039` |"));
    assert!(project.contains(
        "| MCP-037 | Complete MCP-037: add Kiro as the sixth global configuration target"
    ));
    assert!(project.contains("The owner approved `MCP-037` as the ordered M4 successor"));
    assert!(project.contains("without requiring `v0.1.0` adoption evidence"));
    assert!(project.contains("Kiro Crew is a relevant consumer, not a duplicate adapter."));
    assert!(project.contains("`~/.kiro/crew/mcp.json`"));
    assert!(project.contains("`~/.kiro/agents/kirocrew.json`"));
    assert!(project.contains("`MCP-037` must never discover or"));
    assert!(project.contains("mutate either Crew-specific file."));
    assert!(!project.contains("| MCP-036 | Serialize every mutating operation per canonical configuration root | Post-M3 corrective maintenance | P0 | Codex | In progress |"));
    assert!(!project.contains("| MCP-039 | Replace Cargo publication tokens with crates.io Trusted Publishing | Post-M3 release maintenance | P1 | Unassigned | Proposed |"));
    assert!(!project.contains("| MCP-039 | Replace Cargo publication tokens with crates.io Trusted Publishing | Post-M3 release maintenance | P1 | Codex | In progress |"));
    assert!(!project.contains("| MCP-037 | Add a global-user Kiro MCP adapter with inherited Kiro Crew coverage | M4 | P1 | Unassigned | Ready |"));
    assert!(!project.contains("| SIDE-006 | Activate the funded six-target signed-native distribution pipeline | P2 | Codex | In progress |"));
    assert!(!readme.contains("OSPS-AC-01.01"));
    assert!(!readme.contains("OSPS-AC-02.01"));
}
