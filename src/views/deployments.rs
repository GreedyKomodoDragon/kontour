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
}

#[component]
pub fn Deployments() -> Element {
    let client = use_context::<Client>();
    let navigate = use_navigator();

    let mut selected_status = use_signal(|| "All".to_string());
    let mut selected_namespace = use_signal(|| "All".to_string());
    let mut search_query = use_signal(String::new);
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
                {deployments().iter().map(|deployment| {
                    rsx! {
                        DeploymentItem { deployment: deployment.clone() }
                    }
                })}
            }
        }
    }
}
