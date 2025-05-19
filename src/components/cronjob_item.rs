use dioxus::prelude::*;
use k8s_openapi::api::batch::v1::CronJob;
use crate::utils::calculate_age;

#[derive(Props, PartialEq, Clone)]
pub struct CronJobItemProps {
    cronjob: CronJob,
}

use crate::components::{PodContainerInfo, PodContainers, PodEnvVar, PodResources, PodVolumeMount};

#[derive(Clone)]
struct CronJobData {
    name: String,
    namespace: String,
    schedule: String,
    suspend: bool,
    last_schedule: Option<String>,
    active_jobs: i32,
    age: String,
    labels: Vec<(String, String)>,
    annotations: Vec<(String, String)>,
    concurrency_policy: String,
    successful_jobs_history_limit: Option<i32>,
    failed_jobs_history_limit: Option<i32>,
    starting_deadline_seconds: Option<i64>,
    containers: Vec<PodContainerInfo>,
}

#[component]
pub fn CronJobItem(props: CronJobItemProps) -> Element {
    let mut is_expanded = use_signal(|| false);

    let cronjob_data = CronJobData {
        name: props.cronjob.metadata.name.clone().unwrap_or_default(),
        namespace: props.cronjob.metadata.namespace.clone().unwrap_or_default(),
        schedule: props.cronjob.spec.as_ref().map(|s| s.schedule.clone()).unwrap_or_default(),
        suspend: props.cronjob.spec.as_ref().and_then(|s| s.suspend).unwrap_or(false),
        last_schedule: props.cronjob.status.as_ref().and_then(|s| s.last_schedule_time.as_ref().map(|t| t.0.to_string())),
        active_jobs: props.cronjob.status.as_ref().and_then(|s| s.active.as_ref()).map(|a| a.len() as i32).unwrap_or(0),
        age: calculate_age(props.cronjob.metadata.creation_timestamp.as_ref()),
        labels: props.cronjob.metadata.labels.as_ref()
            .map(|labels| labels.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default(),
        annotations: props.cronjob.metadata.annotations.as_ref()
            .map(|annotations| annotations.iter()
                .filter(|(k, _)| *k != "kubectl.kubernetes.io/last-applied-configuration")
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect())
            .unwrap_or_default(),
        concurrency_policy: props.cronjob.spec.as_ref()
            .and_then(|s| s.concurrency_policy.clone())
            .unwrap_or_else(|| "Allow".to_string()),
        successful_jobs_history_limit: props.cronjob.spec.as_ref()
            .and_then(|s| s.successful_jobs_history_limit),
        failed_jobs_history_limit: props.cronjob.spec.as_ref()
            .and_then(|s| s.failed_jobs_history_limit),
        starting_deadline_seconds: props.cronjob.spec.as_ref()
            .and_then(|s| s.starting_deadline_seconds),
        containers: props.cronjob.spec.as_ref()
            .and_then(|s| s.job_template.spec.as_ref())
            .and_then(|s| s.template.spec.as_ref())
            .map(|spec| {
                spec.containers.iter().map(|c| PodContainerInfo {
                    name: c.name.clone(),
                    image: c.image.clone().unwrap_or_default(),
                    command: c.command.clone().unwrap_or_default(),
                    args: c.args.clone().unwrap_or_default(),
                    volume_mounts: c.volume_mounts.clone().unwrap_or_default().into_iter()
                        .map(|vm| PodVolumeMount {
                            name: vm.name,
                            mount_path: vm.mount_path,
                            read_only: vm.read_only.unwrap_or_default(),
                        })
                        .collect(),
                    env: c.env.as_ref().map_or_else(Vec::new, |env_vars| {
                        env_vars.iter().map(|ev| PodEnvVar {
                            name: ev.name.clone(),
                            value: ev.value.clone(),
                            value_from: ev.value_from.as_ref().map(|vf| {
                                if vf.secret_key_ref.is_some() {
                                    "secretKeyRef".to_string()
                                } else if vf.config_map_key_ref.is_some() {
                                    "configMapKeyRef".to_string()
                                } else {
                                    "other".to_string()
                                }
                            }),
                        }).collect()
                    }),
                    resources: PodResources {
                        requests: c.resources.as_ref()
                            .and_then(|r| r.requests.as_ref())
                            .map(|requests| {
                                requests.iter()
                                    .map(|(k, v)| (k.clone(), v.0.clone()))
                                    .collect()
                            })
                            .unwrap_or_default(),
                        limits: c.resources.as_ref()
                            .and_then(|r| r.limits.as_ref())
                            .map(|limits| {
                                limits.iter()
                                    .map(|(k, v)| (k.clone(), v.0.clone()))
                                    .collect()
                            })
                            .unwrap_or_default(),
                    },
                }).collect()
            })
            .unwrap_or_default(),
    };

    let key_base = format!("{}-{}", cronjob_data.namespace, cronjob_data.name);
    let status = if cronjob_data.suspend {
        "Suspended"
    } else if cronjob_data.active_jobs > 0 {
        "Active"
    } else {
        "Scheduled"
    };

    rsx! {
        div {
            key: "{key_base}",
            class: "cronjob-card",
            div {
                class: "cronjob-header-card",
                div { class: "cronjob-title",
                    h3 { "{cronjob_data.name}" }
                    span {
                        class: "status-badge status-{status.to_lowercase()}",
                        "{status}"
                    }
                }
                div { class: "cronjob-info-short",
                    span { class: "info-item-short", title: "Schedule",
                        "{cronjob_data.schedule}"
                    }
                    span { class: "info-item-short", title: "Active Jobs",
                        "{cronjob_data.active_jobs}"
                    }
                    span { class: "info-item-short", title: "Age",
                        "{cronjob_data.age}"
                    }
                }
                div { class: "cronjob-controls",
                    button {
                        class: "btn-icon expand-toggle",
                        onclick: move |evt| {
                            evt.stop_propagation();
                            is_expanded.set(!is_expanded());
                        },
                        title: if is_expanded() { "Collapse" } else { "Expand" },
                        if is_expanded() { "🔼" } else { "🔽" }
                    }
                    button {
                        class: "btn-icon",
                        onclick: move |evt| evt.stop_propagation(),
                        title: "Delete",
                        "🗑️"
                    }
                }
            }

            {is_expanded().then(|| rsx! {
                div { class: "cronjob-details",
                    // Basic Info Section
                    div { class: "info-section",
                        h4 { "Details" }
                        div { class: "info-grid",
                            div { class: "info-item",
                                span { class: "info-label", "Namespace" }
                                span { class: "info-value", "{cronjob_data.namespace}" }
                            }
                            div { class: "info-item",
                                span { class: "info-label", "Schedule" }
                                span { class: "info-value", "{cronjob_data.schedule}" }
                            }
                            div { class: "info-item",
                                span { class: "info-label", "Active Jobs" }
                                span { class: "info-value", "{cronjob_data.active_jobs}" }
                            }
                            div { class: "info-item",
                                span { class: "info-label", "Suspended" }
                                span { class: "info-value", "{cronjob_data.suspend}" }
                            }
                            div { class: "info-item",
                                span { class: "info-label", "Concurrency Policy" }
                                span { class: "info-value", "{cronjob_data.concurrency_policy}" }
                            }
                            div { class: "info-item",
                                span { class: "info-label", "Last Schedule" }
                                span { class: "info-value",
                                    {cronjob_data.last_schedule.clone().unwrap_or_else(|| "Never".to_string())}
                                }
                            }
                            div { class: "info-item",
                                span { class: "info-label", "Age" }
                                span { class: "info-value", "{cronjob_data.age}" }
                            }
                            {cronjob_data.successful_jobs_history_limit.map(|limit| rsx! {
                                div { class: "info-item",
                                    span { class: "info-label", "Success History Limit" }
                                    span { class: "info-value", "{limit}" }
                                }
                            })}
                            {cronjob_data.failed_jobs_history_limit.map(|limit| rsx! {
                                div { class: "info-item",
                                    span { class: "info-label", "Failed History Limit" }
                                    span { class: "info-value", "{limit}" }
                                }
                            })}
                            {cronjob_data.starting_deadline_seconds.map(|deadline| rsx! {
                                div { class: "info-item",
                                    span { class: "info-label", "Starting Deadline" }
                                    span { class: "info-value", "{deadline}s" }
                                }
                            })}
                        }
                    }

                    // Labels Section
                    {(!cronjob_data.labels.is_empty()).then(|| rsx! {
                        div { class: "labels-section",
                            h4 { "Labels" }
                            div { class: "labels-grid",
                                {cronjob_data.labels.iter().map(|(key, value)| rsx! {
                                    div {
                                        key: "lbl-{key_base}-{key}",
                                        class: "label",
                                        span { class: "label-key", "{key}" }
                                        span { class: "label-value", "{value}" }
                                    }
                                })}
                            }
                        }
                    })}

                    // Annotations Section
                    {(!cronjob_data.annotations.is_empty()).then(|| rsx! {
                        div { class: "labels-section",
                            h4 { "Annotations" }
                            div { class: "labels-grid",
                                {cronjob_data.annotations.iter().map(|(key, value)| rsx! {
                                    div {
                                        key: "anno-{key_base}-{key}",
                                        class: "label annotation",
                                        span { class: "label-key", "{key}" }
                                        span { class: "label-value", "{value}" }
                                    }
                                })}
                            }
                        }
                    })}

                    // Containers Section
                    {(!cronjob_data.containers.is_empty()).then(|| rsx! {
                        PodContainers {
                            containers: cronjob_data.containers.clone(),
                            key_base: key_base.clone(),
                        }
                    })}
                }
            })}
        }
    }
}
