use dioxus::prelude::*;
use k8s_openapi::api::core::v1::Pod;
use kube::{api::ListParams, Api, Client};

use crate::components::PodItem;

const PODS_CSS: Asset = asset!("/assets/styling/pods.css");

#[component]
pub fn Pods() -> Element {
    let client = use_context::<Client>();

    let selected_status = use_signal(|| "all");
    let mut selected_namespace = use_signal(|| "all".to_string());
    let search_query = use_signal(String::new);
    let mut pods = use_signal(|| Vec::<Pod>::new());

    use_effect(move || {
        let client = client.clone();
        let ns = selected_namespace();
        spawn(async move {
            let params = ListParams::default();
            
            match if ns == "all" {
                // List pods across all namespaces
                Api::all(client).list(&params).await
            } else {
                // List pods in specific namespace
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
                        select {
                            class: "namespace-select",
                            value: "{selected_namespace.read()}",
                            onchange: move |evt| {
                                selected_namespace.set(evt.value());
                            },
                            option { value: "all", "All Namespaces" }
                            option { value: "default", "default" }
                            option { value: "monitoring", "monitoring" }
                            option { value: "kube-system", "kube-system" }
                        }
                        select {
                            class: "status-select",
                            value: "{selected_status.read()}",
                            option { value: "all", "All Statuses" }
                            option { value: "Running", "Running" }
                            option { value: "Pending", "Pending" }
                            option { value: "Failed", "Failed" }
                            option { value: "Succeeded", "Succeeded" }
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
