use dioxus::prelude::*;

const NODES_CSS: Asset = asset!("/assets/styling/nodes.css");

#[derive(Clone)]
struct NodeData {
    name: String,
    node_type: String,
    status: String,
    kubernetes_version: String,
    os: String,
    architecture: String,
    ip: String,
    pods: (u32, u32),
    cpu_usage: f32,
    memory_usage: f32,
    storage_usage: f32,
}

#[component]
pub fn Nodes() -> Element {
    let selected_node = use_signal(|| "all");
    let search_query = use_signal(String::new);

    let nodes = vec![
        NodeData {
            name: "master-1".into(),
            node_type: "master".into(),
            status: "Ready".into(),
            kubernetes_version: "v1.28.1".into(),
            os: "Ubuntu 22.04.3 LTS".into(),
            architecture: "amd64".into(),
            ip: "10.0.1.10".into(),
            pods: (8, 20),
            cpu_usage: 45.0,
            memory_usage: 62.0,
            storage_usage: 35.0,
        },
        NodeData {
            name: "worker-1".into(),
            node_type: "worker".into(),
            status: "Ready".into(),
            kubernetes_version: "v1.28.1".into(),
            os: "Ubuntu 22.04.3 LTS".into(),
            architecture: "amd64".into(),
            ip: "10.0.1.11".into(),
            pods: (12, 30),
            cpu_usage: 65.0,
            memory_usage: 78.0,
            storage_usage: 45.0,
        },
        NodeData {
            name: "worker-2".into(),
            node_type: "worker".into(),
            status: "Ready".into(),
            kubernetes_version: "v1.28.1".into(),
            os: "Ubuntu 22.04.3 LTS".into(),
            architecture: "amd64".into(),
            ip: "10.0.1.12".into(),
            pods: (15, 30),
            cpu_usage: 72.0,
            memory_usage: 84.0,
            storage_usage: 55.0,
        },
        NodeData {
            name: "worker-3".into(),
            node_type: "worker".into(),
            status: "Ready".into(),
            kubernetes_version: "v1.28.1".into(),
            os: "Ubuntu 22.04.3 LTS".into(),
            architecture: "amd64".into(),
            ip: "10.0.1.13".into(),
            pods: (10, 30),
            cpu_usage: 58.0,
            memory_usage: 71.0,
            storage_usage: 40.0,
        },
    ];

    let filtered_nodes: Vec<_> = nodes
        .iter()
        .filter(|&node| selected_node() == "all" || node.node_type == selected_node())
        .collect();


    rsx! {
        document::Link { rel: "stylesheet", href: NODES_CSS }
        
        div { class: "nodes-container",
            div { class: "nodes-header",
                div { class: "header-left",
                    h1 { "Nodes" }
                    div { class: "header-controls",
                        div { class: "search-container",
                            input {
                                class: "search-input",
                                r#type: "text",
                                placeholder: "Search nodes...",
                                value: "{search_query}",
                            }
                        }
                        select {
                            class: "node-select",
                            value: "{selected_node.read()}",
                            onchange: move |evt| {
                                // selected_node.set(evt.value.clone());
                            },
                            option { value: "all", "All Nodes ({nodes.len()})" }
                            option { value: "worker", "Worker Nodes (3)" }
                            option { value: "master", "Master Node (1)" }
                        }
                        span { class: "node-count", "{filtered_nodes.len()} nodes selected" }
                    }
                }
                div { class: "header-actions",
                    button { class: "btn btn-primary", "Add Node" }
                    button { class: "btn btn-secondary", "Refresh" }
                }
            }

            div { class: "nodes-grid",
                {
                    filtered_nodes.iter().map(|node| {
                    rsx! {
                        div {
                            key: "{node.name}",
                            class: "node-card",
                            div { class: "node-header",
                                div { class: "node-title",
                                    h3 { "{node.name}" }
                                    span { class: "status-badge status-healthy", "{node.status}" }
                                }
                                div { class: "node-controls",
                                    button { class: "btn-icon", title: "Cordon", "🔒" }
                                    button { class: "btn-icon", title: "Drain", "⭕" }
                                    button { class: "btn-icon", title: "Delete", "🗑️" }
                                }
                            }

                            div { class: "resource-metrics",
                                div { class: "metric",
                                    span { class: "metric-label", "CPU" }
                                    div { class: "progress-bar",
                                        div {
                                            class: "progress-fill",
                                            style: "width: {node.cpu_usage}%"
                                        }
                                    }
                                    span { class: "metric-value", "{node.cpu_usage}%" }
                                }
                                div { class: "metric",
                                    span { class: "metric-label", "Memory" }
                                    div { class: "progress-bar",
                                        div {
                                            class: "progress-fill",
                                            style: "width: {node.memory_usage}%"
                                        }
                                    }
                                    span { class: "metric-value", "{node.memory_usage}%" }
                                }
                                div { class: "metric",
                                    span { class: "metric-label", "Storage" }
                                    div { class: "progress-bar",
                                        div {
                                            class: "progress-fill",
                                            style: "width: {node.storage_usage}%"
                                        }
                                    }
                                    span { class: "metric-value", "{node.storage_usage}%" }
                                }
                            }

                            div { class: "node-info",
                                div { class: "info-group",
                                    div { class: "info-item",
                                        span { class: "info-label", "Kubernetes Version" }
                                        span { class: "info-value", "{node.kubernetes_version}" }
                                    }
                                    div { class: "info-item",
                                        span { class: "info-label", "OS" }
                                        span { class: "info-value", "{node.os}" }
                                    }
                                    div { class: "info-item",
                                        span { class: "info-label", "Architecture" }
                                        span { class: "info-value", "{node.architecture}" }
                                    }
                                }
                                div { class: "info-group",
                                    div { class: "info-item",
                                        span { class: "info-label", "Internal IP" }
                                        span { class: "info-value", "{node.ip}" }
                                    }
                                    div { class: "info-item",
                                        span { class: "info-label", "Pods" }
                                        span { class: "info-value", "{node.pods.0}/{node.pods.1}" }
                                    }
                                }
                            }

                            div { class: "node-conditions",
                                h4 { "Node Conditions" }
                                div { class: "conditions-list",
                                    div { class: "condition status-healthy",
                                        span { class: "condition-type", "Ready" }
                                        span { class: "condition-status", "True" }
                                    }
                                    div { class: "condition status-healthy",
                                        span { class: "condition-type", "MemoryPressure" }
                                        span { class: "condition-status", "False" }
                                    }
                                    div { class: "condition status-healthy",
                                        span { class: "condition-type", "DiskPressure" }
                                        span { class: "condition-status", "False" }
                                    }
                                    div { class: "condition status-healthy",
                                        span { class: "condition-type", "PIDPressure" }
                                        span { class: "condition-status", "False" }
                                    }
                                }
                            }
                        }
                    }
                })
                }
            }
        }
    }
}
