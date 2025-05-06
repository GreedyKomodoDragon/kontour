use dioxus::{logger::tracing, prelude::*};
use k8s_openapi::{api::apps::v1::Deployment, chrono::{DateTime, Utc}};
use kube::{api::ListParams, Api, Client};
use std::collections::HashSet;

use crate::components::{NamespaceSelector, StatusSelector, SearchInput};

const DEPLOYMENTS_CSS: Asset = asset!("/assets/styling/deployments.css");

#[derive(Clone)]
struct DeploymentFetcher {
    client: Client,
    deployments: Signal<Vec<Deployment>>,
}

impl DeploymentFetcher {
    fn fetch(&self, ns: String, status: String, query: String) {
        let client = self.client.clone();
        let mut deployments = self.deployments.clone();

        tracing::info!("Starting deployment fetch...");
        
        spawn(async move {
            let params = if status == "All" {
                ListParams::default()
            } else {
                ListParams::default()
                    .fields(&format!("status.phase={}", status))
            };
            
            match if ns == "All" {
                Api::<Deployment>::all(client.clone()).list(&params).await
            } else {
                Api::<Deployment>::namespaced(client.clone(), &ns).list(&params).await
            } {
                Ok(deployment_list) => {
                    let filtered_deployments = if query.is_empty() {
                        deployment_list.items
                    } else {
                        deployment_list.items
                            .into_iter()
                            .filter(|dep: &Deployment| {
                                dep.metadata.name.as_ref()
                                    .map(|name| name.to_lowercase().contains(&query.to_lowercase()))
                                    .unwrap_or(false)
                            })
                            .collect()
                    };
                    deployments.set(filtered_deployments);
                }
                Err(e) => {
                    tracing::error!("Failed to fetch deployments: {:?}", e);
                }
            }
        });
    }

    fn get_strategy(deployment: &Deployment) -> String {
        deployment
            .spec
            .as_ref()
            .and_then(|s| s.strategy.as_ref())
            .and_then(|st| st.type_.as_ref())
            .cloned()
            .unwrap_or_else(|| "Unknown".to_string())
    }
    
    fn get_age(deployment: &Deployment) -> String {
        deployment
            .metadata
            .creation_timestamp
            .as_ref()
            .map(|t| {
                let now = Utc::now();
                let created: DateTime<Utc> = t.0;
                let duration = now.signed_duration_since(created);
                
                if duration.num_days() > 0 {
                    format!("{}d", duration.num_days())
                } else if duration.num_hours() > 0 {
                    format!("{}h", duration.num_hours())
                } else {
                    format!("{}m", duration.num_minutes())
                }
            })
            .unwrap_or_else(|| "Unknown".to_string())
    }
}

#[component]
pub fn Deployments() -> Element {
    let client = use_context::<Client>();
    let navigate = use_navigator();

    let mut selected_status = use_signal(|| "All".to_string());
    let mut selected_namespace = use_signal(|| "All".to_string());
    let mut search_query = use_signal(String::new);
    let mut expanded_deployments = use_signal(|| HashSet::<String>::new());
    let deployments = use_signal(|| Vec::<Deployment>::new());

    let fetcher = DeploymentFetcher {
        client: client.clone(),
        deployments: deployments.clone(),
    };

    use_effect({
        let fetcher = fetcher.clone();
        move || {
            let ns = selected_namespace();
            let status = selected_status();
            let query = search_query();
            fetcher.fetch(ns, status, query);
        }
    });

    let refresh = {
        let fetcher = fetcher.clone();
        move |_| {
            let ns = selected_namespace();
            let status = selected_status();
            let query = search_query();
            fetcher.fetch(ns, status, query);
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: DEPLOYMENTS_CSS }
        div { class: "deployments-container",
            div { class: "deployments-header",
                div { class: "header-left",
                    h1 { "Deployments" }
                    div { class: "header-controls",
                        SearchInput {
                            query: search_query(),
                            on_change: move |q| search_query.set(q)
                        }
                        NamespaceSelector {
                            selected_namespace: selected_namespace(),
                            on_change: move |ns| selected_namespace.set(ns)
                        }
                        StatusSelector {
                            selected_status: selected_status(),
                            on_change: move |status| selected_status.set(status)
                        }
                        span { class: "deployment-count", "{deployments().len()} deployments" }
                    }
                }
                div { class: "header-actions",
                    button { 
                        class: "btn btn-primary",
                        onclick: move |_| {
                            navigate.push("/deployments/create");
                        },
                        "Create Deployment" 
                    }
                    button { 
                        class: "btn btn-secondary",
                        onclick: refresh,
                        "Refresh" 
                    }
                }
            }

            div { class: "deployments-grid",
                {deployments().iter().map(|dep| {
                    let name = dep.metadata.name.as_ref().unwrap_or(&String::new()).to_string();
                    let is_expanded = expanded_deployments.read().contains(&name);
                    let dep_name_clone = name.clone();
                    let ready_replicas = dep.status.as_ref().map_or(0, |s| s.ready_replicas.unwrap_or(0));
                    let desired_replicas = dep.spec.as_ref().map_or(0, |s| s.replicas.unwrap_or(0));
                    let available_replicas = dep.status.as_ref().map_or(0, |s| s.available_replicas.unwrap_or(0));

                    let status_class = if ready_replicas == desired_replicas && available_replicas >= desired_replicas {
                        "status-running"
                    } else if ready_replicas < desired_replicas {
                        "status-pending"
                    } else {
                        "status-warning"
                    };

                    rsx! {
                        div {
                            key: "{dep.metadata.name.as_ref().unwrap_or(&String::new())}",
                            class: "deployment-card",
                            div {
                                class: "deployment-header-card",
                                div { class: "deployment-title",
                                    h3 { "{dep.metadata.name.as_ref().unwrap_or(&String::new())}" }
                                    span { class: "status-badge {status_class}",
                                        "{ready_replicas}/{desired_replicas}"
                                    }
                                }
                                div { class: "deployment-controls",
                                    button {
                                        class: "btn-icon expand-toggle",
                                        onclick: move |evt| {
                                            evt.stop_propagation();
                                            if expanded_deployments.read().contains(&dep_name_clone) {
                                                expanded_deployments.write().remove(&dep_name_clone);
                                            } else {
                                                expanded_deployments.write().insert(dep_name_clone.clone());
                                            }
                                        },
                                        title: if is_expanded { "Collapse" } else { "Expand" },
                                        if is_expanded { "🔼" } else { "🔽" }
                                    }
                                    button { class: "btn-icon", onclick: move |evt| evt.stop_propagation(), title: "View ReplicaSets", "🧱" }
                                    button { class: "btn-icon", onclick: move |evt| evt.stop_propagation(), title: "View Pods", "📦" }
                                    button { class: "btn-icon", onclick: move |evt| evt.stop_propagation(), title: "Edit", "✏️" }
                                    button { class: "btn-icon", onclick: move |evt| evt.stop_propagation(), title: "Delete", "🗑️" }
                                }
                            }

                            {is_expanded.then(|| rsx! {
                                div { class: "deployment-details",
                                    // Basic Info Section
                                    div { class: "deployment-info",
                                        div { class: "info-group",
                                            div { class: "info-item", span { class: "info-label", "Namespace" } span { class: "info-value", "{dep.metadata.namespace.as_ref().unwrap_or(&String::new())}" } }
                                            div { class: "info-item", span { class: "info-label", "Strategy" } span { class: "info-value", "{DeploymentFetcher::get_strategy(dep)}" } }
                                            div { class: "info-item", span { class: "info-label", "Age" } span { class: "info-value", "{DeploymentFetcher::get_age(dep)}" } }
                                        }
                                    }

                                    // Labels Section
                                    div { class: "labels-section",
                                        h4 { "Labels" }
                                        div { class: "labels-grid",
                                            {dep.metadata.labels.as_ref().map_or_else(
                                                || rsx! { 
                                                    span { class: "info-value", i { "No labels" } }
                                                },
                                                |labels| rsx! {
                                                    {labels.iter().map(|(key, value)| rsx! {
                                                        div { 
                                                            key: "{key}",
                                                            class: "label",
                                                            span { class: "label-key", "{key}" }
                                                            span { class: "label-value", "{value}" }
                                                        }
                                                    })}
                                                }
                                            )}
                                        }
                                    }

                                    // Selector Section
                                    div { class: "labels-section",
                                        h4 { "Selector" }
                                        div { class: "labels-grid",
                                            {dep.spec.as_ref()
                                                .and_then(|s| s.selector.match_labels.as_ref())
                                                .map_or_else(
                                                    || rsx! { 
                                                        span { class: "info-value", i { "No selector" } }
                                                    },
                                                    |selector| rsx! {
                                                        {selector.iter().map(|(key, value)| rsx! {
                                                            div { 
                                                                key: "sel-{key}",
                                                                class: "label",
                                                                span { class: "label-key", "{key}" }
                                                                span { class: "label-value", "{value}" }
                                                            }
                                                        })}
                                                    }
                                                )}
                                        }
                                    }

                                    // Conditions Section
                                    div { class: "conditions-section",
                                        h4 { "Conditions" }
                                        div { class: "conditions-grid",
                                            {dep.status.as_ref()
                                                .and_then(|s| s.conditions.as_ref())
                                                .map_or_else(
                                                    || rsx! { 
                                                        span { class: "info-value", i { "No conditions" } }
                                                    },
                                                    |conditions| rsx! {
                                                        {conditions.iter().map(|cond| rsx! {
                                                            div {
                                                                key: "{cond.type_}",
                                                                class: "condition",
                                                                div { class: "condition-info",
                                                                    span { class: "condition-type", "{cond.type_}" }
                                                                    span { 
                                                                        class: "condition-status status-{cond.status.to_lowercase()}",
                                                                        "{cond.status}"
                                                                    }
                                                                }
                                                                div { class: "condition-details",
                                                                    span { class: "condition-reason", "{cond.reason.as_ref().unwrap_or(&String::new())}" }
                                                                    span { class: "condition-time", "{cond.last_transition_time.as_ref().map_or_else(String::new, |t| t.0.to_string())}" }
                                                                }
                                                                div { class: "condition-message", "{cond.message.as_ref().unwrap_or(&String::new())}"}
                                                            }
                                                        })}
                                                    }
                                                )}
                                        }
                                    }
                                }
                            })}
                        }
                    }
                })}
            }
        }
    }
}
