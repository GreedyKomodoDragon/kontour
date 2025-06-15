use dioxus::{logger::tracing, prelude::*};
use k8s_openapi::api::apps::v1::StatefulSet;
use kube::{api::ListParams, Api, Client};

use crate::components::{NamespaceSelector, StatusSelector, SearchInput, StatefulSetItem};

const STATEFULSETS_CSS: Asset = asset!("/assets/styling/statefulsets.css");

#[derive(Clone)]
struct StatefulSetFetcher {
    client: Client,
    statefulsets: Signal<Vec<StatefulSet>>,
}

impl StatefulSetFetcher {
    fn fetch(&self, ns: String, status: String, query: String) {
        let client = self.client.clone();
        let mut statefulsets = self.statefulsets.clone();

        tracing::info!("Starting statefulset fetch...");
        
        spawn(async move {
            let api = if ns == "All" {
                Api::<StatefulSet>::all(client.clone())
            } else {
                Api::<StatefulSet>::namespaced(client.clone(), &ns)
            };
            
            match api.list(&ListParams::default()).await {
                Ok(statefulset_list) => {
                    let mut filtered_statefulsets = if query.is_empty() {
                        statefulset_list.items
                    } else {
                        statefulset_list.items
                            .into_iter()
                            .filter(|sts: &StatefulSet| {
                                sts.metadata.name.as_ref()
                                    .map(|name| name.to_lowercase().contains(&query.to_lowercase()))
                                    .unwrap_or(false)
                            })
                            .collect::<Vec<_>>()
                    };

                    // Filter by status if not "All"
                    if status != "All" {
                        filtered_statefulsets = filtered_statefulsets.into_iter()
                            .filter(|sts| {
                                let ready = sts.status.as_ref().map_or(0, |s| s.ready_replicas.unwrap_or(0));
                                let desired = sts.spec.as_ref().map_or(0, |s| s.replicas.unwrap_or(0));
                                let current = sts.status.as_ref().map_or(0, |s| s.current_replicas.unwrap_or(0));
                                let updated = sts.status.as_ref().map_or(0, |s| s.updated_replicas.unwrap_or(0));

                                match status.as_str() {
                                    "Available" => ready == desired && current == desired && updated == desired,
                                    "Progressing" => updated < desired,
                                    "Rolling Update" => updated < desired && ready < desired,
                                    "Degraded" => ready < desired,
                                    "Scaled Down" => desired == 0,
                                    _ => false
                                }
                            })
                            .collect();
                    }
                    
                    statefulsets.set(filtered_statefulsets);
                }
                Err(e) => {
                    tracing::error!("Failed to fetch statefulsets: {:?}", e);
                }
            }
        });
    }
}

#[component]
pub fn StatefulSets() -> Element {
    let client_signal = use_context::<Signal<Option<Client>>>();
    let navigate = use_navigator();

    let mut selected_status = use_signal(|| "All".to_string());
    let mut selected_namespace = use_signal(|| "All".to_string());
    let mut search_query = use_signal(String::new);
    let statefulsets = use_signal(|| Vec::<StatefulSet>::new());

    // Always call use_effect but handle conditional logic inside
    use_effect({
        move || {
            if let Some(client) = &*client_signal.read() {
                let fetcher = StatefulSetFetcher {
                    client: client.clone(),
                    statefulsets: statefulsets.clone(),
                };
                let ns = selected_namespace();
                let status = selected_status();
                let query = search_query();
                fetcher.fetch(ns, status, query);
            }
        }
    });

    let refresh = {
        move |_| {
            if let Some(client) = &*client_signal.read() {
                let fetcher = StatefulSetFetcher {
                    client: client.clone(),
                    statefulsets: statefulsets.clone(),
                };
                let ns = selected_namespace();
                let status = selected_status();
                let query = search_query();
                fetcher.fetch(ns, status, query);
            }
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: STATEFULSETS_CSS }
        div { class: "statefulsets-container",
            div { class: "statefulsets-header",
                div { class: "header-left",
                    h1 { "StatefulSets" }
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
                            custom_statuses: Some(vec!["All", "Available", "Progressing", "Rolling Update", "Degraded", "Scaled Down"])
                        }
                        span { class: "statefulset-count", "{statefulsets().len()} statefulsets" }
                    }
                }
                div { class: "header-actions",
                    button { 
                        class: "btn btn-primary",
                        onclick: move |_| {
                            navigate.push("/statefulsets/create");
                        },
                        "Create StatefulSet" 
                    }
                    button { 
                        class: "btn btn-secondary",
                        onclick: refresh,
                        "Refresh" 
                    }
                }
            }

            div { class: "statefulsets-grid",
                {statefulsets().iter().map(|statefulset| {
                    rsx! {
                        StatefulSetItem { statefulset: statefulset.clone() }
                    }
                })}
            }
        }
    }
}

