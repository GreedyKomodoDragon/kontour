use dioxus::{logger::tracing, prelude::*};
use k8s_openapi::{
    api::core::v1::{
        LimitRange as K8sLimitRange, Namespace, Pod, ResourceQuota as K8sResourceQuota,
    },
    apimachinery::pkg::api::resource::Quantity,
    chrono,
};
use kube::{api::ListParams, Api, Client};
use std::collections::BTreeMap;

use crate::components::{LimitRange, NamespaceItem, NamespaceItemProps, ResourceQuota};

const NAMESPACES_CSS: Asset = asset!("/assets/styling/namespaces.css");

fn parse_resource_quantity(value: &str) -> Option<f32> {
    if value.is_empty() || value == "0" {
        return None;
    }

    // Handle memory values
    if value.ends_with("Gi") {
        return value.trim_end_matches("Gi").parse::<f32>().ok();
    }
    if value.ends_with("Mi") {
        return value
            .trim_end_matches("Mi")
            .parse::<f32>()
            .ok()
            .map(|v| v / 1024.0);
    }
    if value.ends_with("Ki") {
        return value
            .trim_end_matches("Ki")
            .parse::<f32>()
            .ok()
            .map(|v| v / (1024.0 * 1024.0));
    }

    // Handle CPU values
    if value.ends_with('m') {
        return value
            .trim_end_matches('m')
            .parse::<f32>()
            .ok()
            .map(|v| v / 1000.0);
    }

    // Try parsing as a plain number
    value.parse::<f32>().ok()
}

fn format_metric(value: &str) -> String {
    if let Some(num) = parse_resource_quantity(value) {
        if value.ends_with("Gi") || value.ends_with("Mi") || value.ends_with("Ki") {
            // Memory values
            if num >= 1.0 {
                format!("{:.1}Gi", num)
            } else {
                format!("{:.0}Mi", num * 1024.0)
            }
        } else if value.ends_with('m') || num < 1.0 {
            // CPU values in millicores
            format!("{:.0}m", num * 1000.0)
        } else {
            // CPU values in cores
            format!("{:.1}", num)
        }
    } else {
        "0".to_string()
    }
}

#[derive(Clone)]
struct NamespaceInfo {
    namespace: Namespace,
    pods: Vec<Pod>,
    resource_quota: Option<K8sResourceQuota>,
    limit_range: Option<K8sLimitRange>,
}

#[derive(Clone)]
struct NamespaceFetcher {
    client: Client,
    namespaces: Signal<Vec<NamespaceInfo>>,
}

impl NamespaceFetcher {
    async fn fetch_namespace_info(client: Client, ns_name: &str) -> NamespaceInfo {
        let pods_api: Api<Pod> = Api::namespaced(client.clone(), ns_name);
        let quotas_api: Api<K8sResourceQuota> = Api::namespaced(client.clone(), ns_name);
        let limitranges_api: Api<K8sLimitRange> = Api::namespaced(client.clone(), ns_name);

        let pods = match pods_api.list(&ListParams::default()).await {
            Ok(pod_list) => pod_list.items,
            Err(e) => {
                tracing::error!("Failed to fetch pods for namespace {}: {:?}", ns_name, e);
                Vec::new()
            }
        };

        let resource_quota = match quotas_api.list(&ListParams::default()).await {
            Ok(quota_list) => quota_list.items.into_iter().next(),
            Err(e) => {
                tracing::error!(
                    "Failed to fetch resource quotas for namespace {}: {:?}",
                    ns_name,
                    e
                );
                None
            }
        };

        let limit_range = match limitranges_api.list(&ListParams::default()).await {
            Ok(lr_list) => lr_list.items.into_iter().next(),
            Err(e) => {
                tracing::error!(
                    "Failed to fetch limit ranges for namespace {}: {:?}",
                    ns_name,
                    e
                );
                None
            }
        };

        NamespaceInfo {
            namespace: Namespace::default(), // Will be filled in later
            pods,
            resource_quota,
            limit_range,
        }
    }

    fn fetch(&self, query: String) {
        let client = self.client.clone();
        let mut namespaces = self.namespaces.clone();

        tracing::info!("Starting namespace fetch...");

        spawn(async move {
            let api = Api::<Namespace>::all(client.clone());

            match api.list(&ListParams::default()).await {
                Ok(namespace_list) => {
                    let filtered_namespaces = if query.is_empty() {
                        namespace_list.items
                    } else {
                        namespace_list
                            .items
                            .into_iter()
                            .filter(|ns| {
                                let name_match = ns
                                    .metadata
                                    .name
                                    .as_ref()
                                    .map(|name| name.to_lowercase().contains(&query.to_lowercase()))
                                    .unwrap_or(false);

                                let label_match = ns
                                    .metadata
                                    .labels
                                    .as_ref()
                                    .map(|labels| {
                                        labels.iter().any(|(k, v)| {
                                            k.to_lowercase().contains(&query.to_lowercase())
                                                || v.to_lowercase().contains(&query.to_lowercase())
                                        })
                                    })
                                    .unwrap_or(false);

                                name_match || label_match
                            })
                            .collect::<Vec<_>>()
                    };

                    // Fetch additional info for each namespace
                    let mut namespace_infos = Vec::new();
                    for ns in filtered_namespaces {
                        if let Some(name) = &ns.metadata.name {
                            let mut info = Self::fetch_namespace_info(client.clone(), name).await;
                            info.namespace = ns;
                            namespace_infos.push(info);
                        }
                    }

                    namespaces.set(namespace_infos);
                }
                Err(e) => {
                    tracing::error!("Failed to fetch namespaces: {:?}", e);
                }
            }
        });
    }

    fn parse_quantity(quantity: Option<&Quantity>) -> String {
        quantity
            .map(|q| q.0.clone())
            .unwrap_or_else(|| "0".to_string())
    }

    fn format_quantity(quantity: Option<&Quantity>, is_memory: bool) -> String {
        let raw = quantity
            .map(|q| q.0.clone())
            .unwrap_or_else(|| "0".to_string());

        if is_memory {
            format_metric(&raw)
        } else {
            format_metric(&raw)
        }
    }
}

#[component]
pub fn Namespaces() -> Element {
    let client = use_context::<Client>();
    let selected_status = use_signal(|| "all");
    let search_query = use_signal(String::new);
    let namespaces = use_signal(|| Vec::<NamespaceInfo>::new());

    let fetcher = NamespaceFetcher {
        client: client.clone(),
        namespaces: namespaces.clone(),
    };

    use_effect({
        let fetcher = fetcher.clone();
        move || {
            let query = search_query();
            fetcher.fetch(query);
        }
    });

    let refresh = {
        let fetcher = fetcher.clone();
        move |_: Event<MouseData>| {
            let query = search_query();
            fetcher.fetch(query);
        }
    };

    // Convert k8s Namespace objects to our display format
    let namespace_data: Vec<_> = namespaces()
        .into_iter()
        .map(|ns_info| {
            let ns = &ns_info.namespace;
            let name = ns.metadata.name.clone().unwrap_or_default();
            let age = ns
                .metadata
                .creation_timestamp
                .as_ref()
                .map(|t| {
                    let duration = chrono::Utc::now().signed_duration_since(t.0);
                    if duration.num_days() > 0 {
                        format!("{}d", duration.num_days())
                    } else if duration.num_hours() > 0 {
                        format!("{}h", duration.num_hours())
                    } else {
                        format!("{}m", duration.num_minutes())
                    }
                })
                .unwrap_or_default();

            let labels = ns
                .metadata
                .labels
                .clone()
                .unwrap_or_default()
                .into_iter()
                .collect();

            let status = ns
                .status
                .as_ref()
                .and_then(|s| s.phase.clone())
                .unwrap_or_default();

            let pod_count = ns_info.pods.len() as u32;

            // Parse resource quotas
            let resource_quota = if let Some(quota) = &ns_info.resource_quota {
                let hard = quota.spec.as_ref().map(|s| &s.hard).unwrap_or(&None);

                let used = quota.status.as_ref().map(|s| &s.used).unwrap_or(&None);

                ResourceQuota {
                    cpu_used: NamespaceFetcher::format_quantity(
                        used.as_ref().and_then(|m| m.get("cpu")),
                        false,
                    ),
                    cpu_limit: NamespaceFetcher::format_quantity(
                        hard.as_ref().and_then(|m| m.get("cpu")),
                        false,
                    ),
                    memory_used: NamespaceFetcher::format_quantity(
                        used.as_ref().and_then(|m| m.get("memory")),
                        true,
                    ),
                    memory_limit: NamespaceFetcher::format_quantity(
                        hard.as_ref().and_then(|m| m.get("memory")),
                        true,
                    ),
                    pods_used: pod_count,
                    pods_limit: hard
                        .as_ref()
                        .and_then(|map| map.get("pods"))
                        .and_then(|q| q.0.parse::<u32>().ok())
                        .unwrap_or(0),
                }
            } else {
                ResourceQuota {
                    cpu_used: "0".into(),
                    cpu_limit: "0".into(),
                    memory_used: "0".into(),
                    memory_limit: "0".into(),
                    pods_used: pod_count,
                    pods_limit: 0,
                }
            };

            // Parse limit ranges
            let limit_range = ns_info.limit_range.as_ref().map(|lr| {
                let binding = k8s_openapi::api::core::v1::LimitRangeItem {
                    default: None,
                    default_request: None,
                    max: None,
                    min: None,
                    max_limit_request_ratio: None,
                    type_: String::new(),
                };

                let limits = lr
                    .spec
                    .as_ref()
                    .map(|s| &s.limits)
                    .and_then(|l| l.first())
                    .unwrap_or(&binding);

                let format_limit = |m: Option<&BTreeMap<String, Quantity>>, key: &str| {
                    let value = NamespaceFetcher::parse_quantity(m.and_then(|map| map.get(key)));
                    format_metric(&value)
                };

                LimitRange {
                    default_request_cpu: format_limit(limits.default_request.as_ref(), "cpu"),
                    default_request_memory: format_limit(limits.default_request.as_ref(), "memory"),
                    default_limit_cpu: format_limit(limits.default.as_ref(), "cpu"),
                    default_limit_memory: format_limit(limits.default.as_ref(), "memory"),
                }
            });

            (
                NamespaceItemProps {
                    name: name.clone(),
                    status: status.clone(),
                    age,
                    labels,
                    pod_count,
                    phase: status,
                    resource_quota,
                    limit_range,
                },
                name,
            )
        })
        .collect();

    let filtered_namespaces: Vec<_> = namespace_data
        .iter()
        .filter(|(props, _)| {
            selected_status() == "all" || props.status.to_lowercase() == selected_status()
        })
        .collect();

    rsx! {
        document::Link { rel: "stylesheet", href: NAMESPACES_CSS }

        div { class: "namespaces-container",
            div { class: "namespaces-header",
                div { class: "header-left",
                    h1 { "Namespaces" }
                    div { class: "header-controls",
                        div { class: "search-container",
                            input {
                                class: "search-input",
                                r#type: "text",
                                placeholder: "Search namespaces...",
                                value: "{search_query}",
                            }
                        }
                        select {
                            class: "status-select",
                            value: "{selected_status.read()}",
                            // onchange: move |evt| selected_status.set(evt.value.as_str()),
                            option { value: "all", "All Statuses ({namespaces.len()})" }
                            option { value: "Active", "Active" }
                            option { value: "Terminating", "Terminating" }
                        }
                        span { class: "namespace-count", "{filtered_namespaces.len()} namespaces" }
                    }
                }
                div { class: "header-actions",
                    button { 
                        class: "btn btn-primary",
                        onclick: move |_| {
                            use_navigator().push("/namespaces/create");
                        },
                        "Create Namespace"
                    }
                    button { class: "btn btn-secondary", onclick: refresh, "Refresh" }
                }
            }

            div { class: "namespaces-grid",
                {filtered_namespaces.iter().map(|(props, _name)| rsx!(
                    NamespaceItem {
                        name: props.name.clone(),
                        status: props.status.clone(),
                        age: props.age.clone(),
                        labels: props.labels.clone(),
                        pod_count: props.pod_count,
                        resource_quota: props.resource_quota.clone(),
                        limit_range: props.limit_range.clone(),
                        phase: props.phase.clone(),
                    }
                ))}
            }
        }
    }
}
