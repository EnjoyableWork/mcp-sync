use crate::config::{CanonicalConfig, CanonicalServer};
use std::collections::BTreeSet;
use std::fmt;

/// Computes a deterministic plan from normalized current and desired state.
///
/// Both inputs have already crossed the canonical validation boundary. The
/// comparison is therefore exact and literal: commands are not shell-parsed,
/// arguments are not reordered, and environment values are not expanded.
/// Target-only entries become drift instead of deletion work.
pub fn reconcile(current: &CanonicalConfig, desired: &CanonicalConfig) -> ReconciliationPlan {
    let names: BTreeSet<&str> = current
        .servers()
        .keys()
        .chain(desired.servers().keys())
        .map(String::as_str)
        .collect();

    let entries = names
        .into_iter()
        .map(|name| {
            let current_server = current.servers().get(name);
            let desired_server = desired.servers().get(name);

            match (current_server, desired_server) {
                (None, Some(desired_server)) => PlanEntry::new(
                    name,
                    ReconciliationOutcome::Add {
                        desired: ServerShape::from_server(desired_server),
                    },
                    Some(desired_server.clone()),
                ),
                (Some(current_server), Some(desired_server))
                    if current_server == desired_server =>
                {
                    PlanEntry::new(name, ReconciliationOutcome::NoOp, None)
                }
                (Some(current_server), Some(desired_server)) => {
                    let changes = ServerChanges::between(current_server, desired_server);
                    debug_assert!(!changes.is_empty());
                    PlanEntry::new(
                        name,
                        ReconciliationOutcome::Update { changes },
                        Some(desired_server.clone()),
                    )
                }
                (Some(current_server), None) => PlanEntry::new(
                    name,
                    ReconciliationOutcome::Drift {
                        current: ServerShape::from_server(current_server),
                    },
                    None,
                ),
                (None, None) => unreachable!("a unioned server name must exist in one input"),
            }
        })
        .collect();

    ReconciliationPlan { entries }
}

#[derive(Clone, PartialEq, Eq)]
pub struct ReconciliationPlan {
    entries: Vec<PlanEntry>,
}

impl ReconciliationPlan {
    pub fn entries(&self) -> &[PlanEntry] {
        &self.entries
    }

    pub fn summary(&self) -> PlanSummary {
        self.entries
            .iter()
            .fold(PlanSummary::default(), |mut summary, entry| {
                summary.record(entry.outcome().kind());
                summary
            })
    }

    pub fn requires_mutation(&self) -> bool {
        self.entries.iter().any(PlanEntry::requires_mutation)
    }

    #[cfg(test)]
    pub fn has_drift(&self) -> bool {
        self.entries
            .iter()
            .any(|entry| entry.outcome().kind() == ReconciliationOutcomeKind::Drift)
    }
}

impl fmt::Debug for ReconciliationPlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ReconciliationPlan")
            .field("entries", &self.entries)
            .field("summary", &self.summary())
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct PlanEntry {
    server_name: String,
    outcome: ReconciliationOutcome,
    desired_server: Option<CanonicalServer>,
}

impl PlanEntry {
    fn new(
        server_name: impl Into<String>,
        outcome: ReconciliationOutcome,
        desired_server: Option<CanonicalServer>,
    ) -> Self {
        debug_assert_eq!(
            matches!(
                outcome,
                ReconciliationOutcome::Add { .. } | ReconciliationOutcome::Update { .. }
            ),
            desired_server.is_some()
        );

        Self {
            server_name: server_name.into(),
            outcome,
            desired_server,
        }
    }

    pub fn server_name(&self) -> &str {
        &self.server_name
    }

    pub fn outcome(&self) -> &ReconciliationOutcome {
        &self.outcome
    }

    /// Returns the validated desired definition for add and update work.
    ///
    /// The values can contain credentials. They are deliberately omitted from
    /// every plan `Debug` implementation and must remain behind the apply
    /// boundary.
    pub fn desired_server(&self) -> Option<&CanonicalServer> {
        self.desired_server.as_ref()
    }

    pub fn requires_mutation(&self) -> bool {
        matches!(
            self.outcome,
            ReconciliationOutcome::Add { .. } | ReconciliationOutcome::Update { .. }
        )
    }
}

impl fmt::Debug for PlanEntry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PlanEntry")
            .field("server_name", &self.server_name)
            .field("outcome", &self.outcome)
            .field("has_desired_server", &self.desired_server.is_some())
            .finish()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReconciliationOutcome {
    Add { desired: ServerShape },
    Update { changes: ServerChanges },
    NoOp,
    Drift { current: ServerShape },
}

impl ReconciliationOutcome {
    pub fn kind(&self) -> ReconciliationOutcomeKind {
        match self {
            Self::Add { .. } => ReconciliationOutcomeKind::Add,
            Self::Update { .. } => ReconciliationOutcomeKind::Update,
            Self::NoOp => ReconciliationOutcomeKind::NoOp,
            Self::Drift { .. } => ReconciliationOutcomeKind::Drift,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReconciliationOutcomeKind {
    Add,
    Update,
    NoOp,
    Drift,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerShape {
    argument_count: usize,
    environment_keys: Vec<String>,
}

impl ServerShape {
    fn from_server(server: &CanonicalServer) -> Self {
        Self {
            argument_count: server.args().len(),
            environment_keys: server.env().keys().cloned().collect(),
        }
    }

    pub fn argument_count(&self) -> usize {
        self.argument_count
    }

    pub fn environment_keys(&self) -> &[String] {
        &self.environment_keys
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerChanges {
    command_changed: bool,
    arguments: Option<ArgumentChanges>,
    environment: EnvironmentChanges,
}

impl ServerChanges {
    fn between(current: &CanonicalServer, desired: &CanonicalServer) -> Self {
        let arguments = (current.args() != desired.args()).then_some(ArgumentChanges {
            current_count: current.args().len(),
            desired_count: desired.args().len(),
        });

        Self {
            command_changed: current.command() != desired.command(),
            arguments,
            environment: EnvironmentChanges::between(current, desired),
        }
    }

    pub fn command_changed(&self) -> bool {
        self.command_changed
    }

    pub fn arguments(&self) -> Option<ArgumentChanges> {
        self.arguments
    }

    pub fn environment(&self) -> &EnvironmentChanges {
        &self.environment
    }

    pub fn is_empty(&self) -> bool {
        !self.command_changed && self.arguments.is_none() && self.environment.is_empty()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArgumentChanges {
    current_count: usize,
    desired_count: usize,
}

impl ArgumentChanges {
    pub fn current_count(self) -> usize {
        self.current_count
    }

    pub fn desired_count(self) -> usize {
        self.desired_count
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EnvironmentChanges {
    added_keys: Vec<String>,
    updated_keys: Vec<String>,
    removed_keys: Vec<String>,
}

impl EnvironmentChanges {
    fn between(current: &CanonicalServer, desired: &CanonicalServer) -> Self {
        let keys: BTreeSet<&str> = current
            .env()
            .keys()
            .chain(desired.env().keys())
            .map(String::as_str)
            .collect();
        let mut changes = Self::default();

        for key in keys {
            match (current.env().get(key), desired.env().get(key)) {
                (None, Some(_)) => changes.added_keys.push(key.to_owned()),
                (Some(current_value), Some(desired_value)) if current_value != desired_value => {
                    changes.updated_keys.push(key.to_owned());
                }
                (Some(_), None) => changes.removed_keys.push(key.to_owned()),
                (Some(_), Some(_)) => {}
                (None, None) => unreachable!("a unioned environment key must exist in one input"),
            }
        }

        changes
    }

    pub fn added_keys(&self) -> &[String] {
        &self.added_keys
    }

    pub fn updated_keys(&self) -> &[String] {
        &self.updated_keys
    }

    pub fn removed_keys(&self) -> &[String] {
        &self.removed_keys
    }

    pub fn is_empty(&self) -> bool {
        self.added_keys.is_empty() && self.updated_keys.is_empty() && self.removed_keys.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlanSummary {
    add: usize,
    update: usize,
    no_op: usize,
    drift: usize,
}

impl PlanSummary {
    fn record(&mut self, kind: ReconciliationOutcomeKind) {
        match kind {
            ReconciliationOutcomeKind::Add => self.add += 1,
            ReconciliationOutcomeKind::Update => self.update += 1,
            ReconciliationOutcomeKind::NoOp => self.no_op += 1,
            ReconciliationOutcomeKind::Drift => self.drift += 1,
        }
    }

    pub fn add(self) -> usize {
        self.add
    }

    pub fn update(self) -> usize {
        self.update
    }

    pub fn no_op(self) -> usize {
        self.no_op
    }

    pub fn drift(self) -> usize {
        self.drift
    }

    #[cfg(test)]
    pub fn total(self) -> usize {
        self.add + self.update + self.no_op + self.drift
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::collection;
    use proptest::prelude::*;
    use std::collections::BTreeMap;

    const PROPERTY_CASES: u32 = 128;
    const MAX_SHRINK_ITERATIONS: u32 = 4_096;

    fn server(command: &str, args: &[&str], environment: &[(&str, &str)]) -> CanonicalServer {
        CanonicalServer::new(
            command,
            args.iter().map(|argument| (*argument).to_owned()).collect(),
            environment
                .iter()
                .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
                .collect(),
        )
    }

    fn config(entries: &[(&str, CanonicalServer)]) -> CanonicalConfig {
        CanonicalConfig::new(
            entries
                .iter()
                .map(|(name, server)| ((*name).to_owned(), server.clone()))
                .collect(),
        )
        .expect("test configuration should be valid")
    }

    fn config_with(base: &CanonicalConfig, entries: &[(&str, CanonicalServer)]) -> CanonicalConfig {
        let mut servers = base.servers().clone();
        for (name, server) in entries {
            servers.insert((*name).to_owned(), server.clone());
        }
        CanonicalConfig::new(servers).expect("extended test configuration should be valid")
    }

    fn entry<'a>(plan: &'a ReconciliationPlan, name: &str) -> &'a PlanEntry {
        plan.entries()
            .iter()
            .find(|entry| entry.server_name() == name)
            .unwrap_or_else(|| panic!("plan should contain {name}"))
    }

    #[test]
    fn mixed_plan_is_complete_sorted_and_deterministic() {
        let unchanged = server("same", &["--one"], &[("MODE", "same")]);
        let current = config(&[
            ("updated", server("old", &["--old"], &[("OLD", "one")])),
            ("drifted", server("target-only", &[], &[("LOCAL", "one")])),
            ("unchanged", unchanged.clone()),
        ]);
        let desired = config(&[
            (
                "added",
                server("new", &["--one", "--two"], &[("TOKEN", "private")]),
            ),
            ("unchanged", unchanged),
            (
                "updated",
                server(
                    "new",
                    &["--new", "--second"],
                    &[("ADDED", "two"), ("OLD", "changed")],
                ),
            ),
        ]);

        let first = reconcile(&current, &desired);
        let second = reconcile(&current, &desired);

        assert_eq!(first, second);
        assert_eq!(
            first
                .entries()
                .iter()
                .map(|entry| (entry.server_name(), entry.outcome().kind()))
                .collect::<Vec<_>>(),
            vec![
                ("added", ReconciliationOutcomeKind::Add),
                ("drifted", ReconciliationOutcomeKind::Drift),
                ("unchanged", ReconciliationOutcomeKind::NoOp),
                ("updated", ReconciliationOutcomeKind::Update),
            ]
        );
        assert_eq!(
            first.summary(),
            PlanSummary {
                add: 1,
                update: 1,
                no_op: 1,
                drift: 1,
            }
        );
        assert_eq!(first.summary().add(), 1);
        assert_eq!(first.summary().update(), 1);
        assert_eq!(first.summary().no_op(), 1);
        assert_eq!(first.summary().drift(), 1);
        assert!(first.requires_mutation());
        assert!(first.has_drift());
    }

    #[test]
    fn add_and_drift_show_shape_without_process_values() {
        let current = config(&[(
            "drifted",
            server(
                "private-current-command",
                &["private-current-argument"],
                &[("LOCAL_KEY", "private-current-value")],
            ),
        )]);
        let desired = config(&[(
            "added",
            server(
                "private-desired-command",
                &["one", "two"],
                &[("ALPHA", "private-one"), ("TOKEN", "private-two")],
            ),
        )]);

        let plan = reconcile(&current, &desired);

        let ReconciliationOutcome::Add { desired: shape } = entry(&plan, "added").outcome() else {
            panic!("added server should produce add work");
        };
        assert_eq!(shape.argument_count(), 2);
        assert_eq!(shape.environment_keys(), ["ALPHA", "TOKEN"]);

        let ReconciliationOutcome::Drift { current: shape } = entry(&plan, "drifted").outcome()
        else {
            panic!("target-only server should produce drift");
        };
        assert_eq!(shape.argument_count(), 1);
        assert_eq!(shape.environment_keys(), ["LOCAL_KEY"]);

        assert!(entry(&plan, "added").desired_server().is_some());
        assert!(entry(&plan, "drifted").desired_server().is_none());
    }

    #[test]
    fn update_reports_only_structural_field_changes() {
        let current = config(&[(
            "changed",
            server(
                "old-command",
                &["same-count-old"],
                &[
                    ("KEEP", "same"),
                    ("REMOVE", "private-old"),
                    ("UPDATE", "private-before"),
                ],
            ),
        )]);
        let desired = config(&[(
            "changed",
            server(
                "new-command",
                &["same-count-new"],
                &[
                    ("ADD", "private-new"),
                    ("KEEP", "same"),
                    ("UPDATE", "private-after"),
                ],
            ),
        )]);

        let plan = reconcile(&current, &desired);
        let changed = entry(&plan, "changed");
        let ReconciliationOutcome::Update { changes } = changed.outcome() else {
            panic!("different definitions should produce update work");
        };

        assert!(changes.command_changed());
        let arguments = changes
            .arguments()
            .expect("the argument sequence should be marked as changed");
        assert_eq!(arguments.current_count(), 1);
        assert_eq!(arguments.desired_count(), 1);
        assert_eq!(changes.environment().added_keys(), ["ADD"]);
        assert_eq!(changes.environment().updated_keys(), ["UPDATE"]);
        assert_eq!(changes.environment().removed_keys(), ["REMOVE"]);
        assert!(!changes.is_empty());
        assert_eq!(changed.desired_server(), desired.servers().get("changed"));
    }

    #[test]
    fn identical_state_is_stable_no_op_work() {
        let state = config(&[
            ("alpha", server("one", &[], &[])),
            ("beta", server("two", &["--flag"], &[("TOKEN", "private")])),
        ]);

        let plan = reconcile(&state, &state);

        assert!(!plan.requires_mutation());
        assert!(!plan.has_drift());
        assert_eq!(plan.summary().no_op(), 2);
        assert!(plan.entries().iter().all(|entry| {
            entry.outcome().kind() == ReconciliationOutcomeKind::NoOp
                && entry.desired_server().is_none()
        }));
    }

    #[test]
    fn target_only_state_is_reported_as_non_mutating_drift() {
        let current = config(&[("local", server("local", &[], &[]))]);
        let desired = config(&[]);

        let plan = reconcile(&current, &desired);

        assert!(!plan.requires_mutation());
        assert!(plan.has_drift());
        assert_eq!(plan.summary().drift(), 1);
        assert_eq!(plan.summary().total(), 1);
        assert_eq!(
            entry(&plan, "local").outcome().kind(),
            ReconciliationOutcomeKind::Drift
        );
    }

    #[test]
    fn empty_states_produce_an_empty_non_mutating_plan() {
        let empty = config(&[]);

        let plan = reconcile(&empty, &empty);

        assert!(plan.entries().is_empty());
        assert_eq!(plan.summary(), PlanSummary::default());
        assert!(!plan.requires_mutation());
        assert!(!plan.has_drift());
    }

    #[test]
    fn plan_debug_is_structurally_redacted() {
        let current = config(&[
            (
                "drifted",
                server(
                    "private-drift-command",
                    &["private-drift-argument"],
                    &[("DRIFT_TOKEN", "private-drift-value")],
                ),
            ),
            (
                "updated",
                server(
                    "private-old-command",
                    &["private-old-argument"],
                    &[("UPDATE_TOKEN", "private-old-value")],
                ),
            ),
        ]);
        let desired = config(&[
            (
                "added",
                server(
                    "private-add-command",
                    &["private-add-argument"],
                    &[("ADD_TOKEN", "private-add-value")],
                ),
            ),
            (
                "updated",
                server(
                    "private-new-command",
                    &["private-new-argument"],
                    &[("UPDATE_TOKEN", "private-new-value")],
                ),
            ),
        ]);

        let debug = format!("{:#?}", reconcile(&current, &desired));

        for structural_value in ["added", "drifted", "updated", "ADD_TOKEN", "DRIFT_TOKEN"] {
            assert!(debug.contains(structural_value));
        }
        for secret in [
            "private-drift-command",
            "private-drift-argument",
            "private-drift-value",
            "private-old-command",
            "private-old-argument",
            "private-old-value",
            "private-add-command",
            "private-add-argument",
            "private-add-value",
            "private-new-command",
            "private-new-argument",
            "private-new-value",
        ] {
            assert!(!debug.contains(secret));
        }
    }

    fn canonical_server_strategy() -> BoxedStrategy<CanonicalServer> {
        (
            "[a-z][a-z0-9-]{0,15}",
            collection::vec("[a-zA-Z0-9_./:=+@-]{0,20}", 0..5),
            collection::btree_map("[A-Z][A-Z0-9_]{0,10}", "[a-zA-Z0-9_./:=+@-]{0,20}", 0..5),
        )
            .prop_map(|(command, args, environment)| {
                CanonicalServer::new(command, args, environment)
            })
            .boxed()
    }

    fn canonical_config_strategy() -> BoxedStrategy<CanonicalConfig> {
        collection::btree_map("[a-z][a-z0-9-]{0,12}", canonical_server_strategy(), 0..8)
            .prop_map(|servers| {
                CanonicalConfig::new(servers).expect("generated configuration should be valid")
            })
            .boxed()
    }

    fn reverse_insertion_order(config: &CanonicalConfig) -> CanonicalConfig {
        let servers: BTreeMap<_, _> = config
            .servers()
            .iter()
            .rev()
            .map(|(name, server)| (name.clone(), server.clone()))
            .collect();
        CanonicalConfig::new(servers).expect("reordered configuration should remain valid")
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: PROPERTY_CASES,
            max_shrink_iters: MAX_SHRINK_ITERATIONS,
            ..ProptestConfig::default()
        })]

        #[test]
        fn arbitrary_plans_match_exact_set_relationships(
            current in canonical_config_strategy(),
            desired in canonical_config_strategy(),
            unchanged in canonical_server_strategy(),
            updated_current in canonical_server_strategy(),
            added in canonical_server_strategy(),
            drifted in canonical_server_strategy(),
        ) {
            let updated_desired = CanonicalServer::new(
                format!("{}-changed", updated_current.command()),
                updated_current.args().to_vec(),
                updated_current.env().clone(),
            );
            let current = config_with(
                &current,
                &[
                    ("case-drift", drifted),
                    ("case-no-op", unchanged.clone()),
                    ("case-update", updated_current),
                ],
            );
            let desired = config_with(
                &desired,
                &[
                    ("case-add", added),
                    ("case-no-op", unchanged),
                    ("case-update", updated_desired),
                ],
            );
            let plan = reconcile(&current, &desired);
            let expected_names: BTreeSet<&str> = current
                .servers()
                .keys()
                .chain(desired.servers().keys())
                .map(String::as_str)
                .collect();
            let actual_names: Vec<_> = plan
                .entries()
                .iter()
                .map(PlanEntry::server_name)
                .collect();

            prop_assert_eq!(
                actual_names,
                expected_names.iter().copied().collect::<Vec<_>>()
            );
            prop_assert_eq!(plan.summary().total(), expected_names.len());

            for entry in plan.entries() {
                let current_server = current.servers().get(entry.server_name());
                let desired_server = desired.servers().get(entry.server_name());
                let expected_kind = match (current_server, desired_server) {
                    (None, Some(_)) => ReconciliationOutcomeKind::Add,
                    (Some(current_server), Some(desired_server)) if current_server == desired_server => {
                        ReconciliationOutcomeKind::NoOp
                    }
                    (Some(_), Some(_)) => ReconciliationOutcomeKind::Update,
                    (Some(_), None) => ReconciliationOutcomeKind::Drift,
                    (None, None) => unreachable!("unioned name should be present"),
                };

                prop_assert_eq!(entry.outcome().kind(), expected_kind);
                if matches!(
                    expected_kind,
                    ReconciliationOutcomeKind::Add | ReconciliationOutcomeKind::Update
                ) {
                    prop_assert_eq!(entry.desired_server(), desired_server);
                    prop_assert!(entry.requires_mutation());
                } else {
                    prop_assert!(entry.desired_server().is_none());
                    prop_assert!(!entry.requires_mutation());
                }
            }

            for (name, expected_kind) in [
                ("case-add", ReconciliationOutcomeKind::Add),
                ("case-drift", ReconciliationOutcomeKind::Drift),
                ("case-no-op", ReconciliationOutcomeKind::NoOp),
                ("case-update", ReconciliationOutcomeKind::Update),
            ] {
                prop_assert_eq!(entry(&plan, name).outcome().kind(), expected_kind);
            }
        }

        #[test]
        fn reconciliation_is_deterministic_and_does_not_mutate_inputs(
            current in canonical_config_strategy(),
            desired in canonical_config_strategy(),
        ) {
            let current_before = current.clone();
            let desired_before = desired.clone();

            let first = reconcile(&current, &desired);
            let second = reconcile(&current, &desired);

            prop_assert_eq!(first, second);
            prop_assert_eq!(current, current_before);
            prop_assert_eq!(desired, desired_before);
        }

        #[test]
        fn insertion_order_does_not_change_the_plan(
            current in canonical_config_strategy(),
            desired in canonical_config_strategy(),
        ) {
            let reordered_current = reverse_insertion_order(&current);
            let reordered_desired = reverse_insertion_order(&desired);

            prop_assert_eq!(
                reconcile(&current, &desired),
                reconcile(&reordered_current, &reordered_desired)
            );
        }

        #[test]
        fn reconciling_identical_state_is_stable_no_op_work(
            state in canonical_config_strategy(),
        ) {
            let plan = reconcile(&state, &state);

            prop_assert!(!plan.requires_mutation());
            prop_assert!(!plan.has_drift());
            prop_assert_eq!(plan.summary().no_op(), state.servers().len());
            let all_no_op = plan.entries().iter().all(|entry| {
                entry.outcome().kind() == ReconciliationOutcomeKind::NoOp
            });
            prop_assert!(all_no_op);
        }

        #[test]
        fn generated_process_values_never_appear_in_plan_debug(
            suffix in "[a-zA-Z0-9]{8,24}",
        ) {
            let secrets = [
                format!("private-current-command-{suffix}"),
                format!("private-current-argument-{suffix}"),
                format!("private-current-value-{suffix}"),
                format!("private-desired-command-{suffix}"),
                format!("private-desired-argument-{suffix}"),
                format!("private-desired-value-{suffix}"),
            ];
            let current = config(&[
                (
                    "drifted",
                    server(&secrets[0], &[&secrets[1]], &[("DRIFT_TOKEN", &secrets[2])]),
                ),
                (
                    "updated",
                    server(&secrets[0], &[&secrets[1]], &[("UPDATE_TOKEN", &secrets[2])]),
                ),
                (
                    "unchanged",
                    server(&secrets[0], &[&secrets[1]], &[("NO_OP_TOKEN", &secrets[2])]),
                ),
            ]);
            let desired = config(&[
                (
                    "added",
                    server(&secrets[3], &[&secrets[4]], &[("ADD_TOKEN", &secrets[5])]),
                ),
                (
                    "updated",
                    server(&secrets[3], &[&secrets[4]], &[("UPDATE_TOKEN", &secrets[5])]),
                ),
                (
                    "unchanged",
                    server(&secrets[0], &[&secrets[1]], &[("NO_OP_TOKEN", &secrets[2])]),
                ),
            ]);

            let debug = format!("{:#?}", reconcile(&current, &desired));

            for secret in secrets {
                prop_assert!(!debug.contains(&secret));
            }
        }
    }
}
