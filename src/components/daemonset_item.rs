use dioxus::prelude::*;
use k8s_openapi::api::apps::v1::DaemonSet;

#[derive(Clone)]
struct DaemonSetData {
    name: String,
    namespace: String,
    age: String,
    desired_scheduled: i32,
    current_scheduled: i32,
    ready_scheduled: i32,
    updated_scheduled: i32,
    available_scheduled: i32,
    labels: Vec<(String, String)>,
    selector: Vec<(String, String)>,
    node_selector: Vec<(String, String)>,
    node_selector_requirements: Vec<NodeSelectorRequirement>,
    conditions: Vec<DaemonSetCondition>,
}

#[derive(Clone)]
struct NodeSelectorRequirement {
    key: String,
    operator: String,
    values: Vec<String>,
}

#[derive(Clone)]
struct DaemonSetCondition {
    condition_type: String,
    status: String,
    last_transition_time: String,
    reason: String,
    message: String,
}

#[derive(Props, PartialEq, Clone)]
pub struct DaemonSetItemProps {
    daemonset: DaemonSet,
}

#[component]
pub fn DaemonSetItem(props: DaemonSetItemProps) -> Element {
    let mut is_expanded = use_signal(|| false);

    let daemonset_data = DaemonSetData {
        name: props.daemonset.metadata.name.clone().unwrap_or_default(),
        namespace: props
            .daemonset
            .metadata
            .namespace
            .clone()
            .unwrap_or_default(),
        age: "1h".to_string(), // TODO: Calculate age
        desired_scheduled: props
            .daemonset
            .status
            .as_ref()
            .map_or(0, |s| s.desired_number_scheduled),
        current_scheduled: props
            .daemonset
            .status
            .as_ref()
            .map_or(0, |s| s.current_number_scheduled),
        ready_scheduled: props
            .daemonset
            .status
            .as_ref()
            .map_or(0, |s| s.number_ready),
        updated_scheduled: props.daemonset.status.as_ref().map_or(0, |s| {
            s.updated_number_scheduled
                .unwrap_or(s.current_number_scheduled)
        }),
        available_scheduled: props
            .daemonset
            .status
            .as_ref()
            .map_or(0, |s| s.number_available.unwrap_or(0)),
        labels: props
            .daemonset
            .metadata
            .labels
            .as_ref()
            .map(|labels| labels.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default(),
        selector: props
            .daemonset
            .spec
            .as_ref()
            .and_then(|s| s.selector.match_labels.as_ref())
            .map(|selector| {
                selector
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            })
            .unwrap_or_default(),
        node_selector: props
            .daemonset
            .spec
            .as_ref()
            .and_then(|s| s.template.spec.as_ref())
            .and_then(|s| s.node_selector.as_ref())
            .map(|selector| {
                selector
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            })
            .unwrap_or_default(),
        node_selector_requirements: props
            .daemonset
            .spec
            .as_ref()
            .and_then(|s| s.template.spec.as_ref())
            .and_then(|s| s.affinity.as_ref())
            .and_then(|a| a.node_affinity.as_ref())
            .and_then(|na| {
                na.required_during_scheduling_ignored_during_execution
                    .as_ref()
            })
            .map(|selector| {
                selector
                    .node_selector_terms
                    .iter()
                    .flat_map(|term| {
                        term.match_expressions.iter().flatten().map(|expr| {
                            NodeSelectorRequirement {
                                key: expr.key.clone(),
                                operator: expr.operator.clone(),
                                values: expr.values.clone().unwrap_or_default(),
                            }
                        })
                    })
                    .collect()
            })
            .unwrap_or_default(),
        conditions: props
            .daemonset
            .status
            .as_ref()
            .and_then(|s| s.conditions.as_ref())
            .map(|conditions| {
                conditions
                    .iter()
                    .map(|c| DaemonSetCondition {
                        condition_type: c.type_.clone(),
                        status: c.status.clone(),
                        last_transition_time: c
                            .last_transition_time
                            .as_ref()
                            .map(|t| t.0.to_string())
                            .unwrap_or_default(),
                        reason: c.reason.clone().unwrap_or_default(),
                        message: c.message.clone().unwrap_or_default(),
                    })
                    .collect()
            })
            .unwrap_or_default(),
    };

    let status = determine_status(&props.daemonset);
    let status_class = match status.as_str() {
        "Running" => "status-running",
        "Progressing" => "status-pending",
        "Not Ready" => "status-failed",
        "No Nodes" => "status-warning",
        _ => "status-unknown",
    };

    rsx! {
        div {
            key: "{daemonset_data.name}",
            class: "daemonset-card",
            div {
                class: "daemonset-header-card",
                div { class: "daemonset-title",
                    h3 { "{daemonset_data.name}" }
                    span { class: "status-badge {status_class}",
                        "{status} ({daemonset_data.ready_scheduled}/{daemonset_data.desired_scheduled})"
                    }
                }
                div { class: "daemonset-controls",
                    button {
                        class: "btn-icon expand-toggle",
                        onclick: move |evt| {
                            evt.stop_propagation();
                            is_expanded.set(!is_expanded());
                        },
                        title: if is_expanded() { "Collapse" } else { "Expand" },
                        if is_expanded() { "🔼" } else { "🔽" }
                    }
                    button { class: "btn-icon", onclick: move |evt| evt.stop_propagation(), title: "View Pods", "📦" }
                    button { class: "btn-icon", onclick: move |evt| evt.stop_propagation(), title: "Edit", "✏️" }
                    button { class: "btn-icon", onclick: move |evt| evt.stop_propagation(), title: "Delete", "🗑️" }
                }
            }

            {is_expanded().then(|| rsx! {
                div { class: "daemonset-details",
                    // Basic Info Section
                    div { class: "daemonset-info",
                        div { class: "info-group",
                            div { class: "info-item", span { class: "info-label", "Namespace" } span { class: "info-value", "{daemonset_data.namespace}" } }
                            div { class: "info-item", span { class: "info-label", "Age" } span { class: "info-value", "{daemonset_data.age}" } }
                        }
                        div { class: "info-group",
                            div { class: "info-item", span { class: "info-label", "Current" } span { class: "info-value", "{daemonset_data.current_scheduled}" } }
                            div { class: "info-item", span { class: "info-label", "Updated" } span { class: "info-value", "{daemonset_data.updated_scheduled}" } }
                            div { class: "info-item", span { class: "info-label", "Available" } span { class: "info-value", "{daemonset_data.available_scheduled}" } }
                        }
                    }

                    // Labels Section
                    div { class: "labels-section",
                        h4 { "Labels" }
                        div { class: "labels-grid",
                            {daemonset_data.labels.is_empty().then(|| rsx! {
                                span { class: "info-value", i { "No labels" } }
                            })}
                            {daemonset_data.labels.iter().map(|(key, value)| rsx! {
                                div {
                                    key: "{key}",
                                    class: "label",
                                    span { class: "label-key", "{key}" }
                                    span { class: "label-value", "{value}" }
                                }
                            })}
                        }
                    }

                    // Selector Section
                    div { class: "labels-section",
                        h4 { "Selector" }
                        div { class: "labels-grid",
                            {daemonset_data.selector.is_empty().then(|| rsx! {
                                span { class: "info-value", i { "No selector" } }
                            })}
                            {daemonset_data.selector.iter().map(|(key, value)| rsx! {
                                div {
                                    key: "sel-{key}",
                                    class: "label",
                                    span { class: "label-key", "{key}" }
                                    span { class: "label-value", "{value}" }
                                }
                            })}
                        }
                    }

                    // Node Selector Section
                    div { class: "labels-section",
                        h4 { "Node Selector" }
                        div { class: "labels-grid",
                            {(daemonset_data.node_selector.is_empty() && daemonset_data.node_selector_requirements.is_empty()).then(|| rsx! {
                                span { class: "info-value", i { "None (runs on all eligible nodes)" } }
                            })}
                            // Node Selector Labels
                            {daemonset_data.node_selector.iter().map(|(key, value)| rsx! {
                                div {
                                    key: "node-sel-{key}",
                                    class: "label",
                                    span { class: "label-key", "{key}" }
                                    span { class: "label-value", "{value}" }
                                }
                            })}
                            // Node Selector Requirements
                            {daemonset_data.node_selector_requirements.iter().map(|req| rsx! {
                                div {
                                    key: "node-req-{req.key}",
                                    class: "label node-requirement",
                                    span { class: "label-key", "{req.key} {req.operator}" }
                                    span { class: "label-value", "{req.values.join(\", \")}" }
                                }
                            })}
                        }
                    }

                    // Conditions Section
                    div { class: "conditions-section",
                        h4 { "Conditions" }
                        div { class: "conditions-grid",
                            {daemonset_data.conditions.is_empty().then(|| rsx! {
                                span { class: "info-value", i { "No conditions" } }
                            })}
                            {daemonset_data.conditions.iter().map(|cond| rsx! {
                                div {
                                    key: "{cond.condition_type}",
                                    class: "condition",
                                    div { class: "condition-info",
                                        span { class: "condition-type", "{cond.condition_type}" }
                                        span {
                                            class: "condition-status status-{cond.status.to_lowercase()}",
                                            "{cond.status}"
                                        }
                                    }
                                    div { class: "condition-details",
                                        span { class: "condition-reason", "{cond.reason}" }
                                        span { class: "condition-time", "{cond.last_transition_time}" }
                                    }
                                    div { class: "condition-message", "{cond.message}"}
                                }
                            })}
                        }
                    }
                }
            })}
        }
    }
}

fn determine_status(daemonset: &DaemonSet) -> String {
    let desired = daemonset
        .status
        .as_ref()
        .map_or(0, |s| s.desired_number_scheduled);
    let ready = daemonset.status.as_ref().map_or(0, |s| s.number_ready);
    let current = daemonset
        .status
        .as_ref()
        .map_or(0, |s| s.current_number_scheduled);
    let updated = daemonset.status.as_ref().map_or(0, |s| {
        s.updated_number_scheduled
            .unwrap_or(s.current_number_scheduled)
    });

    if desired == 0 {
        return "No Nodes".to_string();
    }

    if ready == desired && current == desired && updated == desired {
        return "Running".to_string();
    }

    if ready < desired {
        if ready == 0 {
            return "Not Ready".to_string();
        }
        return "Progressing".to_string();
    }

    "Unknown".to_string()
}
