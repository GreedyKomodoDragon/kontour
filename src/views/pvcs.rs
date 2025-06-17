use dioxus::{logger::tracing, prelude::*};
use k8s_openapi::api::core::v1::PersistentVolumeClaim;
use kube::{api::ListParams, Api, Client};

use crate::components::{NamespaceSelector, SearchInput, PvcItem};

const PVCS_CSS: Asset = asset!("/assets/styling/pvcs.css");

#[derive(Clone)]
struct PvcFetcher {
    client: Client,
    pvcs: Signal<Vec<PersistentVolumeClaim>>,
}

impl PvcFetcher {
    fn fetch(&self, ns: String, query: String) {
        let client = self.client.clone();
        let mut pvcs = self.pvcs.clone();

        tracing::info!("Starting PVCs fetch...");
        
        spawn(async move {
            let api = if ns == "All" {
                Api::<PersistentVolumeClaim>::all(client.clone())
            } else {
                Api::<PersistentVolumeClaim>::namespaced(client.clone(), &ns)
            };
            
            match api.list(&ListParams::default()).await {
                Ok(pvc_list) => {
                    let filtered_pvcs = if query.is_empty() {
                        pvc_list.items
                    } else {
                        pvc_list.items
                            .into_iter()
                            .filter(|pvc: &PersistentVolumeClaim| {
                                pvc.metadata.name.as_ref()
                                    .map(|name| name.to_lowercase().contains(&query.to_lowercase()))
                                    .unwrap_or(false) ||
                                pvc.spec.as_ref()
                                    .and_then(|s| s.storage_class_name.as_ref())
                                    .map(|sc| sc.to_lowercase().contains(&query.to_lowercase()))
                                    .unwrap_or(false)
                            })
                            .collect()
                    };
                    
                    pvcs.set(filtered_pvcs);
                }
                Err(e) => {
                    tracing::error!("Failed to fetch PVCs: {:?}", e);
                }
            }
        });
    }
}

#[component]
pub fn Pvcs() -> Element {
    let client_signal = use_context::<Signal<Option<Client>>>();

    let mut selected_namespace = use_signal(|| "All".to_string());
    let mut search_query = use_signal(String::new);
    let pvcs = use_signal(|| Vec::<PersistentVolumeClaim>::new());

    use_effect({
        move || {
            if let Some(client) = &*client_signal.read() {
                let fetcher = PvcFetcher {
                    client: client.clone(),
                    pvcs: pvcs.clone(),
                };
                let ns = selected_namespace();
                let query = search_query();
                fetcher.fetch(ns, query);
            }
        }
    });

    let refresh = {
        move |_| {
            if let Some(client) = &*client_signal.read() {
                let fetcher = PvcFetcher {
                    client: client.clone(),
                    pvcs: pvcs.clone(),
                };
                let ns = selected_namespace();
                let query = search_query();
                fetcher.fetch(ns, query);
            }
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: PVCS_CSS }
        div { class: "pvcs-container",
            div { class: "pvcs-header",
                div { class: "header-left",
                    h1 { "Persistent Volume Claims" }
                    div { class: "header-controls",
                        SearchInput {
                            query: search_query(),
                            on_change: move |q| search_query.set(q)
                        }
                        NamespaceSelector {
                            selected_namespace: selected_namespace(),
                            on_change: move |ns| selected_namespace.set(ns)
                        }
                        span { class: "pvc-count", "{pvcs().len()} PVCs" }
                    }
                }
                div { class: "header-actions",
                    button { 
                        class: "btn btn-secondary",
                        onclick: refresh,
                        "Refresh" 
                    }
                }
            }

            div { class: "pvcs-grid",
                {pvcs().iter().map(|pvc| {
                    rsx! {
                        PvcItem {
                            key: "{pvc.metadata.name.clone().unwrap_or_default()}",
                            pvc: pvc.clone()
                        }
                    }
                })}
            }
        }
    }
}
