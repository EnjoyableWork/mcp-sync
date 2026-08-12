use std::fs;
use std::path::Path;

const NATIVE_TARGETS: [&str; 6] = [
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "aarch64-unknown-linux-gnu",
    "x86_64-unknown-linux-gnu",
    "aarch64-pc-windows-msvc",
    "x86_64-pc-windows-msvc",
];

const NATIVE_RUNNERS: [&str; 6] = [
    "macos-15",
    "macos-15-intel",
    "ubuntu-24.04-arm",
    "ubuntu-24.04",
    "windows-11-arm",
    "windows-2025",
];

fn repository_file(relative_path: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()))
}

fn job_block<'a>(workflow: &'a str, start: &str, end: &str) -> &'a str {
    let start = workflow
        .find(start)
        .unwrap_or_else(|| panic!("workflow should contain {start}"));
    let end = workflow[start..]
        .find(end)
        .map(|offset| start + offset)
        .unwrap_or(workflow.len());
    &workflow[start..end]
}

#[test]
fn cargo_publisher_confines_oidc_and_the_temporary_token_to_one_protected_job() {
    let workflow = repository_file(".github/workflows/cargo-publish.yml");
    let validate = job_block(&workflow, "  validate:\n", "  publish:\n");
    let publish = job_block(&workflow, "  publish:\n", "  cargo-unix:\n");

    assert!(workflow.contains("group: mcp-sync-release\n  cancel-in-progress: false"));
    assert!(!workflow.contains("group: cargo-publish"));

    for required in [
        "workflow_dispatch:",
        "version:",
        "tag:",
        "release_kind:",
        "mode:",
        "REQUEST_REF: ${{ github.ref }}",
        "REQUEST_WORKFLOW_SHA: ${{ github.workflow_sha }}",
        "REQUEST_REF_PROTECTED: ${{ github.ref_protected }}",
        "scripts/validate-cargo-publish-request.sh",
        "needs.validate.outputs.mode == 'authorization-only' ||",
        "steps.package.outputs.publish == 'true'",
    ] {
        assert!(
            workflow.contains(required),
            "publisher should contain {required}"
        );
    }
    for forbidden in [
        "pull_request:",
        "pull_request_target:",
        "push:",
        "repository_dispatch:",
        "workflow_run:",
        "\n  deployment:\n",
        "secrets.",
        "CRATES_IO_TOKEN",
        "cargo login",
        "--token",
    ] {
        assert!(
            !workflow.contains(forbidden),
            "publisher must not contain {forbidden}"
        );
    }

    assert!(validate.contains("ref: ${{ github.workflow_sha }}"));
    assert!(!validate.contains("environment:"));
    assert!(!validate.contains("id-token: write"));
    assert!(!validate.contains("crates-io-auth-action"));

    assert!(publish.contains("environment:\n      name: release\n      deployment: false"));
    assert!(publish.contains("id-token: write"));
    assert!(publish.contains(
        "rust-lang/crates-io-auth-action@c6f97d42243bad5fab37ca0427f495c86d5b1a18 # v1.0.5"
    ));
    assert!(publish.contains("CARGO_REGISTRY_TOKEN: ${{ steps.crates_io.outputs.token }}"));
    assert_eq!(workflow.matches("id-token: write").count(), 1);
    assert_eq!(workflow.matches("crates-io-auth-action@").count(), 1);
    assert_eq!(workflow.matches("CARGO_REGISTRY_TOKEN:").count(), 1);
    assert!(!workflow.contains("outputs.token }}\n    outputs:"));
}

#[test]
fn cargo_publisher_verifies_all_three_package_copies_before_and_after_publication() {
    let workflow = repository_file(".github/workflows/cargo-publish.yml");
    let verify_position = workflow
        .find("Verify immutable release, attested package, and deterministic local bytes")
        .expect("publisher should verify release bytes");
    let auth_position = workflow
        .find("Obtain short-lived crates.io authorization through OIDC")
        .expect("publisher should authenticate through OIDC");
    let publish_position = workflow
        .find("Publish the exact verified package")
        .expect("publisher should contain its publication step");
    assert!(verify_position < auth_position && auth_position < publish_position);

    for required in [
        ".immutable == true",
        "gh release verify",
        "gh attestation verify",
        "scripts/verify-published-source-linux-release.sh",
        "scripts/verify-published-release.sh",
        "cargo package --locked",
        "cmp --silent \"$first_package\" \"$local_package\"",
        "cmp --silent \"$release_package\" \"$local_package\"",
        ".crate.trustpub_only == true",
        "scripts/validate-release-version.sh",
        "enjoyable-mcp-sync-versions-before.json",
        "enjoyable-mcp-sync-versions-after.json",
        "cargo publish --dry-run --locked --registry crates-io",
        "cargo publish --locked --registry crates-io",
        "Require the MCP-039 rehearsal to create no Cargo version",
        "existing Cargo version differs from the immutable GitHub Release bytes",
        "printf 'publish=false\\n'",
        "No OIDC credential was requested and no publish call was made.",
        "cmp --silent \"$release_package\" \"$registry_package\"",
        "published Cargo package did not converge to the verified release bytes",
    ] {
        assert!(
            workflow.contains(required),
            "publisher should enforce {required}"
        );
    }
    assert!(workflow.contains("if: needs.validate.outputs.mode == 'authorization-only'"));
    assert!(workflow.contains("Mode: authorization only; no package was published."));
}

#[test]
fn cargo_publisher_runs_native_install_and_recovery_only_after_real_publication() {
    let workflow = repository_file(".github/workflows/cargo-publish.yml");

    for target in NATIVE_TARGETS {
        assert!(workflow.contains(target), "publisher should cover {target}");
    }
    for runner in NATIVE_RUNNERS {
        assert!(
            workflow.contains(&format!("runner: {runner}")),
            "publisher should use {runner}"
        );
    }
    assert_eq!(
        workflow
            .matches("if: needs.publish.outputs.mode == 'publish'")
            .count(),
        2
    );
    assert!(workflow.contains("cargo install enjoyable-mcp-sync"));
    assert_eq!(workflow.matches("--registry crates-io").count(), 4);
    assert!(workflow.contains("--version \"=$RELEASE_VERSION\""));
    assert!(workflow.contains("--version \"=$env:RELEASE_VERSION\""));
    assert!(workflow.contains("scripts/smoke-installed.sh"));
    assert!(workflow.contains("scripts/smoke-installed.ps1"));
}

#[test]
fn cargo_request_and_workflow_policy_have_acceptance_and_rejection_exercises() {
    let validator = repository_file("scripts/validate-cargo-publish-request.sh");
    let request_test = repository_file("scripts/test-cargo-publish-policy.sh");
    let workflow_test = repository_file("scripts/test-cargo-publish-workflow-policy.sh");
    let supply_chain = repository_file("scripts/verify-workflow-supply-chain.sh");
    let ci = repository_file(".github/workflows/ci.yml");

    for required in [
        "\"$cargo_publish_request_event\" != workflow_dispatch",
        "Authorization-only mode is limited to the fixed MCP-039 v0.1.0 rehearsal",
        "The MCP-039 rehearsal must use the exact protected main workflow revision",
        "Cargo publication must target the exact existing release tag",
        "Cargo 0.1.0 is immutable and cannot be republished",
    ] {
        assert!(
            validator.contains(required),
            "validator should contain {required}"
        );
    }
    for accepted in ["manual-authorization", "manual-publication"] {
        assert!(request_test.contains(accepted));
    }
    for rejected in [
        "wrong-repository",
        "branch-ref",
        "mismatched-tag",
        "immutable-version",
        "token-mode",
        "deployment-event",
        "authorization-tag-ref",
        "authorization-unprotected-main",
        "authorization-wrong-workflow-sha",
        "authorization-wrong-version",
        "publication-unprotected-tag",
        "leading-zero-publication",
        "prerelease-publication",
    ] {
        assert!(request_test.contains(rejected));
    }
    for rejected in [
        "mutable-auth-action",
        "wrong-environment",
        "missing-local-release-comparison",
        "missing-registry-comparison",
        "missing-rehearsal-nonpublication-proof",
        "missing-trusted-only-publication-gate",
        "reusable-token-fallback",
        "untrusted-trigger",
        "historical-deployment-trigger",
        "missing-oidc",
        "recovery-requests-oidc",
        "missing-global-release-serialization",
    ] {
        assert!(workflow_test.contains(rejected));
    }
    assert!(workflow_test.contains("scripts/syft-assets.txt"));
    for gate in [
        "./scripts/test-cargo-publish-policy.sh",
        "./scripts/test-cargo-publish-workflow-policy.sh",
        "./scripts/test-release-version-policy.sh",
    ] {
        assert!(ci.contains(gate));
    }
    assert!(supply_chain.contains(
        "Cargo publisher must verify immutable bytes before requesting OIDC authorization"
    ));
    assert!(
        supply_chain.contains("Cargo publisher contains a forbidden trigger or token fallback")
    );
}

#[test]
fn funded_release_keeps_registry_authority_separate_and_now_attests_the_crate() {
    let manifest = repository_file("Cargo.toml");
    let release = repository_file(".github/workflows/release.yml");
    let preflight = repository_file(".github/workflows/release-preflight.yml");
    let asset_verifier = repository_file("scripts/verify-release-assets.sh");
    let published_verifier = repository_file("scripts/verify-published-release.sh");

    assert!(manifest.contains("\"/.github/workflows/cargo-publish.yml\""));
    for required in [
        "source:\n    name: Package and attest the deterministic Cargo source",
        "cargo package --locked",
        "release-assets/enjoyable-mcp-sync-${{ env.RELEASE_VERSION }}.crate",
        "name: release-source",
        "- source",
    ] {
        assert!(
            release.contains(required),
            "funded release should contain {required}"
        );
    }
    assert!(!release.contains("cargo publish"));
    assert!(!release.contains("crates-io-auth-action"));

    for required in [
        "source-package:",
        "Deterministic Cargo source package",
        "pattern: preflight-*",
        "scripts/verify-release-assets.sh",
        "scripts/verify-published-release.sh",
    ] {
        assert!(preflight.contains(required));
    }
    for verifier in [asset_verifier, published_verifier] {
        assert!(verifier.contains("enjoyable-mcp-sync-$asset_version.crate"));
    }
}

#[test]
fn operator_verifier_reads_back_exact_publisher_and_trusted_only_state() {
    let verifier = repository_file("scripts/verify-cargo-publishing-controls.sh");

    for required in [
        "verify-release-repository-controls.sh",
        "verify-repository-supply-chain-controls.sh",
        "contents/.github/workflows/cargo-publish.yml?ref=main",
        "actions/workflows/cargo-publish.yml",
        "repos/EnjoyableWork/mcp-sync/actions/secrets?per_page=100",
        "repos/EnjoyableWork/mcp-sync/environments/release/secrets?per_page=100",
        "orgs/EnjoyableWork/actions/secrets?per_page=100",
        "verify_no_cargo_credential_names",
        "repos/rust-lang/crates-io-auth-action/commits/c6f97d42243bad5fab37ca0427f495c86d5b1a18",
        ".commit.verification.verified == true",
        ".crate.trustpub_only == true",
        ".github_configs | length",
        ".meta.total == 1",
        ".github_configs[0].repository_owner == \"EnjoyableWork\"",
        ".github_configs[0].repository_owner_id == $expected_owner_id",
        ".github_configs[0].repository_name == \"mcp-sync\"",
        ".github_configs[0].workflow_filename == \"cargo-publish.yml\"",
        ".github_configs[0].environment == \"release\"",
    ] {
        assert!(
            verifier.contains(required),
            "verifier should require {required}"
        );
    }
    for forbidden in ["CRATES_IO_TOKEN", "CARGO_REGISTRY_TOKEN", "secrets."] {
        assert!(!verifier.contains(forbidden));
    }
}
