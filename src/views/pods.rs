use std::collections::BTreeMap;

use dioxus::{logger::tracing, prelude::*};
use k8s_openapi::{api::core::v1::Pod, apimachinery::pkg::api::resource::Quantity};
use kube::{api::ListParams, Api, Client};

const PODS_CSS: Asset = asset!("/assets/styling/pods.css");

#[derive(Clone)]
struct PodData {
    name: String,
    namespace: String,
    status: String,
    phase: String,
    age: String,
    ready_containers: (u32, u32), // (ready, total)
    restart_count: u32,
    ip: String,
    node: String,
    qos_class: String,
    containers: Vec<ContainerData>,
    conditions: Vec<PodCondition>,
    labels: Vec<(String, String)>,
}

#[derive(Clone)]
struct ContainerData {
    name: String,
    image: String,
    status: String,
    restarts: u32,
    cpu_usage: f32,
    memory_usage: String,
    memory_limit: std::option::Option<Quantity>,
}

#[derive(Clone)]
struct PodCondition {
    condition_type: String,
    status: String,
    last_transition: String,
    reason: String,
}

/// Parses a Kubernetes quantity string like "512Mi", "2Gi", "1024Ki" into a float (in MiB)
fn parse_quantity_string(q: &str) -> f32 {
    if let Some(stripped) = q.strip_suffix("Ki") {
        stripped.parse::<f32>().unwrap_or(0.0) / 1024.0
    } else if let Some(stripped) = q.strip_suffix("Mi") {
        stripped.parse::<f32>().unwrap_or(0.0)
    } else if let Some(stripped) = q.strip_suffix("Gi") {
        stripped.parse::<f32>().unwrap_or(0.0) * 1024.0
    } else if let Some(stripped) = q.strip_suffix("Ti") {
        stripped.parse::<f32>().unwrap_or(0.0) * 1024.0 * 1024.0
    } else {
        // Assume it's already a plain number in MiB
        q.parse::<f32>().unwrap_or(0.0)
    }
}


#[component]
pub fn Pods() -> Element { // <-- Remove client prop
    let client = use_context::<Client>();

    let selected_status = use_signal(|| "all");
    let selected_namespace = use_signal(|| "all");
    let search_query = use_signal(String::new);
    let mut expanded_pods = use_signal(|| std::collections::HashSet::<String>::new());

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
                    button { class: "btn btn-primary hover:bg-yellow-600", "Create Pod" } // Test Tailwind hover
                    button { class: "btn btn-secondary", "Refresh" }
                }
            }

            div { class: "pods-grid",
                {pods().iter().map(|pod| {
                    let is_expanded = expanded_pods.read().contains(&pod.metadata.name.clone().unwrap());
                    let pod_data = PodData {
                        name: pod.metadata.name.clone().unwrap(),
                        namespace: pod.metadata.namespace.clone().unwrap(),
                        status: pod.status.clone().unwrap().phase.unwrap_or_default(),
                        phase: pod.status.clone().unwrap().phase.unwrap_or_default(),
                        age: "1h".to_string(), // Placeholder for age
                        ready_containers: {
                            let total = pod.spec.as_ref()
                                .map(|spec| spec.containers.len() as u32)
                                .unwrap_or(0);
                            let ready = pod.status.as_ref()
                                .map(|status| status.container_statuses.as_ref()
                                    .map(|containers| containers.iter()
                                        .filter(|c| c.ready)
                                        .count() as u32)
                                    .unwrap_or(0))
                                .unwrap_or(0);
                            (ready, total)
                        },
                        restart_count: 0, // Placeholder for restart count
                        ip: pod.status.clone().unwrap().pod_ip.unwrap_or_default(),
                        node: pod.spec.clone().unwrap().node_name.unwrap_or_default(),
                        qos_class: "BestEffort".to_string(), // Placeholder for QoS class
                        containers: pod
                            .spec
                            .as_ref()
                            .map(|spec| {
                                spec.containers.iter().map(|c| {
                                    ContainerData {
                                        name: c.name.clone(),
                                        image: c.image.clone().unwrap_or_default(),
                                        status: "Unknown".to_string(), // Placeholder
                                        restarts: 0,                   // Placeholder
                                        cpu_usage: 0.0,                 // Placeholder
                                        memory_usage: "12Mi".to_string(),   // Placeholder
                                        memory_limit: c.resources.as_ref()
                                            .and_then(|res| res.limits.as_ref())
                                            .and_then(|limits| {
                                                limits.get("memory").cloned()
                                            }),
                                    }
                                }).collect()
                            })
                            .unwrap_or_default(),
                        conditions: vec![], // Placeholder for conditions
                        labels: vec![], // Placeholder for labels
                    };
                    rsx! {
                        // Add Tailwind: padding
                        div {
                            key: "{pod_data.name}",
                            class: "pod-card",
                            div { 
                                class: "pod-header",
                                // Optional: Keep header click commented or remove if only button should toggle
                                // onclick: move |_| {
                                //     let mut set = expanded_pods.write();
                                //     if set.contains(&pod_data.name) {
                                //         set.remove(&pod_data.name);
                                //     } else {
                                //         set.insert(pod_data.name.clone());
                                //     }
                                // },
                                div { class: "pod-title",
                                    h3 { "{pod_data.name}" }
                                    span { class: "status-badge status-{pod_data.status.to_lowercase()}", "{pod_data.status}" }
                                }
                                div { class: "pod-controls",
                                    button { 
                                        class: "btn-icon expand-toggle",
                                        onclick: move |evt| { 
                                            evt.stop_propagation();
                                            let mut set = expanded_pods.write(); 
                                            if set.contains(&pod_data.name.clone()) {
                                                set.remove(&pod_data.name.clone());
                                            } else {
                                                set.insert(pod_data.name.clone());
                                            }
                                        },
                                        title: if is_expanded { "Collapse" } else { "Expand" },
                                        if is_expanded { "🔼" } else { "🔽" }
                                    }
                                    button { 
                                        class: "btn-icon", 
                                        onclick: move |evt| evt.stop_propagation(),
                                        title: "View Logs", 
                                        "📄" 
                                    }
                                    button { 
                                        class: "btn-icon", 
                                        onclick: move |evt| evt.stop_propagation(),
                                        title: "Shell", 
                                        "🖥️" 
                                    }
                                    button { 
                                        class: "btn-icon", 
                                        onclick: move |evt| evt.stop_propagation(),
                                        title: "Delete", 
                                        "🗑️" 
                                    }
                                }
                            }

                            {is_expanded.then(|| rsx! {
                                div { class: "pod-details",
                                    div { class: "pod-info",
                                        div { class: "info-group",
                                            div { class: "info-item",
                                                span { class: "info-label", "Namespace" }
                                                span { class: "info-value", "{pod_data.namespace}" }
                                            }
                                            div { class: "info-item",
                                                span { class: "info-label", "Node" }
                                                span { class: "info-value", "{pod_data.node}" }
                                            }
                                            div { class: "info-item",
                                                span { class: "info-label", "IP" }
                                                span { class: "info-value", "{pod_data.ip}" }
                                            }
                                        }
                                        div { class: "info-group",
                                            div { class: "info-item",
                                                span { class: "info-label", "Age" }
                                                span { class: "info-value", "{pod_data.age}" }
                                            }
                                            div { class: "info-item",
                                                span { class: "info-label", "QoS Class" }
                                                span { class: "info-value", "{pod_data.qos_class}" }
                                            }
                                            div { class: "info-item",
                                                span { class: "info-label", "Restarts" }
                                                span { class: "info-value", "{pod_data.restart_count}" }
                                            }
                                        }
                                    }

                                    div { class: "labels-section",
                                        h4 { "Labels" }
                                        div { class: "labels-grid",
                                            {pod_data.labels.iter().map(|(key, value)| rsx! {
                                                div {
                                                    key: "{key}",
                                                    class: "label",
                                                    span { class: "label-key", "{key}" }
                                                    span { class: "label-value", "{value}" }
                                                }
                                            })}
                                        }
                                    }

                                    div { class: "containers-section",
                                        h4 { "Containers ({pod_data.ready_containers.0}/{pod_data.ready_containers.1})" }
                                        div { class: "containers-grid",
                                            {pod_data.containers.iter().map(|container| {
                                                // Calculate memory usage percent safely
                                                // log out the memory usage and limit
                                                // Calculate memory usage percent safely
                                                tracing::debug!(
                                                    "Memory Usage: {}, Memory Limit: {}",
                                                    container.memory_usage,
                                                    container.memory_limit.as_ref().map(|q| q.0.clone()).unwrap_or_default()
                                                );
                                                

                                                let memory_usage = parse_quantity_string(&container.memory_usage);
                                                
                                                let memory_limit = container.memory_limit
                                                    .as_ref()
                                                    .map(|q| parse_quantity_string(&q.0))
                                                    .unwrap_or(0.0);
                                                
                                                let percent = if memory_limit > 0.0 {
                                                    (memory_usage / memory_limit) * 100.0
                                                } else {
                                                    0.0
                                                };
                                                
                                                tracing::debug!("Memory Usage Percent: {}", percent);

                                                // Extract limits and requests for display
                                                let (limits_str, requests_str) = pod
                                                    .spec
                                                    .as_ref()
                                                    .and_then(|spec| {
                                                        spec.containers.iter().find(|c| c.name == container.name)
                                                    })
                                                    .map(|c| {
                                                        let limits = c.resources.as_ref()
                                                            .and_then(|r| r.limits.as_ref())
                                                            .map(|l| {
                                                                l.iter()
                                                                    .map(|(k, v)| format!("{}: {}", k, v.0))
                                                                    .collect::<Vec<_>>()
                                                                    .join(", ")
                                                            })
                                                            .unwrap_or_else(|| "None".to_string());
                                                        let requests = c.resources.as_ref()
                                                            .and_then(|r| r.requests.as_ref())
                                                            .map(|r| {
                                                                r.iter()
                                                                    .map(|(k, v)| format!("{}: {}", k, v.0))
                                                                    .collect::<Vec<_>>()
                                                                    .join(", ")
                                                            })
                                                            .unwrap_or_else(|| "None".to_string());
                                                        (limits, requests)
                                                    })
                                                    .unwrap_or_else(|| ("None".to_string(), "None".to_string()));

                                                rsx! {
                                                    div {
                                                        key: "{container.name}",
                                                        class: "container-card",
                                                        div { class: "container-header",
                                                            div { class: "container-title",
                                                                h5 { "{container.name}" }
                                                                span { class: "container-image", "{container.image}" }
                                                            }
                                                            span { class: "status-badge status-{container.status.to_lowercase()}", "{container.status}" }
                                                        }
                                                        div { class: "resource-metrics",
                                                            div { class: "metric",
                                                                span { class: "metric-label", "CPU" }
                                                                div { class: "progress-bar",
                                                                    div {
                                                                        class: "progress-fill",
                                                                        style: "width: {container.cpu_usage * 100.0}%"
                                                                    }
                                                                }
                                                                span { class: "metric-value", "{container.cpu_usage * 100.0:.1}%" }
                                                            }
                                                            div { class: "metric",
                                                                span { class: "metric-label", "Memory" }
                                                                div { class: "progress-bar",
                                                                    div {
                                                                        class: "progress-fill",
                                                                        style: "width: {percent}%"
                                                                    }
                                                                }
                                                                span { class: "metric-value", "{percent}%" }
                                                            }
                                                        }
                                                        div { class: "container-limits-requests",
                                                            div { class: "limits",
                                                                span { class: "limits-label", "Resource Limits:" }
                                                                span { class: "limits-value", "{limits_str}" }
                                                            }
                                                            div { class: "requests",
                                                                span { class: "requests-label", "Resource Requests:" }
                                                                span { class: "requests-value", "{requests_str}" }
                                                            }
                                                        }
                                                    }
                                                }
                                            })}
                                        }
                                    }

                                    div { class: "conditions-section",
                                        h4 { "Conditions" }
                                        div { class: "conditions-grid",
                                            {pod_data.conditions.iter().map(|condition| rsx! {
                                                div {
                                                    key: "{condition.condition_type}",
                                                    class: "condition",
                                                    div { class: "condition-info",
                                                        span { class: "condition-type", "{condition.condition_type}" }
                                                        span { class: "condition-status status-{condition.status.to_lowercase()}", "{condition.status}" }
                                                    }
                                                    div { class: "condition-details",
                                                        span { class: "condition-reason", "{condition.reason}" }
                                                        span { class: "condition-time", "{condition.last_transition}" }
                                                    }
                                                }
                                            })}
                                        }
                                    }
                                }
                            })}
                        }
                    }
                })}
            }
        }
    }
}
