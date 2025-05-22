use dioxus::{logger::tracing, prelude::*};
use k8s_openapi::{
    api::core::v1::{Node, Pod},
    apimachinery::pkg::api::resource::Quantity,
};
use kube::{api::ListParams, Api, Client};

use crate::components::{NodeItem, NodeItemProps};

const NODES_CSS: Asset = asset!("/assets/styling/nodes.css");

#[derive(Clone)]
struct NodeInfo {
    node: Node,
    pods: Vec<Pod>,
}

#[derive(Clone)]
struct NodeFetcher {
    client: Client,
    nodes: Signal<Vec<NodeInfo>>,
}

impl NodeFetcher {
    async fn fetch_node_info(client: Client, node_name: &str) -> NodeInfo {
        let pods_api: Api<Pod> = Api::all(client.clone());
        
        let pods = match pods_api.list(&ListParams::default()).await {
            Ok(pod_list) => pod_list.items.into_iter()
                .filter(|pod| pod.spec.as_ref()
                    .and_then(|spec| spec.node_name.as_ref())
                    .map(|name| name == node_name)
                    .unwrap_or(false))
                .collect(),
            Err(e) => {
                tracing::error!("Failed to fetch pods for node {}: {:?}", node_name, e);
                Vec::new()
            }
        };

        NodeInfo {
            node: Node::default(), // Will be filled in later
            pods,
        }
    }

    fn fetch(&self) {
        let client = self.client.clone();
        let mut nodes = self.nodes.clone();

        tracing::info!("Starting node fetch...");

        spawn(async move {
            let api = Api::<Node>::all(client.clone());

            match api.list(&ListParams::default()).await {
                Ok(node_list) => {
                    let mut node_infos = Vec::new();
                    for node in node_list.items {
                        if let Some(name) = &node.metadata.name {
                            let mut info = Self::fetch_node_info(client.clone(), name).await;
                            info.node = node;
                            node_infos.push(info);
                        }
                    }
                    nodes.set(node_infos);
                }
                Err(e) => {
                    tracing::error!("Failed to fetch nodes: {:?}", e);
                }
            }
        });
    }

    fn parse_resource_quantity(quantity: &str) -> f32 {
        if quantity.is_empty() || quantity == "0" {
            return 0.0;
        }

        // Parse CPU values
        if quantity.ends_with('m') {
            return quantity.trim_end_matches('m')
                .parse::<f32>()
                .map(|v| v / 1000.0)
                .unwrap_or(0.0);
        }

        // Parse memory/storage values
        if let Some(value) = quantity.strip_suffix("Ki") {
            return value.parse::<f32>().map(|v| v / (1024.0 * 1024.0)).unwrap_or(0.0);
        }
        if let Some(value) = quantity.strip_suffix("Mi") {
            return value.parse::<f32>().map(|v| v / 1024.0).unwrap_or(0.0);
        }
        if let Some(value) = quantity.strip_suffix("Gi") {
            return value.parse::<f32>().ok().unwrap_or(0.0);
        }

        quantity.parse::<f32>().unwrap_or(0.0)
    }
}

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
    let client = use_context::<Client>();
    let mut selected_node = use_signal(|| String::from("all"));
    let search_query = use_signal(String::new);
    let nodes = use_signal(|| Vec::<NodeInfo>::new());

    let fetcher = NodeFetcher {
        client: client.clone(),
        nodes: nodes.clone(),
    };

    use_effect({
        let fetcher = fetcher.clone();
        move || fetcher.fetch()
    });

    let refresh = {
        let fetcher = fetcher.clone();
        move |_: Event<MouseData>| fetcher.fetch()
    };

    // Convert k8s Node objects to our display format
    let node_data: Vec<NodeData> = nodes()
        .into_iter()
        .map(|node_info| {
            let node = &node_info.node;
            let name = node.metadata.name.clone().unwrap_or_default();
            
            // Determine node type based on labels
            let node_type = if node.metadata.labels.as_ref()
                .and_then(|labels| labels.get("node-role.kubernetes.io/control-plane"))
                .is_some() 
            {
                "master"
            } else {
                "worker"
            };

            // Get status
            let status = node.status.as_ref()
                .and_then(|status| status.conditions.as_ref())
                .and_then(|conditions| conditions.iter()
                    .find(|cond| cond.type_ == "Ready"))
                .map(|ready_cond| ready_cond.status.clone())
                .unwrap_or_else(|| "Unknown".into());

            // Get version
            let kubernetes_version = node.status.as_ref()
                .and_then(|status| status.node_info.as_ref())
                .map(|info| info.kubelet_version.clone())
                .unwrap_or_default();

            // Get OS and architecture
            let os = node.status.as_ref()
                .and_then(|status| status.node_info.as_ref())
                .map(|info| format!("{} {}", info.os_image, info.kernel_version))
                .unwrap_or_default();

            let architecture = node.status.as_ref()
                .and_then(|status| status.node_info.as_ref())
                .map(|info| info.architecture.clone())
                .unwrap_or_default();

            // Get IP
            let ip = node.status.as_ref()
                .and_then(|status| status.addresses.as_ref())
                .and_then(|addresses| addresses.iter()
                    .find(|addr| addr.type_ == "InternalIP"))
                .map(|addr| addr.address.clone())
                .unwrap_or_default();

            // Calculate resource usage
            let binding = std::collections::BTreeMap::new();
            let allocatable = node.status.as_ref()
                .and_then(|status| status.allocatable.as_ref())
                .unwrap_or(&binding);

            let capacity = node.status.as_ref()
                .and_then(|status| status.capacity.as_ref())
                .unwrap_or(&binding);

            // Get pod counts
            let max_pods = capacity.get("pods")
                .map(|q| q.0.parse::<u32>().unwrap_or(0))
                .unwrap_or(0);
            let current_pods = node_info.pods.len() as u32;

            // Calculate CPU usage (as percentage of capacity)
            let cpu_total = NodeFetcher::parse_resource_quantity(&capacity.get("cpu")
                .map(|q| q.0.clone())
                .unwrap_or_else(|| "0".into()));
            let cpu_allocatable = NodeFetcher::parse_resource_quantity(&allocatable.get("cpu")
                .map(|q| q.0.clone())
                .unwrap_or_else(|| "0".into()));
            let cpu_usage = if cpu_total > 0.0 {
                ((cpu_total - cpu_allocatable) / cpu_total * 100.0).max(0.0)
            } else {
                0.0
            };

            // Calculate memory usage
            let memory_total = NodeFetcher::parse_resource_quantity(&capacity.get("memory")
                .map(|q| q.0.clone())
                .unwrap_or_else(|| "0".into()));
            let memory_allocatable = NodeFetcher::parse_resource_quantity(&allocatable.get("memory")
                .map(|q| q.0.clone())
                .unwrap_or_else(|| "0".into()));
            let memory_usage = if memory_total > 0.0 {
                ((memory_total - memory_allocatable) / memory_total * 100.0).max(0.0)
            } else {
                0.0
            };

            // Calculate storage usage
            let storage_total = NodeFetcher::parse_resource_quantity(&capacity.get("ephemeral-storage")
                .map(|q| q.0.clone())
                .unwrap_or_else(|| "0".into()));
            let storage_allocatable = NodeFetcher::parse_resource_quantity(&allocatable.get("ephemeral-storage")
                .map(|q| q.0.clone())
                .unwrap_or_else(|| "0".into()));
            let storage_usage = if storage_total > 0.0 {
                ((storage_total - storage_allocatable) / storage_total * 100.0).max(0.0)
            } else {
                0.0
            };

            NodeData {
                name,
                node_type: node_type.into(),
                status,
                kubernetes_version,
                os,
                architecture,
                ip,
                pods: (current_pods, max_pods),
                cpu_usage,
                memory_usage,
                storage_usage,
            }
        })
        .collect();

    let filtered_nodes: Vec<_> = node_data
        .iter()
        .filter(|node| {
            let type_match = selected_node() == "all" || node.node_type == selected_node();
            let search_match = if search_query().is_empty() {
                true
            } else {
                let query = search_query().to_lowercase();
                node.name.to_lowercase().contains(&query) ||
                node.ip.to_lowercase().contains(&query) ||
                node.os.to_lowercase().contains(&query)
            };
            type_match && search_match
        })
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
                                let value = evt.value().to_string();
                                selected_node.set(value);
                            },
                            option { value: "all", "All Nodes ({nodes.len()})" }
                            option { value: "worker", "Worker Nodes ({node_data.iter().filter(|n| n.node_type == \"worker\").count()})" }
                            option { value: "master", "Master Nodes ({node_data.iter().filter(|n| n.node_type == \"master\").count()})" }
                        }
                        span { class: "node-count", "{filtered_nodes.len()} nodes selected" }
                    }
                }
                div { class: "header-actions",
                    button { class: "btn btn-primary", "Add Node" }
                    button { class: "btn btn-secondary", onclick: refresh, "Refresh" }
                }
            }

            div { class: "nodes-grid",
                {filtered_nodes.iter().map(|node| rsx!(
                    NodeItem {
                        name: node.name.clone(),
                        node_type: node.node_type.clone(),
                        status: node.status.clone(),
                        kubernetes_version: node.kubernetes_version.clone(),
                        os: node.os.clone(),
                        architecture: node.architecture.clone(),
                        ip: node.ip.clone(),
                        pods: node.pods,
                        cpu_usage: node.cpu_usage,
                        memory_usage: node.memory_usage,
                        storage_usage: node.storage_usage,
                    }
                ))}
            }
        }
    }
}
