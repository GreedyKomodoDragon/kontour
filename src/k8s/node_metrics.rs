use dioxus::logger::tracing;
use k8s_openapi::{
    api::core::v1::Node,
    apimachinery::pkg::{
        api::resource::Quantity,
        apis::meta::v1::ObjectMeta,
    },
};
use kube::{
    api::{ListParams, Api},
    core::{Resource},
    Client,
};
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};

#[derive(Deserialize, Clone, Debug, Default)]
pub struct NodeMetrics {
    pub metadata: ObjectMeta,
    pub timestamp: String,
    pub window: String,
    pub usage: BTreeMap<String, Quantity>,
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

pub fn parse_resource_quantity(quantity: &str) -> f32 {
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

pub async fn fetch_node_metrics(client: &Client) -> HashMap<String, NodeMetrics> {
    let metrics_api: Api<NodeMetrics> = Api::all(client.clone());
    
    match metrics_api.list(&ListParams::default()).await {
        Ok(metrics_list) => {
            metrics_list.items
                .into_iter()
                .filter_map(|m| m.metadata.name.clone().map(|name| (name, m)))
                .collect()
        }
        Err(e) => {
            tracing::error!("Failed to fetch metrics: {:?}", e);
            HashMap::new()
        }
    }
}

pub fn apply_metrics_to_node(node: &mut Node, metrics: Option<&NodeMetrics>) {
    if let Some(metrics) = metrics {
        // Create or get mutable status
        if node.status.is_none() {
            node.status = Some(k8s_openapi::api::core::v1::NodeStatus::default());
        }
        
        if let Some(status) = &mut node.status {
            // Store metrics in the allocatable field
            status.allocatable = Some(metrics.usage.clone());
        }
    }
}
