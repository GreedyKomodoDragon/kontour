use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct NodeItemProps {
    pub name: String,
    pub node_type: String,
    pub status: String,
    pub kubernetes_version: String,
    pub os: String,
    pub architecture: String,
    pub ip: String,
    pub pods: (u32, u32),
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub storage_usage: f32,
}

#[component]
pub fn NodeItem(props: NodeItemProps) -> Element {
    rsx! {
        div {
            key: "{props.name}",
            class: "node-card",
            div { class: "node-header",
                div { class: "node-title",
                    h3 { "{props.name}" }
                    span { class: "status-badge status-healthy", "{props.status}" }
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
                    div { class: "node-progress-bar",
                        div {
                            class: "progress-fill",
                            style: "width: {props.cpu_usage}%"
                        }
                    }
                    span { class: "metric-value", "{props.cpu_usage}%" }
                }
                div { class: "metric",
                    span { class: "metric-label", "Memory" }
                    div { class: "node-progress-bar",
                        div {
                            class: "progress-fill",
                            style: "width: {props.memory_usage}%"
                        }
                    }
                    span { class: "metric-value", "{props.memory_usage}%" }
                }
                div { class: "metric",
                    span { class: "metric-label", "Storage" }
                    div { class: "node-progress-bar",
                        div {
                            class: "progress-fill",
                            style: "width: {props.storage_usage}%"
                        }
                    }
                    span { class: "metric-value", "{props.storage_usage}%" }
                }
            }

            div { class: "node-info",
                div { class: "info-group",
                    div { class: "info-item",
                        span { class: "info-label", "Kubernetes Version" }
                        span { class: "info-value", "{props.kubernetes_version}" }
                    }
                    div { class: "info-item",
                        span { class: "info-label", "OS" }
                        span { class: "info-value", "{props.os}" }
                    }
                    div { class: "info-item",
                        span { class: "info-label", "Architecture" }
                        span { class: "info-value", "{props.architecture}" }
                    }
                }
                div { class: "info-group",
                    div { class: "info-item",
                        span { class: "info-label", "Internal IP" }
                        span { class: "info-value", "{props.ip}" }
                    }
                    div { class: "info-item",
                        span { class: "info-label", "Pods" }
                        span { class: "info-value", "{props.pods.0}/{props.pods.1}" }
                    }
                }
            }

            div { class: "node-conditions",
                h4 { "Node Conditions" }
                div { class: "conditions-list",
                    div { class: "condition status-healthy",
                        span { class: "node-condition-type", "Ready" }
                        span { class: "node-condition-status", "True" }
                    }
                    div { class: "condition status-healthy",
                        span { class: "node-condition-type", "MemoryPressure" }
                        span { class: "node-condition-status", "False" }
                    }
                    div { class: "condition status-healthy",
                        span { class: "node-condition-type", "DiskPressure" }
                        span { class: "node-condition-status", "False" }
                    }
                    div { class: "condition status-healthy",
                        span { class: "node-condition-type", "PIDPressure" }
                        span { class: "node-condition-status", "False" }
                    }
                }
            }
        }
    }
}
