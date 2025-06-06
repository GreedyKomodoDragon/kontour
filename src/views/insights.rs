use dioxus::{logger::tracing, prelude::*};
use k8s_openapi::api::core::v1::Pod;
use kube::{api::ListParams, Api, Client};
const INSIGHTS_CSS: Asset = asset!("/assets/styling/insights.css");

#[derive(Debug)]
struct ProblemPod {
    name: String,
    namespace: String,
    issue_type: String,
    details: String,
    severity: String,
}

fn check_pod_status(pod: &Pod) -> Option<ProblemPod> {
    let name = pod.metadata.name.clone()?;
    let namespace = pod.metadata.namespace.clone()?;
    let status = pod.status.as_ref()?;

    // Check for CrashLoopBackOff
    if let Some(container_statuses) = &status.container_statuses {
        for container in container_statuses {
            if let Some(waiting) = &container.state.as_ref().and_then(|s| s.waiting.as_ref()) {
                // Check for CrashLoopBackOff
                if waiting.reason.as_deref() == Some("CrashLoopBackOff") {
                    return Some(ProblemPod {
                        name,
                        namespace,
                        issue_type: "CrashLoopBackOff".to_string(),
                        details: format!(
                            "Container {} has crashed {} times",
                            container.name, container.restart_count
                        ),
                        severity: "high".to_string(),
                    });
                }
                
                // Check for image pull issues
                if matches!(waiting.reason.as_deref(), Some("ImagePullBackOff" | "ErrImagePull")) {
                    return Some(ProblemPod {
                        name,
                        namespace,
                        issue_type: "Image Pull Error".to_string(),
                        details: format!(
                            "Container {} failed to pull image: {}",
                            container.name,
                            waiting.message.as_deref().unwrap_or("No details available")
                        ),
                        severity: "high".to_string(),
                    });
                }
            }

            // Check for container termination with error
            if let Some(terminated) = &container.state.as_ref().and_then(|s| s.terminated.as_ref()) {
                if terminated.exit_code != 0 {
                    return Some(ProblemPod {
                        name,
                        namespace,
                        issue_type: "Container Failed".to_string(),
                        details: format!(
                            "Container {} terminated with exit code {}: {}",
                            container.name,
                            terminated.exit_code,
                            terminated.message.as_deref().unwrap_or("No details available")
                        ),
                        severity: "high".to_string(),
                    });
                }
            }

            // Check for container not ready
            if !container.ready && container.restart_count == 0 {
                return Some(ProblemPod {
                    name,
                    namespace,
                    issue_type: "Container Not Ready".to_string(),
                    details: format!(
                        "Container {} is not ready and has never started successfully",
                        container.name
                    ),
                    severity: "medium".to_string(),
                });
            }

            // Check for frequent restarts
            if container.restart_count > 5 {
                return Some(ProblemPod {
                    name,
                    namespace,
                    issue_type: "Frequent Restarts".to_string(),
                    details: format!(
                        "Container {} has restarted {} times",
                        container.name, container.restart_count
                    ),
                    severity: "medium".to_string(),
                });
            }
        }
    }

    // Check for Eviction
    if let Some(reason) = &status.reason {
        if reason == "Evicted" {
            let message = status
                .message
                .clone()
                .unwrap_or_else(|| "No details available".to_string());
            return Some(ProblemPod {
                name,
                namespace,
                issue_type: "Evicted".to_string(),
                details: message,
                severity: "high".to_string(),
            });
        }
    }

    None
}

#[component]
pub fn Insights() -> Element {
    let client = use_context::<Client>();
    let mut problem_pods = use_signal(Vec::<ProblemPod>::new);
    let mut visible_pods = use_signal(|| 6); // Number of pods to show initially

    // Fetch problem pods
    use_effect({
        let client = client.clone();
        let mut problem_pods = problem_pods.clone();

        move || {
            spawn({
                let client = client.clone();
                async move {
                    let pods: Api<Pod> = Api::all(client);
                    match pods.list(&ListParams::default()).await {
                        Ok(pod_list) => {
                            let problems: Vec<ProblemPod> =
                                pod_list.items.iter().filter_map(check_pod_status).collect();
                            problem_pods.set(problems);
                        }
                        Err(e) => {
                            tracing::error!("Failed to fetch pods: {}", e);
                        }
                    }
                }
            });
        }
    });

    rsx! {
        document::Link { rel: "stylesheet", href: INSIGHTS_CSS }
        div { class: "insights-container",
            h1 { "Cluster Insights" }

            // Summary Stats
            div { class: "insights-section insights-stats",
                h2 { "Cluster Summary" }
                div { class: "stats-grid",
                    div { class: "stat-card",
                        span { class: "stat-label", "CrashLoopBackOff Pods" }
                        span { class: "stat-value", "3" }
                    }
                    div { class: "stat-card",
                        span { class: "stat-label", "Frequently Restarting Pods" }
                        span { class: "stat-value", "5" }
                    }
                    div { class: "stat-card",
                        span { class: "stat-label", "Recent Evictions" }
                        span { class: "stat-value", "2" }
                    }
                }
            }

            // Problem Pods Section
            div { class: "insights-section",
                h2 { "Problem Pods" }
                div { class: "problem-pods-grid",
                    {problem_pods.iter()
                        .take(visible_pods())
                        .map(|pod| rsx! {
                            div { class: format_args!("problem-pod-card severity-{}", pod.severity),
                                div { class: "problem-pod-header",
                                    h3 { "{pod.name}" }
                                    span { class: "pod-namespace", "{pod.namespace}" }
                                }
                                div { class: "problem-pod-content",
                                    div { class: "issue-type", "{pod.issue_type}" }
                                    p { class: "issue-details", "{pod.details}" }
                                }
                            }
                        })
                    }
                }
                
                // Show more button if there are more pods to show
                {
                    let total = problem_pods.len();
                    let current = visible_pods();
                    if total > current {
                        let remaining = total - current;
                        rsx! {
                            button {
                                class: "show-more-button",
                                onclick: move |_| {
                                    visible_pods += 6;
                                },
                                "Show More ({remaining} remaining)"
                            }
                        }
                    } else {
                        rsx!("")
                    }
                }
            }

            // Resource Hotspots
            div { class: "insights-section",
                h2 { "Resource Hotspots" }
                div { class: "resource-hotspots-grid",
                    div { class: "hotspot-card",
                        h3 { "High CPU Usage" }
                        div { class: "hotspot-content",
                            p { "web-backend-754fd78c4b-2nlpx" }
                            span { class: "usage-value", "95%" }
                        }
                    }
                    div { class: "hotspot-card",
                        h3 { "High Memory Usage" }
                        div { class: "hotspot-content",
                            p { "kafka-0" }
                            span { class: "usage-value", "87%" }
                        }
                    }
                }
            }

            // Pods Without Resource Limits
            div { class: "insights-section",
                h2 { "Pods Without Resource Limits" }
                div { class: "problem-pods-grid",
                    div { class: "problem-pod-card severity-low",
                        div { class: "problem-pod-header",
                            h3 { "nginx-proxy-65df748474-abc12" }
                            span { class: "pod-namespace", "default" }
                        }
                        div { class: "problem-pod-content",
                            div { class: "issue-type", "No Resource Limits" }
                            p { class: "issue-details", "Pod is running without CPU or memory limits, which could lead to resource contention" }
                        }
                    }
                    div { class: "problem-pod-card severity-low",
                        div { class: "problem-pod-header",
                            h3 { "metrics-collector-7d9b4f556-xyz89" }
                            span { class: "pod-namespace", "monitoring" }
                        }
                        div { class: "problem-pod-content",
                            div { class: "issue-type", "Partial Resource Limits" }
                            p { class: "issue-details", "Pod has memory limits but no CPU limits defined" }
                        }
                    }
                }
            }

            // Deprecated API Usage
            div { class: "insights-section",
                h2 { "Deprecated API Usage" }
                div { class: "problem-pods-grid",
                    div { class: "problem-pod-card severity-medium",
                        div { class: "problem-pod-header",
                            h3 { "my-ingress" }
                            span { class: "pod-namespace", "default" }
                        }
                        div { class: "problem-pod-content",
                            div { class: "issue-type", "Deprecated Ingress API Version" }
                            p { class: "issue-details", "Using networking.k8s.io/v1beta1, migrate to networking.k8s.io/v1 before Kubernetes 1.22" }
                        }
                    }
                    div { class: "problem-pod-card severity-medium",
                        div { class: "problem-pod-header",
                            h3 { "restrict-root" }
                            span { class: "pod-namespace", "kube-system" }
                        }
                        div { class: "problem-pod-content",
                            div { class: "issue-type", "PodSecurityPolicy Deprecation" }
                            p { class: "issue-details", "PodSecurityPolicy API will be removed in Kubernetes 1.25. Migrate to Pod Security Standards" }
                        }
                    }
                    div { class: "problem-pod-card severity-low",
                        div { class: "problem-pod-header",
                            h3 { "my-cronjob" }
                            span { class: "pod-namespace", "batch" }
                        }
                        div { class: "problem-pod-content",
                            div { class: "issue-type", "Deprecated CronJob API Version" }
                            p { class: "issue-details", "Using batch/v1beta1, migrate to batch/v1 for better compatibility" }
                        }
                    }
                }
            }

            // Unused Resources
            div { class: "insights-section",
                h2 { "Unused Resources" }
                div { class: "problem-pods-grid",
                    div { class: "problem-pod-card severity-low",
                        div { class: "problem-pod-header",
                            h3 { "mysql-config" }
                            span { class: "pod-namespace", "database" }
                        }
                        div { class: "problem-pod-content",
                            div { class: "issue-type", "Unused ConfigMap" }
                            p { class: "issue-details", "ConfigMap is not mounted by any pods or referenced by any deployments" }
                        }
                    }
                    div { class: "problem-pod-card severity-low",
                        div { class: "problem-pod-header",
                            h3 { "data-backup-pvc" }
                            span { class: "pod-namespace", "backup" }
                        }
                        div { class: "problem-pod-content",
                            div { class: "issue-type", "Abandoned PVC" }
                            p { class: "issue-details", "PersistentVolumeClaim has not been mounted by any pods for 30 days" }
                        }
                    }
                    div { class: "problem-pod-card severity-medium",
                        div { class: "problem-pod-header",
                            h3 { "api-credentials" }
                            span { class: "pod-namespace", "default" }
                        }
                        div { class: "problem-pod-content",
                            div { class: "issue-type", "Unused Secret" }
                            p { class: "issue-details", "Secret contains sensitive data but is not referenced by any resources" }
                        }
                    }
                    div { class: "problem-pod-card severity-low",
                        div { class: "problem-pod-header",
                            h3 { "legacy-service" }
                            span { class: "pod-namespace", "default" }
                        }
                        div { class: "problem-pod-content",
                            div { class: "issue-type", "Unused Service" }
                            p { class: "issue-details", "Service does not match any pods and has had no traffic for 60 days" }
                        }
                    }
                }
            }
        }
    }
}
