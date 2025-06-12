use dioxus::prelude::*;
use k8s_openapi::api::core::v1::Service;

#[derive(Clone)]
struct ServiceData {
    name: String,
    namespace: String,
    age: String,
    service_type: String,
    cluster_ip: String,
    external_ip: String,
    ports: Vec<ServicePort>,
    labels: Vec<(String, String)>,
    selector: Vec<(String, String)>,
}

#[derive(Clone)]
struct ServicePort {
    name: Option<String>,
    protocol: String,
    port: i32,
    target_port: String,
    node_port: Option<i32>,
}

#[derive(Props, PartialEq, Clone)]
pub struct ServiceItemProps {
    service: Service,
}

#[component]
pub fn ServiceItem(props: ServiceItemProps) -> Element {
    let mut is_expanded = use_signal(|| false);

    let service_data = ServiceData {
        name: props.service.metadata.name.clone().unwrap_or_default(),
        namespace: props.service.metadata.namespace.clone().unwrap_or_default(),
        age: "1h".to_string(), // TODO: Calculate age
        service_type: props.service.spec.as_ref()
            .and_then(|s| s.type_.as_ref())
            .map(|t| t.as_str())
            .unwrap_or("ClusterIP")
            .to_string(),
        cluster_ip: props.service.spec.as_ref()
            .and_then(|s| s.cluster_ip.as_ref())
            .cloned()
            .unwrap_or_else(|| "None".to_string()),
        external_ip: props.service.status.as_ref()
            .and_then(|s| s.load_balancer.as_ref())
            .and_then(|lb| lb.ingress.as_ref())
            .and_then(|ingress| ingress.first())
            .and_then(|ing| ing.ip.as_ref())
            .cloned()
            .unwrap_or_else(|| "None".to_string()),
        ports: props.service.spec.as_ref()
            .and_then(|s| s.ports.as_ref())
            .map(|ports| {
                ports.iter().map(|p| ServicePort {
                    name: p.name.clone(),
                    protocol: p.protocol.clone().unwrap_or_default(),
                    port: p.port,
                    target_port: p.target_port.as_ref().map(|t| match t {
                        k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::Int(i) => i.to_string(),
                        k8s_openapi::apimachinery::pkg::util::intstr::IntOrString::String(s) => s.clone(),
                    }).unwrap_or_default(),
                    node_port: p.node_port,
                }).collect()
            })
            .unwrap_or_default(),
        labels: props.service.metadata.labels.as_ref()
            .map(|labels| labels.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default(),
        selector: props.service.spec.as_ref()
            .and_then(|s| s.selector.as_ref())
            .map(|selector| selector.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default(),
    };

    let status_class = match service_data.service_type.as_str() {
        "LoadBalancer" => "status-running",
        "NodePort" => "status-pending",
        _ => "status-unknown",
    };

    rsx! {
        div {
            key: "{service_data.name}",
            class: "service-card",
            div {
                class: "service-header-card",
                div { class: "service-title",
                    h3 { "{service_data.name}" }
                    span { class: "status-badge {status_class}", "{service_data.service_type}" }
                }
                div { class: "service-controls",
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

            {is_expanded().then(|| rsx! {
                div { class: "service-details",
                    // Basic Info Section
                    div { class: "service-info",
                        div { class: "info-group",
                            div { class: "info-item", 
                                span { class: "info-label", "Namespace" } 
                                span { class: "info-value", "{service_data.namespace}" }
                            }
                            div { class: "info-item", 
                                span { class: "info-label", "Type" } 
                                span { class: "info-value", "{service_data.service_type}" }
                            }
                        }
                        div { class: "info-group",
                            div { class: "info-item", 
                                span { class: "info-label", "Cluster IP" } 
                                span { class: "info-value", "{service_data.cluster_ip}" }
                            }
                            div { class: "info-item", 
                                span { class: "info-label", "External IP" } 
                                span { class: "info-value", "{service_data.external_ip}" }
                            }
                        }
                    }

                    // Labels Section
                    div { class: "labels-section",
                        h4 { "Labels" }
                        div { class: "labels-grid",
                            {service_data.labels.is_empty().then(|| rsx! {
                                span { class: "info-value", i { "No labels" } }
                            })}
                            {service_data.labels.iter().map(|(key, value)| {
                                rsx!(
                                    div {
                                        key: "{key}",
                                        class: "label",
                                        span { class: "label-key", "{key}" }
                                        span { class: "label-value", "{value}" }
                                    }
                                )
                            })}
                        }
                    }

                    // Selector Section
                    div { class: "labels-section",
                        h4 { "Selector" }
                        div { class: "labels-grid",
                            {service_data.selector.is_empty().then(|| rsx! {
                                span { class: "info-value", i { "No selector" } }
                            })}
                            {service_data.selector.iter().map(|(key, value)| {
                                rsx!(
                                    div {
                                        key: "sel-{key}",
                                        class: "label",
                                        span { class: "label-key", "{key}" }
                                        span { class: "label-value", "{value}" }
                                    }
                                )
                            })}
                        }
                    }

                    // Ports Section
                    div { class: "ports-section",
                        h4 { "Ports" }
                        div { class: "ports-grid",
                            {service_data.ports.is_empty().then(|| rsx! {
                                span { class: "info-value", i { "No ports" } }
                            })}
                            {service_data.ports.iter().map(|port| {
                                rsx!(
                                    div {
                                        key: "{port.port}",
                                        class: "port-item",
                                        span { class: "port-detail port-name", "{port.name.as_deref().unwrap_or(\"-\")}" }
                                        span { class: "port-detail port-number", "{port.port}" }
                                        span { class: "port-detail port-protocol", "{port.protocol}" }
                                        span { class: "port-detail port-target", "→ {port.target_port}" }
                                        {port.node_port.map(|np| rsx!{ 
                                            span { class: "port-detail port-nodeport", "(Node: {np})" }
                                        })}
                                    }
                                )
                            })}
                        }
                    }
                }
            })}
        }
    }
}