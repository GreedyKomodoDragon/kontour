use dioxus::{logger::tracing, prelude::*};
use k8s_openapi::api::apps::v1::DaemonSet;
use kube::{api::ListParams, Api, Client};

use crate::components::{NamespaceSelector, StatusSelector, SearchInput, DaemonSetItem};

const DAEMONSETS_CSS: Asset = asset!("/assets/styling/daemonsets.css");

#[derive(Clone)]
struct DaemonSetFetcher {
    client: Client,
    daemonsets: Signal<Vec<DaemonSet>>,
}

impl DaemonSetFetcher {
    fn fetch(&self, ns: String, status: String, query: String) {
        let client = self.client.clone();
        let mut daemonsets = self.daemonsets.clone();

        tracing::info!("Starting daemonset fetch...");
        
        spawn(async move {
            let api = if ns == "All" {
                Api::<DaemonSet>::all(client.clone())
            } else {
                Api::<DaemonSet>::namespaced(client.clone(), &ns)
            };
            
            match api.list(&ListParams::default()).await {
                Ok(daemonset_list) => {
                    let mut filtered_daemonsets = if query.is_empty() {
                        daemonset_list.items
                    } else {
                        daemonset_list.items
                            .into_iter()
                            .filter(|ds: &DaemonSet| {
                                ds.metadata.name.as_ref()
                                    .map(|name| name.to_lowercase().contains(&query.to_lowercase()))
                                    .unwrap_or(false)
                            })
                            .collect::<Vec<_>>()
                    };

                    // Filter by status if not "All"
                    if status != "All" {
                        filtered_daemonsets = filtered_daemonsets.into_iter()
                            .filter(|ds| {
                                let desired = ds.status.as_ref().map_or(0, |s| s.desired_number_scheduled);
                                let ready = ds.status.as_ref().map_or(0, |s| s.number_ready);
                                let current = ds.status.as_ref().map_or(0, |s| s.current_number_scheduled);
                                let updated = ds.status.as_ref().map_or(0, |s| s.updated_number_scheduled.unwrap_or(s.current_number_scheduled));

                                match status.as_str() {
                                    "Running" => ready == desired && current == desired && updated == desired,
                                    "Progressing" => ready < desired && ready > 0,
                                    "Not Ready" => ready == 0 && desired > 0,
                                    "No Nodes" => desired == 0,
                                    _ => false
                                }
                            })
                            .collect();
                    }
                    
                    daemonsets.set(filtered_daemonsets);
                }
                Err(e) => {
                    tracing::error!("Failed to fetch daemonsets: {:?}", e);
                }
            }
        });
    }
}

#[component]
pub fn DaemonSets() -> Element {
    let client_signal = use_context::<Signal<Option<Client>>>();
    let navigate = use_navigator();

    let mut selected_status = use_signal(|| "All".to_string());
    let mut selected_namespace = use_signal(|| "All".to_string());
    let mut search_query = use_signal(String::new);
    let daemonsets = use_signal(|| Vec::<DaemonSet>::new());

    // Always call use_effect but handle conditional logic inside
    use_effect({
        move || {
            if let Some(client) = &*client_signal.read() {
                let fetcher = DaemonSetFetcher {
                    client: client.clone(),
                    daemonsets: daemonsets.clone(),
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
                let fetcher = DaemonSetFetcher {
                    client: client.clone(),
                    daemonsets: daemonsets.clone(),
                };
                let ns = selected_namespace();
                let status = selected_status();
                let query = search_query();
                fetcher.fetch(ns, status, query);
            }
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: DAEMONSETS_CSS }
        div { class: "daemonsets-container",
            div { class: "daemonsets-header",
                div { class: "header-left",
                    h1 { "DaemonSets" }
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
                            custom_statuses: Some(vec!["All", "Running", "Progressing", "Not Ready", "No Nodes"])
                        }
                        span { class: "daemonset-count", "{daemonsets().len()} daemonsets" }
                    }
                }
                div { class: "header-actions",
                    button { 
                        class: "btn btn-primary",
                        onclick: move |_| {
                            navigate.push("/daemonsets/create");
                        },
                        "Create DaemonSet" 
                    }
                    button { 
                        class: "btn btn-secondary",
                        onclick: refresh,
                        "Refresh" 
                    }
                }
            }

            div { class: "daemonsets-grid",
                {daemonsets().iter().map(|daemonset| {
                    rsx! {
                        DaemonSetItem { daemonset: daemonset.clone() }
                    }
                })}
            }
        }
    }
}
