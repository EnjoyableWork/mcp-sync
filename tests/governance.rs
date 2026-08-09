use std::fs;
use std::path::PathBuf;

fn repository_file(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fs::read_to_string(root.join(path)).expect("repository governance file should be readable")
}

#[test]
fn public_main_ruleset_verifier_encodes_the_accepted_contract() {
    let verifier = repository_file("scripts/verify-public-main-ruleset.sh");

    for required_contract in [
        "https://api.github.com/repos/$main_ruleset_repository/rulesets",
        "Protect main",
        ".enforcement == \"active\"",
        ".target == \"branch\"",
        "~DEFAULT_BRANCH",
        "(.bypass_actors | length) == 0",
        "deletion",
        "non_fast_forward",
        "pull_request",
        "required_status_checks",
        "strict_required_status_checks_policy == true",
        "required_review_thread_resolution == true",
        "required_approving_review_count == 0",
        "Dependency policy",
        "Linux x64 — format, Clippy, and test",
        "Linux ARM64 — format, Clippy, and test",
        "Windows x64 — format, Clippy, and test",
        "Windows ARM64 — format, Clippy, and test",
        "integration_id: 15368",
    ] {
        assert!(
            verifier.contains(required_contract),
            "main ruleset verifier should enforce {required_contract}"
        );
    }

    for forbidden_contract in ["gh api", "GH_TOKEN", "secrets.", "refs/tags/v*"] {
        assert!(
            !verifier.contains(forbidden_contract),
            "public verifier must not depend on {forbidden_contract}"
        );
    }
}

#[test]
fn operator_main_control_verifier_keeps_admin_access_local() {
    let verifier = repository_file("scripts/verify-main-repository-controls.sh");

    for required_contract in [
        "gh auth status --hostname github.com",
        "verify-public-main-ruleset.sh",
        "default_branch == \"main\"",
        "allow_merge_commit == true",
        "allow_squash_merge == true",
        "allow_rebase_merge == true",
        "allow_auto_merge == false",
        "delete_branch_on_merge == true",
        "web_commit_signoff_required == false",
    ] {
        assert!(
            verifier.contains(required_contract),
            "operator verifier should enforce {required_contract}"
        );
    }
    assert!(!verifier.contains("secrets."));
}

#[test]
fn repository_governance_keeps_emergency_and_release_controls_separate() {
    let governance = repository_file("docs/repository-governance.md");

    for required_contract in [
        "no standing bypass actor",
        "required approval count is therefore zero",
        "Merge commits, squash merges, and rebase merges remain available",
        "Verified commit signatures are not required yet",
        "smallest ref, rule, and time window",
        "stable-tag or protected-environment controls",
        "OSPS-AC-03.01",
        "OSPS-AC-03.02",
    ] {
        assert!(
            governance.contains(required_contract),
            "repository governance should document {required_contract}"
        );
    }
}

#[test]
fn security_policy_defines_the_private_reporting_contract_without_contact_credentials() {
    let security = repository_file("SECURITY.md");

    for required_contract in [
        "## Supported versions",
        "`0.1.0`",
        "The `mcp-sync` maintainers are the project security contacts",
        "https://github.com/EnjoyableWork/mcp-sync/security/advisories/new",
        "Do not open a public issue",
        "Private security contact requested",
        "synthetic, redacted values",
        "within three business days",
        "within seven business days",
        "every fourteen days",
        "within ninety days",
        "coordinate a disclosure date",
    ] {
        assert!(
            security.contains(required_contract),
            "security policy should define {required_contract}"
        );
    }

    for forbidden_contract in ["mailto:", "@example", "OSPS compliant", "certified"] {
        assert!(
            !security.contains(forbidden_contract),
            "security policy must not publish {forbidden_contract}"
        );
    }
}

#[test]
fn security_control_verifier_is_exact_and_never_prints_private_findings() {
    let verifier = repository_file("scripts/verify-repository-security-controls.sh");

    for required_contract in [
        "gh auth status --hostname github.com",
        "private-vulnerability-reporting",
        "vulnerability-alerts",
        "dependency-graph/sbom",
        "dependabot_security_updates.status == \"enabled\"",
        "secret_scanning.status == \"enabled\"",
        "secret_scanning_push_protection.status == \"enabled\"",
        "secret_scanning_non_provider_patterns.status == \"disabled\"",
        "secret_scanning_validity_checks.status == \"disabled\"",
        "code-scanning/default-setup",
        "([.languages[]] | sort) == ([\"actions\", \"rust\"] | sort)",
        "dependabot/alerts?state=open&per_page=100",
        "code-scanning/alerts?state=open&per_page=100",
        "secret-scanning/alerts?state=open&per_page=100",
        "details remain in GitHub",
    ] {
        assert!(
            verifier.contains(required_contract),
            "security verifier should enforce {required_contract}"
        );
    }

    for forbidden_contract in [
        "set -x",
        ".html_url",
        ".secret |",
        ".secret,",
        ".secret)",
        ".rule",
        ".location",
        ".dismissed_by",
        "secrets.",
    ] {
        assert!(
            !verifier.contains(forbidden_contract),
            "security verifier must not inspect or print {forbidden_contract}"
        );
    }
}

#[test]
fn repository_security_evidence_keeps_paid_and_later_ticket_scope_explicit() {
    let evidence = repository_file("docs/repository-security.md");

    for required_contract in [
        "GitHub Free",
        "Dependency graph and Dependabot alerts",
        "Dependabot security updates",
        "CodeQL",
        "Secret scanning",
        "Push protection",
        "Private vulnerability reporting",
        "non-provider pattern scanning",
        "automatic partner-pattern validity checks",
        "GitHub Team with GitHub Secret Protection enabled",
        "OSPS-BR-07.01",
        "OSPS-VM-02.01",
        "not an assurance badge, certification",
    ] {
        assert!(
            evidence.contains(required_contract),
            "security evidence should document {required_contract}"
        );
    }

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    assert!(
        !root.join(".github/dependabot.yml").exists()
            && !root.join(".github/dependabot.yaml").exists(),
        "dependency version-update configuration remains MCP-033 scope"
    );
}
