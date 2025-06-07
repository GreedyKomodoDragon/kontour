use crate::k8s::{
    find_unused_configmaps,
    problem_pod::{check_pod_status, ProblemPod},
    ClusterStats,
};
use dioxus::{logger::tracing, prelude::*};
use k8s_openapi::api::core::v1::{ConfigMap, Pod};
use kube::{api::ListParams, Api, Client};

const INSIGHTS_CSS: Asset = asset!("/assets/styling/insights.css");

#[component]
pub fn Insights() -> Element {
    let client = use_context::<Client>();
    let mut problem_pods = use_signal(Vec::<ProblemPod>::new);
    let mut cluster_stats = use_signal(ClusterStats::default);
    let mut visible_pods = use_signal(|| 6); // Number of pods to show initially
    let mut is_loading_more = use_signal(|| false);
    let mut unused_configmaps = use_signal(Vec::<(ConfigMap, String)>::new);

    // Fetch problem pods and compute stats
    // Effect to find unused ConfigMaps
    use_effect({
        let client = client.clone();
        let mut unused_configmaps = unused_configmaps.clone();

        move || {
            spawn({
                let client = client.clone();
                async move {
                    let unused = find_unused_configmaps(client).await;
                    unused_configmaps.set(unused);
                }
            });
        }
    });

    // Effect to fetch problem pods and compute stats
    use_effect({
        let client = client.clone();
        let mut problem_pods = problem_pods.clone();
        let mut cluster_stats = cluster_stats.clone();
        let mut is_loading_more = is_loading_more.clone();

        move || {
            spawn({
                let client = client.clone();
                async move {
                    let pods: Api<Pod> = Api::all(client);
                    let mut params = ListParams::default().limit(20); // Fetch 20 pods at a time
                    let mut all_problems = Vec::new();
                    let mut stats = ClusterStats::default();

                    loop {
                        match pods.list(&params).await {
                            Ok(pod_list) => {
                                // Update stats with this batch
                                let batch_stats = ClusterStats::compute_from_pods(&pod_list.items);
                                stats.crashloop_count += batch_stats.crashloop_count;
                                stats.restart_count += batch_stats.restart_count;
                                stats.evicted_count += batch_stats.evicted_count;

                                // Process problem pods from this batch
                                let batch_problems: Vec<ProblemPod> =
                                    pod_list.items.iter().filter_map(check_pod_status).collect();
                                all_problems.extend(batch_problems);

                                // Update the UI with what we have so far
                                cluster_stats.set(stats.clone());
                                problem_pods.set(all_problems.clone());

                                // Check if we've received all pods
                                if let Some(continue_token) = pod_list.metadata.continue_ {
                                    params = params.continue_token(&continue_token);
                                    is_loading_more.set(true);
                                } else {
                                    is_loading_more.set(false);
                                    break;
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to fetch pods: {}", e);
                                is_loading_more.set(false);
                                break;
                            }
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
                    span { class: "stat-value", "{cluster_stats.read().crashloop_count}" }
                }
                div { class: "stat-card",
                    span { class: "stat-label", "Frequently Restarting Pods" }
                    span { class: "stat-value", "{cluster_stats.read().restart_count}" }
                }
                div { class: "stat-card",
                    span { class: "stat-label", "Recent Evictions" }
                    span { class: "stat-value", "{cluster_stats.read().evicted_count}" }
                }
            }
        }

        // Problem Pods Section
        div { class: "insights-section",
            h2 { "Problem Pods" }
            div { class: "problem-pods-grid",
                {problem_pods.read()
                    .iter()
                    .take(*visible_pods.read())
                    .map(|pod| rsx! {
                        div { class: format!("problem-pod-card severity-{}", pod.severity),
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

            div {
                class: "show-more-container",
                {
                    let show_more = move || {
                        if *is_loading_more.read() {
                            rsx! {
                                div {
                                    class: "loading-indicator",
                                    "Loading more pods..."
                                }
                            }
                        } else if problem_pods.read().len() > *visible_pods.read() {
                            let remaining = problem_pods.read().len() - *visible_pods.read();
                            rsx! {
                                button {
                                    class: "show-more-button",
                                    onclick: move |_| {
                                        let current = *visible_pods.read();
                                        visible_pods.set(current + 6);
                                    },
                                    "Show More ({remaining} remaining)"
                                }
                            }
                        } else {
                            rsx! { }
                        }
                    };
                    show_more()
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
                {unused_configmaps.read().iter().map(|(cm, reason)| {
                    let name = cm.metadata.name.clone().unwrap_or_default();
                    let namespace = cm.metadata.namespace.clone().unwrap_or_default();
                    rsx! {
                        div { class: "problem-pod-card severity-low",
                            div { class: "problem-pod-header",
                                h3 { "{name}" }
                                span { class: "pod-namespace", "{namespace}" }
                            }
                            div { class: "problem-pod-content",
                                div { class: "issue-type", "Unused ConfigMap" }
                                p { class: "issue-details", "{reason}" }
                            }
                        }
                    }
                })}
            }
        }
    }
    }
}
