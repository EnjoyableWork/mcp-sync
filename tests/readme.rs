use std::fs;
use std::path::PathBuf;

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn readme() -> String {
    fs::read_to_string(repository_root().join("README.md"))
        .expect("the public product page should be readable")
}

#[test]
fn product_page_frontloads_the_cli_journey_and_progressively_discloses_detail() {
    let readme = readme();

    assert!(readme.starts_with("<h1 align=\"center\">⚡ mcp-sync</h1>"));
    for required_surface in [
        "Define your MCP servers once. Keep every local AI client in sync.",
        "https://github.com/EnjoyableWork/mcp-sync/actions/workflows/ci.yml/badge.svg",
        "https://www.bestpractices.dev/projects/14014/baseline",
        "https://img.shields.io/crates/v/enjoyable-mcp-sync.svg",
        "<a href=\"#why-mcp-sync\">Why mcp-sync?</a>",
        "<a href=\"#install\">Install</a>",
        "<a href=\"#quick-start\">Quick start</a>",
        "<a href=\"#safety-by-construction\">Safety</a>",
        "<a href=\"#security-and-trust\">Trust</a>",
        "$ mcp-sync init",
        "$ mcp-sync sync --dry-run",
        "Kiro: would update with recoverable backup at \"<path>\"",
        "structurally redacted six-client plan",
        "`vscode`, `codex`, or `kiro`",
        "| Kiro | Global-user `.kiro/settings/mcp.json` | Comment-preserving JSON `mcpServers` |",
        "An absolute, traversal-free `KIRO_HOME`",
        "**Serialize writers.**",
        "**Treat returned failures across six targets as one transaction.**",
        "<details>\n<summary><strong>Managed paths by platform</strong></summary>",
        "<details>\n<summary><strong>Build and verify from source</strong></summary>",
    ] {
        assert!(
            readme.contains(required_surface),
            "README should retain its public product surface: {required_surface}"
        );
    }

    let ordered_sections = [
        "## Why mcp-sync?",
        "## Install",
        "## Quick start",
        "## How it works",
        "## Client coverage",
        "## Safety by construction",
        "## Security and trust",
        "## Contributing",
        "## License",
    ];
    let mut previous_position = 0;
    for section in ordered_sections {
        let position = readme
            .find(section)
            .unwrap_or_else(|| panic!("README should contain {section}"));
        assert!(
            position >= previous_position,
            "README should keep {section} in the progressive product-page order"
        );
        previous_position = position;
    }

    let demonstration = readme
        .find("```console")
        .expect("README should contain the terminal-first demonstration");
    let first_section = readme
        .find("## Why mcp-sync?")
        .expect("README should contain its first product section");
    assert!(
        demonstration < first_section,
        "the CLI demonstration should appear before reference prose"
    );

    let code_fence_count = readme
        .lines()
        .filter(|line| line.starts_with("```"))
        .count();
    assert_eq!(
        code_fence_count % 2,
        0,
        "README code fences should remain balanced"
    );
}

#[test]
fn public_and_operational_docs_pin_the_kiro_ownership_boundary() {
    let readme = readme();
    let guide = fs::read_to_string(repository_root().join("docs/m1-usage-and-recovery.md"))
        .expect("the operational guide should be readable");

    for required_contract in [
        "`.kiro/settings/mcp.json` and agent configuration",
        "Kiro Crew's `~/.kiro/crew/mcp.json`",
        "`~/.kiro/agents/kirocrew.json`",
        "remain unmanaged and are never expanded",
    ] {
        assert!(
            readme.contains(required_contract),
            "README should retain the Kiro boundary: {required_contract}"
        );
    }

    for required_contract in [
        "the six implemented global targets",
        "<KIRO_HOME>/settings/mcp.json",
        "valid `${VARIABLE}` reference",
        "Kiro IDE `1.0.288`",
        "Kiro Crew `0.1.3`",
        "`includeMcpJson: false`",
        "`sync` treats returned failures as one six-target",
        "| `kiro` | Global-user Kiro comment-preserving JSON |",
    ] {
        assert!(
            guide.contains(required_contract),
            "the operational guide should retain the Kiro contract: {required_contract}"
        );
    }
}

#[test]
fn operational_guide_documents_the_complete_mutation_lock_contract() {
    let guide = fs::read_to_string(repository_root().join("docs/m1-usage-and-recovery.md"))
        .expect("the operational guide should be readable");

    for required_contract in [
        "## Concurrent mutating operations",
        "mcp-sync/operation.lock",
        "from before canonical and native planning through apply",
        "another mutating mcp-sync operation is already in progress",
        "Do not delete `operation.lock`",
        "Different canonical roots remain independent",
    ] {
        assert!(
            guide.contains(required_contract),
            "the operational guide should retain the mutation-lock contract: {required_contract}"
        );
    }
}

#[test]
fn public_and_operational_docs_pin_the_per_file_crash_recovery_boundary() {
    let readme = readme();
    let guide = fs::read_to_string(repository_root().join("docs/m1-usage-and-recovery.md"))
        .expect("the operational guide should be readable");

    for required_contract in [
        "target-first, journaled same-directory transaction",
        "the next locked mutation safely aborts or",
        "Crash recovery is per file",
        "atomic across process termination or whole-machine power loss",
    ] {
        assert!(
            readme.contains(required_contract),
            "README should retain the crash-recovery boundary: {required_contract}"
        );
    }

    for required_contract in [
        "### An earlier process stopped during existing-file replacement",
        "<target>.mcp-sync-transaction.json",
        "byte counts, SHA-256 fingerprints",
        "lock-free commands never perform recovery",
        "inspects all seven resolved managed",
        "Recovery is idempotent",
        "whole-machine or power-loss durability",
        "nor does it make all six target commits",
    ] {
        assert!(
            guide.contains(required_contract),
            "the operational guide should retain the crash-recovery contract: {required_contract}"
        );
    }
}

#[test]
fn every_readme_repository_link_resolves_to_a_tracked_path() {
    for relative_path in [
        "LICENSE",
        "CONTRIBUTING.md",
        "SUPPORT.md",
        "CODE_OF_CONDUCT.md",
        "SECURITY.md",
        "examples/config.v1.json",
        "docs/m1-usage-and-recovery.md",
        "docs/security-assurance.md",
        "docs/project-community-and-licensing.md",
        "scripts/check.sh",
        "scripts/check.ps1",
    ] {
        assert!(
            repository_root().join(relative_path).exists(),
            "README repository link should resolve: {relative_path}"
        );
    }
}
