use dioxus::{logger::tracing, prelude::*};
use k8s_openapi::api::apps::v1::Deployment;
use kube::{api::ListParams, Api, Client};

use crate::components::{NamespaceSelector, StatusSelector, SearchInput, DeploymentItem};

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
            // First get all deployments since we need to examine conditions
            let api = if ns == "All" {
                Api::<Deployment>::all(client.clone())
            } else {
                Api::<Deployment>::namespaced(client.clone(), &ns)
            };
            
            match api.list(&ListParams::default()).await {
                Ok(deployment_list) => {
                    let mut filtered_deployments = if query.is_empty() {
                        deployment_list.items
                    } else {
                        deployment_list.items
                            .into_iter()
                            .filter(|dep: &Deployment| {
                                dep.metadata.name.as_ref()
                                    .map(|name| name.to_lowercase().contains(&query.to_lowercase()))
                                    .unwrap_or(false)
                            })
                            .collect::<Vec<_>>()
                    };

                    // Filter by status if not "All"
                    if status != "All" {
                        filtered_deployments = filtered_deployments.into_iter()
                            .filter(|dep| {
                                let desired = dep.spec.as_ref().map_or(0, |s| s.replicas.unwrap_or(0));
                                let updated = dep.status.as_ref().map_or(0, |s| s.updated_replicas.unwrap_or(0));
                                let available = dep.status.as_ref().map_or(0, |s| s.available_replicas.unwrap_or(0));
                                
                                let conditions = dep.status.as_ref()
                                    .and_then(|s| s.conditions.as_ref())
                                    .map(|c| c.iter().map(|cond| (cond.type_.clone(), cond.status.clone(), cond.reason.clone())).collect::<Vec<_>>())
                                    .unwrap_or_default();

                                let is_progressing = conditions.iter().any(|(t, s, _)| t == "Progressing" && s == "True");
                                let is_available = conditions.iter().any(|(t, s, _)| t == "Available" && s == "True");
                                let has_replica_failure = conditions.iter().any(|(t, _, r)| t == "Progressing" && r.as_ref().map_or(false, |r| r == "ReplicaFailure"));
                                
                                match status.as_str() {
                                    "Available" => is_available && is_progressing && updated == desired && available == desired,
                                    "Progressing" => is_progressing && (!is_available || updated != desired),
                                    "Degraded" => has_replica_failure || (!is_progressing && !is_available),
                                    "Scaled Down" => desired == 0,
                                    _ => false
                                }
                            })
                            .collect();
                    }
                    
                    deployments.set(filtered_deployments);
                }
                Err(e) => {
                    tracing::error!("Failed to fetch deployments: {:?}", e);
                }
            }
        });
    }
}

#[component]
pub fn Deployments() -> Element {
    let client_signal = use_context::<Signal<Option<Client>>>();
    let navigate = use_navigator();

    let mut selected_status = use_signal(|| "All".to_string());
    let mut selected_namespace = use_signal(|| "All".to_string());
    let mut search_query = use_signal(String::new);
    let deployments = use_signal(|| Vec::<Deployment>::new());

    // Always call use_effect but handle conditional logic inside
    use_effect({
        let client_signal = client_signal.clone();
        let deployments = deployments.clone();
        move || {
            if let Some(client) = &*client_signal.read() {
                let fetcher = DeploymentFetcher {
                    client: client.clone(),
                    deployments: deployments.clone(),
                };
                let ns = selected_namespace();
                let status = selected_status();
                let query = search_query();
                fetcher.fetch(ns, status, query);
            }
        }
    });

    let refresh = {
        let client_signal = client_signal.clone();
        let deployments = deployments.clone();
        move |_| {
            if let Some(client) = &*client_signal.read() {
                let fetcher = DeploymentFetcher {
                    client: client.clone(),
                    deployments: deployments.clone(),
                };
                let ns = selected_namespace();
                let status = selected_status();
                let query = search_query();
                fetcher.fetch(ns, status, query);
            }
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
                            on_change: move |status| selected_status.set(status),
                            custom_statuses: Some(vec!["All", "Available", "Progressing", "Degraded", "Scaled Down"])
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
                {deployments().iter().map(|deployment| {
                    rsx! {
                        DeploymentItem { deployment: deployment.clone() }
                    }
                })}
            }
        }
    }
}
