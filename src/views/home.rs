use crate::k8s::{get_cluster_resources, get_recent_events, ClusterResourceUsage};
use dioxus::prelude::*;
use k8s_openapi::api::core::v1::Event;
use kube::Client;

const OVERVIEW_CSS: Asset = asset!("/assets/styling/overview.css");

/// The Home page component that renders the Kubernetes dashboard overview
#[component]
pub fn Home() -> Element {
    let client = use_context::<Client>();
    let events = use_signal(Vec::<Event>::new);
    let resources = use_signal(|| ClusterResourceUsage::default());

    // Fetch recent events
    use_effect({
        let client = client.clone();
        let mut events = events.clone();

        move || {
            spawn({
                let client = client.clone();
                async move {
                    let recent_events = get_recent_events(client).await;
                    events.set(recent_events);
                }
            });
        }
    });
    // Fetch cluster resources
    use_effect({
        let client = client.clone();
        let mut resources = resources.clone();

        move || {
            spawn({
                let client = client.clone();
                async move {
                    let usage = get_cluster_resources(client).await;
                    resources.set(usage);
                }
            });
        }
    });

    rsx! {
        link { rel: "stylesheet", href: OVERVIEW_CSS }
        div { class: "overview-container",
            div { class: "overview-header",
                h1 { "Cluster Overview" }
            }

            // Cluster Status Cards
            div { class: "cluster-status",
                div { class: "status-card",
                    h3 { "Cluster Status" }
                    p {
                        class: format!("status-value status-{}", resources.read().cluster_status.status.to_lowercase()),
                        { format!("{}", resources.read().cluster_status.status) }
                    }
                    p {
                        class: "status-subtext",
                        { format!("{}", resources.read().cluster_status.message) }
                    }
                }
                div { class: "status-card",
                    h3 { "Node Count" }
                    p { class: "status-value", { format!("{}", resources.read().node_count) } }
                    p { class: "status-subtext", "Active nodes" }
                }
                div { class: "status-card",
                    h3 { "Pod Count" }
                    p {
                        class: "status-value",
                        {
                            format!("{}/{}", resources.read().running_pods, resources.read().pod_count)
                        }
                    }
                    p { class: "status-subtext", "Running/Total" }
                }
                div { class: "status-card",
                    h3 { "Namespace Count" }
                    p {
                        class: "status-value",
                        { format!("{}", resources.read().namespace_count) }
                    }
                    p { class: "status-subtext", "Active namespaces" }
                }
            }

            // Resource Usage Section
            div { class: "resource-section",
                h2 { "Resource Usage" }
                div { class: "resource-grid",
                    div { class: "resource-card",
                        div { class: "resource-header",
                            h3 { class: "resource-title", "CPU Usage" }
                        }
                        div { class: "resource-stats",
                            div { class: "stat-item",
                                span {
                                    class: "stat-value",
                                    {
                                        let percentage = if resources.read().cpu_total > 0.0 {
                                            (resources.read().cpu_used / resources.read().cpu_total) * 100.0
                                        } else {
                                            0.0
                                        };
                                        format!("{:.1}%", percentage)
                                    }
                                }
                                span { class: "stat-label", "Utilization" }
                            }
                            div { class: "stat-item",
                                span {
                                    class: "stat-value",
                                    {
                                        format!("{:.1}/{:.1}",
                                            resources.read().cpu_used,
                                            resources.read().cpu_total)
                                    }
                                }
                                span { class: "stat-label", "Cores Used" }
                            }
                        }
                    }
                    div { class: "resource-card",
                        div { class: "resource-header",
                            h3 { class: "resource-title", "Memory Usage" }
                        }
                        div { class: "resource-stats",
                            div { class: "stat-item",
                                span {
                                    class: "stat-value",
                                    {
                                        let percentage = if resources.read().memory_total > 0.0 {
                                            (resources.read().memory_used / resources.read().memory_total) * 100.0
                                        } else {
                                            0.0
                                        };
                                        format!("{:.1}%", percentage)
                                    }
                                }
                                span { class: "stat-label", "Utilization" }
                            }
                            div { class: "stat-item",
                                span {
                                    class: "stat-value",
                                    {
                                        format!("{:.1}/{:.1}",
                                            resources.read().memory_used,
                                            resources.read().memory_total)
                                    }
                                }
                                span { class: "stat-label", "GB Used" }
                            }
                        }
                    }
                    div { class: "resource-card",
                        div { class: "resource-header",
                            h3 { class: "resource-title", "Storage Usage" }
                        }
                        div { class: "resource-stats",
                            div { class: "stat-item",
                                span {
                                    class: "stat-value",
                                    {
                                        let percentage = if resources.read().storage_total > 0.0 {
                                            (resources.read().storage_used / resources.read().storage_total) * 100.0
                                        } else {
                                            0.0
                                        };
                                        format!("{:.1}%", percentage)
                                    }
                                }
                                span { class: "stat-label", "Utilization" }
                            }
                            div { class: "stat-item",
                                span {
                                    class: "stat-value",
                                    {
                                        format!("{:.1}/{:.1}",
                                            resources.read().storage_used,
                                            resources.read().storage_total)
                                    }
                                }
                                span { class: "stat-label", "GB Used" }
                            }
                        }
                    }
                }
            }

            // Recent Events Section
            div { class: "events-section",
                h2 { "Recent Events" }
                div { class: "events-list",
                    {events.read().iter().map(|event| {
                        let reason = event.reason.as_deref().unwrap_or("Unknown");
                        let message = event.message.as_deref().unwrap_or("No message");
                        let namespace = event.metadata.namespace.as_deref().unwrap_or("default");
                        let involved_object = &event.involved_object;
                        let kind = involved_object.kind.as_deref().unwrap_or("unknown");
                        let name = involved_object.name.as_deref().unwrap_or("unknown");
                        let time = event.last_timestamp
                            .as_ref()
                            .or(event.first_timestamp.as_ref())
                            .map(|t| t.0.to_string())
                            .unwrap_or_else(|| "Unknown time".to_string());

                        let severity = event.type_.as_deref().unwrap_or("Normal");
                        let severity_class = match severity {
                            "Warning" => "event-warning",
                            "Normal" => "event-normal",
                            _ => "event-normal"
                        };

                        rsx! {
                            div {
                                class: format!("event-card {}", severity_class),
                                div { class: "event-line",
                                    span { class: "event-reason", "{reason}" }
                                    span { class: "event-object", "{kind}/{name}" }
                                    span { class: "event-namespace", "{namespace}" }
                                    span { class: "event-time", "{time}" }
                                }
                                p { class: "event-message", "{message}" }
                            }
                        }
                    })}
                }
            }
        }
    }
}
