use dioxus::{logger::tracing, prelude::*};
use k8s_openapi::api::core::v1::Service;
use kube::{api::ListParams, Api, Client};
use std::collections::HashSet;

use crate::components::{NamespaceSelector, StatusSelector, SearchInput, ServiceItem};

const SERVICES_CSS: Asset = asset!("/assets/styling/services.css");

#[derive(Clone)]
struct ServiceFetcher {
    client: Client,
    services: Signal<Vec<Service>>,
}

impl ServiceFetcher {
    fn fetch(&self, ns: String, service_type: String, query: String) {
        let client = self.client.clone();
        let mut services = self.services.clone();

        tracing::info!("Starting services fetch...");
        
        spawn(async move {
            let api = if ns == "All" {
                Api::<Service>::all(client.clone())
            } else {
                Api::<Service>::namespaced(client.clone(), &ns)
            };
            
            match api.list(&ListParams::default()).await {
                Ok(service_list) => {
                    let mut filtered_services = if query.is_empty() {
                        service_list.items
                    } else {
                        service_list.items
                            .into_iter()
                            .filter(|svc: &Service| {
                                svc.metadata.name.as_ref()
                                    .map(|name| name.to_lowercase().contains(&query.to_lowercase()))
                                    .unwrap_or(false)
                            })
                            .collect::<Vec<_>>()
                    };

                    // Filter by type if not "All"
                    if service_type != "All" {
                        filtered_services = filtered_services.into_iter()
                            .filter(|svc| {
                                svc.spec.as_ref()
                                    .and_then(|s| s.type_.as_ref())
                                    .map(|t| t == &service_type)
                                    .unwrap_or(false)
                            })
                            .collect();
                    }
                    
                    services.set(filtered_services);
                }
                Err(e) => {
                    tracing::error!("Failed to fetch services: {:?}", e);
                }
            }
        });
    }
}

#[component]
pub fn Services() -> Element {
    let client = use_context::<Client>();
    let navigate = use_navigator();

    let mut selected_namespace = use_signal(|| "All".to_string());
    let mut selected_type = use_signal(|| "All".to_string());
    let mut search_query = use_signal(String::new);
    let services = use_signal(|| Vec::<Service>::new());

    let fetcher = ServiceFetcher {
        client: client.clone(),
        services: services.clone(),
    };

    use_effect({
        let fetcher = fetcher.clone();
        move || {
            let ns = selected_namespace();
            let type_ = selected_type();
            let query = search_query();
            fetcher.fetch(ns, type_, query);
        }
    });

    let refresh = {
        let fetcher = fetcher.clone();
        move |_| {
            let ns = selected_namespace();
            let type_ = selected_type();
            let query = search_query();
            fetcher.fetch(ns, type_, query);
        }
    };

    // --- Unique Service Types for Filter ---
    let service_types = use_memo(move || {
        let mut types: Vec<String> = services.read().iter()
            .filter_map(|s| s.spec.as_ref()?.type_.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        types.sort();
        types
    });

    rsx! {
        document::Link { rel: "stylesheet", href: SERVICES_CSS }
        div { class: "services-container",
            div { class: "services-header",
                div { class: "header-left",
                    h1 { "Services" }
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
                            selected_status: selected_type(),
                            on_change: move |t| selected_type.set(t),
                            custom_statuses: Some(vec!["All", "ClusterIP", "NodePort", "LoadBalancer", "ExternalName"])
                        }
                        span { class: "service-count", "{services().len()} services" }
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

            div { class: "services-grid",
                {services().iter().map(|service| {
                    rsx! {
                        ServiceItem {
                            key: "{service.metadata.name.clone().unwrap_or_default()}",
                            service: service.clone()
                        }
                    }
                })}
            }
        }
    }
}
