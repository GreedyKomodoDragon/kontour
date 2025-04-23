use dioxus::prelude::*;

const PODS_CSS: Asset = asset!("/assets/styling/pods.css");

#[derive(Clone)]
struct PodData {
    name: String,
    namespace: String,
    status: String,
    phase: String,
    age: String,
    ready_containers: (u32, u32), // (ready, total)
    restart_count: u32,
    ip: String,
    node: String,
    qos_class: String,
    containers: Vec<ContainerData>,
    conditions: Vec<PodCondition>,
    labels: Vec<(String, String)>,
}

#[derive(Clone)]
struct ContainerData {
    name: String,
    image: String,
    status: String,
    restarts: u32,
    cpu_usage: f32,
    memory_usage: String,
    memory_limit: String,
}

#[derive(Clone)]
struct PodCondition {
    condition_type: String,
    status: String,
    last_transition: String,
    reason: String,
}

#[component]
pub fn Pods() -> Element {
    let selected_status = use_signal(|| "all");
    let selected_namespace = use_signal(|| "all");
    let search_query = use_signal(String::new);
    let mut expanded_pods = use_signal(|| std::collections::HashSet::<String>::new()); // Added `mut`
    let pods = use_signal(|| vec![
        PodData {
            name: "nginx-deployment-6b474476c4-x8t7r".into(),
            namespace: "default".into(),
            status: "Running".into(),
            phase: "Running".into(),
            age: "2d".into(),
            ready_containers: (1, 1),
            restart_count: 0,
            ip: "10.244.0.15".into(),
            node: "worker-1".into(),
            qos_class: "Burstable".into(),
            containers: vec![
                ContainerData {
                    name: "nginx".into(),
                    image: "nginx:1.25".into(),
                    status: "Running".into(),
                    restarts: 0,
                    cpu_usage: 0.15,
                    memory_usage: "32Mi".into(),
                    memory_limit: "128Mi".into(),
                }
            ],
            conditions: vec![
                PodCondition {
                    condition_type: "Ready".into(),
                    status: "True".into(),
                    last_transition: "2d".into(),
                    reason: "PodHasBeenReady".into(),
                },
                PodCondition {
                    condition_type: "ContainersReady".into(),
                    status: "True".into(),
                    last_transition: "2d".into(),
                    reason: "ContainersAreReady".into(),
                },
            ],
            labels: vec![
                ("app".into(), "nginx".into()),
                ("pod-template-hash".into(), "6b474476c4".into()),
            ],
        },
        PodData {
            name: "prometheus-7d7d4769b8-zk9j5".into(),
            namespace: "monitoring".into(),
            status: "Running".into(),
            phase: "Running".into(),
            age: "5d".into(),
            ready_containers: (2, 2),
            restart_count: 1,
            ip: "10.244.0.18".into(),
            node: "worker-2".into(),
            qos_class: "Guaranteed".into(),
            containers: vec![
                ContainerData {
                    name: "prometheus".into(),
                    image: "prom/prometheus:v2.45".into(),
                    status: "Running".into(),
                    restarts: 1,
                    cpu_usage: 0.45,
                    memory_usage: "750Mi".into(),
                    memory_limit: "1Gi".into(),
                },
                ContainerData {
                    name: "config-reloader".into(),
                    image: "jimmidyson/configmap-reload:v0.8.0".into(),
                    status: "Running".into(),
                    restarts: 0,
                    cpu_usage: 0.05,
                    memory_usage: "12Mi".into(),
                    memory_limit: "25Mi".into(),
                }
            ],
            conditions: vec![
                PodCondition {
                    condition_type: "Ready".into(),
                    status: "True".into(),
                    last_transition: "5d".into(),
                    reason: "PodHasBeenReady".into(),
                },
                PodCondition {
                    condition_type: "ContainersReady".into(),
                    status: "True".into(),
                    last_transition: "5d".into(),
                    reason: "ContainersAreReady".into(),
                },
            ],
            labels: vec![
                ("app".into(), "prometheus".into()),
                ("pod-template-hash".into(), "7d7d4769b8".into()),
            ],
        },
    ]);

    let filtered_pods = {
        let pods = pods.clone();
        let selected_status = selected_status.clone();
        let selected_namespace = selected_namespace.clone();

        use_signal(move || {
            let pods = pods.read();
            pods.iter()
                .filter(|&pod| {
                    (selected_status() == "all" || pod.status == selected_status()) &&
                    (selected_namespace() == "all" || pod.namespace == selected_namespace())
                })
                .cloned() // if needed
                .collect::<Vec<_>>()
        })
    };

    rsx! {
        document::Link { rel: "stylesheet", href: PODS_CSS }
        // Add Tailwind: background color
        div { class: "pods-container", // Test Tailwind bg
            // Add Tailwind: border
            div { class: "pods-header", // Test Tailwind border
                div { class: "header-left",
                    // Add Tailwind: text color
                    h1 { class: "text-yellow-300", "Pods" } // Test Tailwind text color
                    div { class: "header-controls",
                        div { class: "search-container",
                            input {
                                class: "search-input",
                                r#type: "text",
                                placeholder: "Search pods...",
                                value: "{search_query}"
                            }
                        }
                        select {
                            class: "namespace-select",
                            value: "{selected_namespace.read()}",
                            option { value: "all", "All Namespaces" }
                            option { value: "default", "default" }
                            option { value: "monitoring", "monitoring" }
                        }
                        select {
                            class: "status-select",
                            value: "{selected_status.read()}",
                            option { value: "all", "All Statuses" }
                            option { value: "Running", "Running" }
                            option { value: "Pending", "Pending" }
                            option { value: "Failed", "Failed" }
                            option { value: "Succeeded", "Succeeded" }
                        }
                        span { class: "pod-count", "{filtered_pods.read().len()} pods" }
                    }
                }
                div { class: "header-actions",
                    // Add Tailwind: hover effect
                    button { class: "btn btn-primary hover:bg-yellow-600", "Create Pod" } // Test Tailwind hover
                    button { class: "btn btn-secondary", "Refresh" }
                }
            }

            div { class: "pods-grid",
                {filtered_pods.read().iter().map(|pod| {
                    let is_expanded = expanded_pods.read().contains(&pod.name);
                    let pod_name_clone = pod.name.clone();
                    rsx! {
                        // Add Tailwind: padding
                        div {
                            key: "{pod.name}",
                            class: "pod-card p-10", // Test Tailwind padding
                            div { 
                                class: "pod-header",
                                // Optional: Keep header click commented or remove if only button should toggle
                                // onclick: move |_| {
                                //     let mut set = expanded_pods.write();
                                //     if set.contains(&pod.name) {
                                //         set.remove(&pod.name);
                                //     } else {
                                //         set.insert(pod.name.clone());
                                //     }
                                // },
                                div { class: "pod-title",
                                    h3 { "{pod.name}" }
                                    span { class: "status-badge status-{pod.status.to_lowercase()}", "{pod.status}" }
                                }
                                div { class: "pod-controls",
                                    button { 
                                        class: "btn-icon expand-toggle",
                                        onclick: move |evt| { 
                                            evt.stop_propagation();
                                            // Now this is allowed because expanded_pods is mutable
                                            let mut set = expanded_pods.write(); 
                                            if set.contains(&pod_name_clone) {
                                                set.remove(&pod_name_clone);
                                            } else {
                                                set.insert(pod_name_clone.clone());
                                            }
                                        },
                                        title: if is_expanded { "Collapse" } else { "Expand" },
                                        if is_expanded { "🔼" } else { "🔽" }
                                    }
                                    button { 
                                        class: "btn-icon", 
                                        onclick: move |evt| evt.stop_propagation(),
                                        title: "View Logs", 
                                        "📄" 
                                    }
                                    button { 
                                        class: "btn-icon", 
                                        onclick: move |evt| evt.stop_propagation(),
                                        title: "Shell", 
                                        "🖥️" 
                                    }
                                    button { 
                                        class: "btn-icon", 
                                        onclick: move |evt| evt.stop_propagation(),
                                        title: "Delete", 
                                        "🗑️" 
                                    }
                                }
                            }

                            {is_expanded.then(|| rsx! {
                                div { class: "pod-details",
                                    div { class: "pod-info",
                                        div { class: "info-group",
                                            div { class: "info-item",
                                                span { class: "info-label", "Namespace" }
                                                span { class: "info-value", "{pod.namespace}" }
                                            }
                                            div { class: "info-item",
                                                span { class: "info-label", "Node" }
                                                span { class: "info-value", "{pod.node}" }
                                            }
                                            div { class: "info-item",
                                                span { class: "info-label", "IP" }
                                                span { class: "info-value", "{pod.ip}" }
                                            }
                                        }
                                        div { class: "info-group",
                                            div { class: "info-item",
                                                span { class: "info-label", "Age" }
                                                span { class: "info-value", "{pod.age}" }
                                            }
                                            div { class: "info-item",
                                                span { class: "info-label", "QoS Class" }
                                                span { class: "info-value", "{pod.qos_class}" }
                                            }
                                            div { class: "info-item",
                                                span { class: "info-label", "Restarts" }
                                                span { class: "info-value", "{pod.restart_count}" }
                                            }
                                        }
                                    }

                                    div { class: "labels-section",
                                        h4 { "Labels" }
                                        div { class: "labels-grid",
                                            {pod.labels.iter().map(|(key, value)| rsx! {
                                                div {
                                                    key: "{key}",
                                                    class: "label",
                                                    span { class: "label-key", "{key}" }
                                                    span { class: "label-value", "{value}" }
                                                }
                                            })}
                                        }
                                    }

                                    div { class: "containers-section",
                                        h4 { "Containers ({pod.ready_containers.0}/{pod.ready_containers.1})" }
                                        div { class: "containers-grid",
                                            {pod.containers.iter().map(|container| rsx! {
                                                div {
                                                    key: "{container.name}",
                                                    class: "container-card",
                                                    div { class: "container-header",
                                                        div { class: "container-title",
                                                            h5 { "{container.name}" }
                                                            span { class: "container-image", "{container.image}" }
                                                        }
                                                        span { class: "status-badge status-{container.status.to_lowercase()}", "{container.status}" }
                                                    }
                                                    div { class: "resource-metrics",
                                                        div { class: "metric",
                                                            span { class: "metric-label", "CPU" }
                                                            div { class: "progress-bar",
                                                                div {
                                                                    class: "progress-fill",
                                                                    style: "width: {container.cpu_usage * 100.0}%"
                                                                }
                                                            }
                                                            span { class: "metric-value", "{container.cpu_usage * 100.0:.1}%" }
                                                        }
                                                        div { class: "metric",
                                                            span { class: "metric-label", "Memory" }
                                                            div { class: "progress-bar",
                                                                div {
                                                                    class: "progress-fill",
                                                                    style: "width: {(container.memory_usage.replace(\"Gi\", \"\").replace(\"Mi\", \"\").parse::<f32>().unwrap() / container.memory_limit.replace(\"Gi\", \"\").replace(\"Mi\", \"\").parse::<f32>().unwrap() * 100.0)}%"
                                                                }
                                                            }
                                                            span { class: "metric-value", "{container.memory_usage}/{container.memory_limit}" }
                                                        }
                                                    }
                                                }
                                            })}
                                        }
                                    }

                                    div { class: "conditions-section",
                                        h4 { "Conditions" }
                                        div { class: "conditions-grid",
                                            {pod.conditions.iter().map(|condition| rsx! {
                                                div {
                                                    key: "{condition.condition_type}",
                                                    class: "condition",
                                                    div { class: "condition-info",
                                                        span { class: "condition-type", "{condition.condition_type}" }
                                                        span { class: "condition-status status-{condition.status.to_lowercase()}", "{condition.status}" }
                                                    }
                                                    div { class: "condition-details",
                                                        span { class: "condition-reason", "{condition.reason}" }
                                                        span { class: "condition-time", "{condition.last_transition}" }
                                                    }
                                                }
                                            })}
                                        }
                                    }
                                }
                            })}
                        }
                    }
                })}
            }
        }
    }
}
