use std::fs;
use std::path::PathBuf;

const LEVEL_ONE_CONTROLS: [&str; 24] = [
    "OSPS-AC-01.01",
    "OSPS-AC-02.01",
    "OSPS-AC-03.01",
    "OSPS-AC-03.02",
    "OSPS-BR-01.01",
    "OSPS-BR-01.03",
    "OSPS-BR-03.01",
    "OSPS-BR-03.02",
    "OSPS-BR-07.01",
    "OSPS-DO-01.01",
    "OSPS-DO-02.01",
    "OSPS-GV-02.01",
    "OSPS-GV-03.01",
    "OSPS-LE-02.01",
    "OSPS-LE-02.02",
    "OSPS-LE-03.01",
    "OSPS-LE-03.02",
    "OSPS-QA-01.01",
    "OSPS-QA-01.02",
    "OSPS-QA-02.01",
    "OSPS-QA-04.01",
    "OSPS-QA-05.01",
    "OSPS-QA-05.02",
    "OSPS-VM-02.01",
];

fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn repository_file(path: &str) -> String {
    fs::read_to_string(repository_root().join(path))
        .unwrap_or_else(|error| panic!("{path} should be readable: {error}"))
}

#[test]
fn assurance_contract_passes_every_level_one_control_with_scoped_evidence() {
    let assurance = repository_file("docs/security-assurance.md");

    assert!(
        assurance.contains(
            "OpenSSF Open Source Project\nSecurity (OSPS) Baseline [`v2026.02.19` Level 1"
        )
    );
    assert!(
        assurance
            .contains("This is a maintainer self-assessment, not an independent certification")
    );
    assert!(assurance.contains("All 24 controls apply"));
    assert_eq!(assurance.matches("| Pass |").count(), 24);

    for control in LEVEL_ONE_CONTROLS {
        assert!(
            assurance.contains(&format!("| `{control}` | Pass |")),
            "assurance crosswalk should pass {control}"
        );
    }

    for revalidated_control in [
        "`OSPS-DO-01.01`: every current basic command",
        "`OSPS-QA-01.01`: the authoritative repository",
        "`OSPS-QA-01.02`: its public Git history",
        "`OSPS-QA-02.01`: locked Cargo metadata",
    ] {
        assert!(
            assurance.contains(revalidated_control),
            "assurance should directly revalidate {revalidated_control}"
        );
    }
}

#[test]
fn badgeapp_automation_proposes_exactly_the_reviewed_level_one_results() {
    let proposal: serde_json::Value = serde_json::from_str(&repository_file(".bestpractices.json"))
        .expect("BadgeApp automation proposal should be valid JSON");
    let proposal = proposal
        .as_object()
        .expect("BadgeApp automation proposal should be an object");

    assert_eq!(proposal.len(), 52);
    assert_eq!(proposal["name"], "mcp-sync");
    assert_eq!(proposal["license"], "MIT");
    assert_eq!(
        proposal["implementation_languages"],
        "Rust, Shell, PowerShell"
    );
    assert!(
        proposal["description"]
            .as_str()
            .is_some_and(|description| description.contains("local configuration CLI"))
    );

    for control in LEVEL_ONE_CONTROLS {
        let field = control.to_ascii_lowercase().replace(['-', '.'], "_");
        let status = format!("{field}_status");
        let justification = format!("{field}_justification");
        assert_eq!(
            proposal[&status], "Met",
            "proposal should mark {control} Met"
        );
        assert!(
            proposal[&justification]
                .as_str()
                .is_some_and(|value| value.contains("https://github.com/EnjoyableWork/")),
            "proposal should give public evidence for {control}"
        );
    }
}

#[test]
fn achieved_badge_and_readme_claim_link_to_the_exact_public_assessment() {
    let readme = repository_file("README.md");
    let assurance = repository_file("docs/security-assurance.md");
    let badge = "[![OpenSSF Baseline](https://www.bestpractices.dev/projects/14014/baseline)](https://www.bestpractices.dev/projects/14014)";

    for document in [&readme, &assurance] {
        assert!(document.contains(badge));
    }
    assert!(readme.contains("## 🔐 Security and Trust"));
    assert!(readme.contains("https://www.bestpractices.dev/en/projects/14014/baseline-1"));
    assert!(readme.contains("all 24 OpenSSF OSPS Baseline `v2026.02.19` Level 1 controls"));
    assert!(readme.contains("maintainer self-certification"));
    assert!(readme.contains("seven verified `v0.1.0` GitHub\nRelease assets"));
    assert!(
        assurance.contains("BadgeApp first recorded `baseline-1` on 2026-08-09 at 21:00:26 UTC")
    );
    assert!(assurance.contains("all 24 controls `Met`"));
    assert!(!assurance.contains("MCP-035: replace this line"));

    for overclaim in [
        "certified by OpenSSF",
        "OSPS Level 2 certified",
        "OSPS Level 3 certified",
        "NIST certified",
        "OpenChain certified",
    ] {
        assert!(!readme.contains(overclaim));
        assert!(!assurance.contains(overclaim));
    }
}

#[test]
fn exact_artifact_statement_is_bounded_and_revalidatable() {
    let assurance = repository_file("docs/security-assurance.md");

    for required in [
        "SLSA v1.0 Build Level 2 artifact statement",
        "The seven exact assets attached to the immutable `v0.1.0` GitHub Release",
        "https://slsa.dev/provenance/v1",
        "github-hosted",
        "08365900497bf6fde3749695078794e0f3ecf790",
        "dc48488c20725abc4d773834544acd6965b78b2b2a963b0386b09752a2c3288e",
        "d4b364917faf1d8497a57cf44c2a3e351aa388a1260c678bb1da71f8f7e58f7f",
        "5cc6627fdba5b7cb6ad51c1fd9a88dea4e7ddcec140deb0f22fe7e3f5051f1ce",
        "95aee76134327702a509c270dcd8c221d1794347b08da6f6580417ea6723ea41",
        "bbb5d685f24e61985c74887bd603de5bb262815627cad39be6e0c8efc099fd8b",
        "3c4564c6fd8b91fe88c2fcd5b70aa305453a069e87d27f0fae13c9e9cc9db8e6",
        "b6a4fd20110bf8a2f4dbf3952239e7ee6bc8698afb73c3aff58637a44bc7720d",
        "does not claim a newer SLSA specification,\nBuild Level 3",
    ] {
        assert!(
            assurance.contains(required),
            "artifact statement should retain {required}"
        );
    }

    for trigger in [
        "Reassess the complete baseline at least annually",
        "the named OSPS checklist, BadgeApp criteria, badge status, or badge URL",
        "organization ownership, authentication, collaborator, application",
        "a security, secret, access, release, or recovery incident occurs",
        "any evidence link, verifier, public claim, or exact-artifact statement",
        "remove or\nqualify the README badge and trust statement",
    ] {
        assert!(
            assurance.contains(trigger),
            "maintenance contract should retain {trigger}"
        );
    }
}

#[test]
fn public_assurance_verifier_checks_the_official_badge_and_exact_main() {
    let verifier = repository_file("scripts/verify-public-security-assurance.sh");

    for required in [
        "badge_percentage_baseline_1 == 100",
        "baseline_tiered_percentage == 100",
        ".description ==",
        ".implementation_languages == \"Rust, Shell, PowerShell\"",
        ".achieved_baseline_1_at",
        ".lost_baseline_1_at == null",
        "These are criteria version v2026.02.19.",
        "<span class=\"satisfaction-text\">24/24</span>",
        "aria-label=\"openssf baseline v2026.02.19: 1\"",
        "data-canonical-src",
        "--require-closed",
        "tr '[:upper:].-' '[:lower:]__'",
        "commits/main",
        "README.md",
        "docs/security-assurance.md",
        ".bestpractices.json",
        "type == \"object\" and length == 52",
        "| Pass |",
        "| M3 | Trusted project — enterprise assurance and adoption ",
    ] {
        assert!(
            verifier.contains(required),
            "public assurance verifier should require {required}"
        );
    }

    for control in LEVEL_ONE_CONTROLS {
        assert!(
            verifier.contains(control),
            "public assurance verifier should inspect {control}"
        );
    }
}

#[test]
fn project_policy_requires_official_proof_for_every_activated_target() {
    let project = repository_file("PROJECT.md");

    for required in [
        "When the framework provides an\nofficial badge or conformance mark for the achieved scope, obtaining that\nissuer-provided proof",
        "[BadgeApp](https://www.bestpractices.dev/) `baseline-1` self-certification badge linked to the public assessment",
        "Official BadgeApp Passing badge linked to the public project assessment",
        "Applicable [official OpenChain conformance badge]",
        "no issuer-awarded project badge is currently identified, so no certification-like shield may be invented",
        "no official NIST certification badge is currently identified",
    ] {
        assert!(
            project.contains(required),
            "project assurance policy should retain {required}"
        );
    }
}
