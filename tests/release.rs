use std::fs;
use std::path::Path;

const SIGNED_TARGETS: [&str; 6] = [
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "aarch64-unknown-linux-gnu",
    "x86_64-unknown-linux-gnu",
    "aarch64-pc-windows-msvc",
    "x86_64-pc-windows-msvc",
];

const SIGNED_RUNNERS: [&str; 6] = [
    "macos-15",
    "macos-15-intel",
    "ubuntu-24.04-arm",
    "ubuntu-24.04",
    "windows-11-arm",
    "windows-2025",
];

const SOURCE_LINUX_TARGETS: [&str; 2] = ["aarch64-unknown-linux-gnu", "x86_64-unknown-linux-gnu"];

const SOURCE_LINUX_RUNNERS: [&str; 2] = ["ubuntu-24.04-arm", "ubuntu-24.04"];

fn repository_file(relative_path: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()))
}

fn assert_actions_are_commit_pinned(workflow: &str) {
    for line in workflow.lines() {
        let Some(reference) = line.trim().strip_prefix("uses:") else {
            continue;
        };
        let reference = reference
            .split_whitespace()
            .next()
            .expect("an action reference should follow uses:");
        let (_, revision) = reference
            .rsplit_once('@')
            .unwrap_or_else(|| panic!("action reference {reference} should have a revision"));
        assert!(
            revision.len() == 40 && revision.bytes().all(|byte| byte.is_ascii_hexdigit()),
            "action reference {reference} should use a full commit SHA"
        );
    }
}

fn assert_crates_io_requests_identify_the_client(workflow: &str, user_agent: &str) {
    let lines: Vec<_> = workflow.lines().collect();
    let mut request_count = 0;

    for (index, line) in lines.iter().enumerate() {
        if !line.contains("https://crates.io/api/v1/crates") {
            continue;
        }
        request_count += 1;
        let request_start = index.saturating_sub(6);
        assert!(
            lines[request_start..index]
                .iter()
                .any(|candidate| candidate.contains(user_agent)),
            "crates.io request on line {} must identify the client with {user_agent}",
            index + 1
        );
    }

    assert!(
        request_count > 0,
        "workflow should contain a crates.io request"
    );
}

fn assert_unix_sbom_generation(workflow: &str, expected_calls: usize) {
    assert_eq!(
        workflow.matches("scripts/generate-sbom.sh").count(),
        expected_calls,
        "workflow should use the repository-owned Unix SBOM generator"
    );
    assert!(workflow.contains("ARCHIVE_PATH:"));
    assert!(workflow.contains("SBOM_PATH:"));
    assert!(!workflow.contains("anchore/sbom-action"));
    assert!(!workflow.contains("syft-version:"));
}

fn assert_windows_sbom_generation(workflow: &str, expected_calls: usize) {
    assert_eq!(
        workflow.matches("scripts/generate-sbom.ps1").count(),
        expected_calls,
        "workflow should use the repository-owned Windows SBOM generator"
    );
    if expected_calls > 0 {
        assert!(workflow.contains(
            "./scripts/generate-sbom.ps1 -Archive $env:ARCHIVE_PATH -Output $env:SBOM_PATH"
        ));
        assert!(workflow.contains("MCP_SYNC_SYFT_HOST_ARCHITECTURE: ${{ matrix.architecture }}"));
    }
}

#[test]
fn release_preflight_covers_every_native_artifact_without_credentials() {
    let workflow = repository_file(".github/workflows/release-preflight.yml");

    for target in SIGNED_TARGETS {
        assert!(
            workflow.contains(target),
            "release preflight should include native target {target}"
        );
    }
    for runner in SIGNED_RUNNERS {
        assert!(
            workflow.contains(&format!("runner: {runner}")),
            "release preflight should use native runner {runner}"
        );
    }

    assert!(workflow.contains("scripts/smoke-archive.sh"));
    assert!(workflow.contains("scripts/smoke-archive.ps1"));
    assert_eq!(
        workflow
            .matches("Prove native health-process containment")
            .count(),
        2,
        "both Unix and Windows preflight matrices should run the focused containment proof"
    );
    assert_eq!(
        workflow
            .matches("cargo test --test health_process_containment --locked")
            .count(),
        2,
        "all six native preflight hosts should execute the harness-free containment regression"
    );
    assert!(workflow.contains("Deterministic Cargo source package"));
    assert!(workflow.contains("cargo package --locked"));
    assert!(workflow.contains("scripts/verify-release-assets.sh"));
    assert!(workflow.contains("scripts/verify-published-release.sh"));
    assert_unix_sbom_generation(&workflow, 1);
    assert_windows_sbom_generation(&workflow, 1);
    assert!(workflow.contains("retention-days: 1"));
    assert!(
        !workflow.contains("secrets."),
        "untrusted preflight events must not receive release secrets"
    );
    assert!(
        !workflow.contains("attest-build-provenance"),
        "test builds should not create release provenance"
    );
    assert!(
        !workflow.contains("id-token: write") && !workflow.contains("contents: write"),
        "preflight should remain read-only"
    );
    assert_actions_are_commit_pinned(&workflow);
}

#[test]
fn funded_signed_workflow_is_explicit_and_preserves_the_full_trust_contract() {
    let workflow = repository_file(".github/workflows/release.yml");

    for target in SIGNED_TARGETS {
        assert!(
            workflow.contains(target),
            "stable release should include native target {target}"
        );
    }
    for runner in SIGNED_RUNNERS {
        assert!(
            workflow.contains(&format!("runner: {runner}")),
            "stable release should use native runner {runner}"
        );
    }

    for required_contract in [
        "workflow_dispatch:",
        "confirm_funded_signing:",
        "confirm_repository_controls:",
        "funded signing must be explicitly confirmed on manual dispatch",
        "operator-side release repository controls must be explicitly confirmed",
        "funded release dispatch must select an existing stable tag",
        "scripts/validate-release-version.sh",
        "future",
        "group: mcp-sync-release",
        "an existing Cargo version is recoverable only from its immutable GitHub Release",
        "environment: release",
        "name: release",
        "refs/tags/v*",
        "actions: read",
        "release-authorize.yml",
        "release has no successful protected authorization for this exact commit",
        "scripts/verify-public-stable-tag-ruleset.sh",
        "Developer ID Application",
        "--identifier com.enjoyablework.mcp-sync",
        "--options runtime",
        "notarytool submit",
        ".status == \"Accepted\"",
        "spctl --assess --type execute",
        "Azure/login@",
        "Azure/artifact-signing-action@",
        "Get-AuthenticodeSignature",
        "TimeStamperCertificate",
        "actions/attest-build-provenance@",
        "Package and attest the deterministic Cargo source",
        "enjoyable-mcp-sync-${{ env.RELEASE_VERSION }}.crate",
        "name: release-source",
        "scripts/verify-release-assets.sh",
        "scripts/verify-published-release.sh",
        "gh release create",
        "--draft",
        "gh release edit \"$GITHUB_REF_NAME\" --draft=false",
        "gh release verify",
    ] {
        assert!(
            workflow.contains(required_contract),
            "stable workflow should enforce {required_contract}"
        );
    }

    for protected_input in [
        "MACOS_CERTIFICATE_P12_BASE64",
        "MACOS_CERTIFICATE_PASSWORD",
        "APPLE_NOTARY_KEY_P8_BASE64",
        "APPLE_NOTARY_KEY_ID",
        "APPLE_NOTARY_ISSUER_ID",
        "AZURE_CLIENT_ID",
        "AZURE_TENANT_ID",
        "AZURE_SUBSCRIPTION_ID",
        "ARTIFACT_SIGNING_ENDPOINT",
        "ARTIFACT_SIGNING_ACCOUNT",
        "ARTIFACT_SIGNING_PROFILE",
    ] {
        assert!(
            workflow.contains(protected_input),
            "stable workflow should require protected input {protected_input}"
        );
    }

    assert!(workflow.contains("id-token: write"));
    assert!(workflow.contains("attestations: write"));
    assert!(workflow.contains("contents: write"));
    assert!(workflow.contains("persist-credentials: false"));
    assert!(!workflow.contains("repos/$GITHUB_REPOSITORY/immutable-releases"));
    assert!(
        !workflow.contains("  push:"),
        "a normal stable tag must not invoke paid signing"
    );
    assert!(
        !workflow.contains("cargo publish")
            && !workflow.contains("brew install")
            && !workflow.contains("winget install"),
        "downstream publication must wait for the immutable GitHub Release"
    );
    assert_unix_sbom_generation(&workflow, 2);
    assert_windows_sbom_generation(&workflow, 1);
    assert_actions_are_commit_pinned(&workflow);
}

#[test]
fn source_linux_tag_workflow_publishes_only_attested_linux_and_source_outputs() {
    let workflow = repository_file(".github/workflows/source-linux-release.yml");

    for target in SOURCE_LINUX_TARGETS {
        assert!(workflow.contains(target));
    }
    for runner in SOURCE_LINUX_RUNNERS {
        assert!(workflow.contains(&format!("runner: {runner}")));
    }
    for excluded_target in [
        "aarch64-apple-darwin",
        "x86_64-apple-darwin",
        "aarch64-pc-windows-msvc",
        "x86_64-pc-windows-msvc",
    ] {
        assert!(
            !workflow.contains(excluded_target),
            "source and GNU/Linux release must not contain project-issued target {excluded_target}"
        );
    }
    for forbidden_contract in [
        "Developer ID Application",
        "notarytool",
        "Azure/login@",
        "Azure/artifact-signing-action@",
        "Get-AuthenticodeSignature",
        "MACOS_CERTIFICATE",
        "ARTIFACT_SIGNING",
        "secrets.",
    ] {
        assert!(
            !workflow.contains(forbidden_contract),
            "source and GNU/Linux release must not depend on {forbidden_contract}"
        );
    }
    for required_contract in [
        "workflow_dispatch:",
        "default: rehearse",
        "- rehearse",
        "- publish",
        "group: mcp-sync-release",
        "Validate the source release request before protected access",
        "needs: request",
        "canonical later version on its exact protected tag",
        "source and GNU/Linux publication requires an exact protected tag dispatch",
        "scripts/validate-release-version.sh",
        "future",
        "registry_version_exists",
        "an existing Cargo version is recoverable only from its immutable GitHub Release",
        "environment: release",
        "actions: read",
        "release-authorize.yml",
        "release has no successful protected authorization for this exact commit",
        "scripts/verify-public-stable-tag-ruleset.sh",
        "cargo package --locked",
        "scripts/generate-source-linux-release-channels.sh",
        "scripts/verify-source-linux-release-assets.sh",
        "scripts/verify-published-source-linux-release.sh",
        "actions/attest-build-provenance@",
        "enjoyable-mcp-sync-${{ env.RELEASE_VERSION }}.crate",
        "release-assets/mcp-sync.rb",
        "gh release create",
        "--draft",
        "gh release verify",
        "contents: write",
        "id-token: write",
        "attestations: write",
        "Rehearse immutable source and downstream byte handoffs",
        "the live repeat-release rehearsal must use exact protected main",
        "Synthetic Cargo and Homebrew mismatches: rejected",
        "No tag, release, crate, formula, or credential was created or changed.",
    ] {
        assert!(
            workflow.contains(required_contract),
            "source and GNU/Linux workflow should enforce {required_contract}"
        );
    }
    let request_start = workflow
        .find("  request:\n")
        .expect("source release should validate the request first");
    let validate_start = workflow
        .find("  validate:\n")
        .expect("source release should retain protected validation");
    assert!(request_start < validate_start);
    let request = &workflow[request_start..validate_start];
    assert!(request.contains("permissions: {}"));
    assert!(!request.contains("environment:"));
    assert!(!request.contains("secrets."));
    assert!(!request.contains("uses: actions/checkout"));
    assert!(!workflow.contains("repos/$GITHUB_REPOSITORY/immutable-releases"));
    assert!(!workflow.contains("  push:"));
    assert_unix_sbom_generation(&workflow, 1);
    assert_windows_sbom_generation(&workflow, 0);
    assert_actions_are_commit_pinned(&workflow);
}

#[test]
fn source_linux_preflight_proves_native_source_installs_and_exact_payload_without_secrets() {
    let workflow = repository_file(".github/workflows/source-linux-release-preflight.yml");

    for target in SIGNED_TARGETS {
        assert!(
            workflow.contains(target),
            "source and GNU/Linux preflight should source-install on {target}"
        );
    }
    for runner in SIGNED_RUNNERS {
        assert!(workflow.contains(&format!("runner: {runner}")));
    }
    for required_contract in [
        "scripts/verify-public-stable-tag-ruleset.sh",
        "cargo package --locked",
        "scripts/generate-source-linux-release-channels.sh",
        "cargo install",
        "brew install --build-from-source",
        "brew audit --strict \"$formula_name\"",
        "scripts/smoke-installed.sh",
        "scripts/smoke-installed.ps1",
        "scripts/smoke-archive.sh",
        "scripts/verify-source-linux-release-assets.sh",
        "scripts/verify-published-source-linux-release.sh",
        "retention-days: 1",
    ] {
        assert!(workflow.contains(required_contract));
    }
    assert!(!workflow.contains("secrets."));
    assert!(!workflow.contains("contents: write"));
    assert!(!workflow.contains("id-token: write"));
    assert!(!workflow.contains("attest-build-provenance"));
    assert_unix_sbom_generation(&workflow, 1);
    assert_windows_sbom_generation(&workflow, 0);
    assert_actions_are_commit_pinned(&workflow);
}

#[test]
fn syft_acquisition_is_exact_native_bounded_and_fail_closed() {
    let manifest = repository_file("scripts/syft-assets.txt");
    let unix = repository_file("scripts/generate-sbom.sh");
    let windows = repository_file("scripts/generate-sbom.ps1");
    let unix_test = repository_file("scripts/test-generate-sbom.sh");
    let windows_test = repository_file("scripts/test-generate-sbom.ps1");

    for expected_record in [
        "darwin arm64 syft_1.50.0_darwin_arm64.tar.gz e32fdb9d47823fa633748a1efca2528fd77c37469ea93c9e40ab835da44e4cce",
        "darwin x86_64 syft_1.50.0_darwin_amd64.tar.gz d11a8c7bc27114853bd7c1e1b2f3be3ddda3a1de17aee585329f04c369341c75",
        "linux arm64 syft_1.50.0_linux_arm64.tar.gz 887c57cbcc2d0e8c5c110a4571a3fc7150058b24d74f993ee4663516e5c8ce86",
        "linux x86_64 syft_1.50.0_linux_amd64.tar.gz bf7b29ff57f06da30918266a0e1c2885a8f99784798d1bdb1628886aa015d788",
        "windows arm64 syft_1.50.0_windows_arm64.zip 5eb435eb8750737d12e66f5a145975b4027adf20076b518079af38b2148d55a5",
        "windows amd64 syft_1.50.0_windows_amd64.zip 815ee6973ec5dff6a671d7f41b0e78835a8c45b91d5a39f4743ea1cee833d3be",
    ] {
        assert!(
            manifest.contains(expected_record),
            "Syft manifest should retain {expected_record}"
        );
    }

    for required_contract in [
        "https://github.com/anchore/syft/releases/download/v${sbom_syft_version}",
        "sbom_maximum_attempts=5",
        "--proto '=https'",
        "--proto-redir '=https'",
        "408 | 429 | 500 | 502 | 503 | 504",
        "5 | 6 | 7 | 16 | 18 | 28 | 35 | 52 | 55 | 56 | 92",
        "sbom_delay=$((1 << (sbom_attempt - 1)))",
        "downloaded Syft asset failed SHA-256 verification",
        "SYFT_CHECK_FOR_APP_UPDATE=false",
        "spdx-json=$sbom_output_temp",
    ] {
        assert!(
            unix.contains(required_contract),
            "Unix Syft generator should retain {required_contract}"
        );
    }
    for required_contract in [
        "$mcpSyncMaximumAttempts = 5",
        "408, 429, 500, 502, 503, 504",
        "System.Net.Http.HttpRequestException",
        "System.TimeoutException",
        "System.OperationCanceledException",
        "System.Net.Sockets.SocketException",
        "Syft download URI must use HTTPS.",
        "Get-FileHash -LiteralPath $mcpSyncDownload -Algorithm SHA256",
        "$env:SYFT_CHECK_FOR_APP_UPDATE = 'false'",
        "spdx-json=$mcpSyncOutputTemp",
    ] {
        assert!(
            windows.contains(required_contract),
            "Windows Syft generator should retain {required_contract}"
        );
    }
    for forbidden_installer in [
        "raw.githubusercontent.com/anchore/syft/main/install.sh",
        "anchore/sbom-action",
    ] {
        assert!(!unix.contains(forbidden_installer));
        assert!(!windows.contains(forbidden_installer));
    }
    for focused_evidence in [
        "SBOM generator accepted a checksum mismatch",
        "SBOM generator accepted a permanent HTTP failure",
        "SBOM generator accepted exhausted transient failures",
        "SBOM generator accepted an unsupported host",
        "Syft SBOM generation policy tests passed",
    ] {
        assert!(unix_test.contains(focused_evidence));
    }
    assert!(windows_test.contains("Unsupported Syft host architecture."));
    assert!(windows_test.contains("Windows Syft SBOM generation policy tests passed."));
}

#[test]
fn source_linux_channel_verifier_is_read_only_and_covers_every_represented_install() {
    let workflow = repository_file(".github/workflows/source-linux-release-channels.yml");

    for target in SIGNED_TARGETS {
        assert!(
            workflow.contains(target),
            "Cargo channel smoke should include {target}"
        );
    }
    for runner in SIGNED_RUNNERS {
        assert!(workflow.contains(&format!("runner: {runner}")));
    }
    for required_contract in [
        "scripts/verify-published-source-linux-release.sh",
        ".immutable == true",
        "gh release verify",
        "gh attestation verify",
        ".github/workflows/source-linux-release.yml",
        "cargo install enjoyable-mcp-sync",
        "brew install --build-from-source EnjoyableWork/tap/mcp-sync",
        "scripts/smoke-archive.sh",
        "scripts/smoke-installed.sh",
        "scripts/smoke-installed.ps1",
        "EnjoyableWork/homebrew-tap/main/Formula/mcp-sync.rb",
        "release-policy/scripts/validate-release-version.sh",
        "published-channel verification must use exact protected main",
        "canonical stable version",
        ".crate.trustpub_only == true",
    ] {
        assert!(workflow.contains(required_contract));
    }
    for forbidden_contract in [
        "winget install",
        "verify-macos-release-trust",
        "verify-windows-release-trust",
        "contents: write",
        "id-token: write",
        "secrets.",
    ] {
        assert!(!workflow.contains(forbidden_contract));
    }
    assert_crates_io_requests_identify_the_client(
        &workflow,
        "User-Agent: mcp-sync-source-linux-channel-verifier/0.1",
    );
    assert_actions_are_commit_pinned(&workflow);
}

#[test]
fn homebrew_publication_is_tap_owned_and_source_retains_no_cross_repository_authority() {
    let obsolete_workflow = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".github/workflows/source-linux-release-publish-homebrew.yml");
    let formula_policy = repository_file("scripts/validate-homebrew-formula-update.sh");
    let formula_policy_test = repository_file("scripts/test-homebrew-formula-update-policy.sh");
    let authority_policy_test = repository_file("scripts/test-homebrew-authority-policy.sh");
    let workflow_policy = repository_file("scripts/verify-workflow-supply-chain.sh");
    let runbook = repository_file("docs/source-linux-release.md");
    let project = repository_file("PROJECT.md");
    let ci = repository_file(".github/workflows/ci.yml");

    assert!(
        !obsolete_workflow.exists(),
        "source repository must not retain its former Homebrew writer"
    );
    for required_contract in [
        "publish-mcp-sync.yml",
        "--repo EnjoyableWork/homebrew-tap",
        "--ref main",
        "-f mode=publish",
        "job-scoped `GITHUB_TOKEN`",
    ] {
        assert!(
            runbook.contains(required_contract),
            "source runbook should delegate {required_contract} to the tap"
        );
    }
    for forbidden_contract in [
        "HOMEBREW_TAP_DEPLOY_KEY",
        "git@github.com:EnjoyableWork/homebrew-tap.git",
        "GIT_SSH_COMMAND=",
        "repository_dispatch:",
    ] {
        assert!(workflow_policy.contains(forbidden_contract));
        assert!(
            workflow_policy.contains("source workflows retain forbidden Homebrew write authority")
        );
        assert!(authority_policy_test.contains(forbidden_contract));
    }
    assert!(workflow_policy.contains("must not retain a cross-repository Homebrew publisher"));
    assert!(authority_policy_test.contains("publish-mcp-sync.yml"));
    assert!(authority_policy_test.contains("zero source secrets and zero tap deploy keys"));
    assert!(project.contains("| DEC-054 | Move source-built Homebrew publication into the tap with job-scoped authority | Accepted |"));
    for required_contract in [
        "canonical immutable release URL",
        "published formula must be a regular, non-symbolic-link file",
        "refusing to replace different formula bytes for the same immutable version",
        "future",
        "printf 'create\\n'",
        "printf 'noop\\n'",
        "printf 'update %s\\n'",
    ] {
        assert!(formula_policy.contains(required_contract));
    }
    for policy_case in [
        "monotonic-update",
        "downgrade",
        "same-version-mismatch",
        "mismatched-url",
        "malformed-formula",
        "symbolic-link",
    ] {
        assert!(formula_policy_test.contains(policy_case));
    }
    assert!(ci.contains("./scripts/test-homebrew-formula-update-policy.sh"));
    assert!(ci.contains("./scripts/test-homebrew-authority-policy.sh"));
}

#[test]
fn source_linux_generator_and_verifiers_enforce_source_builds_and_linux_only_assets() {
    let generator = repository_file("scripts/generate-source-linux-release-channels.sh");
    let asset_verifier = repository_file("scripts/verify-source-linux-release-assets.sh");
    let published_verifier = repository_file("scripts/verify-published-source-linux-release.sh");

    for required_contract in [
        "enjoyable-mcp-sync-$release_version.crate",
        "class McpSync < Formula",
        "releases/download/v$release_version/$release_package_name",
        "depends_on \"rust\" => :build",
        "std_cargo_args(path: \".\")",
        "Cargo package contains a path outside its versioned root",
        "cargo metadata",
        "canonical stable semantic version",
    ] {
        assert!(generator.contains(required_contract));
    }
    for forbidden_contract in ["on_macos do", "bin.install", "PackageIdentifier:"] {
        assert!(!generator.contains(forbidden_contract));
    }

    for target in SOURCE_LINUX_TARGETS {
        assert!(asset_verifier.contains(target));
        assert!(published_verifier.contains(target));
    }
    for excluded_target in ["apple-darwin", "pc-windows-msvc", "EnjoyableWork.mcp-sync"] {
        assert!(!asset_verifier.contains(excluded_target));
        assert!(!published_verifier.contains(excluded_target));
    }
    for required_asset in [
        "enjoyable-mcp-sync-$release_asset_version.crate",
        "mcp-sync.rb",
        "SHA256SUMS",
    ] {
        assert!(asset_verifier.contains(required_asset));
    }
}

#[test]
fn authorization_workflow_proves_main_without_receiving_tag_write_authority() {
    let workflow = repository_file(".github/workflows/release-authorize.yml");

    for required_contract in [
        "workflow_dispatch:",
        "confirm_repository_controls:",
        "environment: release-control",
        "actions: read",
        "contents: read",
        "source-linux-release-preflight.yml",
        "release-preflight.yml",
        "ci.yml",
        "operator-side release repository controls must be explicitly confirmed",
        "scripts/verify-public-stable-tag-ruleset.sh",
        "scripts/validate-release-version.sh",
        "new tag authorization refuses an already-published Cargo version",
        "The tag push publishes nothing by itself.",
        "Dispatch exactly one release producer on the selected tag:",
        "separately funded signed workflow",
        "Actions deliberately has no tag-creation or bypass credential.",
    ] {
        assert!(workflow.contains(required_contract));
    }
    assert!(!workflow.contains("contents: write"));
    assert!(!workflow.contains("secrets."));
    assert!(!workflow.contains("git push"));
    assert!(!workflow.contains("repos/$GITHUB_REPOSITORY/immutable-releases"));
    assert_actions_are_commit_pinned(&workflow);
}

#[test]
fn release_version_policy_is_canonical_monotonic_and_recovery_safe() {
    let validator = repository_file("scripts/validate-release-version.sh");
    let focused_test = repository_file("scripts/test-release-version-policy.sh");
    let ci = repository_file(".github/workflows/ci.yml");

    for required_contract in [
        "future|published|rehearsal",
        "canonical stable v-prefixed semantic version",
        "release tag and Cargo package version do not match",
        "repeat-release automation refuses v0.1.0 and earlier versions",
        "release version is older than an already published stable version",
        "continue",
        "the nonpublishing live rehearsal reuses only immutable v0.1.0",
    ] {
        assert!(
            validator.contains(required_contract),
            "version validator should enforce {required_contract}"
        );
    }
    for accepted_case in [
        "patch-release",
        "partial-publication-recovery",
        "minor-release",
        "major-release",
        "unbounded-numeric-components",
        "fixed-live-rehearsal",
    ] {
        assert!(focused_test.contains(accepted_case));
    }
    for rejected_case in [
        "missing-inventory",
        "immutable-initial-version",
        "manifest-mismatch",
        "older-than-published",
        "leading-zero",
        "prerelease",
        "malformed-inventory",
    ] {
        assert!(focused_test.contains(rejected_case));
    }
    assert!(ci.contains("./scripts/test-release-version-policy.sh"));
}

#[test]
fn repository_control_verifiers_keep_admin_access_operator_side() {
    let public_ruleset = repository_file("scripts/verify-public-stable-tag-ruleset.sh");
    let operator_controls = repository_file("scripts/verify-release-repository-controls.sh");

    for required_contract in [
        "https://api.github.com/repos/$release_ruleset_repository/rulesets",
        "Protect stable release tags",
        ".enforcement == \"active\"",
        ".target == \"tag\"",
        "refs/tags/v*",
        "creation",
        "deletion",
        "update",
    ] {
        assert!(public_ruleset.contains(required_contract));
    }
    for forbidden_contract in ["gh api", "GH_TOKEN", "secrets.", "immutable-releases"] {
        assert!(!public_ruleset.contains(forbidden_contract));
    }

    for required_contract in [
        "gh auth status --hostname github.com",
        "commits/main",
        "immutable-releases",
        "bypass_mode == \"always\"",
        "verify-public-stable-tag-ruleset.sh",
        "verify_release_environment release-control main branch",
        "verify_release_environment release 'v*' tag",
        "--allow-mcp-039-main-rehearsal",
        ".total_count == 2",
        "{name: \"main\", type: \"branch\"}",
        "{name: \"v*\", type: \"tag\"}",
        "required_reviewers",
        "deployment-branch-policies",
    ] {
        assert!(operator_controls.contains(required_contract));
    }
    assert!(!operator_controls.contains("secrets."));
}

#[test]
fn package_and_public_docs_keep_the_accepted_release_identities() {
    let manifest = repository_file("Cargo.toml");
    let readme = repository_file("README.md");
    let runbook = repository_file("docs/source-linux-release.md");
    let signed_runbook = repository_file("docs/release.md");
    let historical_release_notes = repository_file("docs/releases/v0.1.0.md");
    let repeat_release_notes = repository_file("docs/releases/v0.1.1.md");
    let release_asset_verifier = repository_file("scripts/verify-published-release.sh");
    let source_linux_asset_verifier =
        repository_file("scripts/verify-published-source-linux-release.sh");

    for required_manifest_value in [
        "name = \"enjoyable-mcp-sync\"",
        "version = \"0.1.1\"",
        "name = \"mcp-sync\"",
        "publish = [\"crates-io\"]",
        "\"/.github/workflows/cargo-publish.yml\"",
        "\"/.github/workflows/release*.yml\"",
        "\"/.github/workflows/source-linux-release*.yml\"",
        "\"/tests/**\"",
        "\"/scripts/**\"",
        "\"/docs/source-linux-release.md\"",
        "\"/docs/releases/**\"",
        "\"/Cargo.lock\"",
        "\"/LICENSE\"",
        "\"/README.md\"",
    ] {
        assert!(manifest.contains(required_manifest_value));
    }

    for install_command in [
        "brew install EnjoyableWork/tap/mcp-sync",
        "winget install EnjoyableWork.mcp-sync",
        "cargo install enjoyable-mcp-sync",
    ] {
        assert!(
            readme.contains(install_command),
            "README should retain target install command {install_command}"
        );
    }

    for target in SIGNED_TARGETS {
        assert!(signed_runbook.contains(target));
        assert!(release_asset_verifier.contains(target));
    }
    for target in SOURCE_LINUX_TARGETS {
        assert!(runbook.contains(target));
        assert!(source_linux_asset_verifier.contains(target));
    }
    assert!(runbook.contains("v0.1.0"));
    assert!(runbook.contains("verify-release-repository-controls.sh"));
    assert!(runbook.contains("project-issued macOS or Windows binary"));
    assert!(runbook.contains("one first-publication exception"));
    assert!(runbook.contains("revoked server-side and removed locally"));
    assert!(runbook.contains("never create a replacement token"));
    for repeat_release_contract in [
        "The tag push publishes nothing by itself.",
        "Choose exactly one GitHub Release producer.",
        "-f mode=publish",
        "Rerunning after crates.io accepted the candidate is safe.",
        "requests no OIDC credential",
        "strictly newer",
        "-f mode=rehearse",
    ] {
        assert!(
            runbook.contains(repeat_release_contract),
            "source/Linux runbook should document {repeat_release_contract}"
        );
    }
    for trusted_publication_contract in [
        "`.github/workflows/cargo-publish.yml`",
        "`rust-lang/crates-io-auth-action`",
        "Require trusted publishing for all new versions",
        "no API-token fallback",
        "aggregate zero-token result",
        "scripts/verify-cargo-publishing-controls.sh",
    ] {
        assert!(
            runbook.contains(trusted_publication_contract),
            "source/Linux runbook should retain {trusted_publication_contract}"
        );
        assert!(
            signed_runbook.contains(trusted_publication_contract),
            "signed-native runbook should retain {trusted_publication_contract}"
        );
    }
    assert!(signed_runbook.contains("com.enjoyablework.mcp-sync"));
    assert!(signed_runbook.contains("confirm_funded_signing"));
    assert!(signed_runbook.contains("all 14 downloaded assets"));
    assert!(signed_runbook.contains("release_kind=funded"));
    assert!(historical_release_notes.contains("# mcp-sync v0.1.0"));
    assert!(historical_release_notes.contains("immutable GitHub Release"));
    assert!(historical_release_notes.contains("No project-issued macOS or Windows binary"));
    for repeat_release_contract in [
        "# mcp-sync v0.1.1",
        "`mcp-sync` does not update itself",
        "brew upgrade --build-from-source EnjoyableWork/tap/mcp-sync",
        "cargo install enjoyable-mcp-sync --version '=0.1.1' --locked",
        "gh release download v0.1.1",
        "gh attestation verify",
        "Installing or replacing the executable does not read or modify",
        "No project-issued macOS or Windows binary",
        "not published to WinGet",
    ] {
        assert!(
            repeat_release_notes.contains(repeat_release_contract),
            "repeat release notes should document {repeat_release_contract}"
        );
    }
}

#[test]
fn published_channel_workflow_proves_exact_metadata_and_every_native_install_path() {
    let workflow = repository_file(".github/workflows/release-channels.yml");
    let generator = repository_file("scripts/generate-release-channels.sh");
    let macos_trust = repository_file("scripts/verify-macos-release-trust.sh");
    let windows_trust = repository_file("scripts/verify-windows-release-trust.ps1");

    for target in SIGNED_TARGETS {
        assert!(
            workflow.contains(target),
            "published-channel smoke should include native target {target}"
        );
    }
    for runner in SIGNED_RUNNERS {
        assert!(
            workflow.contains(&format!("runner: {runner}")),
            "published-channel smoke should use native runner {runner}"
        );
    }

    for required_contract in [
        "workflow_dispatch:",
        "Verify funded signed release channels",
        "v0.1.0 belongs to the source and GNU/Linux channel verifier",
        "Verify immutable release and downstream metadata",
        ".immutable == true",
        "scripts/verify-published-release.sh",
        "gh release verify",
        "gh attestation verify",
        "--signer-workflow",
        "--source-ref",
        "--source-digest",
        "https://crates.io/api/v1/crates/enjoyable-mcp-sync",
        "scripts/generate-release-channels.sh",
        "EnjoyableWork/homebrew-tap/main/Formula/mcp-sync.rb",
        "microsoft/winget-pkgs/master/manifests/e/EnjoyableWork/mcp-sync",
        "cargo install enjoyable-mcp-sync",
        "brew install EnjoyableWork/tap/mcp-sync",
        "--id EnjoyableWork.mcp-sync",
        "scripts/smoke-installed.sh",
        "scripts/smoke-installed.ps1",
        "scripts/verify-macos-release-trust.sh",
        "scripts/verify-windows-release-trust.ps1",
        "Repair-WinGetPackageManager -Force -Latest",
    ] {
        assert!(
            workflow.contains(required_contract),
            "published-channel workflow should enforce {required_contract}"
        );
    }
    assert!(workflow.contains("attestations: read"));
    assert!(!workflow.contains("contents: write"));
    assert!(!workflow.contains("id-token: write"));
    assert!(!workflow.contains("secrets."));
    assert_actions_are_commit_pinned(&workflow);

    for generated_contract in [
        "class McpSync < Formula",
        "on_macos do",
        "on_linux do",
        "on_arm do",
        "on_intel do",
        "bin.install \"mcp-sync\"",
        "PackageIdentifier: EnjoyableWork.mcp-sync",
        "PackageVersion: $channel_version",
        "InstallerType: zip",
        "NestedInstallerType: portable",
        "Commands:",
        "Architecture: arm64",
        "Architecture: x64",
        "RelativeFilePath: mcp-sync.exe",
        "PortableCommandAlias: mcp-sync",
        "ManifestVersion: 1.12.0",
        "winget-manifest.installer.1.12.0.schema.json",
        "verify-published-release.sh",
        "release_hash",
    ] {
        assert!(
            generator.contains(generated_contract),
            "channel generator should retain {generated_contract}"
        );
    }

    for trust_contract in [
        "Identifier=com.enjoyablework.mcp-sync",
        "flags=.*runtime",
        "Timestamp=",
        "spctl --assess --type execute",
    ] {
        assert!(macos_trust.contains(trust_contract));
    }
    for trust_contract in [
        "SignatureStatus]::Valid",
        "SignerCertificate",
        "TimeStamperCertificate",
    ] {
        assert!(windows_trust.contains(trust_contract));
    }
}
