use dioxus::{logger::tracing, prelude::*};
use k8s_openapi::{api::core::v1::{Node, Pod}, apimachinery::pkg::api::resource::Quantity};
use kube::{api::{ListParams, Api}, Client};
use std::collections::{BTreeMap, HashMap};

use crate::k8s::{fetch_node_metrics, parse_resource_quantity};

use crate::components::NodeItem;

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
    async fn fetch_node_info(client: Client, node: Node) -> NodeInfo {
        let pods_api: Api<Pod> = Api::all(client.clone());
        let node_name = node.metadata.name.as_deref().unwrap_or_default();
        
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
            node,
            pods,
        }
    }

    fn fetch(&self) {
        let client = self.client.clone();
        let mut nodes = self.nodes.clone();

        tracing::info!("Starting node fetch...");

        spawn(async move {
            let nodes_api = Api::<Node>::all(client.clone());
            
            // Fetch both nodes and metrics in parallel
            let node_list = nodes_api.list(&ListParams::default()).await;

            match node_list {
                Ok(node_list) => {
                    let mut node_infos = Vec::new();
                    for node in node_list.items {
                        let info = Self::fetch_node_info(client.clone(), node).await;
                        node_infos.push(info);
                    }
                    nodes.set(node_infos);
                }
                Err(e) => {
                    tracing::error!("Failed to fetch nodes: {:?}", e);
                }
            }
        });
    }

    // Moved to k8s::node_metrics::parse_resource_quantity
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
    let client_signal = use_context::<Signal<Option<Client>>>();
    let mut selected_node = use_signal(|| String::from("all"));
    let search_query = use_signal(String::new);
    let nodes = use_signal(|| Vec::<NodeInfo>::new());

    use_effect({
        move || {
            if let Some(client) = &*client_signal.read() {
                let fetcher = NodeFetcher {
                    client: client.clone(),
                    nodes: nodes.clone(),
                };
                fetcher.fetch();
            }
        }
    });

    let refresh = {
        move |_: Event<MouseData>| {
            if let Some(client) = &*client_signal.read() {
                let fetcher = NodeFetcher {
                    client: client.clone(),
                    nodes: nodes.clone(),
                };
                fetcher.fetch();
            }
        }
    };

    // Use a signal to track metrics and fetch them asynchronously
    let metrics_map = use_signal(|| HashMap::<String, crate::k8s::node_metrics::NodeMetrics>::new());

    // Fetch metrics asynchronously when client changes
    use_effect({
        let mut metrics_map = metrics_map.clone();
        move || {
            if let Some(client) = &*client_signal.read() {
                let client = client.clone();
                spawn(async move {
                    let metrics = fetch_node_metrics(&client).await;
                    metrics_map.set(metrics);
                });
            } else {
                metrics_map.set(HashMap::new());
            }
        }
    });

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
            let binding = std::collections::BTreeMap::<String, Quantity>::new();
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

            // Use the pre-fetched metrics
            let binding = BTreeMap::new();
            let metrics_map_value = metrics_map();
            let metrics = metrics_map_value
                .get(&name)
                .map(|m| &m.usage)
                .unwrap_or(&binding);

            // Calculate CPU usage
            let cpu_total = parse_resource_quantity(&capacity.get("cpu")
                .map(|q| q.0.clone())
                .unwrap_or_else(|| "0".into()));

            let cpu_used = parse_resource_quantity(&metrics.get("cpu")
                .map(|q| q.0.clone())
                .unwrap_or_else(|| "0".into()));

            let cpu_usage = if cpu_total > 0.0 {
                ((cpu_used / cpu_total * 100.0).min(100.0) * 100.0).round() / 100.0
            } else {
                0.0
            };

            // Calculate memory usage
            let memory_total = parse_resource_quantity(&capacity.get("memory")
                .map(|q| q.0.clone())
                .unwrap_or_else(|| "0".into()));
            let memory_used = parse_resource_quantity(&metrics.get("memory")
                .map(|q| q.0.clone())
                .unwrap_or_else(|| "0".into()));
            let memory_usage = if memory_total > 0.0 {
                ((memory_used / memory_total * 100.0).min(100.0) * 100.0).round() / 100.0
            } else {
                0.0
            };

            // Calculate storage usage (still based on capacity since it's not in metrics)
            let storage_total = parse_resource_quantity(&capacity.get("ephemeral-storage")
                .map(|q| q.0.clone())
                .unwrap_or_else(|| "0".into()));
            let storage_used = if let Some(fs) = metrics.get("ephemeral-storage") {
                parse_resource_quantity(&fs.0)
            } else {
                // Fallback to capacity-allocatable if metrics aren't available
                let storage_allocatable = parse_resource_quantity(&allocatable.get("ephemeral-storage")
                    .map(|q| q.0.clone())
                    .unwrap_or_else(|| "0".into()));
                storage_total - storage_allocatable
            };
            let storage_usage = if storage_total > 0.0 {
                ((storage_used / storage_total * 100.0).min(100.0) * 100.0).round() / 100.0
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
                cpu_usage: cpu_usage as f32,
                memory_usage: memory_usage as f32,
                storage_usage: storage_usage as f32,
            }
        })
        .collect();

    let search_query = search_query().to_lowercase();
    let filtered_nodes: Vec<_> = node_data
        .iter()
        .filter(|node| {
            // First check the type filter since it's a simple equality check
            if selected_node() != "all" && node.node_type != selected_node() {
                return false;
            }
            
            // If no search query, include all nodes that match the type filter
            if search_query.is_empty() {
                return true;
            }
            
            // Search across multiple fields, ordered by importance
            node.name.to_lowercase().contains(&search_query) ||
            node.ip.to_lowercase().contains(&search_query) ||
            node.os.to_lowercase().contains(&search_query) ||
            node.status.to_lowercase().contains(&search_query) ||
            node.kubernetes_version.to_lowercase().contains(&search_query) ||
            node.architecture.to_lowercase().contains(&search_query)
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
                    button { class: "btn btn-secondary", onclick: refresh, "Refresh" }
                }
            }

            div { class: "nodes-grid",
                {filtered_nodes.iter().map(|node| {
                    let binding = nodes();
                    let original_node = binding.iter()
                        .find(|n| n.node.metadata.name.as_ref().map_or(false, |name| name == &node.name));
                    rsx!(NodeItem {
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
                        conditions: original_node
                            .and_then(|n| n.node.status.as_ref())
                            .and_then(|status| status.conditions.as_ref())
                            .map(|conditions| {
                                conditions.iter().map(|c| crate::components::NodeCondition {
                                    condition_type: c.type_.clone(),
                                    status: c.status.clone(),
                                }).collect()
                            })
                            .unwrap_or_default(),
                    })
                })}
            }
        }
    }
}
