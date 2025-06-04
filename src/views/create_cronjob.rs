use dioxus::prelude::*;
use k8s_openapi::{
    api::batch::v1::{CronJob, CronJobSpec, JobSpec, JobTemplateSpec},
    api::core::v1::{Container, PodSpec, PodTemplateSpec},
    apimachinery::pkg::api::resource::Quantity,
};
use kube::{api::PostParams, Api, Client};
use std::collections::BTreeMap;

const CREATE_FORMS_CSS: Asset = asset!("/assets/styling/create_forms.css");

#[derive(Default, Clone)]
struct ContainerPort {
    container_port: String,
    protocol: String,
}

#[derive(Default, Clone)]
struct EnvVar {
    name: String,
    value: String,
}

#[derive(Default, Clone)]
struct KeyValuePair {
    key: String,
    value: String,
}

#[component]
pub fn CreateCronJob() -> Element {
    let client = use_context::<Client>();
    let navigate = use_navigator();
    
    let mut name = use_signal(String::new);
    let mut namespace = use_signal(|| "default".to_string());
    let mut schedule = use_signal(|| "0 * * * *".to_string());  // Default to hourly
    let mut image = use_signal(String::new);
    let mut concurrency_policy = use_signal(|| "Allow".to_string());
    let mut starting_deadline_seconds = use_signal(String::new);
    
    // Resource requests/limits
    let mut cpu_request = use_signal(|| "100m".to_string());
    let mut memory_request = use_signal(|| "128Mi".to_string());
    let mut cpu_limit = use_signal(|| "200m".to_string());
    let mut memory_limit = use_signal(|| "256Mi".to_string());

    // Container ports
    let mut ports = use_signal(|| vec![ContainerPort::default()]);
    
    // Environment variables
    let mut env_vars = use_signal(|| vec![EnvVar::default()]);

    // Labels and annotations
    let mut labels = use_signal(|| vec![KeyValuePair::default()]);
    let mut annotations = use_signal(|| vec![KeyValuePair::default()]);

    let mut sections_state = use_signal(|| {
        vec![
            ("basic", false),
            ("schedule", false),
            ("labels", false),
            ("annotations", false),
            ("resources", false),
            ("ports", false),
            ("env", false),
        ].into_iter().collect::<std::collections::HashMap<&'static str, bool>>()
    });

    let mut toggle_section = move |section: &'static str| {
        let current = sections_state.read().get(section).copied().unwrap_or(false);
        sections_state.write().insert(section, !current);
    };

    let section_class = move |section: &'static str| {
        let is_open = sections_state.read().get(section).copied().unwrap_or(false);
        if is_open {
            "section section-open"
        } else {
            "section"
        }
    };

    let mut error = use_signal(|| None::<String>);

    let submit = move |_| {
        let name = name().clone();
        let cronjob_name = name.clone();
        let namespace = namespace();
        let schedule = schedule();
        let image = image();
        let client = client.clone();

        // Basic validation
        if name.is_empty() {
            error.set(Some("CronJob name is required".to_string()));
            return;
        }
        if schedule.is_empty() {
            error.set(Some("Schedule is required".to_string()));
            return;
        }
        if image.is_empty() {
            error.set(Some("Container image is required".to_string()));
            return;
        }

        error.set(None);
        
        spawn(async move {
            use k8s_openapi::api::core::v1::ResourceRequirements;

            // Create resource requirements
            let mut requests = BTreeMap::new();
            let mut limits = BTreeMap::new();
            
            if !cpu_request().is_empty() {
                requests.insert("cpu".to_string(), Quantity(cpu_request()));
            }
            if !memory_request().is_empty() {
                requests.insert("memory".to_string(), Quantity(memory_request()));
            }
            if !cpu_limit().is_empty() {
                limits.insert("cpu".to_string(), Quantity(cpu_limit()));
            }
            if !memory_limit().is_empty() {
                limits.insert("memory".to_string(), Quantity(memory_limit()));
            }

            let resources = if !requests.is_empty() || !limits.is_empty() {
                Some(ResourceRequirements {
                    requests: if requests.is_empty() { None } else { Some(requests) },
                    limits: if limits.is_empty() { None } else { Some(limits) },
                    claims: None,
                })
            } else {
                None
            };

            // Convert container ports
            let ports = if ports().is_empty() {
                None
            } else {
                Some(ports().into_iter().filter_map(|p| {
                    if p.container_port.is_empty() {
                        return None;
                    }
                    Some(k8s_openapi::api::core::v1::ContainerPort {
                        container_port: p.container_port.parse().ok()?,
                        protocol: Some(p.protocol),
                        ..Default::default()
                    })
                }).collect())
            };

            // Convert environment variables
            let env = if env_vars().is_empty() {
                None
            } else {
                Some(env_vars().into_iter().filter_map(|e| {
                    if e.name.is_empty() {
                        return None;
                    }
                    Some(k8s_openapi::api::core::v1::EnvVar {
                        name: e.name,
                        value: Some(e.value),
                        value_from: None,
                    })
                }).collect())
            };

            // Convert labels and annotations
            let mut label_map = BTreeMap::new();
            for label in labels() {
                if !label.key.is_empty() && !label.value.is_empty() {
                    label_map.insert(label.key, label.value);
                }
            }

            let mut annotation_map = BTreeMap::new();
            for annotation in annotations() {
                if !annotation.key.is_empty() && !annotation.value.is_empty() {
                    annotation_map.insert(annotation.key, annotation.value);
                }
            }

            let cronjob = CronJob {
                metadata: k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                    name: Some(name),
                    namespace: Some(namespace.clone()),
                    labels: if label_map.is_empty() { None } else { Some(label_map.clone()) },
                    annotations: if annotation_map.is_empty() { None } else { Some(annotation_map) },
                    ..Default::default()
                },
                spec: Some(CronJobSpec {
                    schedule,
                    concurrency_policy: Some(concurrency_policy()),
                    starting_deadline_seconds: starting_deadline_seconds()
                        .parse()
                        .ok()
                        .map(|s: i64| s),
                    job_template: JobTemplateSpec {
                        metadata: None,
                        spec: Some(JobSpec {
                            template: PodTemplateSpec {
                                metadata: Some(k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
                                    labels: Some(label_map),
                                    ..Default::default()
                                }),
                                spec: Some(PodSpec {
                                    containers: vec![Container {
                                        name: cronjob_name,
                                        image: Some(image),
                                        ports,
                                        env,
                                        resources,
                                        ..Default::default()
                                    }],
                                    restart_policy: Some("OnFailure".to_string()),
                                    ..Default::default()
                                }),
                            },
                            ..Default::default()
                        }),
                    },
                    ..Default::default()
                }),
                status: None,
            };

            let cronjobs: Api<CronJob> = Api::namespaced(client, &namespace);
            match cronjobs.create(&PostParams::default(), &cronjob).await {
                Ok(_) => {
                    navigate.push("/cronjobs");
                }
                Err(e) => {
                    error.set(Some(format!("Failed to create cronjob: {}", e)));
                }
            }
        });
    };

    rsx! {
        document::Link { rel: "stylesheet", href: CREATE_FORMS_CSS }
        div { class: "create-cronjob-container",
            h1 { class: "create-cronjob-title", "Create CronJob" }
            
            // Basic Info Section
            div { class: section_class("basic"),
                div {
                    class: "section-header",
                    onclick: move |_| toggle_section("basic"),
                    h2 { class: "section-title", "Basic Information" }
                    span { class: "section-toggle", "▼" }
                }
                div { class: "section-content",
                    div { class: "form-grid",
                        div { class: "form-group",
                            label { class: "form-label", "CronJob Name" }
                            input {
                                class: "form-input",
                                placeholder: "Enter cronjob name",
                                value: "{name}",
                                oninput: move |evt| name.set(evt.value().clone())
                            }
                        }

                        div { class: "form-group",
                            label { class: "form-label", "Namespace" }
                            input {
                                class: "form-input",
                                value: "{namespace}",
                                oninput: move |evt| namespace.set(evt.value().clone())
                            }
                        }

                        div { class: "form-group",
                            label { class: "form-label", "Container Image" }
                            input {
                                class: "form-input",
                                placeholder: "e.g., backup-tool:latest",
                                value: "{image}",
                                oninput: move |evt| image.set(evt.value().clone())
                            }
                        }
                    }
                }
            }

            // Schedule Section 
            div { class: section_class("schedule"),
                div {
                    class: "section-header",
                    onclick: move |_| toggle_section("schedule"),
                    h2 { class: "section-title", "Schedule Configuration" }
                    span { class: "section-toggle", "▼" }
                }
                div { class: "section-content",
                    div { class: "form-grid",
                        div { class: "form-group",
                            label { class: "form-label", "Schedule (Cron Format)" }
                            input {
                                class: "form-input",
                                placeholder: "0 * * * *",
                                value: "{schedule}",
                                oninput: move |evt| schedule.set(evt.value().clone())
                            }
                            div { class: "form-help",
                                "Format: minute hour day-of-month month day-of-week"
                            }
                        }

                        div { class: "form-group",
                            label { class: "form-label", "Concurrency Policy" }
                            select {
                                class: "form-input",
                                value: "{concurrency_policy}",
                                onchange: move |evt| concurrency_policy.set(evt.value().clone()),
                                option { value: "Allow", "Allow" }
                                option { value: "Forbid", "Forbid" }
                                option { value: "Replace", "Replace" }
                            }
                        }

                        div { class: "form-group",
                            label { class: "form-label", "Starting Deadline (Seconds)" }
                            input {
                                class: "form-input",
                                r#type: "number",
                                min: "0",
                                placeholder: "Optional",
                                value: "{starting_deadline_seconds}",
                                oninput: move |evt| starting_deadline_seconds.set(evt.value().clone())
                            }
                        }
                    }
                }
            }

            // Labels Section
            div { class: section_class("labels"),
                div {
                    class: "section-header",
                    onclick: move |_| toggle_section("labels"),
                    h2 { class: "section-title", "Labels" }
                    span { class: "section-toggle", "▼" }
                }
                div { class: "section-content",
                    div { class: "repeatable-section",
                        div { class: "form-row form-header",
                            div { class: "form-group",
                                label { class: "form-label", "Key" }
                            }
                            div { class: "form-group",
                                label { class: "form-label", "Value" }
                            }
                        }
                        {labels().iter().enumerate().map(|(i, label)| {
                            let i = i.clone();
                            rsx! {
                                div { class: "form-row",
                                    div { class: "form-group",
                                        input {
                                            class: "form-input",
                                            placeholder: "app",
                                            value: "{label.key}",
                                            oninput: move |evt| {
                                                let mut new_labels = labels();
                                                new_labels[i].key = evt.value().clone();
                                                labels.set(new_labels);
                                            }
                                        }
                                    }
                                    div { class: "form-group",
                                        input {
                                            class: "form-input",
                                            placeholder: "backup-job",
                                            value: "{label.value}",
                                            oninput: move |evt| {
                                                let mut new_labels = labels();
                                                new_labels[i].value = evt.value().clone();
                                                labels.set(new_labels);
                                            }
                                        }
                                    }
                                }
                            }
                        })}
                        button {
                            class: "create-form-btn create-form-btn-secondary",
                            onclick: move |_| {
                                let mut new_labels = labels();
                                new_labels.push(KeyValuePair::default());
                                labels.set(new_labels);
                            },
                            "Add Label"
                        }
                    }
                }
            }

            // Annotations Section
            div { class: section_class("annotations"),
                div {
                    class: "section-header",
                    onclick: move |_| toggle_section("annotations"),
                    h2 { class: "section-title", "Annotations" }
                    span { class: "section-toggle", "▼" }
                }
                div { class: "section-content",
                    div { class: "repeatable-section",
                        div { class: "form-row form-header",
                            div { class: "form-group",
                                label { class: "form-label", "Key" }
                            }
                            div { class: "form-group",
                                label { class: "form-label", "Value" }
                            }
                        }
                        {annotations().iter().enumerate().map(|(i, annotation)| {
                            let i = i.clone();
                            rsx! {
                                div { class: "form-row",
                                    div { class: "form-group",
                                        input {
                                            class: "form-input",
                                            placeholder: "description",
                                            value: "{annotation.key}",
                                            oninput: move |evt| {
                                                let mut new_annotations = annotations();
                                                new_annotations[i].key = evt.value().clone();
                                                annotations.set(new_annotations);
                                            }
                                        }
                                    }
                                    div { class: "form-group",
                                        input {
                                            class: "form-input",
                                            placeholder: "Database backup job",
                                            value: "{annotation.value}",
                                            oninput: move |evt| {
                                                let mut new_annotations = annotations();
                                                new_annotations[i].value = evt.value().clone();
                                                annotations.set(new_annotations);
                                            }
                                        }
                                    }
                                }
                            }
                        })}
                        button {
                            class: "create-form-btn create-form-btn-secondary",
                            onclick: move |_| {
                                let mut new_annotations = annotations();
                                new_annotations.push(KeyValuePair::default());
                                annotations.set(new_annotations);
                            },
                            "Add Annotation"
                        }
                    }
                }
            }

            // Resource Section
            div { class: section_class("resources"),
                div {
                    class: "section-header",
                    onclick: move |_| toggle_section("resources"),
                    h2 { class: "section-title", "Container Resources" }
                    span { class: "section-toggle", "▼" }
                }
                div { class: "section-content",
                    div { class: "form-grid",
                        div { class: "form-group",
                            label { class: "form-label", "CPU Request" }
                            input {
                                class: "form-input",
                                placeholder: "100m",
                                value: "{cpu_request}",
                                oninput: move |evt| cpu_request.set(evt.value().clone())
                            }
                        }
                        div { class: "form-group",
                            label { class: "form-label", "Memory Request" }
                            input {
                                class: "form-input",
                                placeholder: "128Mi",
                                value: "{memory_request}",
                                oninput: move |evt| memory_request.set(evt.value().clone())
                            }
                        }
                        div { class: "form-group",
                            label { class: "form-label", "CPU Limit" }
                            input {
                                class: "form-input",
                                placeholder: "200m",
                                value: "{cpu_limit}",
                                oninput: move |evt| cpu_limit.set(evt.value().clone())
                            }
                        }
                        div { class: "form-group",
                            label { class: "form-label", "Memory Limit" }
                            input {
                                class: "form-input",
                                placeholder: "256Mi",
                                value: "{memory_limit}",
                                oninput: move |evt| memory_limit.set(evt.value().clone())
                            }
                        }
                    }
                }
            }

            // Ports Section
            div { class: section_class("ports"),
                div {
                    class: "section-header",
                    onclick: move |_| toggle_section("ports"),
                    h2 { class: "section-title", "Container Ports" }
                    span { class: "section-toggle", "▼" }
                }
                div { class: "section-content",
                    div { class: "repeatable-section",
                        div { class: "form-row form-header",
                            div { class: "form-group",
                                label { class: "form-label", "Port" }
                            }
                            div { class: "form-group",
                                label { class: "form-label", "Protocol" }
                            }
                        }
                        {ports().iter().enumerate().map(|(i, port)| {
                            let i = i.clone();
                            rsx! {
                                div { class: "form-row",
                                    div { class: "form-group",
                                        input {
                                            class: "form-input",
                                            placeholder: "8080",
                                            value: "{port.container_port}",
                                            oninput: move |evt| {
                                                let mut new_ports = ports();
                                                new_ports[i].container_port = evt.value().clone();
                                                ports.set(new_ports);
                                            }
                                        }
                                    }
                                    div { class: "form-group",
                                        select {
                                            class: "form-input",
                                            value: "{port.protocol}",
                                            onchange: move |evt| {
                                                let mut new_ports = ports();
                                                new_ports[i].protocol = evt.value().clone();
                                                ports.set(new_ports);
                                            },
                                            option { value: "TCP", "TCP" }
                                            option { value: "UDP", "UDP" }
                                        }
                                    }
                                }
                            }
                        })}
                        button {
                            class: "create-form-btn create-form-btn-secondary",
                            onclick: move |_| {
                                let mut new_ports = ports();
                                new_ports.push(ContainerPort::default());
                                ports.set(new_ports);
                            },
                            "Add Port"
                        }
                    }
                }
            }

            // Environment Variables Section
            div { class: section_class("env"),
                div {
                    class: "section-header",
                    onclick: move |_| toggle_section("env"),
                    h2 { class: "section-title", "Environment Variables" }
                    span { class: "section-toggle", "▼" }
                }
                div { class: "section-content",
                    div { class: "repeatable-section",
                        div { class: "form-row form-header",
                            div { class: "form-group",
                                label { class: "form-label", "Name" }
                            }
                            div { class: "form-group",
                                label { class: "form-label", "Value" }
                            }
                        }
                        {env_vars().iter().enumerate().map(|(i, env)| {
                            let i = i.clone();
                            rsx! {
                                div { class: "form-row",
                                    div { class: "form-group",
                                        input {
                                            class: "form-input",
                                            value: "{env.name}",
                                            oninput: move |evt| {
                                                let mut new_envs = env_vars();
                                                new_envs[i].name = evt.value().clone();
                                                env_vars.set(new_envs);
                                            }
                                        }
                                    }
                                    div { class: "form-group",
                                        input {
                                            class: "form-input",
                                            value: "{env.value}",
                                            oninput: move |evt| {
                                                let mut new_envs = env_vars();
                                                new_envs[i].value = evt.value().clone();
                                                env_vars.set(new_envs);
                                            }
                                        }
                                    }
                                }
                            }
                        })}
                        button {
                            class: "create-form-btn create-form-btn-secondary",
                            onclick: move |_| {
                                let mut new_envs = env_vars();
                                new_envs.push(EnvVar::default());
                                env_vars.set(new_envs);
                            },
                            "Add Environment Variable"
                        }
                    }
                }
            }

            {error().map(|err| rsx!(
                div { class: "error-message", "{err}" }
            ))}

            div { class: "button-group",
                button {
                    class: "create-form-btn create-form-btn-primary",
                    onclick: submit,
                    "Create CronJob"
                }
                button {
                    class: "create-form-btn create-form-btn-secondary",
                    onclick: move |_| {
                        navigate.push("/cronjobs");
                    },
                    "Cancel"
                }
            }
        }
    }
}
