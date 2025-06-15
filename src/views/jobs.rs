use dioxus::{logger::tracing, prelude::*};
use k8s_openapi::api::batch::v1::Job;
use kube::{api::ListParams, Api, Client};

use crate::components::{NamespaceSelector, SearchInput, JobItem};

const JOBS_CSS: Asset = asset!("/assets/styling/jobs.css");

#[derive(Clone)]
struct JobFetcher {
    client: Client,
    jobs: Signal<Vec<Job>>,
}

impl JobFetcher {
    fn fetch(&self, ns: String, query: String) {
        let client = self.client.clone();
        let mut jobs = self.jobs.clone();

        tracing::info!("Starting jobs fetch...");
        
        spawn(async move {
            let api = if ns == "All" {
                Api::<Job>::all(client.clone())
            } else {
                Api::<Job>::namespaced(client.clone(), &ns)
            };
            
            match api.list(&ListParams::default()).await {
                Ok(job_list) => {
                    let filtered_jobs = if query.is_empty() {
                        job_list.items
                    } else {
                        job_list.items
                            .into_iter()
                            .filter(|j: &Job| {
                                let name_match = j.metadata.name.as_ref()
                                    .map(|name| name.to_lowercase().contains(&query.to_lowercase()))
                                    .unwrap_or(false);
                                
                                let namespace_match = j.metadata.namespace.as_ref()
                                    .map(|ns| ns.to_lowercase().contains(&query.to_lowercase()))
                                    .unwrap_or(false);
                                
                                name_match || namespace_match
                            })
                            .collect()
                    };
                    
                    jobs.set(filtered_jobs);
                }
                Err(e) => {
                    tracing::error!("Failed to fetch jobs: {:?}", e);
                }
            }
        });
    }
}

#[component]
pub fn Jobs() -> Element {
    let client_signal = use_context::<Signal<Option<Client>>>();
    let mut selected_namespace = use_signal(|| "All".to_string());
    let mut search_query = use_signal(String::new);
    let jobs = use_signal(|| Vec::<Job>::new());

    use_effect({
        move || {
            if let Some(client) = &*client_signal.read() {
                let fetcher = JobFetcher {
                    client: client.clone(),
                    jobs: jobs.clone(),
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
                let fetcher = JobFetcher {
                    client: client.clone(),
                    jobs: jobs.clone(),
                };
                let ns = selected_namespace();
                let query = search_query();
                fetcher.fetch(ns, query);
            }
        }
    };

    rsx! {
        document::Link { rel: "stylesheet", href: JOBS_CSS }
        div { class: "jobs-container",
            div { class: "jobs-header",
                div { class: "header-left",
                    h1 { "Jobs" }
                    div { class: "header-controls",
                        SearchInput {
                            query: search_query(),
                            on_change: move |q| search_query.set(q)
                        }
                        NamespaceSelector {
                            selected_namespace: selected_namespace(),
                            on_change: move |ns| selected_namespace.set(ns)
                        }
                        span { class: "job-count", "{jobs().len()} Jobs" }
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

            div { class: "jobs-grid",
                {jobs.read().iter().map(|j| {
                    let key = format!("{}-{}", 
                        j.metadata.namespace.clone().unwrap_or_default(),
                        j.metadata.name.clone().unwrap_or_default()
                    );
                    rsx! {
                        JobItem {
                            key: "{key}",
                            job: j.clone()
                        }
                    }
                })}
            }
        }
    }
}
