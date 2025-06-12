use dioxus::prelude::*;
use k8s_openapi::api::batch::v1::Job;
use crate::components::{PodContainerInfo, PodEnvVar, PodContainers, PodResources, PodVolumeMount};

#[derive(Props, PartialEq, Clone)]
pub struct JobItemProps {
    job: Job,
}

#[component]
pub fn JobItem(props: JobItemProps) -> Element {
    let mut is_expanded = use_signal(|| false);

    let job_data = JobData {
        name: props.job.metadata.name.clone().unwrap_or_default(),
        namespace: props.job.metadata.namespace.clone().unwrap_or_default(),
        completions: props
            .job
            .spec
            .as_ref()
            .and_then(|spec| spec.completions)
            .unwrap_or(1),
        parallelism: props
            .job
            .spec
            .as_ref()
            .and_then(|spec| spec.parallelism)
            .unwrap_or(1),
        succeeded: props
            .job
            .status
            .as_ref()
            .and_then(|status| status.succeeded)
            .unwrap_or(0),
        failed: props
            .job
            .status
            .as_ref()
            .and_then(|status| status.failed)
            .unwrap_or(0),
        active: props
            .job
            .status
            .as_ref()
            .and_then(|status| status.active)
            .unwrap_or(0),
        age: "1h".to_string(), // TODO: Calculate age
        labels: props
            .job
            .metadata
            .labels
            .as_ref()
            .map(|labels| labels.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default(),
        annotations: props
            .job
            .metadata
            .annotations
            .as_ref()
            .map(|annotations| {
                annotations
                    .iter()
                    .filter(|(k, _)| *k != "kubectl.kubernetes.io/last-applied-configuration")
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            })
            .unwrap_or_default(),
        backoff_limit: props
            .job
            .spec
            .as_ref()
            .and_then(|spec| spec.backoff_limit)
            .unwrap_or(6),
        ttl_seconds_after_finished: props
            .job
            .spec
            .as_ref()
            .and_then(|spec| spec.ttl_seconds_after_finished),
        restart_policy: props
            .job
            .spec
            .as_ref()
            .and_then(|spec| spec.template.spec.as_ref())
            .and_then(|pod_spec| pod_spec.restart_policy.clone())
            .unwrap_or_else(|| "Never".to_string()),
        service_account: props
            .job
            .spec
            .as_ref()
            .and_then(|spec| spec.template.spec.as_ref())
            .and_then(|pod_spec| pod_spec.service_account_name.clone())
            .unwrap_or_default(),
        node_selector: props
            .job
            .spec
            .as_ref()
            .and_then(|spec| spec.template.spec.as_ref())
            .and_then(|pod_spec| pod_spec.node_selector.as_ref())
            .map(|selector| {
                selector
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            })
            .unwrap_or_default(),
        tolerations: props
            .job
            .spec
            .as_ref()
            .and_then(|spec| spec.template.spec.as_ref())
            .and_then(|pod_spec| pod_spec.tolerations.as_ref())
            .map(|tolerations| {
                tolerations
                    .iter()
                    .map(|t| Toleration {
                        key: t.key.clone().unwrap_or_default(),
                        operator: t.operator.clone().unwrap_or_default(),
                        value: t.value.clone().unwrap_or_default(),
                        effect: t.effect.clone().unwrap_or_default(),
                    })
                    .collect()
            })
            .unwrap_or_default(),
        containers: props
            .job
            .spec
            .as_ref()
            .and_then(|spec| spec.template.spec.as_ref())
            .map(|pod_spec| {
                pod_spec
                    .containers
                    .iter()
                    .map(|c| PodContainerInfo {
                        name: c.name.clone(),
                        image: c.image.clone().unwrap_or_default(),
                        command: c.command.clone().unwrap_or_default(),
                        args: c.args.clone().unwrap_or_default(),
                        env: c.env.as_ref().map_or_else(Vec::new, |env_vars| {
                            env_vars
                                .iter()
                                .map(|ev| PodEnvVar {
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
                                })
                                .collect()
                        }),
                        resources: PodResources {
                            requests: c
                                .resources
                                .as_ref()
                                .and_then(|r| r.requests.as_ref())
                                .map(|requests| {
                                    requests
                                        .iter()
                                        .map(|(k, v)| (k.clone(), v.0.clone()))
                                        .collect()
                                })
                                .unwrap_or_default(),
                            limits: c
                                .resources
                                .as_ref()
                                .and_then(|r| r.limits.as_ref())
                                .map(|limits| {
                                    limits
                                        .iter()
                                        .map(|(k, v)| (k.clone(), v.0.clone()))
                                        .collect()
                                })
                                .unwrap_or_default(),
                        },
                        volume_mounts: c.volume_mounts.as_ref().map_or_else(Vec::new, |mounts| {
                            mounts
                                .iter()
                                .map(|vm| PodVolumeMount {
                                    name: vm.name.clone(),
                                    mount_path: vm.mount_path.clone(),
                                    read_only: vm.read_only.unwrap_or(false),
                                })
                                .collect()
                        }),
                    })
                    .collect()
            })
            .unwrap_or_default(),
        volumes: props
            .job
            .spec
            .as_ref()
            .and_then(|spec| spec.template.spec.as_ref())
            .and_then(|pod_spec| pod_spec.volumes.as_ref())
            .map(|volumes| {
                volumes
                    .iter()
                    .map(|v| VolumeInfo {
                        name: v.name.clone(),
                        volume_type: if v.config_map.is_some() {
                            "ConfigMap"
                        } else if v.secret.is_some() {
                            "Secret"
                        } else if v.empty_dir.is_some() {
                            "EmptyDir"
                        } else {
                            "Other"
                        }
                        .to_string(),
                        source: if let Some(cm) = &v.config_map {
                            cm.name.clone()
                        } else if let Some(secret) = &v.secret {
                            secret.secret_name.clone().unwrap_or_default()
                        } else {
                            String::new()
                        },
                    })
                    .collect()
            })
            .unwrap_or_default(),
    };

    let key_base = format!("{}-{}", job_data.namespace, job_data.name);
    let status = if job_data.succeeded > 0 {
        "Succeeded"
    } else if job_data.failed > 0 {
        "Failed"
    } else if job_data.active > 0 {
        "Active"
    } else {
        "Pending"
    };

    rsx! {
        div {
            key: "{key_base}",
            class: "job-card",
            div {
                class: "job-header-card",
                div { class: "job-title",
                    h3 { "{job_data.name}" }
                    span {
                        class: "status-badge status-{status.to_lowercase()}",
                        "{status}"
                    }
                }
                div { class: "job-info-short",
                    span { class: "info-item-short", title: "Namespace",
                        "{job_data.namespace}"
                    }
                    span { class: "info-item-short", title: "Completions",
                        "{job_data.succeeded}/{job_data.completions}"
                    }
                    span { class: "info-item-short", title: "Age",
                        "{job_data.age}"
                    }
                }
                div { class: "job-controls",
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
                div { class: "job-details",
                    // Basic Info Section
                    div { class: "info-section",
                        h4 { "Details" }
                        div { class: "info-grid",
                            div { class: "info-item",
                                span { class: "info-label", "Namespace" }
                                span { class: "info-value", "{job_data.namespace}" }
                            }
                            div { class: "info-item",
                                span { class: "info-label", "Completions" }
                                span { class: "info-value", "{job_data.succeeded}/{job_data.completions}" }
                            }
                            div { class: "info-item",
                                span { class: "info-label", "Parallelism" }
                                span { class: "info-value", "{job_data.parallelism}" }
                            }
                            div { class: "info-item",
                                span { class: "info-label", "Active" }
                                span { class: "info-value", "{job_data.active}" }
                            }
                            div { class: "info-item",
                                span { class: "info-label", "Failed" }
                                span { class: "info-value", "{job_data.failed}" }
                            }
                            div { class: "info-item",
                                span { class: "info-label", "Age" }
                                span { class: "info-value", "{job_data.age}" }
                            }
                            div { class: "info-item",
                                span { class: "info-label", "Backoff Limit" }
                                span { class: "info-value", "{job_data.backoff_limit}" }
                            }
                            div { class: "info-item",
                                span { class: "info-label", "TTL After Finished" }
                                span { class: "info-value",
                                    {if let Some(ttl) = job_data.ttl_seconds_after_finished {
                                        format!("{ttl}s")
                                    } else {
                                        "Not set".to_string()
                                    }}
                                }
                            }
                            div { class: "info-item",
                                span { class: "info-label", "Restart Policy" }
                                span { class: "info-value", "{job_data.restart_policy}" }
                            }
                            div { class: "info-item",
                                span { class: "info-label", "Service Account" }
                                span { class: "info-value", "{job_data.service_account}" }
                            }
                        }
                    }

                    // Labels Section
                    {(!job_data.labels.is_empty()).then(|| rsx! {
                        div { class: "labels-section",
                            h4 { "Labels" }
                            div { class: "labels-grid",
                                {job_data.labels.iter().map(|(key, value)| rsx! {
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
                    {(!job_data.annotations.is_empty()).then(|| rsx! {
                        div { class: "labels-section",
                            h4 { "Annotations" }
                            div { class: "labels-grid",
                                {job_data.annotations.iter().map(|(key, value)| rsx! {
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

                    // Node Selector Section
                    {(!job_data.node_selector.is_empty()).then(|| rsx! {
                        div { class: "labels-section",
                            h4 { "Node Selector" }
                            div { class: "labels-grid",
                                {job_data.node_selector.iter().map(|(key, value)| rsx! {
                                    div {
                                        key: "selector-{key_base}-{key}",
                                        class: "label",
                                        span { class: "label-key", "{key}" }
                                        span { class: "label-value", "{value}" }
                                    }
                                })}
                            }
                        }
                    })}

                    // Tolerations Section
                    {(!job_data.tolerations.is_empty()).then(|| rsx! {
                        div { class: "tolerations-section",
                            h4 { "Tolerations" }
                            div { class: "tolerations-grid",
                                {job_data.tolerations.iter().map(|toleration| rsx! {
                                    div { class: "toleration-item",
                                        div {
                                            span { class: "toleration-key", "key: " }
                                            span { class: "toleration-value quoted", "\"{toleration.key}\"" }
                                        }
                                        div {
                                            span { class: "toleration-key", "operator: " }
                                            span { class: "toleration-value", "{toleration.operator}" }
                                        }
                                        div {
                                            span { class: "toleration-key", "value: " }
                                            span { class: "toleration-value quoted", "\"{toleration.value}\"" }
                                        }
                                        div {
                                            span { class: "toleration-key", "effect: " }
                                            span { class: "toleration-value", "{toleration.effect}" }
                                        }
                                    }
                                })}
                            }
                        }
                    })}

                    // Containers Section
                    {(!job_data.containers.is_empty()).then(|| rsx! {
                        PodContainers {
                            containers: job_data.containers.clone(),
                            key_base: key_base.clone(),
                        }
                    })}

                    // Volumes Section
                    {(!job_data.volumes.is_empty()).then(|| rsx! {
                        div { class: "volumes-section",
                            h4 { "Volumes" }
                            div { class: "volumes-grid",
                                {job_data.volumes.iter().map(|volume| rsx! {
                                    div { class: "volume-item",
                                        div { class: "volume-header",
                                            span { class: "volume-name", "{volume.name}" }
                                            span { class: "volume-type", "({volume.volume_type})" }
                                        }
                                        {(!volume.source.is_empty()).then(|| rsx! {
                                            span { class: "volume-source", "Source: {volume.source}" }
                                        })}
                                    }
                                })}
                            }
                        }
                    })}
                }
            })}
        }
    }
}

#[derive(Clone)]
struct JobData {
    name: String,
    namespace: String,
    completions: i32,
    parallelism: i32,
    succeeded: i32,
    failed: i32,
    active: i32,
    age: String,
    labels: Vec<(String, String)>,
    annotations: Vec<(String, String)>,
    backoff_limit: i32,
    ttl_seconds_after_finished: Option<i32>,
    restart_policy: String,
    service_account: String,
    node_selector: Vec<(String, String)>,
    tolerations: Vec<Toleration>,
    containers: Vec<PodContainerInfo>,
    volumes: Vec<VolumeInfo>,
}

#[derive(Clone)]
struct Toleration {
    key: String,
    operator: String,
    value: String,
    effect: String,
}

#[derive(Clone)]
struct VolumeInfo {
    name: String,
    volume_type: String,
    source: String,
}
