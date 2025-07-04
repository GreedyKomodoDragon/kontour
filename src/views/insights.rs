use crate::k8s::{
    find_pods_without_limits, find_resource_hotspots, find_unused_configmaps, find_unused_pvcs,
    problem_pod::{check_pod_status, ProblemPod},
    resource_limits::PodResourceIssue,
    resource_metrics::ResourceHotspot,
    ClusterStats,
};
use dioxus::{logger::tracing, prelude::*};
use k8s_openapi::api::core::v1::{ConfigMap, PersistentVolumeClaim, Pod};
use kube::{api::ListParams, Api, Client};

const INSIGHTS_CSS: Asset = asset!("/assets/styling/insights.css");

#[component]
pub fn Insights() -> Element {
    let client_signal = use_context::<Signal<Option<Client>>>();
    let problem_pods = use_signal(Vec::<ProblemPod>::new);
    let cluster_stats = use_signal(ClusterStats::default);
    let mut visible_pods = use_signal(|| 6); // Number of pods to show initially
    let mut visible_resources = use_signal(|| 6); // Number of unused resources to show initially
    let mut visible_limit_issues = use_signal(|| 6); // Number of resource limit issues to show
    let mut visible_hotspots = use_signal(|| 6); // Number of hotspots to show initially
    let is_loading_more = use_signal(|| false);
    let unused_configmaps = use_signal(Vec::<(ConfigMap, String)>::new);
    let unused_pvcs = use_signal(Vec::<(PersistentVolumeClaim, String)>::new);
    let pods_without_limits = use_signal(Vec::<PodResourceIssue>::new);
    let resource_hotspots = use_signal(Vec::<ResourceHotspot>::new);

    // Fetch problem pods and compute stats
    // Effect to find unused ConfigMaps
    use_effect({
        let mut unused_configmaps = unused_configmaps.clone();
        let mut unused_pvcs = unused_pvcs.clone();

        move || {
            if let Some(client) = &*client_signal.read() {
                spawn({
                    let client = client.clone();
                    async move {
                        let unused_cm = find_unused_configmaps(client.clone()).await;
                        unused_configmaps.set(unused_cm);

                        let unused_pvc = find_unused_pvcs(client).await;
                        unused_pvcs.set(unused_pvc);
                    }
                });
            }
        }
    });

    // Effect to fetch problem pods and compute stats
    use_effect({
        let mut problem_pods = problem_pods.clone();
        let mut cluster_stats = cluster_stats.clone();
        let mut is_loading_more = is_loading_more.clone();

        move || {
            if let Some(client) = &*client_signal.read() {
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
                                    let batch_stats =
                                        ClusterStats::compute_from_pods(&pod_list.items);
                                    stats.crashloop_count += batch_stats.crashloop_count;
                                    stats.restart_count += batch_stats.restart_count;
                                    stats.evicted_count += batch_stats.evicted_count;

                                    // Process problem pods from this batch
                                    let batch_problems: Vec<ProblemPod> = pod_list
                                        .items
                                        .iter()
                                        .filter_map(check_pod_status)
                                        .collect();
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
        }
    });

    // Effect to find pods without resource limits
    use_effect({
        let mut pods_without_limits = pods_without_limits.clone();

        move || {
            if let Some(client) = &*client_signal.read() {
                spawn({
                    let client = client.clone();
                    async move {
                        let issues = find_pods_without_limits(client).await;
                        pods_without_limits.set(issues);
                    }
                });
            }
        }
    });

    // Effect to find resource hotspots
    use_effect({
        let mut resource_hotspots = resource_hotspots.clone();

        move || {
            if let Some(client) = &*client_signal.read() {
                spawn({
                    let client = client.clone();
                    async move {
                        let hotspots = find_resource_hotspots(client).await;
                        resource_hotspots.set(hotspots);
                    }
                });
            }
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
        }}

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
            h2 { "Resource Hot & Cold spots" }
            div { class: "resource-hotspots-grid",
                {resource_hotspots.read()
                    .iter()
                    .take(*visible_hotspots.read())
                    .map(|hotspot| rsx! {
                        div {
                            class: format!("hotspot-card {}", if hotspot.severity == "high" { "high-usage" } else { "low-usage" }),
                            h3 { "{hotspot.hotspot_type}" }
                            div { class: "hotspot-content",
                                p { "{hotspot.name}" }
                                span { class: "pod-namespace", "{hotspot.namespace}" }
                                span {
                                    class: format!("usage-value {}", if hotspot.severity == "high" { "high" } else { "low" }),
                                    if hotspot.hotspot_type.contains("CPU") {
                                        "{hotspot.cpu_usage:.1}%"
                                    } else {
                                        "{hotspot.memory_usage:.1}%"
                                    }
                                }
                            }
                        }
                    })
                }
            }

            div {
                class: "show-more-container",
                {
                    let total_hotspots = resource_hotspots.read().len();
                    if total_hotspots > *visible_hotspots.read() {
                        let remaining = total_hotspots - *visible_hotspots.read();
                        rsx! {
                            button {
                                class: "show-more-button",
                                onclick: move |_| {
                                    let current = *visible_hotspots.read();
                                    visible_hotspots.set(current + 6);
                                },
                                "Show More ({remaining} remaining)"
                            }
                        }
                    } else {
                        rsx! { }
                    }
                }
            }
        }

        // Pods Without Resource Limits
        div { class: "insights-section",
            h2 { "Pods Without Resource Limits" }
            div { class: "problem-pods-grid",
                {pods_without_limits.read()
                    .iter()
                    .take(*visible_limit_issues.read())
                    .map(|issue| rsx! {
                        div { class: "problem-pod-card severity-low",
                            div { class: "problem-pod-header",
                                h3 { "{issue.name}" }
                                span { class: "pod-namespace", "{issue.namespace}" }
                            }
                            div { class: "problem-pod-content",
                                div { class: "issue-type", "{issue.issue_type}" }
                                p { class: "issue-details", "{issue.details}" }
                            }
                        }
                    })
                }
            }

            div {
                class: "show-more-container",
                {
                    let total_issues = pods_without_limits.read().len();
                    if total_issues > *visible_limit_issues.read() {
                        let remaining = total_issues - *visible_limit_issues.read();
                        rsx! {
                            button {
                                class: "show-more-button",
                                onclick: move |_| {
                                    let current = *visible_limit_issues.read();
                                    visible_limit_issues.set(current + 6);
                                },
                                "Show More ({remaining} remaining)"
                            }
                        }
                    } else {
                        rsx! { }
                    }
                }
            }
        }

        // Unused Resources
        div { class: "insights-section",
            h2 { "Unused Resources" }
            div { class: "problem-pods-grid",
                // Create a combined iterator of configmaps and pvcs for pagination
                {
                    unused_configmaps.read()
                        .iter()
                        .map(|(cm, reason)| {
                            let name = cm.metadata.name.clone().unwrap_or_default();
                            let namespace = cm.metadata.namespace.clone().unwrap_or_default();
                            (name, namespace, "ConfigMap", reason)
                        })
                        .chain(
                            unused_pvcs.read()
                                .iter()
                                .map(|(pvc, reason)| {
                                    let name = pvc.metadata.name.clone().unwrap_or_default();
                                    let namespace = pvc.metadata.namespace.clone().unwrap_or_default();
                                    (name, namespace, "PersistentVolumeClaim", reason)
                                })
                        )
                        .take(*visible_resources.read())
                        .map(|(name, namespace, resource_type, reason)| {
                            rsx! {
                                div { class: "problem-pod-card severity-low",
                                    div { class: "problem-pod-header",
                                        h3 { "{name}" }
                                        span { class: "pod-namespace", "{namespace}" }
                                    }
                                    div { class: "problem-pod-content",
                                        div { class: "issue-type", "Unused {resource_type}" }
                                        p { class: "issue-details", "{reason}" }
                                    }
                                }
                            }
                        })
                }
            }

            // Show more button for unused resources
            div {
                class: "show-more-container",
                {
                    let total_resources = unused_configmaps.read().len() + unused_pvcs.read().len();
                    if total_resources > *visible_resources.read() {
                        let remaining = total_resources - *visible_resources.read();
                        rsx! {
                            button {
                                class: "show-more-button",
                                onclick: move |_| {
                                    let current = *visible_resources.read();
                                    visible_resources.set(current + 6);
                                },
                                "Show More ({remaining} remaining)"
                            }
                        }
                    } else {
                        rsx! { }
                    }
                }
            }
        }
    }
}
