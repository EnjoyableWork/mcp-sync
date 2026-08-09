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
        "web_commit_signoff_required == false",
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
