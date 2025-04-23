use dioxus::prelude::*;
    use std::collections::HashSet;

    const DEPLOYMENTS_CSS: Asset = asset!("/assets/styling/deployments.css");

    // --- Data Structures ---

    #[derive(Clone, PartialEq)]
    struct DeploymentData {
        name: String,
        namespace: String,
        ready_replicas: u32,
        desired_replicas: u32,
        updated_replicas: u32,
        available_replicas: u32,
        age: String,
        strategy: String,
        selector: Vec<(String, String)>,
        conditions: Vec<DeploymentCondition>,
        labels: Vec<(String, String)>,
        // Add other relevant fields like annotations, image info if needed
    }

    #[derive(Clone, PartialEq)]
    struct DeploymentCondition {
        condition_type: String,
        status: String,
        last_update_time: String,
        last_transition_time: String,
        reason: String,
        message: String,
    }

    // --- Sample Data ---

    fn get_sample_deployments() -> Vec<DeploymentData> {
        vec![
            DeploymentData {
                name: "nginx-deployment".into(),
                namespace: "default".into(),
                ready_replicas: 1,
                desired_replicas: 1,
                updated_replicas: 1,
                available_replicas: 1,
                age: "3d".into(),
                strategy: "RollingUpdate".into(),
                selector: vec![("app".into(), "nginx".into())],
                conditions: vec![
                    DeploymentCondition {
                        condition_type: "Available".into(),
                        status: "True".into(),
                        last_update_time: "3d".into(),
                        last_transition_time: "3d".into(),
                        reason: "MinimumReplicasAvailable".into(),
                        message: "Deployment has minimum availability.".into(),
                    },
                    DeploymentCondition {
                        condition_type: "Progressing".into(),
                        status: "True".into(),
                        last_update_time: "3d".into(),
                        last_transition_time: "3d".into(),
                        reason: "NewReplicaSetAvailable".into(),
                        message: "ReplicaSet \"nginx-deployment-6b474476c4\" has successfully progressed.".into(), // Escaped quotes
                    },
                ],
                labels: vec![("app".into(), "nginx".into())],
            },
            DeploymentData {
                name: "prometheus-deployment".into(),
                namespace: "monitoring".into(),
                ready_replicas: 2,
                desired_replicas: 2,
                updated_replicas: 2,
                available_replicas: 2,
                age: "6d".into(),
                strategy: "RollingUpdate".into(),
                selector: vec![("app".into(), "prometheus".into())],
                conditions: vec![
                     DeploymentCondition {
                        condition_type: "Available".into(),
                        status: "True".into(),
                        last_update_time: "6d".into(),
                        last_transition_time: "6d".into(),
                        reason: "MinimumReplicasAvailable".into(),
                        message: "Deployment has minimum availability.".into(),
                    },
                    DeploymentCondition {
                        condition_type: "Progressing".into(),
                        status: "True".into(),
                        last_update_time: "6d".into(),
                        last_transition_time: "6d".into(),
                        reason: "NewReplicaSetAvailable".into(),
                        message: "ReplicaSet \"prometheus-deployment-7d7d4769b8\" has successfully progressed.".into(),
                    },
                ],
                labels: vec![("app".into(), "prometheus".into())],
            },
             DeploymentData {
                name: "grafana-deployment".into(),
                namespace: "monitoring".into(),
                ready_replicas: 0,
                desired_replicas: 1,
                updated_replicas: 1,
                available_replicas: 0,
                age: "1h".into(),
                strategy: "Recreate".into(),
                selector: vec![("app".into(), "grafana".into())],
                conditions: vec![
                     DeploymentCondition {
                        condition_type: "Available".into(),
                        status: "False".into(),
                        last_update_time: "1h".into(),
                        last_transition_time: "1h".into(),
                        reason: "MinimumReplicasUnavailable".into(),
                        message: "Deployment does not have minimum availability.".into(),
                    },
                    DeploymentCondition {
                        condition_type: "Progressing".into(),
                        status: "False".into(),
                        last_update_time: "1h".into(),
                        last_transition_time: "1h".into(),
                        reason: "ProgressDeadlineExceeded".into(),
                        message: "ReplicaSet \"grafana-deployment-xxxxx\" has failed progressing.".into(), // Escaped quotes
                    },
                ],
                labels: vec![("app".into(), "grafana".into())],
            },
        ]
    }


    // --- Component ---

    #[component]
    pub fn Deployments() -> Element {
        let selected_namespace = use_signal(|| "all");
        let search_query = use_signal(String::new);
        let mut expanded_deployments = use_signal(|| HashSet::<String>::new());
        let deployments = use_signal(get_sample_deployments);

        // --- Filtering Logic ---
        let filtered_deployments = {
            let deployments = deployments.clone();
            let selected_namespace = selected_namespace.clone();
            let search_query = search_query.clone();

            use_signal(move || {
                let deployments = deployments.read();
                let query = search_query.read().to_lowercase();
                deployments.iter()
                    .filter(|&dep| {
                        let ns_match = selected_namespace() == "all" || dep.namespace == selected_namespace();
                        let search_match = query.is_empty() || dep.name.to_lowercase().contains(&query) || dep.namespace.to_lowercase().contains(&query);
                        ns_match && search_match
                    })
                    .cloned()
                    .collect::<Vec<_>>()
            })
        };

        // --- Unique Namespaces for Filter ---
        let namespaces = use_memo(move || {
            let mut ns = deployments.read().iter().map(|d| d.namespace.clone()).collect::<HashSet<_>>().into_iter().collect::<Vec<_>>();
            ns.sort();
            ns
        });


        rsx! {
            document::Link { rel: "stylesheet", href: DEPLOYMENTS_CSS }
            div { class: "deployments-container",
                // --- Header ---
                div { class: "deployments-header",
                    div { class: "header-left",
                        h1 { "Deployments" }
                        div { class: "header-controls",
                            // Search Input
                            div { class: "search-container",
                                input {
                                    class: "search-input",
                                    r#type: "text",
                                    placeholder: "Search deployments...",
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
                            span { class: "deployment-count", "{filtered_deployments.read().len()} deployments" }
                        }
                    }
                    // Header Actions
                    div { class: "header-actions",
                        button { class: "btn btn-primary", "Create Deployment" } // Placeholder action
                        button { class: "btn btn-secondary", "Refresh" } // Placeholder action
                    }
                }

                // --- Deployments Grid ---
                div { class: "deployments-grid",
                    {filtered_deployments.read().iter().map(|dep| {
                        let is_expanded = expanded_deployments.read().contains(&dep.name);
                        let dep_name_clone = dep.name.clone();
                        let status_class = if dep.ready_replicas == dep.desired_replicas && dep.available_replicas >= dep.desired_replicas {
                            "status-running" // Or "status-available"
                        } else if dep.ready_replicas < dep.desired_replicas {
                            "status-pending" // Or "status-progressing"
                        } else {
                            "status-warning" // Or some other state
                        };

                        rsx! {
                            div {
                                key: "{dep.name}",
                                class: "deployment-card",
                                // --- Card Header ---
                                div {
                                    class: "deployment-header-card", // Renamed to avoid conflict
                                    div { class: "deployment-title",
                                        h3 { "{dep.name}" }
                                        span { class: "status-badge {status_class}",
                                            "{dep.ready_replicas}/{dep.desired_replicas}"
                                        }
                                    }
                                    div { class: "deployment-controls",
                                        // Expand/Collapse Button
                                        button {
                                            class: "btn-icon expand-toggle",
                                            onclick: move |evt| {
                                                evt.stop_propagation();
                                                let mut set = expanded_deployments.write();
                                                if set.contains(&dep_name_clone) {
                                                    set.remove(&dep_name_clone);
                                                } else {
                                                    set.insert(dep_name_clone.clone());
                                                }
                                            },
                                            title: if is_expanded { "Collapse" } else { "Expand" },
                                            if is_expanded { "🔼" } else { "🔽" }
                                        }
                                        // Placeholder Action Buttons
                                        button { class: "btn-icon", onclick: move |evt| evt.stop_propagation(), title: "View ReplicaSets", "🧱" }
                                        button { class: "btn-icon", onclick: move |evt| evt.stop_propagation(), title: "View Pods", "📦" }
                                        button { class: "btn-icon", onclick: move |evt| evt.stop_propagation(), title: "Edit", "✏️" }
                                        button { class: "btn-icon", onclick: move |evt| evt.stop_propagation(), title: "Delete", "🗑️" }
                                    }
                                }

                                // --- Expanded Details ---
                                {is_expanded.then(|| rsx! {
                                    div { class: "deployment-details",
                                        // Basic Info Row
                                        div { class: "deployment-info",
                                            div { class: "info-group",
                                                div { class: "info-item", span { class: "info-label", "Namespace" } span { class: "info-value", "{dep.namespace}" } }
                                                div { class: "info-item", span { class: "info-label", "Strategy" } span { class: "info-value", "{dep.strategy}" } }
                                                div { class: "info-item", span { class: "info-label", "Age" } span { class: "info-value", "{dep.age}" } }
                                            }
                                             div { class: "info-group",
                                                div { class: "info-item", span { class: "info-label", "Desired" } span { class: "info-value", "{dep.desired_replicas}" } }
                                                div { class: "info-item", span { class: "info-label", "Current" } span { class: "info-value", "{dep.ready_replicas}" } } // Assuming ready = current for simplicity
                                                div { class: "info-item", span { class: "info-label", "Updated" } span { class: "info-value", "{dep.updated_replicas}" } }
                                                div { class: "info-item", span { class: "info-label", "Available" } span { class: "info-value", "{dep.available_replicas}" } }
                                            }
                                        }

                                        // Labels Section
                                        div { class: "labels-section",
                                            h4 { "Labels" }
                                            div { class: "labels-grid",
                                                {dep.labels.iter().map(|(key, value)| rsx! {
                                                    div { key: "{key}", class: "label", span { class: "label-key", "{key}" } span { class: "label-value", "{value}" } }
                                                })}
                                            }
                                        }

                                         // Selector Section
                                        div { class: "labels-section", // Re-use label styling for selector
                                            h4 { "Selector" }
                                            div { class: "labels-grid",
                                                {dep.selector.iter().map(|(key, value)| rsx! {
                                                    div { key: "sel-{key}", class: "label", span { class: "label-key", "{key}" } span { class: "label-value", "{value}" } }
                                                })}
                                            }
                                        }

                                        // Conditions Section
                                        div { class: "conditions-section",
                                            h4 { "Conditions" }
                                            div { class: "conditions-grid", // Use grid layout
                                                // Header Row (Optional)
                                                // div { class: "condition condition-header", ... }
                                                {dep.conditions.iter().map(|cond| rsx! {
                                                    div {
                                                        key: "{cond.condition_type}",
                                                        class: "condition", // Reuse pod condition styling
                                                        div { class: "condition-info",
                                                            span { class: "condition-type", "{cond.condition_type}" }
                                                            span { class: "condition-status status-{cond.status.to_lowercase()}", "{cond.status}" }
                                                        }
                                                        div { class: "condition-details",
                                                            span { class: "condition-reason", "{cond.reason}" }
                                                            span { class: "condition-time", "{cond.last_transition_time}" } // Or last_update_time
                                                        }
                                                        div { class: "condition-message", "{cond.message}"} // Added message display
                                                    }
                                                })}
                                            }
                                        }
                                        // Placeholder for ReplicaSets/Pods sections if needed later
                                    }
                                })}
                            }
                        }
                    })}
                }
            }
        }
    }
