use dioxus::{logger::tracing, prelude::*};
use k8s_openapi::api::networking::v1::Ingress;
use kube::{api::ListParams, Api, Client};

use crate::components::{NamespaceSelector, SearchInput, IngressItem};

const INGRESSES_CSS: Asset = asset!("/assets/styling/ingresses.css");

#[derive(Clone)]
struct IngressFetcher {
    client: Client,
    ingresses: Signal<Vec<Ingress>>,
}

impl IngressFetcher {
    fn fetch(&self, ns: String, query: String) {
        let client = self.client.clone();
        let mut ingresses = self.ingresses.clone();

        tracing::info!("Starting ingresses fetch...");
        
        spawn(async move {
            let api = if ns == "All" {
                Api::<Ingress>::all(client.clone())
            } else {
                Api::<Ingress>::namespaced(client.clone(), &ns)
            };
            
            match api.list(&ListParams::default()).await {
                Ok(ingress_list) => {
                    let filtered_ingresses = if query.is_empty() {
                        ingress_list.items
                    } else {
                        ingress_list.items
                            .into_iter()
                            .filter(|ing: &Ingress| {
                                ing.metadata.name.as_ref()
                                    .map(|name| name.to_lowercase().contains(&query.to_lowercase()))
                                    .unwrap_or(false) ||
                                ing.spec.as_ref()
                                    .and_then(|s| s.rules.as_ref())
                                    .map(|rules| rules.iter().any(|r| {
                                        r.host.as_ref()
                                            .map(|h| h.to_lowercase().contains(&query.to_lowercase()))
                                            .unwrap_or(false)
                                    }))
                                    .unwrap_or(false)
                            })
                            .collect()
                    };
                    
                    ingresses.set(filtered_ingresses);
                }
                Err(e) => {
                    tracing::error!("Failed to fetch ingresses: {:?}", e);
                }
            }
        });
    }
}

#[component]
pub fn Ingresses() -> Element {
    let client = use_context::<Client>();
    let navigate = use_navigator();

    let mut selected_namespace = use_signal(|| "All".to_string());
    let mut search_query = use_signal(String::new);
    let ingresses = use_signal(|| Vec::<Ingress>::new());

    let fetcher = IngressFetcher {
        client: client.clone(),
        ingresses: ingresses.clone(),
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
        document::Link { rel: "stylesheet", href: INGRESSES_CSS }
        div { class: "ingresses-container",
            div { class: "ingresses-header",
                div { class: "header-left",
                    h1 { "Ingresses" }
                    div { class: "header-controls",
                        SearchInput {
                            query: search_query(),
                            on_change: move |q| search_query.set(q)
                        }
                        NamespaceSelector {
                            selected_namespace: selected_namespace(),
                            on_change: move |ns| selected_namespace.set(ns)
                        }
                        span { class: "ingress-count", "{ingresses().len()} ingresses" }
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

            div { class: "ingresses-grid",
                {ingresses().iter().map(|ingress| {
                    rsx! {
                        IngressItem {
                            key: "{ingress.metadata.name.clone().unwrap_or_default()}",
                            ingress: ingress.clone()
                        }
                    }
                })}
            }
        }
    }
}
