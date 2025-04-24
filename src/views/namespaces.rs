use dioxus::prelude::*;

const NAMESPACES_CSS: Asset = asset!("/assets/styling/namespaces.css");

#[derive(Clone)]
struct NamespaceData {
    name: String,
    status: String,
    age: String,
    labels: Vec<(String, String)>,
    pod_count: u32,
    resource_quota: ResourceQuota,
    limit_range: Option<LimitRange>,
    phase: String,
}

#[derive(Clone)]
struct ResourceQuota {
    cpu_used: String,
    cpu_limit: String,
    memory_used: String,
    memory_limit: String,
    pods_used: u32,
    pods_limit: u32,
}

#[derive(Clone)]
struct LimitRange {
    default_request_cpu: String,
    default_request_memory: String,
    default_limit_cpu: String,
    default_limit_memory: String,
}

#[component]
pub fn Namespaces() -> Element {
    let selected_status = use_signal(|| "all");
    let search_query = use_signal(String::new);

    // Mock data for namespaces
    let namespaces = vec![
        NamespaceData {
            name: "default".into(),
            status: "Active".into(),
            age: "145d".into(),
            labels: vec![
                ("environment".into(), "production".into()),
                ("team".into(), "platform".into()),
            ],
            pod_count: 12,
            resource_quota: ResourceQuota {
                cpu_used: "2.5".into(),
                cpu_limit: "4".into(),
                memory_used: "4.2Gi".into(),
                memory_limit: "8Gi".into(),
                pods_used: 12,
                pods_limit: 20,
            },
            limit_range: Some(LimitRange {
                default_request_cpu: "100m".into(),
                default_request_memory: "128Mi".into(),
                default_limit_cpu: "500m".into(),
                default_limit_memory: "512Mi".into(),
            }),
            phase: "Active".into(),
        },
        NamespaceData {
            name: "kube-system".into(),
            status: "Active".into(),
            age: "145d".into(),
            labels: vec![
                ("kubernetes.io/metadata.name".into(), "kube-system".into()),
            ],
            pod_count: 8,
            resource_quota: ResourceQuota {
                cpu_used: "1.8".into(),
                cpu_limit: "4".into(),
                memory_used: "3.5Gi".into(),
                memory_limit: "8Gi".into(),
                pods_used: 8,
                pods_limit: 20,
            },
            limit_range: None,
            phase: "Active".into(),
        },
        NamespaceData {
            name: "monitoring".into(),
            status: "Active".into(),
            age: "98d".into(),
            labels: vec![
                ("environment".into(), "production".into()),
                ("team".into(), "sre".into()),
            ],
            pod_count: 15,
            resource_quota: ResourceQuota {
                cpu_used: "3.2".into(),
                cpu_limit: "6".into(),
                memory_used: "6.8Gi".into(),
                memory_limit: "12Gi".into(),
                pods_used: 15,
                pods_limit: 30,
            },
            limit_range: Some(LimitRange {
                default_request_cpu: "200m".into(),
                default_request_memory: "256Mi".into(),
                default_limit_cpu: "1".into(),
                default_limit_memory: "1Gi".into(),
            }),
            phase: "Active".into(),
        },
    ];

    let filtered_namespaces: Vec<_> = namespaces
        .iter()
        .filter(|&ns| selected_status() == "all" || ns.status == selected_status())
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
                    button { class: "btn btn-primary", "Create Namespace" }
                    button { class: "btn btn-secondary", "Refresh" }
                }
            }

            div { class: "namespaces-grid",
                {filtered_namespaces.iter().map(|ns| rsx!(
                    div { 
                        key: "{ns.name}",
                        class: "namespace-card",
                        div { class: "namespace-header",
                            div { class: "namespace-title",
                                h3 { "{ns.name}" }
                                span { class: "status-badge status-healthy", "{ns.status}" }
                            }
                            div { class: "namespace-controls",
                                button { class: "btn-icon", title: "Edit", "✏️" }
                                button { class: "btn-icon", title: "Delete", "🗑️" }
                            }
                        }

                        div { class: "labels-section",
                            h4 { "Labels" }
                            div { class: "labels-grid",
                                {ns.labels.iter().map(|(key, value)| rsx!(
                                    div { 
                                        key: "{key}",
                                        class: "label",
                                        span { class: "label-key", "{key}" }
                                        span { class: "label-value", "{value}" }
                                    }
                                ))}
                            }
                        }

                        div { class: "resource-section",
                            h4 { "Resource Quota" }
                            div { class: "resource-metrics",
                                div { class: "metric",
                                    span { class: "metric-label", "CPU" }
                                    div { class: "namespace-progress-bar",
                                        div {
                                            class: "progress-fill",
                                            style: "width: {(ns.resource_quota.cpu_used.parse::<f32>().unwrap() / ns.resource_quota.cpu_limit.parse::<f32>().unwrap() * 100.0)}%"
                                        }
                                    }
                                    span { class: "metric-value", "{ns.resource_quota.cpu_used}/{ns.resource_quota.cpu_limit}" }
                                }
                                div { class: "metric",
                                    span { class: "metric-label", "Memory" }
                                    div { class: "namespace-progress-bar",
                                        div {
                                            class: "progress-fill",
                                            style: "width: {(ns.resource_quota.memory_used.replace(\"Gi\", \"\").parse::<f32>().unwrap() / ns.resource_quota.memory_limit.replace(\"Gi\", \"\").parse::<f32>().unwrap() * 100.0)}%"
                                        }
                                    }
                                    span { class: "metric-value", "{ns.resource_quota.memory_used}/{ns.resource_quota.memory_limit}" }
                                }
                                div { class: "metric",
                                    span { class: "metric-label", "Pods" }
                                    div { class: "namespace-progress-bar",
                                        div {
                                            class: "progress-fill",
                                            style: "width: {(ns.resource_quota.pods_used as f32 / ns.resource_quota.pods_limit as f32 * 100.0)}%"
                                        }
                                    }
                                    span { class: "metric-value", "{ns.resource_quota.pods_used}/{ns.resource_quota.pods_limit}" }
                                }
                            }
                        }

                        {ns.limit_range.as_ref().map(|lr| rsx!(
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
                                span { class: "info-value", "{ns.age}" }
                            }
                            div { class: "info-item",
                                span { class: "info-label", "Phase" }
                                span { class: "info-value", "{ns.phase}" }
                            }
                            div { class: "info-item",
                                span { class: "info-label", "Pods" }
                                span { class: "info-value", "{ns.pod_count}" }
                            }
                        }
                    }
                ))}
            }
        }
    }
}
