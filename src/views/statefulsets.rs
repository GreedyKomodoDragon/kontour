use dioxus::prelude::*;
use std::collections::HashSet;

const STATEFULSETS_CSS: Asset = asset!("/assets/styling/statefulsets.css");

// --- Data Structures ---

#[derive(Clone, PartialEq)]
struct StatefulSetData {
    name: String,
    namespace: String,
    ready_replicas: u32,
    desired_replicas: u32,
    current_replicas: u32, // StatefulSets often track 'current'
    updated_replicas: u32,
    age: String,
    service_name: String, // Specific to StatefulSets
    selector: Vec<(String, String)>,
    conditions: Vec<StatefulSetCondition>, // Use a specific condition type if needed, or reuse DeploymentCondition
    labels: Vec<(String, String)>,
    // Add other relevant fields like pod management policy, update strategy
}

#[derive(Clone, PartialEq)]
struct StatefulSetCondition { // Can reuse DeploymentCondition if fields are identical
    condition_type: String,
    status: String,
    last_update_time: String,
    last_transition_time: String,
    reason: String,
    message: String,
}

// --- Sample Data ---

fn get_sample_statefulsets() -> Vec<StatefulSetData> {
    vec![
        StatefulSetData {
            name: "web-app".into(),
            namespace: "production".into(),
            ready_replicas: 3,
            desired_replicas: 3,
            current_replicas: 3,
            updated_replicas: 3,
            age: "10d".into(),
            service_name: "nginx-svc".into(),
            selector: vec![("app".into(), "nginx-web".into())],
            conditions: vec![
                StatefulSetCondition {
                    condition_type: "Ready".into(), // Example condition type
                    status: "True".into(),
                    last_update_time: "1d".into(),
                    last_transition_time: "10d".into(),
                    reason: "PodsReady".into(),
                    message: "All replicas are ready.".into(),
                },
            ],
            labels: vec![("app".into(), "nginx-web".into()), ("tier".into(), "frontend".into())],
        },
        StatefulSetData {
            name: "database-cluster".into(),
            namespace: "data".into(),
            ready_replicas: 2,
            desired_replicas: 3,
            current_replicas: 3,
            updated_replicas: 2,
            age: "5h".into(),
            service_name: "db-headless".into(),
            selector: vec![("app".into(), "postgres-db".into())],
            conditions: vec![
                 StatefulSetCondition {
                    condition_type: "Ready".into(),
                    status: "False".into(),
                    last_update_time: "5m".into(),
                    last_transition_time: "5h".into(),
                    reason: "UpdatingReplicas".into(),
                    message: "Waiting for replica database-cluster-2 to become ready.".into(),
                },
            ],
            labels: vec![("app".into(), "postgres-db".into()), ("role".into(), "master".into())],
        },
    ]
}


// --- Component ---

#[component]
pub fn StatefulSets() -> Element {
    let selected_namespace = use_signal(|| "all");
    let search_query = use_signal(String::new);
    let mut expanded_statefulsets = use_signal(|| HashSet::<String>::new());
    let statefulsets = use_signal(get_sample_statefulsets);

    // --- Filtering Logic ---
    let filtered_statefulsets = {
        let statefulsets = statefulsets.clone();
        let selected_namespace = selected_namespace.clone();
        let search_query = search_query.clone();

        use_signal(move || {
            let statefulsets = statefulsets.read();
            let query = search_query.read().to_lowercase();
            statefulsets.iter()
                .filter(|&sts| {
                    let ns_match = selected_namespace() == "all" || sts.namespace == selected_namespace();
                    let search_match = query.is_empty() || sts.name.to_lowercase().contains(&query) || sts.namespace.to_lowercase().contains(&query);
                    ns_match && search_match
                })
                .cloned()
                .collect::<Vec<_>>()
        })
    };

    // --- Unique Namespaces for Filter ---
    let namespaces = use_memo(move || {
        let mut ns = statefulsets.read().iter().map(|d| d.namespace.clone()).collect::<HashSet<_>>().into_iter().collect::<Vec<_>>();
        ns.sort();
        ns
    });


    rsx! {
        document::Link { rel: "stylesheet", href: STATEFULSETS_CSS }
        div { class: "statefulsets-container",
            // --- Header ---
            div { class: "statefulsets-header",
                div { class: "header-left",
                    h1 { "StatefulSets" }
                    div { class: "header-controls",
                        // Search Input
                        div { class: "search-container",
                            input {
                                class: "search-input",
                                r#type: "text",
                                placeholder: "Search statefulsets...",
                                value: "{search_query}",
                                // oninput: move |evt| search_query.set(evt.value().clone()),
                            }
                        }
                        // Namespace Select
                        select {
                            class: "namespace-select",
                            value: "{selected_namespace.read()}",
                            // onchange: move |evt| selected_namespace.set(evt.value().clone()),
                            option { value: "all", "All Namespaces" }
                            {namespaces.read().iter().map(|ns| rsx!{
                                option { key: "{ns}", value: "{ns}", "{ns}" }
                            })}
                        }
                        // Count
                        span { class: "statefulset-count", "{filtered_statefulsets.read().len()} statefulsets" }
                    }
                }
                // Header Actions
                div { class: "header-actions",
                    button { class: "btn btn-primary", "Create StatefulSet" } // Placeholder action
                    button { class: "btn btn-secondary", "Refresh" } // Placeholder action
                }
            }

            // --- StatefulSets Grid ---
            div { class: "statefulsets-grid",
                {filtered_statefulsets.read().iter().map(|sts| {
                    let is_expanded = expanded_statefulsets.read().contains(&sts.name);
                    let sts_name_clone = sts.name.clone();
                    // Determine status based on ready vs desired replicas
                    let status_class = if sts.ready_replicas == sts.desired_replicas {
                        "status-running"
                    } else if sts.ready_replicas < sts.desired_replicas {
                        "status-pending" // Or a specific updating status
                    } else {
                        "status-warning" // Or handle scale-down case
                    };

                    rsx! {
                        div {
                            key: "{sts.name}",
                            class: "statefulset-card",
                            // --- Card Header ---
                            div {
                                class: "statefulset-header-card",
                                div { class: "statefulset-title",
                                    h3 { "{sts.name}" }
                                    span { class: "status-badge {status_class}",
                                        "{sts.ready_replicas}/{sts.desired_replicas}"
                                    }
                                }
                                div { class: "statefulset-controls",
                                    // Expand/Collapse Button
                                    button {
                                        class: "btn-icon expand-toggle",
                                        onclick: move |evt| {
                                            evt.stop_propagation();
                                            let mut set = expanded_statefulsets.write();
                                            if set.contains(&sts_name_clone) {
                                                set.remove(&sts_name_clone);
                                            } else {
                                                set.insert(sts_name_clone.clone());
                                            }
                                        },
                                        title: if is_expanded { "Collapse" } else { "Expand" },
                                        if is_expanded { "🔼" } else { "🔽" }
                                    }
                                    // Placeholder Action Buttons
                                    button { class: "btn-icon", onclick: move |evt| evt.stop_propagation(), title: "View Pods", "📦" }
                                    button { class: "btn-icon", onclick: move |evt| evt.stop_propagation(), title: "Edit", "✏️" }
                                    button { class: "btn-icon", onclick: move |evt| evt.stop_propagation(), title: "Delete", "🗑️" }
                                }
                            }

                            // --- Expanded Details ---
                            {is_expanded.then(|| rsx! {
                                div { class: "statefulset-details",
                                    // Basic Info Row
                                    div { class: "statefulset-info",
                                        div { class: "info-group",
                                            div { class: "info-item", span { class: "info-label", "Namespace" } span { class: "info-value", "{sts.namespace}" } }
                                            div { class: "info-item", span { class: "info-label", "Service Name" } span { class: "info-value", "{sts.service_name}" } }
                                            div { class: "info-item", span { class: "info-label", "Age" } span { class: "info-value", "{sts.age}" } }
                                            // Add Pod Management Policy, Update Strategy etc. if needed
                                        }
                                         div { class: "info-group",
                                            div { class: "info-item", span { class: "info-label", "Desired" } span { class: "info-value", "{sts.desired_replicas}" } }
                                            div { class: "info-item", span { class: "info-label", "Current" } span { class: "info-value", "{sts.current_replicas}" } }
                                            div { class: "info-item", span { class: "info-label", "Ready" } span { class: "info-value", "{sts.ready_replicas}" } }
                                            div { class: "info-item", span { class: "info-label", "Updated" } span { class: "info-value", "{sts.updated_replicas}" } }
                                        }
                                    }

                                    // Labels Section
                                    div { class: "labels-section",
                                        h4 { "Labels" }
                                        div { class: "labels-grid",
                                            {sts.labels.iter().map(|(key, value)| rsx! {
                                                div { key: "{key}", class: "label", span { class: "label-key", "{key}" } span { class: "label-value", "{value}" } }
                                            })}
                                        }
                                    }

                                     // Selector Section
                                    div { class: "labels-section", // Re-use label styling for selector
                                        h4 { "Selector" }
                                        div { class: "labels-grid",
                                            {sts.selector.iter().map(|(key, value)| rsx! {
                                                div { key: "sel-{key}", class: "label", span { class: "label-key", "{key}" } span { class: "label-value", "{value}" } }
                                            })}
                                        }
                                    }

                                    // Conditions Section
                                    div { class: "conditions-section",
                                        h4 { "Conditions" }
                                        div { class: "conditions-grid",
                                            {sts.conditions.iter().map(|cond| rsx! {
                                                div {
                                                    key: "{cond.condition_type}",
                                                    class: "condition",
                                                    div { class: "condition-info",
                                                        span { class: "condition-type", "{cond.condition_type}" }
                                                        span { class: "condition-status status-{cond.status.to_lowercase()}", "{cond.status}" }
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
                                    // Placeholder for Pods section if needed later
                                }
                            })}
                        }
                    }
                })}
            }
        }
    }
}

