use std::fs;
use std::path::PathBuf;

fn repository_file(path: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    fs::read_to_string(root.join(path)).expect("repository governance file should be readable")
}

fn assert_official_channels_exclude_insecure_transports(name: &str, document: &str) {
    for forbidden_transport in ["http://", "git://", "ftp://", "ssh://", "git@"] {
        assert!(
            !document.contains(forbidden_transport),
            "{name} must not advertise {forbidden_transport}"
        );
    }
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
        root.join(".github/dependabot.yml").is_file(),
        "MCP-033 should supply the scheduled dependency-update configuration"
    );
}

#[test]
fn contribution_and_conduct_guidance_define_safe_public_routes() {
    let contributing = repository_file("CONTRIBUTING.md");
    let conduct = repository_file("CODE_OF_CONDUCT.md");
    let support = repository_file("SUPPORT.md");

    for required_contract in [
        "## Choose the right public path",
        "bug form",
        "feature form",
        "GitHub Issues and pull-request conversations",
        "## Keep security reports and sensitive data private",
        "https://github.com/EnjoyableWork/mcp-sync/security/advisories/new",
        "synthetic, redacted values",
        "## Report a defect",
        "## Prepare a source checkout",
        "cargo clippy --workspace --all-targets --all-features --locked -- -D warnings",
        "## Submit a pull request",
        "## Contribution licensing, CLA, and DCO",
        "does not require a Contributor License Agreement (CLA)",
        "does not currently require a Developer",
        "Certificate of Origin (DCO)",
    ] {
        assert!(
            contributing.contains(required_contract),
            "contribution guide should define {required_contract}"
        );
    }

    for required_contract in [
        "## Expected behavior",
        "## Unacceptable behavior",
        "## Scope",
        "## Reporting a conduct concern",
        "03-conduct-contact.yml",
        "do not identify",
        "people in the public issue",
        "private security-reporting path",
        "## Enforcement",
        "request reconsideration",
        "single-maintainer limitation",
    ] {
        assert!(
            conduct.contains(required_contract),
            "code of conduct should define {required_contract}"
        );
    }

    for (name, document) in [
        ("CONTRIBUTING.md", contributing.as_str()),
        ("CODE_OF_CONDUCT.md", conduct.as_str()),
        ("SUPPORT.md", support.as_str()),
    ] {
        assert_official_channels_exclude_insecure_transports(name, document);
        for forbidden_claim in [
            "OSPS compliant",
            "OSPS certified",
            "independently certified",
        ] {
            assert!(
                !document.contains(forbidden_claim),
                "{name} must not publish the assurance claim {forbidden_claim}"
            );
        }
    }

    for required_contract in [
        "# mcp-sync support",
        "best-effort basis",
        "bug form",
        "feature and usage-obstacle form",
        "Only the latest public release",
        "synthetic, redacted values",
        "GitHub private vulnerability reporting",
        "detail-free public contact request",
    ] {
        assert!(
            support.contains(required_contract),
            "support policy should define {required_contract}"
        );
    }
}

#[test]
fn issue_forms_and_pull_request_template_enforce_routing_and_redaction() {
    let bug = repository_file(".github/ISSUE_TEMPLATE/01-bug-report.yml");
    let feature = repository_file(".github/ISSUE_TEMPLATE/02-feature-request.yml");
    let conduct = repository_file(".github/ISSUE_TEMPLATE/03-conduct-contact.yml");
    let chooser = repository_file(".github/ISSUE_TEMPLATE/config.yml");
    let pull_request = repository_file(".github/pull_request_template.md");

    for required_contract in [
        "name: Bug report",
        "description:",
        "labels:\n  - bug",
        "id: reproduction",
        "Minimal synthetic reproduction",
        "private vulnerability reporting",
        "raw unreviewed log",
        "required: true",
    ] {
        assert!(
            bug.contains(required_contract),
            "bug form should define {required_contract}"
        );
    }

    for required_contract in [
        "name: Feature request or usage obstacle",
        "labels:\n  - enhancement",
        "id: obstacle",
        "id: outcome",
        "id: safety",
        "opening an issue does not add the change",
        "private reporting path",
    ] {
        assert!(
            feature.contains(required_contract),
            "feature form should define {required_contract}"
        );
    }

    for required_contract in [
        "name: Private conduct contact request",
        "This issue is public",
        "do not name people or describe the incident",
        "I have not included incident details",
    ] {
        assert!(
            conduct.contains(required_contract),
            "conduct contact form should define {required_contract}"
        );
    }

    for required_contract in [
        "blank_issues_enabled: false",
        "Report a security vulnerability privately",
        "https://github.com/EnjoyableWork/mcp-sync/security/advisories/new",
        "Read the contribution and support guide",
        "Read the usage and recovery guide",
    ] {
        assert!(
            chooser.contains(required_contract),
            "issue chooser should define {required_contract}"
        );
    }

    for required_contract in [
        "## User and safety impact",
        "## Validation",
        "## Documentation and release effects",
        "private reporting",
        "synthetic, redacted examples",
        "right to submit this contribution",
        "does not require a CLA or DCO",
    ] {
        assert!(
            pull_request.contains(required_contract),
            "pull-request template should define {required_contract}"
        );
    }

    for (name, document) in [
        ("bug form", bug.as_str()),
        ("feature form", feature.as_str()),
        ("conduct form", conduct.as_str()),
        ("issue chooser", chooser.as_str()),
        ("pull-request template", pull_request.as_str()),
    ] {
        assert_official_channels_exclude_insecure_transports(name, document);
    }
}

#[test]
fn repository_inventory_and_license_evidence_cover_the_two_codebases() {
    let evidence = repository_file("docs/project-community-and-licensing.md");
    let manifest = repository_file("Cargo.toml");
    let license = repository_file("LICENSE");
    let unix_packager = repository_file("scripts/package-release.sh");
    let windows_packager = repository_file("scripts/package-release.ps1");

    for required_contract in [
        "## Official project channels",
        "https://github.com/EnjoyableWork/mcp-sync",
        "https://github.com/EnjoyableWork/homebrew-tap",
        "https://crates.io/crates/enjoyable-mcp-sync",
        "GitHub Discussions is not an enabled project channel",
        "## Repository and codebase inventory",
        "`EnjoyableWork/courtside-mcp`",
        "`EnjoyableWork/enjoyable-mcp`",
        "`EnjoyableWork/mcp-doctor`",
        "## Source and release licensing",
        "OSI-approved MIT License",
        "NOASSERTION",
        "## Inbound contribution terms",
        "no `CODEOWNERS` file",
        "OSPS-BR-03.01",
        "OSPS-DO-02.01",
        "OSPS-GV-02.01",
        "OSPS-GV-03.01",
        "OSPS-LE-02.01",
        "OSPS-LE-02.02",
        "OSPS-LE-03.01",
        "OSPS-LE-03.02",
        "OSPS-QA-04.01",
        "not an assurance badge, independent certification",
    ] {
        assert!(
            evidence.contains(required_contract),
            "project inventory should document {required_contract}"
        );
    }

    assert!(manifest.contains("license = \"MIT\""));
    assert!(manifest.contains("\"/LICENSE\""));
    assert!(license.contains("MIT License"));
    assert!(unix_packager.contains("$release_repository_root/LICENSE"));
    assert!(windows_packager.contains("(Join-Path $mcpSyncRepositoryRoot 'LICENSE')"));
    assert_official_channels_exclude_insecure_transports("project inventory", &evidence);
}

#[test]
fn public_project_contract_verifier_is_credential_free_and_exact() {
    let verifier = repository_file("scripts/verify-public-project-contract.sh");

    for required_contract in [
        "https://api.github.com/repos/$public_contract_repository/community/profile",
        ".health_percentage >= 87",
        ".files.code_of_conduct_file != null",
        ".files.contributing != null",
        ".files.pull_request_template != null",
        "contents/.github/ISSUE_TEMPLATE",
        "01-bug-report.yml",
        "02-feature-request.yml",
        "03-conduct-contact.yml",
        "contents/SUPPORT.md",
        "contents/SECURITY.md",
        "$public_contract_tap_repository/license",
        ".license.spdx_id == \"MIT\"",
        "(http://|git://|ftp://|ssh://|git@)",
        ".immutable == true",
        "verify-published-source-linux-release.sh",
        "raw.githubusercontent.com/$public_contract_repository/v$public_contract_version/LICENSE",
        "https://crates.io/api/v1/crates/enjoyable-mcp-sync/$public_contract_version/download",
        "raw.githubusercontent.com/$public_contract_tap_repository/main/Formula/mcp-sync.rb",
        "license \"MIT\"",
    ] {
        assert!(
            verifier.contains(required_contract),
            "public project verifier should enforce {required_contract}"
        );
    }

    for forbidden_contract in ["gh api", "GH_TOKEN", "Authorization:", "secrets."] {
        assert!(
            !verifier.contains(forbidden_contract),
            "public project verifier must not depend on {forbidden_contract}"
        );
    }

    assert!(
        !verifier.contains(".files.issue_template != null"),
        "GitHub does not expose YAML issue forms through that community-profile field"
    );
    assert!(
        !verifier.contains(".health_percentage == 100"),
        "GitHub's REST score must not override recognized YAML forms in the UI"
    );
}

#[test]
fn codeowners_remains_absent_without_an_independent_reviewer() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    for codeowners_path in [
        root.join("CODEOWNERS"),
        root.join(".github/CODEOWNERS"),
        root.join("docs/CODEOWNERS"),
    ] {
        assert!(
            !codeowners_path.exists(),
            "CODEOWNERS requires a real independent reviewer: {}",
            codeowners_path.display()
        );
    }
}
