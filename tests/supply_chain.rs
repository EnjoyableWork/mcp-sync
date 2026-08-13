use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const EXPECTED_DIRECT_ACTIONS: [&str; 9] = [
    "Azure/artifact-signing-action",
    "Azure/login",
    "EmbarkStudios/cargo-deny-action",
    "Homebrew/actions/setup-homebrew",
    "actions/attest-build-provenance",
    "actions/checkout",
    "actions/download-artifact",
    "actions/upload-artifact",
    "rust-lang/crates-io-auth-action",
];

const EXPECTED_ALLOWED_PATTERNS: [&str; 10] = [
    "Azure/artifact-signing-action@*",
    "Azure/login@*",
    "EmbarkStudios/cargo-deny-action@*",
    "Homebrew/actions/setup-homebrew@*",
    "actions/attest-build-provenance@*",
    "actions/attest@*",
    "actions/checkout@*",
    "actions/download-artifact@*",
    "actions/upload-artifact@*",
    "rust-lang/crates-io-auth-action@*",
];

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn repository_file(path: &str) -> String {
    fs::read_to_string(repository_root().join(path))
        .unwrap_or_else(|error| panic!("{path} should be readable: {error}"))
}

fn workflow_paths() -> Vec<PathBuf> {
    let mut paths: Vec<_> = fs::read_dir(repository_root().join(".github/workflows"))
        .expect("workflow directory should be readable")
        .map(|entry| entry.expect("workflow entry should be readable").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "yml"))
        .collect();
    paths.sort();
    paths
}

fn workflow_name(path: &Path) -> &str {
    path.file_name()
        .and_then(|name| name.to_str())
        .expect("workflow filename should be UTF-8")
}

#[test]
fn dependabot_groups_weekly_cargo_and_actions_updates() {
    let dependabot = repository_file(".github/dependabot.yml");

    assert_eq!(dependabot.matches("package-ecosystem:").count(), 2);
    assert_eq!(dependabot.matches("interval: weekly").count(), 2);
    assert_eq!(
        dependabot.matches("patterns:\n          - \"*\"").count(),
        2
    );
    for required in [
        "package-ecosystem: cargo",
        "package-ecosystem: github-actions",
        "directory: /",
        "cargo-dependencies:",
        "github-actions:",
        "timezone: America/Toronto",
        "open-pull-requests-limit: 5",
        "rebase-strategy: auto",
    ] {
        assert!(
            dependabot.contains(required),
            "Dependabot should define {required}"
        );
    }
}

#[test]
fn every_workflow_action_is_sha_pinned_hinted_and_selected() {
    let evidence = repository_file("docs/repository-supply-chain.md");
    let policy: serde_json::Value =
        serde_json::from_str(&repository_file(".github/actions-policy.json"))
            .expect("selected-action policy should be valid JSON");
    assert_eq!(policy["github_owned_allowed"], false);
    assert_eq!(policy["verified_allowed"], false);

    let allowed: BTreeSet<_> = policy["patterns_allowed"]
        .as_array()
        .expect("selected action patterns should be an array")
        .iter()
        .map(|value| value.as_str().expect("action pattern should be a string"))
        .collect();
    assert_eq!(
        allowed,
        EXPECTED_ALLOWED_PATTERNS.into_iter().collect(),
        "the selected-action policy should contain only reviewed direct and transitive actions"
    );

    let mut direct_actions = BTreeSet::new();
    let mut reference_count = 0;
    for path in workflow_paths() {
        let workflow = fs::read_to_string(&path).expect("workflow should be readable");
        for line in workflow.lines() {
            let Some(raw_reference) = line.trim().strip_prefix("uses: ") else {
                continue;
            };
            if raw_reference.starts_with("./") {
                continue;
            }
            let (reference, hint) = raw_reference.split_once(" # ").unwrap_or_else(|| {
                panic!(
                    "{} action reference should have an update hint: {raw_reference}",
                    workflow_name(&path)
                )
            });
            let (action, revision) = reference.rsplit_once('@').unwrap_or_else(|| {
                panic!(
                    "{} action reference should have a revision: {reference}",
                    workflow_name(&path)
                )
            });
            assert!(
                revision.len() == 40 && revision.bytes().all(|byte| byte.is_ascii_hexdigit()),
                "{} action {action} should use a full commit SHA",
                workflow_name(&path)
            );
            assert!(
                !hint.trim().is_empty(),
                "{} action {action} should have a nonempty update hint",
                workflow_name(&path)
            );
            let hint_token = hint
                .split([';', ' '])
                .next()
                .expect("action hint should have a first token");
            for reviewed_value in [action, revision, hint_token] {
                assert!(
                    evidence.contains(reviewed_value),
                    "the reviewed inventory should record {reviewed_value}"
                );
            }
            assert!(
                allowed.contains(format!("{action}@*").as_str()),
                "{action} should be in the selected-action policy"
            );
            direct_actions.insert(action.to_owned());
            reference_count += 1;
        }
    }

    assert!(
        reference_count > 40,
        "all workflow references should be inspected"
    );
    assert_eq!(
        direct_actions,
        EXPECTED_DIRECT_ACTIONS
            .into_iter()
            .map(str::to_owned)
            .collect(),
        "the direct action inventory should not drift silently"
    );

    let transitive = &policy["reviewed_transitive_actions"][0];
    assert_eq!(transitive["action"], "actions/attest");
    assert_eq!(
        transitive["parent_action"],
        "actions/attest-build-provenance"
    );
    assert_eq!(
        transitive["parent_sha"],
        "4d101475d8b20a2381f78447822ac1eab6504dd8"
    );
    assert_eq!(
        transitive["sha"],
        "508db95dd578ae2727ebd6217d5ba78e4fbda05d"
    );
    assert_eq!(transitive["update_hint"], "v4.2.1");
}

#[test]
fn untrusted_workflows_are_read_only_secretless_and_not_privileged() {
    for path in workflow_paths() {
        let name = workflow_name(&path);
        let workflow = fs::read_to_string(&path).expect("workflow should be readable");

        assert!(!workflow.contains("pull_request_target:"));
        assert!(!workflow.contains("workflow_run:"));

        let lines: Vec<_> = workflow.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            if line.contains("uses: actions/checkout@") {
                let end = usize::min(index + 6, lines.len());
                assert!(
                    lines[index..end]
                        .iter()
                        .any(|candidate| candidate.contains("persist-credentials: false")),
                    "{name} checkout on line {} should not persist credentials",
                    index + 1
                );
            }
        }

        if workflow.contains("  pull_request:") {
            for forbidden in [
                "secrets.",
                "secrets[",
                "secrets:",
                "environment:",
                "run-id:",
                "github-token:",
                ": write",
                "permissions: write-all",
            ] {
                assert!(
                    !workflow.contains(forbidden),
                    "pull-request workflow {name} must not contain {forbidden}"
                );
            }
        }
    }

    for untrusted_workflow in [
        ".github/workflows/ci.yml",
        ".github/workflows/release-preflight.yml",
        ".github/workflows/source-linux-release-preflight.yml",
    ] {
        let workflow = repository_file(untrusted_workflow);
        assert!(workflow.contains("pull_request:"));
        assert!(workflow.contains("contents: read"));
    }
}

#[test]
fn ci_runs_once_per_pull_request_commit_and_again_after_main_merge() {
    let ci = repository_file(".github/workflows/ci.yml");
    let trigger_end = ci
        .find("\npermissions:")
        .expect("CI should define an explicit permissions boundary");
    let triggers = &ci[..trigger_end];

    assert!(triggers.contains("  pull_request:\n"));
    assert!(
        triggers.contains("  push:\n    branches:\n      - main\n"),
        "CI branch pushes should be limited to main so an open pull request does not run twice"
    );
    assert_eq!(triggers.matches("  pull_request:").count(), 1);
    assert_eq!(triggers.matches("  push:").count(), 1);

    for protected_context in [
        "name: Dependency policy",
        "name: Linux ${{ matrix.architecture }} — format, Clippy, and test",
        "name: Windows ${{ matrix.architecture }} — format, Clippy, and test",
    ] {
        assert!(
            ci.contains(protected_context),
            "CI should retain protected check context template {protected_context}"
        );
    }
}

#[test]
fn windows_arm64_health_readiness_is_native_sequential_and_fail_fast() {
    let manifest = repository_file("Cargo.toml");
    let health = repository_file("tests/health.rs");
    let containment = repository_file("tests/health_process_containment.rs");
    let integration_support = repository_file("tests/support/mod.rs");
    let health_unit = repository_file("src/health.rs");
    let ci = repository_file(".github/workflows/ci.yml");

    assert!(
        manifest
            .contains("[[test]]\nname = \"health\"\npath = \"tests/health.rs\"\nharness = false")
    );
    for native_readiness_contract in [
        "std::env::current_exe()",
        "READY_PATH",
        "RELEASE_PATH",
        "let deadline = Instant::now() + COMMAND_TIMEOUT",
        "prove_explicit_fixture_readiness_and_complete_handshake",
    ] {
        assert!(
            health.contains(native_readiness_contract),
            "the sequential health harness should retain {native_readiness_contract}"
        );
    }
    assert!(!health.contains("FIXTURE_STATE_TIMEOUT"));
    for forbidden_interpreter_fixture in ["powershell", "PowerShell", "shell_server"] {
        assert!(!health.contains(forbidden_interpreter_fixture));
        assert!(!containment.contains(forbidden_interpreter_fixture));
        assert!(!integration_support.contains(forbidden_interpreter_fixture));
    }
    for removed_unit_timing_mechanism in [
        "RESPONSIVE_PROCESS_FIXTURE_TIMEOUT",
        "PROCESS_FIXTURE_LOCK",
        "process_fixture_lock",
    ] {
        assert!(!health_unit.contains(removed_unit_timing_mechanism));
    }
    assert!(health_unit.contains("bounded_response_framing_is_proved_without_starting_a_process"));
    assert!(health_unit.contains("assert_eq!(RESPONSE_TIMEOUT, Duration::from_secs(5))"));

    let repeat_start = ci
        .find("- name: Repeat deterministic native health readiness on Windows ARM64")
        .expect("CI should retain the focused Windows ARM64 evidence step");
    let repeat = &ci[repeat_start..];
    for required in [
        "if: matrix.architecture == 'ARM64'",
        "$mcpSyncIteration -le 10",
        "cargo test --test health --locked",
        "if ($LASTEXITCODE -ne 0)",
        "throw \"Focused native health run $mcpSyncIteration failed",
    ] {
        assert!(
            repeat.contains(required),
            "focused Windows ARM64 evidence should retain {required}"
        );
    }
    for forbidden_mask in ["continue-on-error", "retry", "rerun"] {
        assert!(!repeat.contains(forbidden_mask));
    }
}

#[test]
fn source_workflow_policy_rejects_cross_repository_homebrew_authority() {
    let obsolete =
        repository_root().join(".github/workflows/source-linux-release-publish-homebrew.yml");
    let verifier = repository_file("scripts/verify-workflow-supply-chain.sh");
    let focused = repository_file("scripts/test-homebrew-authority-policy.sh");

    assert!(!obsolete.exists());
    for forbidden in [
        "HOMEBREW_TAP_DEPLOY_KEY",
        "git@github.com:EnjoyableWork/homebrew-tap.git",
        "GIT_SSH_COMMAND=",
        "repository_dispatch:",
    ] {
        assert!(verifier.contains(forbidden));
        assert!(focused.contains(forbidden));
    }
    assert!(verifier.contains("source workflows retain forbidden Homebrew write authority"));
    assert!(focused.contains("publish-mcp-sync.yml"));
}

#[test]
fn tracked_artifact_gate_has_acceptance_and_rejection_evidence() {
    let ci = repository_file(".github/workflows/ci.yml");
    let verifier = repository_file("scripts/verify-repository-artifacts.sh");
    let focused_test = repository_file("scripts/test-repository-artifact-policy.sh");
    let ignore = repository_file(".gitignore");

    for gate in [
        "./scripts/verify-workflow-supply-chain.sh",
        "./scripts/verify-repository-artifacts.sh",
        "./scripts/test-repository-artifact-policy.sh",
    ] {
        assert!(ci.contains(gate), "Dependency policy should run {gate}");
    }
    for required in [
        "git -C \"$repository_artifact_root\" ls-files --stage -z",
        "repository_artifact_mode\" == 160000",
        "tracked generated or executable artifact extension is not permitted",
        "file -b --mime-encoding -",
        "tracked binary content is not permitted",
    ] {
        assert!(
            verifier.contains(required),
            "artifact verifier should contain {required}"
        );
    }
    for required in [
        "reviewable.sh",
        "generated-program",
        "generated.exe",
        "repository artifact policy accepted binary content",
        "accepted a generated executable extension",
    ] {
        assert!(
            focused_test.contains(required),
            "artifact test should contain {required}"
        );
    }
    for ignored in [
        "/release-assets/",
        "*.crate",
        "*.exe",
        "*.dylib",
        "*.so",
        "*.wasm",
        "*.zip",
    ] {
        assert!(
            ignore.contains(ignored),
            "source ignore policy should include {ignored}"
        );
    }
}

#[test]
fn operator_verifiers_encode_live_policy_and_authenticated_distribution() {
    let operator = repository_file("scripts/verify-repository-supply-chain-controls.sh");
    let workflow = repository_file("scripts/verify-workflow-supply-chain.sh");
    let distribution = repository_file("scripts/verify-distribution-authentication.sh");
    let evidence = repository_file("docs/repository-supply-chain.md");

    for required in [
        "actions/permissions",
        "allowed_actions == \"selected\"",
        "sha_pinning_required == true",
        "actions/permissions/selected-actions",
        "github_owned_allowed == false",
        "verified_allowed == false",
        "default_workflow_permissions == \"read\"",
        "can_approve_pull_request_reviews == false",
        "first_time_contributors",
        "code-scanning/default-setup",
        "contents/.github/dependabot.yml?ref=main",
        "commits/main",
        "clean checkout of exact main",
        "git -C \"$repository_supply_chain_root\" hash-object .github/dependabot.yml",
        "--verify-upstreams",
    ] {
        assert!(
            operator.contains(required),
            "operator verifier should require {required}"
        );
    }
    assert!(!operator.contains("secrets."));
    assert!(operator.contains("EnjoyableWork/mcp-sync 0.1.1"));
    assert!(!operator.contains("EnjoyableWork/mcp-sync 0.1.0"));

    for required in [
        "permissions:.*write",
        "secrets(\\.|\\[)",
        "environment:",
        "run-id:",
        "github-token:",
    ] {
        assert!(
            workflow.contains(required),
            "workflow verifier should reject {required} in pull-request workflows"
        );
    }
    assert!(distribution.contains("EnjoyableWork/mcp-sync 0.1.1"));
    assert!(!distribution.contains("EnjoyableWork/mcp-sync 0.1.0"));

    for required in [
        "scripts/syft-assets.txt",
        "https://api.github.com/repos/anchore/syft/releases/tags/v1.50.0",
        ".digest == $digest",
    ] {
        assert!(
            workflow.contains(required),
            "workflow verifier should enforce pinned Syft contract {required}"
        );
    }

    for required in [
        ".immutable == true",
        "gh release verify",
        "gh attestation verify",
        "--signer-workflow",
        "--source-ref",
        "--source-digest",
        "https://slsa.dev/provenance/v1",
        "https://actions.github.io/buildtypes/workflow/v1",
        "runner_environment",
        "https://token.actions.githubusercontent.com",
        "certificate.runnerEnvironment",
        "--proto '=https'",
        "crates.io/api/v1/crates/enjoyable-mcp-sync",
        "raw.githubusercontent.com/EnjoyableWork/homebrew-tap/main/Formula/mcp-sync.rb",
        "cmp --silent",
    ] {
        assert!(
            distribution.contains(required),
            "distribution verifier should require {required}"
        );
    }

    for control in [
        "OSPS-BR-01.01",
        "OSPS-BR-01.03",
        "OSPS-BR-03.02",
        "OSPS-QA-05.01",
        "OSPS-QA-05.02",
    ] {
        assert!(evidence.contains(control), "evidence should map {control}");
    }
    for limitation in [
        "does not modify either immutable release",
        "not a project-wide conformance or certification claim",
        "There is no current WinGet distribution",
        "Cargo publication after `0.1.0` uses one protected",
        "No crates.io API token may be created, stored, or used as fallback",
    ] {
        assert!(
            evidence.contains(limitation),
            "evidence should retain {limitation}"
        );
    }
}
