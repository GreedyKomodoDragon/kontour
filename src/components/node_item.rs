use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct NodeCondition {
    pub condition_type: String,
    pub status: String,
}

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
    pub conditions: Vec<NodeCondition>,
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
                    span { class: "status-badge status-unknown", "{props.status}" }
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
                    {
                        props.conditions.iter().map(|condition| {
                            let status_class = match (condition.condition_type.as_str(), condition.status.as_str()) {
                                ("Ready", "True") => "status-healthy",
                                ("Ready", _) => "status-critical",
                                (_, "True") => "status-critical",
                                (_, "False") => "status-healthy",
                                _ => "status-warning"
                            };

                            rsx! {
                                div {
                                    class: format!("condition {}", status_class),
                                    span {
                                        class: "node-condition-type",
                                        "{condition.condition_type}"
                                    }
                                    span {
                                        class: "node-condition-status",
                                        "{condition.status}"
                                    }
                                }
                            }
                        })
                    }
                }
            }
        }
    }
}
