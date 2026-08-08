use std::fs;
use std::path::Path;

const TARGETS: [&str; 6] = [
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "aarch64-unknown-linux-gnu",
    "x86_64-unknown-linux-gnu",
    "aarch64-pc-windows-msvc",
    "x86_64-pc-windows-msvc",
];

const RUNNERS: [&str; 6] = [
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

#[test]
fn release_preflight_covers_every_native_artifact_without_credentials() {
    let workflow = repository_file(".github/workflows/release-preflight.yml");

    for target in TARGETS {
        assert!(
            workflow.contains(target),
            "release preflight should include native target {target}"
        );
    }
    for runner in RUNNERS {
        assert!(
            workflow.contains(&format!("runner: {runner}")),
            "release preflight should use native runner {runner}"
        );
    }

    assert!(workflow.contains("scripts/smoke-archive.sh"));
    assert!(workflow.contains("scripts/smoke-archive.ps1"));
    assert!(workflow.contains("syft-version: v1.50.0"));
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
fn stable_tag_workflow_enforces_signing_provenance_and_draft_first_publication() {
    let workflow = repository_file(".github/workflows/release.yml");

    for target in TARGETS {
        assert!(
            workflow.contains(target),
            "stable release should include native target {target}"
        );
    }
    for runner in RUNNERS {
        assert!(
            workflow.contains(&format!("runner: {runner}")),
            "stable release should use native runner {runner}"
        );
    }

    for required_contract in [
        "environment: release",
        "name: release",
        "refs/tags/v*",
        "immutable-releases",
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
    assert!(
        !workflow.contains("cargo publish")
            && !workflow.contains("brew install")
            && !workflow.contains("winget install"),
        "downstream publication must wait for the immutable GitHub Release"
    );
    assert_actions_are_commit_pinned(&workflow);
}

#[test]
fn authorization_workflow_proves_main_without_receiving_tag_write_authority() {
    let workflow = repository_file(".github/workflows/release-authorize.yml");

    for required_contract in [
        "workflow_dispatch:",
        "environment: release-control",
        "actions: read",
        "contents: read",
        "release-preflight.yml",
        "ci.yml",
        "immutable-releases",
        "Protect stable release tags",
        "Actions deliberately has no tag-creation or bypass credential.",
    ] {
        assert!(workflow.contains(required_contract));
    }
    assert!(!workflow.contains("contents: write"));
    assert!(!workflow.contains("secrets."));
    assert!(!workflow.contains("git push"));
    assert_actions_are_commit_pinned(&workflow);
}

#[test]
fn package_and_public_docs_keep_the_accepted_release_identities() {
    let manifest = repository_file("Cargo.toml");
    let readme = repository_file("README.md");
    let runbook = repository_file("docs/release.md");
    let release_notes = repository_file("docs/releases/v0.1.0.md");
    let release_asset_verifier = repository_file("scripts/verify-published-release.sh");

    for required_manifest_value in [
        "name = \"enjoyable-mcp-sync\"",
        "version = \"0.1.0\"",
        "name = \"mcp-sync\"",
        "publish = [\"crates-io\"]",
        "\"/.github/workflows/release*.yml\"",
        "\"/tests/**\"",
        "\"/scripts/**\"",
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

    for target in TARGETS {
        assert!(runbook.contains(target));
        assert!(release_asset_verifier.contains(target));
    }
    assert!(runbook.contains("v0.1.0"));
    assert!(runbook.contains("com.enjoyablework.mcp-sync"));
    assert!(release_notes.contains("# mcp-sync v0.1.0"));
    assert!(release_notes.contains("immutable GitHub Release"));
}

#[test]
fn published_channel_workflow_proves_exact_metadata_and_every_native_install_path() {
    let workflow = repository_file(".github/workflows/release-channels.yml");
    let generator = repository_file("scripts/generate-release-channels.sh");
    let macos_trust = repository_file("scripts/verify-macos-release-trust.sh");
    let windows_trust = repository_file("scripts/verify-windows-release-trust.ps1");

    for target in TARGETS {
        assert!(
            workflow.contains(target),
            "published-channel smoke should include native target {target}"
        );
    }
    for runner in RUNNERS {
        assert!(
            workflow.contains(&format!("runner: {runner}")),
            "published-channel smoke should use native runner {runner}"
        );
    }

    for required_contract in [
        "workflow_dispatch:",
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
