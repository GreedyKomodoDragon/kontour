use dioxus::prelude::*;
use k8s_openapi::api::apps::v1::Deployment;

#[derive(Clone)]
struct DeploymentData {
    name: String,
    namespace: String,
    strategy: String,
    age: String,
    ready_replicas: i32,
    desired_replicas: i32,
    available_replicas: i32,
    updated_replicas: i32,
    status: String,
    labels: Vec<(String, String)>,
    selector: Vec<(String, String)>,
    conditions: Vec<DeploymentCondition>,
}

#[derive(Clone)]
struct DeploymentCondition {
    condition_type: String,
    status: String,
    last_transition_time: String,
    reason: String,
    message: String,
}

#[derive(Props, PartialEq, Clone)]
pub struct DeploymentItemProps {
    deployment: Deployment,
}

#[component]
pub fn DeploymentItem(props: DeploymentItemProps) -> Element {
    let mut is_expanded = use_signal(|| false);

    let deployment_data = DeploymentData {
        name: props.deployment.metadata.name.clone().unwrap_or_default(),
        namespace: props.deployment.metadata.namespace.clone().unwrap_or_default(),
        strategy: props.deployment.spec.as_ref()
            .and_then(|s| s.strategy.as_ref())
            .and_then(|st| st.type_.as_ref())
            .cloned()
            .unwrap_or_else(|| "Unknown".to_string()),
        age: "1h".to_string(), // TODO: Calculate age
        ready_replicas: props.deployment.status.as_ref().map_or(0, |s| s.ready_replicas.unwrap_or(0)),
        desired_replicas: props.deployment.spec.as_ref().map_or(0, |s| s.replicas.unwrap_or(0)),
        available_replicas: props.deployment.status.as_ref().map_or(0, |s| s.available_replicas.unwrap_or(0)),
        updated_replicas: props.deployment.status.as_ref().map_or(0, |s| s.updated_replicas.unwrap_or(0)),
        status: {
            let conditions = props.deployment.status.as_ref()
                .and_then(|s| s.conditions.as_ref())
                .map(|c| c.iter().map(|cond| (cond.type_.clone(), cond.status.clone(), cond.reason.clone())).collect::<Vec<_>>())
                .unwrap_or_default();

            let desired = props.deployment.spec.as_ref().map_or(0, |s| s.replicas.unwrap_or(0));
            let updated = props.deployment.status.as_ref().map_or(0, |s| s.updated_replicas.unwrap_or(0));
            let available = props.deployment.status.as_ref().map_or(0, |s| s.available_replicas.unwrap_or(0));

            if desired == 0 {
                "Scaled Down".to_string()
            } else {
                let is_progressing = conditions.iter().any(|(t, s, _)| t == "Progressing" && s == "True");
                let is_available = conditions.iter().any(|(t, s, _)| t == "Available" && s == "True");
                let has_replica_failure = conditions.iter().any(|(t, _, r)| t == "Progressing" && r.as_ref().map_or(false, |r| r == "ReplicaFailure"));

                if has_replica_failure || (!is_progressing && !is_available) {
                    "Degraded".to_string()
                } else if is_available && is_progressing && updated == desired && available == desired {
                    "Available".to_string()
                } else {
                    "Progressing".to_string()
                }
            }
        },
        labels: props.deployment.metadata.labels.as_ref()
            .map(|labels| labels.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default(),
        selector: props.deployment.spec.as_ref()
            .and_then(|s| s.selector.match_labels.as_ref())
            .map(|selector| selector.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default(),
        conditions: props.deployment.status.as_ref()
            .and_then(|s| s.conditions.as_ref())
            .map(|conditions| {
                conditions.iter().map(|c| DeploymentCondition {
                    condition_type: c.type_.clone(),
                    status: c.status.clone(),
                    last_transition_time: c.last_transition_time.as_ref()
                        .map(|t| t.0.to_string())
                        .unwrap_or_default(),
                    reason: c.reason.clone().unwrap_or_default(),
                    message: c.message.clone().unwrap_or_default(),
                }).collect()
            })
            .unwrap_or_default(),
    };

    let status_class = match deployment_data.status.as_str() {
        "Available" => "status-running",
        "Progressing" => "status-pending",
        "Degraded" => "status-failed",
        "Scaled Down" => "status-warning",
        _ => "status-unknown"
    };

    rsx! {
        div {
            key: "{deployment_data.name}",
            class: "deployment-card",
            div {
                class: "deployment-header-card",
                div { class: "deployment-title",
                    h3 { "{deployment_data.name}" }
                    span { class: "status-badge {status_class}",
                        "{deployment_data.status} ({deployment_data.ready_replicas}/{deployment_data.desired_replicas})"
                    }
                }
                div { class: "deployment-controls",
                    button {
                        class: "btn-icon expand-toggle",
                        onclick: move |evt| {
                            evt.stop_propagation();
                            is_expanded.set(!is_expanded());
                        },
                        title: if is_expanded() { "Collapse" } else { "Expand" },
                        if is_expanded() { "🔼" } else { "🔽" }
                    }
                    button { class: "btn-icon", onclick: move |evt| evt.stop_propagation(), title: "View ReplicaSets", "🧱" }
                    button { class: "btn-icon", onclick: move |evt| evt.stop_propagation(), title: "View Pods", "📦" }
                    button { class: "btn-icon", onclick: move |evt| evt.stop_propagation(), title: "Edit", "✏️" }
                    button { class: "btn-icon", onclick: move |evt| evt.stop_propagation(), title: "Delete", "🗑️" }
                }
            }

            {is_expanded().then(|| rsx! {
                div { class: "deployment-details",
                    // Basic Info Section
                    div { class: "deployment-info",
                        div { class: "info-group",
                            div { class: "info-item", span { class: "info-label", "Namespace" } span { class: "info-value", "{deployment_data.namespace}" } }
                            div { class: "info-item", span { class: "info-label", "Strategy" } span { class: "info-value", "{deployment_data.strategy}" } }
                            div { class: "info-item", span { class: "info-label", "Age" } span { class: "info-value", "{deployment_data.age}" } }
                        }
                    }

                    // Labels Section
                    div { class: "labels-section",
                        h4 { "Labels" }
                        div { class: "labels-grid",
                            {deployment_data.labels.is_empty().then(|| rsx! {
                                span { class: "info-value", i { "No labels" } }
                            })}
                            {deployment_data.labels.iter().map(|(key, value)| rsx! {
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
                            {deployment_data.selector.is_empty().then(|| rsx! {
                                span { class: "info-value", i { "No selector" } }
                            })}
                            {deployment_data.selector.iter().map(|(key, value)| rsx! {
                                div { 
                                    key: "sel-{key}",
                                    class: "label",
                                    span { class: "label-key", "{key}" }
                                    span { class: "label-value", "{value}" }
                                }
                            })}
                        }
                    }

                    // Conditions Section
                    div { class: "conditions-section",
                        h4 { "Conditions" }
                        div { class: "conditions-grid",
                            {deployment_data.conditions.is_empty().then(|| rsx! {
                                span { class: "info-value", i { "No conditions" } }
                            })}
                            {deployment_data.conditions.iter().map(|cond| rsx! {
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