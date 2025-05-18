use dioxus::{logger::tracing, prelude::*};
use k8s_openapi::api::core::v1::ConfigMap;
use kube::{api::ListParams, Api, Client};

use crate::components::{NamespaceSelector, SearchInput, ConfigMapItem};

const CONFIGMAPS_CSS: Asset = asset!("/assets/styling/configmaps.css");

#[derive(Clone)]
struct ConfigMapFetcher {
    client: Client,
    configmaps: Signal<Vec<ConfigMap>>,
}

impl ConfigMapFetcher {
    fn fetch(&self, ns: String, query: String) {
        let client = self.client.clone();
        let mut configmaps = self.configmaps.clone();

        tracing::info!("Starting configmaps fetch...");
        
        spawn(async move {
            let api = if ns == "All" {
                Api::<ConfigMap>::all(client.clone())
            } else {
                Api::<ConfigMap>::namespaced(client.clone(), &ns)
            };
            
            match api.list(&ListParams::default()).await {
                Ok(configmap_list) => {
                    let filtered_configmaps = if query.is_empty() {
                        configmap_list.items
                    } else {
                        configmap_list.items
                            .into_iter()
                            .filter(|cm: &ConfigMap| {
                                let name_match = cm.metadata.name.as_ref()
                                    .map(|name| name.to_lowercase().contains(&query.to_lowercase()))
                                    .unwrap_or(false);
                                
                                let namespace_match = cm.metadata.namespace.as_ref()
                                    .map(|ns| ns.to_lowercase().contains(&query.to_lowercase()))
                                    .unwrap_or(false);
                                
                                let data_match = cm.data.as_ref()
                                    .map(|data| data.iter().any(|(k, v)| {
                                        k.to_lowercase().contains(&query.to_lowercase()) ||
                                        v.to_lowercase().contains(&query.to_lowercase())
                                    }))
                                    .unwrap_or(false);
                                
                                name_match || namespace_match || data_match
                            })
                            .collect()
                    };
                    
                    configmaps.set(filtered_configmaps);
                }
                Err(e) => {
                    tracing::error!("Failed to fetch configmaps: {:?}", e);
                }
            }
        });
    }
}

#[component]
pub fn ConfigMaps() -> Element {
    let client = use_context::<Client>();
    let navigate = use_navigator();

    let mut selected_namespace = use_signal(|| "All".to_string());
    let mut search_query = use_signal(String::new);
    let configmaps = use_signal(|| Vec::<ConfigMap>::new());

    let fetcher = ConfigMapFetcher {
        client: client.clone(),
        configmaps: configmaps.clone(),
    };

    use_effect({
        let fetcher = fetcher.clone();
        move || {
            let ns = selected_namespace();
            let query = search_query();
            fetcher.fetch(ns, query);
        }
    });

    let refresh = {
        let fetcher = fetcher.clone();
        move |_| {
            let ns = selected_namespace();
            let query = search_query();
            fetcher.fetch(ns, query);
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: CONFIGMAPS_CSS }
        div { class: "configmaps-container",
            div { class: "configmaps-header",
                div { class: "header-left",
                    h1 { "Config Maps" }
                    div { class: "header-controls",
                        SearchInput {
                            query: search_query(),
                            on_change: move |q| search_query.set(q)
                        }
                        NamespaceSelector {
                            selected_namespace: selected_namespace(),
                            on_change: move |ns| selected_namespace.set(ns)
                        }
                        span { class: "configmap-count", "{configmaps().len()} ConfigMaps" }
                    }
                }
                div { class: "header-actions",
                    button { 
                        class: "btn btn-primary",
                        onclick: move |_| {
                            navigate.push("/configmaps/create");
                        },
                        "Create ConfigMap" 
                    }
                    button { 
                        class: "btn btn-secondary",
                        onclick: refresh,
                        "Refresh" 
                    }
                }
            }

            div { class: "configmaps-grid",
                {configmaps.read().iter().map(|cm| {
                    let key = format!("{}-{}", 
                        cm.metadata.namespace.clone().unwrap_or_default(),
                        cm.metadata.name.clone().unwrap_or_default()
                    );
                    rsx! {
                        ConfigMapItem {
                            key: "{key}",
                            configmap: cm.clone()
                        }
                    }
                })}
            }
        }
    }
}
