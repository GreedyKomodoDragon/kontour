use dioxus::prelude::*;

#[derive(PartialEq, Clone)]
pub struct ResourceQuota {
    pub cpu_used: String,
    pub cpu_limit: String,
    pub memory_used: String,
    pub memory_limit: String,
    pub pods_used: u32,
    pub pods_limit: u32,
}

#[derive(PartialEq, Clone)]
pub struct LimitRange {
    pub default_request_cpu: String,
    pub default_request_memory: String,
    pub default_limit_cpu: String,
    pub default_limit_memory: String,
}

#[derive(Props, PartialEq, Clone)]
pub struct NamespaceItemProps {
    pub name: String,
    pub status: String,
    pub age: String,
    pub labels: Vec<(String, String)>,
    pub pod_count: u32,
    pub resource_quota: ResourceQuota,
    pub limit_range: Option<LimitRange>,
    pub phase: String,
}

fn calculate_pod_progress(used: u32, limit: u32) -> f32 {
    if limit > 0 {
        ((used as f32) / (limit as f32) * 100.0).min(100.0)
    } else {
        0.0
    }
}

fn calculate_progress_width(used: &str, limit: &str) -> f32 {
    match (parse_resource_value(used), parse_resource_value(limit)) {
        (Some(used), Some(limit)) if limit > 0.0 => (used / limit * 100.0).min(100.0),
        _ => 0.0,
    }
}

fn parse_resource_value(value: &str) -> Option<f32> {
    if value.is_empty() || value == "0" {
        return None;
    }

    // Parse memory values (Gi, Mi, Ki)
    if let Some(value) = value.strip_suffix("Gi") {
        return value.parse::<f32>().ok();
    }
    if let Some(value) = value.strip_suffix("Mi") {
        return value.parse::<f32>().ok().map(|v| v / 1024.0);
    }
    if let Some(value) = value.strip_suffix("Ki") {
        return value.parse::<f32>().ok().map(|v| v / (1024.0 * 1024.0));
    }

    // Parse CPU values (m for millicores)
    if let Some(value) = value.strip_suffix('m') {
        return value.parse::<f32>().ok().map(|v| v / 1000.0);
    }

    // Try parsing as a plain number
    value.parse::<f32>().ok()
}

#[component]
pub fn NamespaceItem(props: NamespaceItemProps) -> Element {
    let mut is_expanded = use_signal(|| false);

    rsx! {
        div {
            key: "{props.name}",
            class: "namespace-card",
            div {
                class: "namespace-header",
                div {
                    class: "namespace-title",
                    h3 {
                        "{props.name}"
                    }
                }
                div {
                    class: "namespace-controls",
                    button {
                        class: "btn-icon expand-toggle",
                        onclick: move |evt| {
                            evt.stop_propagation();
                            is_expanded.set(!is_expanded());
                        },
                        title: if is_expanded() { "Collapse" } else { "Expand" },
                        if is_expanded() { "🔼" } else { "🔽" }
                    }
                }
            }

            {is_expanded().then(|| rsx!(
                div {
                    class: "labels-section margin-top-6",
                    h4 {
                        "Labels"
                    }
                    div {
                        class: "labels-grid",
                        {props.labels.iter().map(|(key, value)| rsx!(
                            div {
                                key: "{key}",
                                class: "label",
                                span {
                                    class: "label-key",
                                    "{key}"
                                }
                                span {
                                    class: "label-value",
                                    "{value}"
                                }
                            }
                        ))}
                    }
                }

                div { class: "resource-section",
                    h4 { class: "resource-header", "Resource Quota" }
                    div { class: "resource-metrics",
                        div { class: "metric",
                            span { class: "metric-label", "CPU" }
                            div { class: "namespace-progress-bar",
                                div {
                                    class: "progress-fill",
                                    style: "width: {calculate_progress_width(&props.resource_quota.cpu_used, &props.resource_quota.cpu_limit)}%"
                                }
                            }
                            span { class: "metric-value", "{props.resource_quota.cpu_used}/{props.resource_quota.cpu_limit}" }
                        }
                        div { class: "metric",
                            span { class: "metric-label", "Mem" }
                            div { class: "namespace-progress-bar",
                                div {
                                    class: "progress-fill",
                                    style: "width: {calculate_progress_width(&props.resource_quota.memory_used, &props.resource_quota.memory_limit)}%"
                                }
                            }
                            span { class: "metric-value", "{props.resource_quota.memory_used}/{props.resource_quota.memory_limit}" }
                        }
                        div { class: "metric",
                            span { class: "metric-label", "Pods" }
                            div { class: "namespace-progress-bar",
                                div {
                                    class: "progress-fill",
                                    style: "width: {calculate_pod_progress(props.resource_quota.pods_used, props.resource_quota.pods_limit)}%"
                                }
                            }
                            span { class: "metric-value", "{props.resource_quota.pods_used}/{props.resource_quota.pods_limit}" }
                        }
                    }
                }

                {props.limit_range.as_ref().map(|lr| rsx!(
                    div { class: "limit-section",
                        h4 { "Limit Range" }
                        div { class: "limit-grid",
                            div { class: "limit-group",
                                div { class: "limit-item",
                                    span { class: "limit-label", "Default Request CPU" }
                                    span { class: "limit-value", "{lr.default_request_cpu}" }
                                }
                                div { class: "limit-item",
                                    span { class: "limit-label", "Default Request Memory" }
                                    span { class: "limit-value", "{lr.default_request_memory}" }
                                }
                            }
                            div { class: "limit-group",
                                div { class: "limit-item",
                                    span { class: "limit-label", "Default Limit CPU" }
                                    span { class: "limit-value", "{lr.default_limit_cpu}" }
                                }
                                div { class: "limit-item",
                                    span { class: "limit-label", "Default Limit Memory" }
                                    span { class: "limit-value", "{lr.default_limit_memory}" }
                                }
                            }
                        }
                    }
                ))}

                div { class: "namespace-footer",
                    div { class: "info-item",
                        span { class: "info-label", "Age" }
                        span { class: "info-value", "{props.age}" }
                    }
                    div { class: "info-item",
                        span { class: "info-label", "Phase" }
                        span { class: "info-value", "{props.phase}" }
                    }
                    div { class: "info-item",
                        span { class: "info-label", "Pods" }
                        span { class: "info-value", "{props.pod_count}" }
                    }
                }))
            }
        }
    }
}
