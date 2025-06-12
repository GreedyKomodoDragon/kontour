use dioxus::prelude::*;
use k8s_openapi::api::networking::v1::Ingress;

#[derive(Clone)]
struct IngressData {
    name: String,
    namespace: String,
    class_name: Option<String>,
    age: String,
    labels: Vec<(String, String)>,
    annotations: Vec<(String, String)>,
    rules: Vec<IngressRule>,
    tls: Vec<IngressTls>,
    load_balancer_ips: Vec<String>,
}

#[derive(Clone)]
struct IngressRule {
    host: Option<String>,
    paths: Vec<IngressPath>,
}

#[derive(Clone)]
struct IngressPath {
    path: String,
    path_type: String,
    backend: IngressBackend,
}

#[derive(Clone)]
struct IngressBackend {
    service_name: String,
    service_port_name: Option<String>,
    service_port_number: Option<i32>,
}

#[derive(Clone)]
struct IngressTls {
    hosts: Vec<String>,
    secret_name: String,
}

#[derive(Props, PartialEq, Clone)]
pub struct IngressItemProps {
    ingress: Ingress,
}

#[component]
pub fn IngressItem(props: IngressItemProps) -> Element {
    let mut is_expanded = use_signal(|| false);

    let ingress_data = IngressData {
        name: props.ingress.metadata.name.clone().unwrap_or_default(),
        namespace: props.ingress.metadata.namespace.clone().unwrap_or_default(),
        class_name: props.ingress.spec.as_ref()
            .and_then(|s| s.ingress_class_name.clone()),
        age: "1h".to_string(), // TODO: Calculate age
        labels: props.ingress.metadata.labels.as_ref()
            .map(|labels| labels.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default(),
        annotations: props.ingress.metadata.annotations.as_ref()
            .map(|annotations| annotations.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default(),
        rules: props.ingress.spec.as_ref()
            .and_then(|s| s.rules.as_ref())
            .map(|rules| {
                rules.iter().map(|rule| IngressRule {
                    host: rule.host.clone(),
                    paths: rule.http.as_ref().map_or(Vec::new(), |http| {
                        http.paths.iter().map(|path| IngressPath {
                            path: path.path.clone().unwrap_or_default(),
                            path_type: path.path_type.clone(),
                            backend: IngressBackend {
                                service_name: path.backend.service.as_ref()
                                    .map(|svc| svc.name.clone())
                                    .unwrap_or_default(),
                                service_port_name: path.backend.service.as_ref()
                                    .and_then(|svc| svc.port.as_ref())
                                    .and_then(|port| port.name.clone()),
                                service_port_number: path.backend.service.as_ref()
                                    .and_then(|svc| svc.port.as_ref())
                                    .and_then(|port| port.number),
                            },
                        }).collect()
                    }),
                }).collect()
            })
            .unwrap_or_default(),
        tls: props.ingress.spec.as_ref()
            .and_then(|s| s.tls.as_ref())
            .map(|tls_list| {
                tls_list.iter().map(|tls| IngressTls {
                    hosts: tls.hosts.clone().unwrap_or_default(),
                    secret_name: tls.secret_name.clone().unwrap_or_default(),
                }).collect()
            })
            .unwrap_or_default(),
        load_balancer_ips: props.ingress.status.as_ref()
            .and_then(|s| s.load_balancer.as_ref())
            .and_then(|lb| lb.ingress.as_ref())
            .map(|ingress| {
                ingress.iter()
                    .filter_map(|ing| ing.ip.clone().or(ing.hostname.clone()))
                    .collect()
            })
            .unwrap_or_default(),
    };

    let status_class = if ingress_data.load_balancer_ips.is_empty() {
        "status-pending"
    } else if ingress_data.load_balancer_ips.iter().any(|ip| ip == "pending") {
        "status-pending"
    } else {
        "status-running"
    };

    let display_hosts = ingress_data.rules.iter()
        .filter_map(|r| r.host.as_ref())
        .map(|h| h.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    let display_hosts_truncated = if display_hosts.len() > 50 {
        format!("{}...", &display_hosts[..47])
    } else {
        display_hosts
    };

    let status_display = if ingress_data.load_balancer_ips.is_empty() {
        "No LB".to_string()
    } else {
        ingress_data.load_balancer_ips.join(", ")
    };

    rsx! {
        div {
            key: "{ingress_data.name}",
            class: "ingress-card",
            div {
                class: "ingress-header-card",
                div { class: "ingress-title",
                    h3 { "{ingress_data.name}" }
                    span { 
                        class: "status-badge {status_class}",
                        "{status_display}"
                    }
                }
                div { class: "ingress-info-short",
                    span { class: "info-item-short", title: "Namespace", "{ingress_data.namespace}" }
                    span { class: "info-item-short", title: "Class", "{ingress_data.class_name.as_deref().unwrap_or(\"<default>\")}" }
                    span { class: "info-item-short", title: "Hosts", "{display_hosts_truncated}" }
                }
                div { class: "ingress-controls",
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
                div { class: "ingress-details",
                    // Basic Info Section
                    div { class: "info-section",
                        h4 { "Basic Information" }
                        div { class: "info-grid",
                            div { class: "info-item", span { class: "info-label", "Namespace" } span { class: "info-value", "{ingress_data.namespace}" } }
                            div { class: "info-item", span { class: "info-label", "Class" } span { class: "info-value", "{ingress_data.class_name.as_deref().unwrap_or(\"<default>\")}" } }
                            div { class: "info-item", span { class: "info-label", "Age" } span { class: "info-value", "{ingress_data.age}" } }
                            div { class: "info-item", span { class: "info-label", "LoadBalancer IPs" } span { class: "info-value", "{ingress_data.load_balancer_ips.join(\", \")}" } }
                        }
                    }

                    // Labels Section
                    {(!ingress_data.labels.is_empty()).then(|| rsx! {
                        div { class: "labels-section",
                            h4 { "Labels" }
                            div { class: "labels-grid",
                                {ingress_data.labels.iter().map(|(key, value)| {
                                    rsx!(
                                        div {
                                            key: "lbl-{key}",
                                            class: "label",
                                            span { class: "label-key", "{key}" }
                                            span { class: "label-value", "{value}" }
                                        }
                                    )
                                })}
                            }
                        }
                    })}

                    // Annotations Section
                    {(!ingress_data.annotations.is_empty()).then(|| rsx! {
                        div { class: "labels-section",
                            h4 { "Annotations" }
                            div { class: "labels-grid",
                                {ingress_data.annotations.iter().map(|(key, value)| {
                                    rsx!(
                                        div {
                                            key: "anno-{key}",
                                            class: "label annotation",
                                            span { class: "label-key", "{key}" }
                                            span { class: "label-value", "{value}" }
                                        }
                                    )
                                })}
                            }
                        }
                    })}

                    // TLS Section
                    {(!ingress_data.tls.is_empty()).then(|| rsx! {
                        div { class: "tls-section",
                            h4 { "TLS" }
                            div { class: "tls-grid",
                                {ingress_data.tls.iter().map(|tls| {
                                    rsx!(
                                        div { class: "tls-item",
                                            span { class: "tls-secret", "Secret: {tls.secret_name}" }
                                            span { class: "tls-hosts", "Hosts: {tls.hosts.join(\", \")}" }
                                        }
                                    )
                                })}
                            }
                        }
                    })}

                    // Rules Section
                    {(!ingress_data.rules.is_empty()).then(|| rsx! {
                        div { class: "rules-section",
                            h4 { "Rules" }
                            {ingress_data.rules.iter().map(|rule| {
                                rsx!(
                                    div { class: "rule-item",
                                        div { class: "rule-host", "Host: {rule.host.as_deref().unwrap_or(\"*\")}" }
                                        div { class: "paths-grid",
                                            {rule.paths.iter().map(|path| {
                                                rsx!(
                                                    div { class: "path-item",
                                                        span { class: "path-info", "{path.path} ({path.path_type})" }
                                                        span { class: "path-arrow", "→" }
                                                        span { class: "path-backend",
                                                            "{path.backend.service_name}:"
                                                            {if let Some(port_name) = &path.backend.service_port_name {
                                                                rsx!{ "{port_name}" }
                                                            } else if let Some(port_num) = path.backend.service_port_number {
                                                                rsx!{ "{port_num}" }
                                                            } else {
                                                                rsx!{ "<e>" }
                                                            }}
                                                        }
                                                    }
                                                )
                                            })}
                                        }
                                    }
                                )
                            })}
                        }
                    })}
                }
            })}
        }
    }
}