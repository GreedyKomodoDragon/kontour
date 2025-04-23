use dioxus::prelude::*;
use std::collections::HashSet;

const DAEMONSETS_CSS: Asset = asset!("/assets/styling/daemonsets.css");

// --- Data Structures ---

#[derive(Clone, PartialEq)]
struct DaemonSetData {
    name: String,
    namespace: String,
    desired_scheduled: u32,
    current_scheduled: u32,
    ready_scheduled: u32,
    updated_scheduled: u32,
    available_scheduled: u32,
    age: String,
    selector: Vec<(String, String)>,
    // Conditions might be simpler for DaemonSets, but we can reuse for consistency
    conditions: Vec<DaemonSetCondition>,
    labels: Vec<(String, String)>,
    node_selector: Vec<(String, String)>, // Specific to DaemonSets
    // Add other relevant fields like update strategy
}

#[derive(Clone, PartialEq)]
struct DaemonSetCondition { // Can reuse DeploymentCondition if fields are identical
    condition_type: String,
    status: String,
    last_update_time: String,
    last_transition_time: String,
    reason: String,
    message: String,
}

// --- Sample Data ---

fn get_sample_daemonsets() -> Vec<DaemonSetData> {
    vec![
        DaemonSetData {
            name: "fluentd-agent".into(),
            namespace: "kube-system".into(),
            desired_scheduled: 3,
            current_scheduled: 3,
            ready_scheduled: 3,
            updated_scheduled: 3,
            available_scheduled: 3,
            age: "30d".into(),
            selector: vec![("app".into(), "fluentd".into())],
            conditions: vec![
                // DaemonSets might not have standard conditions like Deployments
                // Add relevant status info if available from API
            ],
            labels: vec![("app".into(), "fluentd".into()), ("k8s-app".into(), "fluentd-logging".into())],
            node_selector: vec![], // Example: empty node selector means run on all nodes
        },
        DaemonSetData {
            name: "node-exporter".into(),
            namespace: "monitoring".into(),
            desired_scheduled: 2, // Assume 2 nodes match selector
            current_scheduled: 2,
            ready_scheduled: 1,
            updated_scheduled: 2,
            available_scheduled: 1,
            age: "2d".into(),
            selector: vec![("app".into(), "node-exporter".into())],
            conditions: vec![],
            labels: vec![("app".into(), "node-exporter".into())],
            node_selector: vec![("disktype".into(), "ssd".into())], // Example node selector
        },
    ]
}


// --- Component ---

#[component]
pub fn DaemonSets() -> Element {
    let selected_namespace = use_signal(|| "all");
    let search_query = use_signal(String::new);
    let mut expanded_daemonsets = use_signal(|| HashSet::<String>::new());
    let daemonsets = use_signal(get_sample_daemonsets);

    // --- Filtering Logic ---
    let filtered_daemonsets = {
        let daemonsets = daemonsets.clone();
        let selected_namespace = selected_namespace.clone();
        let search_query = search_query.clone();

        use_signal(move || {
            let daemonsets = daemonsets.read();
            let query = search_query.read().to_lowercase();
            daemonsets.iter()
                .filter(|&ds| {
                    let ns_match = selected_namespace() == "all" || ds.namespace == selected_namespace();
                    let search_match = query.is_empty() || ds.name.to_lowercase().contains(&query) || ds.namespace.to_lowercase().contains(&query);
                    ns_match && search_match
                })
                .cloned()
                .collect::<Vec<_>>()
        })
    };

    // --- Unique Namespaces for Filter ---
    let namespaces = use_memo(move || {
        let mut ns = daemonsets.read().iter().map(|d| d.namespace.clone()).collect::<HashSet<_>>().into_iter().collect::<Vec<_>>();
        ns.sort();
        ns
    });


    rsx! {
        document::Link { rel: "stylesheet", href: DAEMONSETS_CSS }
        div { class: "daemonsets-container",
            // --- Header ---
            div { class: "daemonsets-header",
                div { class: "header-left",
                    h1 { "DaemonSets" }
                    div { class: "header-controls",
                        // Search Input
                        div { class: "search-container",
                            input {
                                class: "search-input",
                                r#type: "text",
                                placeholder: "Search daemonsets...",
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
                        span { class: "daemonset-count", "{filtered_daemonsets.read().len()} daemonsets" }
                    }
                }
                // Header Actions
                div { class: "header-actions",
                    button { class: "btn btn-primary", "Create DaemonSet" } // Placeholder action
                    button { class: "btn btn-secondary", "Refresh" } // Placeholder action
                }
            }

            // --- DaemonSets Grid ---
            div { class: "daemonsets-grid",
                {filtered_daemonsets.read().iter().map(|ds| {
                    let is_expanded = expanded_daemonsets.read().contains(&ds.name);
                    let ds_name_clone = ds.name.clone();
                    // Determine status based on ready vs desired scheduled pods
                    let status_class = if ds.ready_scheduled == ds.desired_scheduled && ds.desired_scheduled > 0 {
                        "status-running"
                    } else if ds.ready_scheduled < ds.desired_scheduled {
                        "status-pending" // Or a specific updating status
                    } else if ds.desired_scheduled == 0 {
                         "status-warning" // Indicate if no nodes match selector?
                    } else {
                        "status-warning" // Other cases
                    };

                    rsx! {
                        div {
                            key: "{ds.name}",
                            class: "daemonset-card",
                            // --- Card Header ---
                            div {
                                class: "daemonset-header-card",
                                div { class: "daemonset-title",
                                    h3 { "{ds.name}" }
                                    span { class: "status-badge {status_class}",
                                        "{ds.ready_scheduled}/{ds.desired_scheduled}" // Show ready/desired
                                    }
                                }
                                div { class: "daemonset-controls",
                                    // Expand/Collapse Button
                                    button {
                                        class: "btn-icon expand-toggle",
                                        onclick: move |evt| {
                                            evt.stop_propagation();
                                            let mut set = expanded_daemonsets.write();
                                            if set.contains(&ds_name_clone) {
                                                set.remove(&ds_name_clone);
                                            } else {
                                                set.insert(ds_name_clone.clone());
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
                                div { class: "daemonset-details",
                                    // Basic Info Row
                                    div { class: "daemonset-info",
                                        div { class: "info-group",
                                            div { class: "info-item", span { class: "info-label", "Namespace" } span { class: "info-value", "{ds.namespace}" } }
                                            div { class: "info-item", span { class: "info-label", "Age" } span { class: "info-value", "{ds.age}" } }
                                            // Add Update Strategy etc. if needed
                                        }
                                         div { class: "info-group",
                                            div { class: "info-item", span { class: "info-label", "Desired" } span { class: "info-value", "{ds.desired_scheduled}" } }
                                            div { class: "info-item", span { class: "info-label", "Current" } span { class: "info-value", "{ds.current_scheduled}" } }
                                            div { class: "info-item", span { class: "info-label", "Ready" } span { class: "info-value", "{ds.ready_scheduled}" } }
                                            div { class: "info-item", span { class: "info-label", "Updated" } span { class: "info-value", "{ds.updated_scheduled}" } }
                                            div { class: "info-item", span { class: "info-label", "Available" } span { class: "info-value", "{ds.available_scheduled}" } }
                                        }
                                    }

                                    // Labels Section
                                    div { class: "labels-section",
                                        h4 { "Labels" }
                                        div { class: "labels-grid",
                                            {ds.labels.iter().map(|(key, value)| rsx! {
                                                div { key: "{key}", class: "label", span { class: "label-key", "{key}" } span { class: "label-value", "{value}" } }
                                            })}
                                        }
                                    }

                                     // Selector Section
                                    div { class: "labels-section",
                                        h4 { "Selector" }
                                        div { class: "labels-grid",
                                            {ds.selector.iter().map(|(key, value)| rsx! {
                                                div { key: "sel-{key}", class: "label", span { class: "label-key", "{key}" } span { class: "label-value", "{value}" } }
                                            })}
                                        }
                                    }

                                     // Node Selector Section
                                    div { class: "labels-section",
                                        h4 { "Node Selector" }
                                        if ds.node_selector.is_empty() {
                                             div { class: "labels-grid", span { class: "info-value", i { "None (runs on all eligible nodes)" } } }
                                        } else {
                                            div { class: "labels-grid",
                                                {ds.node_selector.iter().map(|(key, value)| rsx! {
                                                    div { key: "node-sel-{key}", class: "label", span { class: "label-key", "{key}" } span { class: "label-value", "{value}" } }
                                                })}
                                            }
                                        }
                                    }

                                    // Conditions Section (Optional for DaemonSets)
                                    if !ds.conditions.is_empty() {
                                        {rsx! {
                                            div { class: "conditions-section",
                                                h4 { "Conditions" }
                                                div { class: "conditions-grid",
                                                    {ds.conditions.iter().map(|cond| rsx! {
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
                                        }
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
