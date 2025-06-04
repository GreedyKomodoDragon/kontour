use dioxus::prelude::*;

const INSIGHTS_CSS: Asset = asset!("/assets/styling/insights.css");

#[derive(Debug)]
struct ProblemPod {
    name: String,
    namespace: String,
    issue_type: String,
    details: String,
    severity: String,
}

#[component]
pub fn Insights() -> Element {
    let problem_pods = vec![
        ProblemPod {
            name: "frontend-6d4b87bf5-x8j9k".to_string(),
            namespace: "default".to_string(),
            issue_type: "CrashLoopBackOff".to_string(),
            details: "Container exited 12 times in the last hour".to_string(),
            severity: "high".to_string(),
        },
        ProblemPod {
            name: "redis-cache-5d7b98cf67-p2m3n".to_string(),
            namespace: "backend".to_string(),
            issue_type: "Frequent Restarts".to_string(),
            details: "Pod restarted 8 times in the last 24 hours".to_string(),
            severity: "medium".to_string(),
        },
        ProblemPod {
            name: "elasticsearch-0".to_string(),
            namespace: "logging".to_string(),
            issue_type: "Evicted".to_string(),
            details: "Pod evicted due to node memory pressure".to_string(),
            severity: "high".to_string(),
        },
    ];

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
                    {problem_pods.iter().map(|pod| rsx! {
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
                    })}
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
