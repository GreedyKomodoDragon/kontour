use dioxus::prelude::*;
use k8s_openapi::api::core::v1::Pod;
use kube::{api::ListParams, Api, Client};

use crate::components::{PodItem, NamespaceSelector, StatusSelector};

const PODS_CSS: Asset = asset!("/assets/styling/pods.css");

#[component]
pub fn Pods() -> Element {
    let client: Client = use_context::<Client>();

    let mut selected_status = use_signal(|| "All".to_string());
    let mut selected_namespace = use_signal(|| "All".to_string());
    let search_query = use_signal(String::new);
    let mut pods = use_signal(|| Vec::<Pod>::new());

    use_effect(move || {
        let client = client.clone();
        let ns = selected_namespace();
        let status = selected_status();
        spawn(async move {
            let params = if status == "All" {
                ListParams::default()
            } else {
                ListParams::default()
                    .fields(&format!("status.phase={}", status))
            };
            
            match if ns == "All" {
                Api::all(client).list(&params).await
            } else {
                Api::namespaced(client, &ns).list(&params).await
            } {
                Ok(pod_list) => {
                    pods.set(pod_list.items);
                }
                Err(e) => {
                    println!("Error fetching pods: {:?}", e);
                }
            }
        });
    });

    rsx! {
        document::Link { rel: "stylesheet", href: PODS_CSS }
        div { class: "pods-container",
            div { class: "pods-header",
                div { class: "header-left",
                    h1 { class: "text-yellow-300", "Pods" }
                    div { class: "header-controls",
                        div { class: "search-container",
                            input {
                                class: "search-input",
                                r#type: "text",
                                placeholder: "Search pods...",
                                value: "{search_query}"
                            }
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
                    button { class: "btn btn-secondary", "Refresh" }
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
