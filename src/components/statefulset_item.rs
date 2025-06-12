use dioxus::prelude::*;
use k8s_openapi::api::apps::v1::StatefulSet;

#[derive(Clone)]
struct StatefulSetData {
    name: String,
    namespace: String,
    service_name: String,
    age: String,
    ready_replicas: i32,
    desired_replicas: i32,
    current_replicas: i32,
    updated_replicas: i32,
    status: String,
    labels: Vec<(String, String)>,
    selector: Vec<(String, String)>,
    conditions: Vec<StatefulSetCondition>,
}

#[derive(Clone)]
struct StatefulSetCondition {
    condition_type: String,
    status: String,
    last_transition_time: String,
    reason: String,
    message: String,
}

#[derive(Props, PartialEq, Clone)]
pub struct StatefulSetItemProps {
    statefulset: StatefulSet,
}

#[component]
pub fn StatefulSetItem(props: StatefulSetItemProps) -> Element {
    let mut is_expanded = use_signal(|| false);

    let statefulset_data = StatefulSetData {
        name: props.statefulset.metadata.name.clone().unwrap_or_default(),
        namespace: props.statefulset.metadata.namespace.clone().unwrap_or_default(),
        service_name: props.statefulset.spec.as_ref()
            .map(|s| s.service_name.clone())
            .unwrap_or_else(|| "Unknown".to_string()),
        age: "1h".to_string(), // TODO: Calculate age
        ready_replicas: props.statefulset.status.as_ref().map_or(0, |s| s.ready_replicas.unwrap_or(0)),
        desired_replicas: props.statefulset.spec.as_ref().map_or(0, |s| s.replicas.unwrap_or(0)),
        current_replicas: props.statefulset.status.as_ref().map_or(0, |s| s.current_replicas.unwrap_or(0)),
        updated_replicas: props.statefulset.status.as_ref().map_or(0, |s| s.updated_replicas.unwrap_or(0)),
        status: determine_status(&props.statefulset),
        labels: props.statefulset.metadata.labels.as_ref()
            .map(|labels| labels.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default(),
        selector: props.statefulset.spec.as_ref()
            .and_then(|s| s.selector.match_labels.as_ref())
            .map(|selector| selector.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default(),
        conditions: props.statefulset.status.as_ref()
            .and_then(|s| s.conditions.as_ref())
            .map(|conditions| {
                conditions.iter().map(|c| StatefulSetCondition {
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

    let status_class = match statefulset_data.status.as_str() {
        "Available" => "status-running",
        "Progressing" => "status-pending",
        "Rolling Update" => "status-warning",
        "Degraded" => "status-failed",
        "Scaled Down" => "status-warning",
        _ => "status-unknown"
    };

    // Format the replica count display
    let replica_display = if statefulset_data.status == "Scaled Down" {
        "(0/0)".to_string()
    } else {
        format!("({}/{})", statefulset_data.ready_replicas, statefulset_data.desired_replicas)
    };

    rsx! {
        div {
            key: "{statefulset_data.name}",
            class: "statefulset-card",
            div {
                class: "statefulset-header-card",
                div { class: "statefulset-title",
                    h3 { "{statefulset_data.name}" }
                    span { class: "status-badge {status_class}",
                        "{statefulset_data.status} {replica_display}"
                    }
                }
                div { class: "statefulset-controls",
                    button {
                        class: "btn-icon expand-toggle",
                        onclick: move |evt| {
                            evt.stop_propagation();
                            is_expanded.set(!is_expanded());
                        },
                        title: if is_expanded() { "Collapse" } else { "Expand" },
                        if is_expanded() { "🔼" } else { "🔽" }
                    }
                }
            }

            {is_expanded().then(|| rsx! {
                div { class: "statefulset-details",
                    // Basic Info Section
                    div { class: "statefulset-info",
                        div { class: "info-group",
                            div { class: "info-item", span { class: "info-label", "Namespace" } span { class: "info-value", "{statefulset_data.namespace}" } }
                            div { class: "info-item", span { class: "info-label", "Service Name" } span { class: "info-value", "{statefulset_data.service_name}" } }
                            div { class: "info-item", span { class: "info-label", "Age" } span { class: "info-value", "{statefulset_data.age}" } }
                        }
                        div { class: "info-group",
                            div { class: "info-item", span { class: "info-label", "Current" } span { class: "info-value", "{statefulset_data.current_replicas}" } }
                            div { class: "info-item", span { class: "info-label", "Updated" } span { class: "info-value", "{statefulset_data.updated_replicas}" } }
                        }
                    }

                    // Labels Section
                    div { class: "labels-section",
                        h4 { "Labels" }
                        div { class: "labels-grid",
                            {statefulset_data.labels.is_empty().then(|| rsx! {
                                span { class: "info-value", i { "No labels" } }
                            })}
                            {statefulset_data.labels.iter().map(|(key, value)| rsx! {
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
                            {statefulset_data.selector.is_empty().then(|| rsx! {
                                span { class: "info-value", i { "No selector" } }
                            })}
                            {statefulset_data.selector.iter().map(|(key, value)| rsx! {
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
                            {statefulset_data.conditions.is_empty().then(|| rsx! {
                                span { class: "info-value", i { "No conditions" } }
                            })}
                            {statefulset_data.conditions.iter().map(|cond| rsx! {
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

fn determine_status(statefulset: &StatefulSet) -> String {
    let ready_replicas = statefulset.status.as_ref().map_or(0, |s| s.ready_replicas.unwrap_or(0));
    let desired_replicas = statefulset.spec.as_ref().map_or(0, |s| s.replicas.unwrap_or(0));
    let current_replicas = statefulset.status.as_ref().map_or(0, |s| s.current_replicas.unwrap_or(0));
    let current_revision = statefulset.status.as_ref().and_then(|s| s.current_revision.as_ref());
    let update_revision = statefulset.status.as_ref().and_then(|s| s.update_revision.as_ref());

    if desired_replicas == 0 {
        return "Scaled Down".to_string();
    }

    if current_revision != update_revision {
        return "Rolling Update".to_string();
    }

    if ready_replicas < desired_replicas || current_replicas < desired_replicas {
        if ready_replicas == 0 {
            return "Degraded".to_string();
        }
        return "Progressing".to_string();
    }

    if ready_replicas == desired_replicas && current_replicas == desired_replicas {
        return "Available".to_string();
    }

    "Unknown".to_string()
}