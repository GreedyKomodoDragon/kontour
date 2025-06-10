use dioxus::{html::tr, logger::tracing};
use k8s_openapi::api::core::v1::Pod;
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use kube::{
    api::{Api, ListParams},
    core::ObjectList,
    Resource, ResourceExt,
};
use serde::Deserialize;

#[derive(Clone)]
pub struct ResourceHotspot {
    pub name: String,
    pub namespace: String,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub hotspot_type: String,
    pub severity: String,  // "high" or "low"
}

#[derive(Deserialize, Clone, Debug)]
struct MetricsContainerUsage {
    cpu: Quantity,
    memory: Quantity,
}

#[derive(Deserialize, Clone, Debug)]
struct MetricsContainer {
    name: String,
    usage: MetricsContainerUsage,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct PodMetrics {
    metadata: ObjectMeta,
    #[serde(default)]
    containers: Vec<MetricsContainer>,
    timestamp: String,
    window: String,
}

impl Resource for PodMetrics {
    type DynamicType = ();
    type Scope = kube::core::NamespaceResourceScope;

    fn group(dt: &()) -> std::borrow::Cow<'static, str> {
        "metrics.k8s.io".into()
    }
    
    fn version(dt: &()) -> std::borrow::Cow<'static, str> {
        "v1beta1".into()
    }
    
    fn kind(dt: &()) -> std::borrow::Cow<'static, str> {
        "PodMetrics".into()
    }
    
    fn plural(dt: &()) -> std::borrow::Cow<'static, str> {
        "pods".into()
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

fn parse_cpu_value(cpu: &Quantity) -> f64 {
    let cpu_str = cpu.0.to_string();
    if cpu_str.ends_with('m') {
        // Convert millicores to cores
        cpu_str.trim_end_matches('m').parse::<f64>().unwrap_or(0.0) / 1000.0
    } else {
        // Already in cores
        cpu_str.parse::<f64>().unwrap_or(0.0)
    }
}

fn parse_memory_value(memory: &Quantity) -> f64 {
    let mem_str = memory.0.to_string();
    if mem_str.ends_with("Ki") {
        mem_str.trim_end_matches("Ki").parse::<f64>().unwrap_or(0.0) * 1024.0
    } else if mem_str.ends_with("Mi") {
        mem_str.trim_end_matches("Mi").parse::<f64>().unwrap_or(0.0) * 1024.0 * 1024.0
    } else if mem_str.ends_with("Gi") {
        mem_str.trim_end_matches("Gi").parse::<f64>().unwrap_or(0.0) * 1024.0 * 1024.0 * 1024.0
    } else {
        // Assume bytes
        mem_str.parse::<f64>().unwrap_or(0.0)
    }
}

pub async fn find_resource_hotspots(client: kube::Client) -> Vec<ResourceHotspot> {
    let mut hotspots = Vec::new();
    
    // Create an API for the metrics endpoint
    let metrics_api: Api<PodMetrics> = Api::all(client.clone());    

    // Get all pods first to access their resource limits
    let pods: Api<Pod> = Api::all(client);
    let pod_list = match pods.list(&ListParams::default()).await {
        Ok(list) => list.items,
        Err(_e) => {
            tracing::error!("Failed to fetch pods for resource hotspots: {}", _e);
            return Vec::new();
        }
    };

    // Get pod metrics
    let pod_metrics = match metrics_api.list(&ListParams::default()).await {
        Ok(metrics) => metrics.items,
        Err(e) => {
            tracing::error!("Failed to fetch pod metrics: {}", e);
            return Vec::new();
        }
    };

    // Build a map of pod name to its limits
    let mut pod_limits = std::collections::HashMap::new();
    for pod in &pod_list {
        if let Some(name) = &pod.metadata.name {
            if let Some(spec) = &pod.spec {
                let mut total_cpu_limit = 0.0;
                let mut total_memory_limit = 0.0;

                for container in &spec.containers {
                    if let Some(resources) = &container.resources {
                        if let Some(limits) = &resources.limits {
                            if let Some(cpu) = limits.get("cpu") {
                                total_cpu_limit += parse_cpu_value(cpu);
                            }
                            if let Some(memory) = limits.get("memory") {
                                total_memory_limit += parse_memory_value(memory);
                            }
                        }
                    }
                }

                pod_limits.insert(name.clone(), (total_cpu_limit, total_memory_limit));
            }
        }
    }

    // Process metrics and compare against limits
    for metric in pod_metrics {
        let name = metric.metadata.name.clone().unwrap_or_default();
        let namespace = metric.metadata.namespace.clone().unwrap_or_default();

        // Skip system namespaces
        if namespace == "kube-system" || namespace == "kube-public" {
            continue;
        }

        let mut total_cpu_usage = 0.0;
        let mut total_memory_usage = 0.0;

        // Sum up container metrics
        for container in &metric.containers {
            total_cpu_usage += parse_cpu_value(&container.usage.cpu);
            total_memory_usage += parse_memory_value(&container.usage.memory);
        }

        // Compare with limits if available
        if let Some((cpu_limit, memory_limit)) = pod_limits.get(&name) {
            let cpu_usage_percent = if *cpu_limit > 0.0 {
                (total_cpu_usage / cpu_limit) * 100.0
            } else {
                0.0
            };

            let memory_usage_percent = if *memory_limit > 0.0 {
                (total_memory_usage / memory_limit) * 100.0
            } else {
                0.0
            };

            // Check for high usage (over 80%) and low usage (under 10%)
            if cpu_usage_percent >= 20.0 {
                hotspots.push(ResourceHotspot {
                    name: name.clone(),
                    namespace: namespace.clone(),
                    cpu_usage: cpu_usage_percent,
                    memory_usage: memory_usage_percent,
                    hotspot_type: "High CPU Usage".to_string(),
                    severity: "high".to_string(),
                });
            } else if memory_usage_percent >= 80.0 {
                hotspots.push(ResourceHotspot {
                    name: name.clone(),
                    namespace: namespace.clone(),
                    cpu_usage: cpu_usage_percent,
                    memory_usage: memory_usage_percent,
                    hotspot_type: "High Memory Usage".to_string(),
                    severity: "high".to_string(),
                });
            }

            // Check for underutilization
            if cpu_usage_percent <= 10.0 && *cpu_limit > 0.0 {
                hotspots.push(ResourceHotspot {
                    name: name.clone(),
                    namespace: namespace.clone(),
                    cpu_usage: cpu_usage_percent,
                    memory_usage: memory_usage_percent,
                    hotspot_type: "Low CPU Usage".to_string(),
                    severity: "low".to_string(),
                });
            } 
            if memory_usage_percent <= 10.0 && *memory_limit > 0.0 {
                hotspots.push(ResourceHotspot {
                    name,
                    namespace,
                    cpu_usage: cpu_usage_percent,
                    memory_usage: memory_usage_percent,
                    hotspot_type: "Low Memory Usage".to_string(),
                    severity: "low".to_string(),
                });
            }
        }
    }

    // Sort hotspots by usage percentage (highest first)
    hotspots.sort_by(|a, b| {
        let a_max = a.cpu_usage.max(a.memory_usage);
        let b_max = b.cpu_usage.max(b.memory_usage);
        b_max.partial_cmp(&a_max).unwrap_or(std::cmp::Ordering::Equal)
    });

    hotspots
}
