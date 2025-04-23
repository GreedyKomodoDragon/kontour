use dioxus::prelude::*;
use std::collections::{HashMap, HashSet}; // Use HashMap for annotations/labels if needed

const INGRESSES_CSS: Asset = asset!("/assets/styling/ingresses.css"); // Link to the new CSS

// --- Data Structures ---

#[derive(Clone, PartialEq)]
struct IngressData {
    name: String,
    namespace: String,
    class_name: Option<String>, // IngressClass name
    rules: Vec<IngressRule>,
    tls: Vec<IngressTLS>,
    load_balancer_ips: Vec<String>, // IPs or hostnames assigned by the controller
    age: String,
    labels: Vec<(String, String)>,
    annotations: Vec<(String, String)>,
}

#[derive(Clone, PartialEq)]
struct IngressRule {
    host: Option<String>,
    paths: Vec<IngressPath>,
}

#[derive(Clone, PartialEq)]
struct IngressPath {
    path: String,
    path_type: String, // e.g., Prefix, Exact, ImplementationSpecific
    backend: IngressBackend,
}

#[derive(Clone, PartialEq)]
struct IngressBackend {
    service_name: String,
    service_port_name: Option<String>,
    service_port_number: Option<u32>,
    // Could also include Resource backend type later
}

#[derive(Clone, PartialEq)]
struct IngressTLS {
    hosts: Vec<String>,
    secret_name: String,
}

// --- Sample Data ---

fn get_sample_ingresses() -> Vec<IngressData> {
    vec![
        IngressData {
            name: "webapp-ingress".into(),
            namespace: "production".into(),
            class_name: Some("nginx-public".into()),
            rules: vec![
                IngressRule {
                    host: Some("example.com".into()),
                    paths: vec![
                        IngressPath {
                            path: "/app".into(),
                            path_type: "Prefix".into(),
                            backend: IngressBackend {
                                service_name: "webapp-service".into(),
                                service_port_name: None,
                                service_port_number: Some(8080),
                            },
                        },
                        IngressPath {
                            path: "/api".into(),
                            path_type: "Prefix".into(),
                            backend: IngressBackend {
                                service_name: "api-service".into(),
                                service_port_name: Some("http-api".into()),
                                service_port_number: None,
                            },
                        },
                    ],
                },
                IngressRule {
                    host: Some("admin.example.com".into()),
                    paths: vec![
                        IngressPath {
                            path: "/".into(),
                            path_type: "Prefix".into(),
                            backend: IngressBackend {
                                service_name: "admin-service".into(),
                                service_port_number: Some(80),
                                service_port_name: None,
                            },
                        },
                    ],
                },
            ],
            tls: vec![
                IngressTLS {
                    hosts: vec!["example.com".into(), "admin.example.com".into()],
                    secret_name: "example-tls-secret".into(),
                },
            ],
            load_balancer_ips: vec!["192.168.1.100".into()],
            age: "2h".into(),
            labels: vec![("app".into(), "webapp".into())],
            annotations: vec![("nginx.ingress.kubernetes.io/rewrite-target".into(), "/".into())],
        },
        IngressData {
            name: "default-backend-ingress".into(),
            namespace: "kube-system".into(),
            class_name: Some("internal-lb".into()),
            rules: vec![], // Example with default backend only (often configured via class)
            tls: vec![],
            load_balancer_ips: vec!["10.0.0.5".into()],
            age: "5d".into(),
            labels: vec![],
            annotations: vec![("kubernetes.io/ingress.class".into(), "internal-lb".into())],
            // A default backend might be implicitly defined or via annotations/class params
        },
         IngressData {
            name: "simple-fanout".into(),
            namespace: "default".into(),
            class_name: None, // Using default ingress controller
            rules: vec![
                 IngressRule {
                    host: None, // Applies to all hosts if not specified
                    paths: vec![
                        IngressPath {
                            path: "/foo".into(),
                            path_type: "Prefix".into(),
                            backend: IngressBackend {
                                service_name: "service-foo".into(),
                                service_port_number: Some(80),
                                service_port_name: None,
                            },
                        },
                         IngressPath {
                            path: "/bar".into(),
                            path_type: "Prefix".into(),
                            backend: IngressBackend {
                                service_name: "service-bar".into(),
                                service_port_number: Some(80),
                                service_port_name: None,
                            },
                        },
                    ],
                },
            ],
            tls: vec![],
            load_balancer_ips: vec!["pending".into()],
            age: "10m".into(),
            labels: vec![],
            annotations: vec![],
        },
    ]
}

// --- Component ---

#[component]
pub fn Ingresses() -> Element {
    let mut selected_namespace = use_signal(|| "all".to_string());
    let mut search_query = use_signal(String::new);
    let mut expanded_ingresses = use_signal(|| HashSet::<String>::new()); // Keyed by name+namespace? Just name for now.
    let ingresses = use_signal(get_sample_ingresses);

    // --- Filtering Logic ---
    let filtered_ingresses = {
        let ingresses = ingresses.clone();
        let selected_namespace = selected_namespace.clone();
        let search_query = search_query.clone();

        use_signal(move || {
            let ingresses = ingresses.read();
            let query = search_query.read().to_lowercase();
            let current_ns = selected_namespace.read();

            ingresses.iter()
                .filter(|&ing| {
                    let ns_match = *current_ns == "all" || ing.namespace == *current_ns;
                    let search_match = query.is_empty()
                        || ing.name.to_lowercase().contains(&query)
                        || ing.namespace.to_lowercase().contains(&query)
                        || ing.rules.iter().any(|r| r.host.as_deref().unwrap_or("").to_lowercase().contains(&query)) // Search hosts
                        || ing.load_balancer_ips.iter().any(|ip| ip.to_lowercase().contains(&query)); // Search IPs
                    ns_match && search_match
                })
                .cloned()
                .collect::<Vec<_>>()
        })
    };

    // --- Unique Namespaces for Filter ---
    let namespaces = use_memo(move || {
        let mut ns = ingresses.read().iter().map(|d| d.namespace.clone()).collect::<HashSet<_>>().into_iter().collect::<Vec<_>>();
        ns.sort();
        ns
    });

    rsx! {
        document::Link { rel: "stylesheet", href: INGRESSES_CSS }
        div { class: "ingresses-container", // Use new CSS classes
            // --- Header ---
            div { class: "ingresses-header",
                div { class: "header-left",
                    h1 { "Ingresses" }
                    div { class: "header-controls",
                        // Search Input
                        div { class: "search-container",
                            input {
                                class: "search-input",
                                r#type: "text",
                                placeholder: "Search ingresses...",
                                value: "{search_query}",
                                oninput: move |evt| search_query.set(evt.value()),
                            }
                        }
                        // Namespace Select
                        select {
                            class: "namespace-select",
                            value: "{selected_namespace.read()}",
                            onchange: move |evt| selected_namespace.set(evt.value()),
                            option { value: "all", "All Namespaces" }
                            {namespaces.read().iter().map(|ns| rsx!{
                                option { key: "{ns}", value: "{ns}", "{ns}" }
                            })}
                        }
                        // Count
                        span { class: "ingress-count", "{filtered_ingresses.read().len()} ingresses" }
                    }
                }
                // Header Actions
                div { class: "header-actions",
                    button { class: "btn btn-primary", "Create Ingress" } // Placeholder
                    button { class: "btn btn-secondary", "Refresh" } // Placeholder
                }
            }

            // --- Ingresses Grid ---
            div { class: "ingresses-grid",
                {filtered_ingresses.read().iter().map(|ing| {
                    let ingress_key = format!("{}-{}", ing.namespace, ing.name); // Unique key
                    let is_expanded = expanded_ingresses.read().contains(&ingress_key);
                    let ingress_key_clone = ingress_key.clone();

                    // Determine status based on LB IPs?
                    let status_class = if ing.load_balancer_ips.is_empty() || ing.load_balancer_ips.contains(&"pending".to_string()) {
                        "status-pending"
                    } else {
                        "status-running" // Assuming assigned IP means active
                    };
                     let display_hosts = ing.rules.iter()
                        .filter_map(|r| r.host.as_ref())
                        .map(|h| h.as_str())
                        .collect::<Vec<_>>()
                        .join(", ");
                     let display_hosts_truncated = if display_hosts.len() > 50 {
                         format!("{}...", &display_hosts[..47])
                     } else {
                         display_hosts.clone()
                     };

                    // Calculate status text beforehand
                    let status_text = if ing.load_balancer_ips.is_empty() {
                        "No LB".to_string()
                    } else {
                        ing.load_balancer_ips.join(", ")
                    };


                    rsx! {
                        div {
                            key: "{ingress_key}",
                            class: "ingress-card",
                            // --- Card Header ---
                            div {
                                class: "ingress-header-card",
                                div { class: "ingress-title",
                                    h3 { "{ing.name}" }
                                    // Use the pre-calculated status_text variable
                                    span { class: "status-badge {status_class}", "{status_text}" }
                                }
                                div { class: "ingress-info-short", // Show key info in header
                                     span { class: "info-item-short", title: "Namespace", "{ing.namespace}" }
                                     span { class: "info-item-short", title: "Class", "{ing.class_name.as_deref().unwrap_or(\"<default>\")}" }
                                     span { class: "info-item-short", title: "Hosts", "{display_hosts_truncated}" }
                                }
                                div { class: "ingress-controls",
                                    // Expand/Collapse Button
                                    button {
                                        class: "btn-icon expand-toggle",
                                        onclick: move |evt| {
                                            evt.stop_propagation();
                                            let mut set = expanded_ingresses.write();
                                            if set.contains(&ingress_key_clone) {
                                                set.remove(&ingress_key_clone);
                                            } else {
                                                set.insert(ingress_key_clone.clone());
                                            }
                                        },
                                        title: if is_expanded { "Collapse" } else { "Expand" },
                                        if is_expanded { "🔼" } else { "🔽" }
                                    }
                                    // Placeholder Action Buttons
                                    button { class: "btn-icon", onclick: move |evt| evt.stop_propagation(), title: "Edit", "✏️" }
                                    button { class: "btn-icon", onclick: move |evt| evt.stop_propagation(), title: "Delete", "🗑️" }
                                }
                            }

                            // --- Expanded Details ---
                            {is_expanded.then(|| rsx! {
                                div { class: "ingress-details",
                                    // Basic Info Section
                                    div { class: "info-section",
                                        h4 { "Basic Information" }
                                        div { class: "info-grid",
                                            div { class: "info-item", span { class: "info-label", "Namespace" } span { class: "info-value", "{ing.namespace}" } }
                                            div { class: "info-item", span { class: "info-label", "Class" } span { class: "info-value", "{ing.class_name.as_deref().unwrap_or(\"<default>\")}" } }
                                            div { class: "info-item", span { class: "info-label", "Age" } span { class: "info-value", "{ing.age}" } }
                                            div { class: "info-item", span { class: "info-label", "LoadBalancer IPs" } span { class: "info-value", "{ing.load_balancer_ips.join(\", \")}" } }
                                        }
                                    }

                                    // Labels Section
                                    {(!ing.labels.is_empty()).then(|| rsx! {
                                        div { class: "labels-section",
                                            h4 { "Labels" }
                                            div { class: "labels-grid",
                                                {ing.labels.iter().map(|(key, value)| rsx! {
                                                    div { key: "lbl-{key}", class: "label", span { class: "label-key", "{key}" } span { class: "label-value", "{value}" } }
                                                })}
                                            }
                                        }
                                    })}

                                     // Annotations Section
                                    {(!ing.annotations.is_empty()).then(|| rsx! {
                                        div { class: "labels-section", // Reuse label styling
                                            h4 { "Annotations" }
                                            div { class: "labels-grid",
                                                {ing.annotations.iter().map(|(key, value)| rsx! {
                                                    div { key: "anno-{key}", class: "label annotation", // Add annotation class if specific styling needed
                                                        span { class: "label-key", "{key}" }
                                                        span { class: "label-value", "{value}" }
                                                    }
                                                })}
                                            }
                                        }
                                    })}

                                    // TLS Section
                                    {(!ing.tls.is_empty()).then(|| rsx! {
                                        div { class: "tls-section",
                                            h4 { "TLS" }
                                            div { class: "tls-grid",
                                                {ing.tls.iter().map(|tls_item| rsx! {
                                                    div { class: "tls-item",
                                                        span { class: "tls-secret", "Secret: {tls_item.secret_name}" }
                                                        span { class: "tls-hosts", "Hosts: {tls_item.hosts.join(\", \")}" }
                                                    }
                                                })}
                                            }
                                        }
                                    })}

                                    // Rules Section
                                    {(!ing.rules.is_empty()).then(|| rsx! {
                                        div { class: "rules-section",
                                            h4 { "Rules" }
                                            {ing.rules.iter().map(|rule| rsx! {
                                                div { class: "rule-item",
                                                    div { class: "rule-host", "Host: {rule.host.as_deref().unwrap_or(\"*\")}" }
                                                    div { class: "paths-grid",
                                                        {rule.paths.iter().map(|path| rsx! {
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
                                                                        rsx!{ "<error>" } // Should have one or the other
                                                                    }}
                                                                }
                                                            }
                                                        })}
                                                    }
                                                }
                                            })}
                                        }
                                    })}
                                    // Default Backend (if applicable and not in rules) - Needs logic
                                    // { if ing.rules.is_empty() && ing.default_backend.is_some() ... }
                                }
                            })}
                        }
                    }
                })}
            }
        }
    }
}
