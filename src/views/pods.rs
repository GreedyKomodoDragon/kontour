use dioxus::prelude::*;
use k8s_openapi::api::core::v1::Pod;
use kube::{api::ListParams, Api, Client};

use crate::components::PodItem;

const PODS_CSS: Asset = asset!("/assets/styling/pods.css");

#[component]
pub fn Pods() -> Element { // <-- Remove client prop
    let client = use_context::<Client>();

    let selected_status = use_signal(|| "all");
    let selected_namespace = use_signal(|| "all");
    let search_query = use_signal(String::new);

    let mut pods = use_signal(|| Vec::<Pod>::new());


    use_effect(move || {
        let client = client.clone();
        let ns = selected_namespace();
        spawn(async move {
            // let pods_api: Api<Pod> = Api::namespaced(client, namespace);

            // Create an Api for Pod
            let pds: Api<Pod> = Api::all(client);

            // List all pods in the specified namespace
            // let namespace = if ns == "all" { "" } else { ns };
            let params = ListParams::default();
            match pds.list(&params).await {
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
        // Add Tailwind: background color
        div { class: "pods-container", // Test Tailwind bg
            // Add Tailwind: border
            div { class: "pods-header", // Test Tailwind border
                div { class: "header-left",
                    // Add Tailwind: text color
                    h1 { class: "text-yellow-300", "Pods" } // Test Tailwind text color
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
                            option { value: "all", "All Namespaces" }
                            option { value: "default", "default" }
                            option { value: "monitoring", "monitoring" }
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
                    // Add Tailwind: hover effect
                    button { class: "btn btn-primary", "Create Pod" } // Test Tailwind hover
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
