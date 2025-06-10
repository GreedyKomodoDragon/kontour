use dioxus::logger::tracing;
use k8s_openapi::api::core::v1::Node;
use k8s_openapi::apimachinery::pkg::{api::resource::Quantity, apis::meta::v1::ObjectMeta};
use kube::api::TypeMeta;
use kube::{
    api::{Api, ListParams},
    core::{ObjectList, Resource, ResourceExt},
    Client,
};
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug)]
struct NodeMetrics {
    metadata: ObjectMeta,
    timestamp: String,
    window: String,
    usage: std::collections::BTreeMap<String, Quantity>,
}

impl Resource for NodeMetrics {
    type DynamicType = ();
    type Scope = kube::core::NamespaceResourceScope;

    fn group(dt: &()) -> std::borrow::Cow<'static, str> {
        "metrics.k8s.io".into()
    }
    
    fn version(dt: &()) -> std::borrow::Cow<'static, str> {
        "v1beta1".into()
    }
    
    fn kind(dt: &()) -> std::borrow::Cow<'static, str> {
        "NodeMetrics".into()
    }
    
    fn plural(dt: &()) -> std::borrow::Cow<'static, str> {
        "nodes".into()
    }

    fn api_version(dt: &()) -> std::borrow::Cow<'static, str> {
        "metrics.k8s.io/v1beta1".into()
    }

    fn meta(&self) -> &ObjectMeta {
        &self.metadata
    }

    fn meta_mut(&mut self) -> &mut ObjectMeta {
        &mut self.metadata
    }
}

#[derive(Clone, Debug, Default)]
pub struct ClusterResourceUsage {
    pub cpu_total: f64,
    pub cpu_used: f64,
    pub memory_total: f64,
    pub memory_used: f64,
    pub storage_total: f64,
    pub storage_used: f64,
}

pub async fn get_cluster_resources(client: Client) -> ClusterResourceUsage {
    let nodes: Api<Node> = Api::all(client.clone());
    let metrics_api: Api<NodeMetrics> = Api::all(client);
    let mut usage = ClusterResourceUsage::default();

    // Get all nodes and metrics
    let node_list = match nodes.list(&ListParams::default()).await {
        Ok(list) => list,
        Err(_) => return usage,
    };
    
    let metrics_list = match metrics_api.list(&ListParams::default()).await {
        Ok(list) => list,
        Err(_) => ObjectList {
            types: TypeMeta::default(),
            metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ListMeta::default(),
            items: vec![],
        },
    };

    // Create metrics lookup map for efficient access
    let metrics_map: std::collections::HashMap<String, &NodeMetrics> = metrics_list.items.iter()
        .filter_map(|m| m.metadata.name.as_ref().map(|name| (name.clone(), m)))
        .collect();

    for node in &node_list.items {
        // Get node name
        let empty_string = String::new();
        let node_name = node.metadata.name.as_ref().unwrap_or(&empty_string);

        // Get allocatable resources
        if let Some(allocatable) = &node.status.as_ref().and_then(|s| s.allocatable.as_ref()) {
            // CPU
            if let Some(cpu) = allocatable.get("cpu") {
                usage.cpu_total += parse_cpu_value(cpu);
            }
            // Memory
            if let Some(memory) = allocatable.get("memory") {
                usage.memory_total += parse_memory_value(memory);
            }
            // Storage
            if let Some(storage) = allocatable.get("ephemeral-storage") {
                usage.storage_total += parse_storage_value(storage);
            }
        }

        // Get metrics for this node
        if let Some(node_metrics) = metrics_map.get(node_name) {
            // CPU usage from metrics
            if let Some(cpu) = node_metrics.usage.get("cpu") {
                let cpu_value = parse_cpu_value(cpu);
                tracing::debug!("Node {} CPU usage: {}", node_name, cpu_value);
                usage.cpu_used += cpu_value;
            }
            // Memory usage from metrics
            if let Some(memory) = node_metrics.usage.get("memory") {
                let memory_value = parse_memory_value(memory);
                tracing::debug!("Node {} Memory usage: {}", node_name, memory_value);
                usage.memory_used += memory_value;
            }
        } else {
            tracing::debug!("No metrics found for node {}", node_name);
        }

        // Storage usage from node status (since it's not in metrics)
        if let Some(fs) = node.status.as_ref()
            .and_then(|s| s.capacity.as_ref())
            .and_then(|a| a.get("ephemeral-storage")) 
        {
            usage.storage_used += parse_storage_value(fs);
        }
    }

    usage
}

fn parse_cpu_value(cpu: &k8s_openapi::apimachinery::pkg::api::resource::Quantity) -> f64 {
    let cpu_str = cpu.0.to_string();
    if cpu_str.ends_with('m') {
        // Convert millicores to cores
        cpu_str.trim_end_matches('m').parse::<f64>().unwrap_or(0.0) / 1000.0
    } else {
        // Already in cores
        cpu_str.parse::<f64>().unwrap_or(0.0)
    }
}

fn parse_memory_value(memory: &k8s_openapi::apimachinery::pkg::api::resource::Quantity) -> f64 {
    let mem_str = memory.0.to_string();
    if mem_str.ends_with("Ki") {
        mem_str.trim_end_matches("Ki").parse::<f64>().unwrap_or(0.0) * 1024.0 / (1024.0 * 1024.0 * 1024.0) // Convert to GB
    } else if mem_str.ends_with("Mi") {
        mem_str.trim_end_matches("Mi").parse::<f64>().unwrap_or(0.0) / 1024.0 // Convert to GB
    } else if mem_str.ends_with("Gi") {
        mem_str.trim_end_matches("Gi").parse::<f64>().unwrap_or(0.0) // Already in GB
    } else {
        // Assume bytes
        mem_str.parse::<f64>().unwrap_or(0.0) / (1024.0 * 1024.0 * 1024.0) // Convert to GB
    }
}

fn parse_storage_value(storage: &k8s_openapi::apimachinery::pkg::api::resource::Quantity) -> f64 {
    parse_memory_value(storage) // Storage uses same format as memory
}
