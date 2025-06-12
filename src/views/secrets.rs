use dioxus::{logger::tracing, prelude::*};
use k8s_openapi::api::core::v1::Secret;
use kube::{api::ListParams, Api, Client};

use crate::components::{NamespaceSelector, SearchInput, SecretItem};

const SECRETS_CSS: Asset = asset!("/assets/styling/secrets.css");

#[derive(Clone)]
struct SecretFetcher {
    client: Client,
    secrets: Signal<Vec<Secret>>,
}

impl SecretFetcher {
    fn fetch(&self, ns: String, query: String) {
        let client = self.client.clone();
        let mut secrets = self.secrets.clone();

        tracing::info!("Starting secrets fetch...");
        
        spawn(async move {
            let api = if ns == "All" {
                Api::<Secret>::all(client.clone())
            } else {
                Api::<Secret>::namespaced(client.clone(), &ns)
            };
            
            match api.list(&ListParams::default()).await {
                Ok(secret_list) => {
                    let filtered_secrets = if query.is_empty() {
                        secret_list.items
                    } else {
                        secret_list.items
                            .into_iter()
                            .filter(|s: &Secret| {
                                let name_match = s.metadata.name.as_ref()
                                    .map(|name| name.to_lowercase().contains(&query.to_lowercase()))
                                    .unwrap_or(false);
                                
                                let namespace_match = s.metadata.namespace.as_ref()
                                    .map(|ns| ns.to_lowercase().contains(&query.to_lowercase()))
                                    .unwrap_or(false);
                                
                                let type_match = s.type_.as_ref()
                                    .map(|t| t.to_lowercase().contains(&query.to_lowercase()))
                                    .unwrap_or(false);

                                let key_match = s.data.as_ref()
                                    .map(|data| data.keys().any(|k| k.to_lowercase().contains(&query.to_lowercase())))
                                    .unwrap_or(false);
                                
                                name_match || namespace_match || type_match || key_match
                            })
                            .collect()
                    };
                    
                    secrets.set(filtered_secrets);
                }
                Err(e) => {
                    tracing::error!("Failed to fetch secrets: {:?}", e);
                }
            }
        });
    }
}

#[component]
pub fn Secrets() -> Element {
    let client = use_context::<Client>();
    let navigate = use_navigator();

    let mut selected_namespace = use_signal(|| "All".to_string());
    let mut search_query = use_signal(String::new);
    let secrets = use_signal(|| Vec::<Secret>::new());

    let fetcher = SecretFetcher {
        client: client.clone(),
        secrets: secrets.clone(),
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
        document::Link { rel: "stylesheet", href: SECRETS_CSS }
        div { class: "secrets-container",
            div { class: "secrets-header",
                div { class: "header-left",
                    h1 { "Secrets" }
                    div { class: "header-controls",
                        SearchInput {
                            query: search_query(),
                            on_change: move |q| search_query.set(q)
                        }
                        NamespaceSelector {
                            selected_namespace: selected_namespace(),
                            on_change: move |ns| selected_namespace.set(ns)
                        }
                        span { class: "secret-count", "{secrets().len()} Secrets" }
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

            div { class: "secrets-grid",
                {secrets.read().iter().map(|s| {
                    let key = format!("{}-{}", 
                        s.metadata.namespace.clone().unwrap_or_default(),
                        s.metadata.name.clone().unwrap_or_default()
                    );
                    rsx! {
                        SecretItem {
                            key: "{key}",
                            secret: s.clone()
                        }
                    }
                })}
            }
        }
    }
}
