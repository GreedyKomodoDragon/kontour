use dioxus::{logger::tracing, prelude::*};
use k8s_openapi::api::batch::v1::CronJob;
use kube::{api::ListParams, Api, Client};

use crate::components::{NamespaceSelector, SearchInput, CronJobItem};

const CRONJOBS_CSS: Asset = asset!("/assets/styling/cronjobs.css");

#[derive(Clone)]
struct CronJobFetcher {
    client: Client,
    cronjobs: Signal<Vec<CronJob>>,
}

impl CronJobFetcher {
    fn fetch(&self, ns: String, query: String) {
        let client = self.client.clone();
        let mut cronjobs = self.cronjobs.clone();

        tracing::info!("Starting cronjobs fetch...");
        
        spawn(async move {
            let api = if ns == "All" {
                Api::<CronJob>::all(client.clone())
            } else {
                Api::<CronJob>::namespaced(client.clone(), &ns)
            };
            
            match api.list(&ListParams::default()).await {
                Ok(cronjob_list) => {
                    let filtered_cronjobs = if query.is_empty() {
                        cronjob_list.items
                    } else {
                        cronjob_list.items
                            .into_iter()
                            .filter(|cj| {
                                cj.metadata.name.as_ref()
                                    .map(|name| name.to_lowercase().contains(&query.to_lowercase()))
                                    .unwrap_or(false) ||
                                cj.spec.as_ref()
                                    .map(|s| s.schedule.to_lowercase().contains(&query.to_lowercase()))
                                    .unwrap_or(false)
                            })
                            .collect()
                    };
                    
                    cronjobs.set(filtered_cronjobs);
                }
                Err(e) => {
                    tracing::error!("Failed to fetch cronjobs: {:?}", e);
                }
            }
        });
    }
}

#[component]
pub fn CronJobs() -> Element {
    let client = use_context::<Client>();
    let navigate = use_navigator();

    let mut selected_namespace = use_signal(|| "All".to_string());
    let mut search_query = use_signal(String::new);
    let cronjobs = use_signal(|| Vec::<CronJob>::new());

    let fetcher = CronJobFetcher {
        client: client.clone(),
        cronjobs: cronjobs.clone(),
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
        document::Link { rel: "stylesheet", href: CRONJOBS_CSS }
        div { class: "cronjobs-container",
            div { class: "cronjobs-header",
                div { class: "header-left",
                    h1 { "CronJobs" }
                }
                div { class: "header-controls",
                    div { class: "header-controls-left",
                        SearchInput {
                            query: search_query(),
                            on_change: move |q| search_query.set(q)
                        }
                        NamespaceSelector {
                            selected_namespace: selected_namespace(),
                            on_change: move |ns| selected_namespace.set(ns)
                        }
                        span { class: "cronjob-count", "{cronjobs().len()} cronjobs" }
                    }
                    div { class: "header-actions",
                        button { 
                            class: "btn btn-primary",
                            onclick: move |_| {
                                navigate.push("/cronjobs/create");
                            },
                            "Create CronJob" 
                        }
                        button { 
                            class: "btn btn-secondary",
                            onclick: refresh,
                            "Refresh" 
                        }
                    }
                }
            }

            div { class: "cronjobs-grid",
                {cronjobs().iter().map(|cronjob| {
                    rsx! {
                        CronJobItem {
                            key: "{cronjob.metadata.name.clone().unwrap_or_default()}",
                            cronjob: cronjob.clone()
                        }
                    }
                })}
            }
        }
    }
}
