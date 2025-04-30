use dioxus::{logger::tracing, prelude::*};
use k8s_openapi::api::core::v1::Pod;
use kube::{api::ListParams, Api, Client};

use crate::components::{PodItem, NamespaceSelector, StatusSelector, SearchInput};

const PODS_CSS: Asset = asset!("/assets/styling/pods.css");

#[derive(Clone)]
struct PodFetcher {
    client: Client,
    pods: Signal<Vec<Pod>>,
}

impl PodFetcher {
    fn fetch(&self, ns: String, status: String, query: String) {
        let client = self.client.clone();
        let mut pods = self.pods.clone();

        tracing::info!("Fetching pods with status: {} in namespace: {}", status, ns);
        
        spawn(async move {
            let params = if status == "All" {
                ListParams::default()
            } else {
                ListParams::default()
                    .fields(&format!("status.phase={}", status))
            };
            
            match if ns == "All" {
                Api::all(client.clone()).list(&params).await
            } else {
                Api::namespaced(client.clone(), &ns).list(&params).await
            } {
                Ok(pod_list) => {
                    let filtered_pods = if query.is_empty() {
                        pod_list.items
                    } else {
                        pod_list.items
                            .into_iter()
                            .filter(|pod: &Pod| {
                                pod.metadata.name.as_ref()
                                    .map(|name| name.to_lowercase().contains(&query.to_lowercase()))
                                    .unwrap_or(false)
                            })
                            .collect()
                    };
                    pods.set(filtered_pods);
                }
                Err(e) => {
                    println!("Error fetching pods: {:?}", e);
                }
            }
        });
    }
}

#[component]
pub fn Pods() -> Element {
    let client = use_context::<Client>();

    let mut selected_status = use_signal(|| "All".to_string());
    let mut selected_namespace = use_signal(|| "All".to_string());
    let mut search_query = use_signal(String::new);
    let pods = use_signal(|| Vec::<Pod>::new());

    let fetcher = PodFetcher {
        client: client.clone(),
        pods: pods.clone(),
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
        document::Link { rel: "stylesheet", href: PODS_CSS }
        div { class: "pods-container",
            div { class: "pods-header",
                div { class: "header-left",
                    h1 { class: "text-yellow-300", "Pods" }
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
                        span { class: "pod-count", "{pods().len()} pods" }
                    }
                }
                div { class: "header-actions",
                    button { class: "btn btn-primary", "Create Pod" }
                    button { 
                        class: "btn btn-secondary",
                        onclick: refresh,
                        "Refresh" 
                    }
                }
            }

            div { class: "pods-grid",
                {pods().iter().map(|pod| {
                    rsx! {
                        PodItem { pod: pod.clone() }
                    }
                })}
            }
        }
    }
}
