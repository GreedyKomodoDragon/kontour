use dioxus::prelude::*;
use std::collections::HashMap;

#[derive(Clone, PartialEq)]
pub struct PodContainerInfo {
    pub name: String,
    pub image: String,
    pub command: Vec<String>,
    pub args: Vec<String>,
    pub env: Vec<PodEnvVar>,
    pub resources: PodResources,
    pub volume_mounts: Vec<PodVolumeMount>,
}

#[derive(Clone, PartialEq)]
pub struct PodEnvVar {
    pub name: String,
    pub value: Option<String>,
    pub value_from: Option<String>,
}

#[derive(Clone, PartialEq)]
pub struct PodResources {
    pub requests: HashMap<String, String>,
    pub limits: HashMap<String, String>,
}

#[derive(Clone, PartialEq)]
pub struct PodVolumeMount {
    pub name: String,
    pub mount_path: String,
    pub read_only: bool,
}

#[derive(Props, PartialEq, Clone)]
pub struct PodContainersProps {
    pub containers: Vec<PodContainerInfo>,
    pub key_base: String,
}

#[component]
pub fn PodContainers(props: PodContainersProps) -> Element {
    let mut expanded_states = use_signal(|| HashMap::<String, bool>::new());
    
    let mut toggle_container = move |name: String| {
        expanded_states.with_mut(|states| {
            let entry = states.entry(name).or_insert(false);
            *entry = !*entry;
        });
    };

    rsx! {
        div { class: "labels-section",
            h4 { "Containers" }
            div { class: "containers-grid",
                {props.containers.iter().cloned().map(|container| {
                    let is_expanded = expanded_states.read().get(&container.name).copied().unwrap_or(false);
                    rsx! {
                        div { 
                            key: "container-{props.key_base}-{container.name}",
                            class: "container-card",
                            div { class: "container-section-header",
                                div { class: "container-title",
                                    h5 { "{container.name}" }
                                }
                                div { class: "container-right",
                                    
                                    span { class: "image-tag", "{container.image}" }
                                    button {
                                        class: "btn-icon expand-toggle",
                                        onclick: move |_| toggle_container(container.name.clone()),
                                        title: if is_expanded { "Collapse" } else { "Expand" },
                                        if is_expanded { "🔼" } else { "🔽" }
                                    }
                                }
                            }
                            {is_expanded.then(|| rsx! {
                                div { class: "container-content",
                                    // Command and Args
                                    {(!container.command.is_empty() || !container.args.is_empty()).then(|| rsx! {
                                        div { class: "command-section",
                                            {(!container.command.is_empty()).then(|| rsx! {
                                                div { class: "command-item",
                                                    span { class: "command-label", "Command:" }
                                                    span { class: "command-value", "{container.command.join(\" \")}" }
                                                }
                                            })}
                                            {(!container.args.is_empty()).then(|| rsx! {
                                                div { class: "command-item",
                                                    span { class: "command-label", "Args:" }
                                                    span { class: "command-value", "{container.args.join(\" \")}" }
                                                }
                                            })}
                                        }
                                    })}

                                    // Environment Variables
                                    {(!container.env.is_empty()).then(|| rsx! {
                                        div { class: "env-section",
                                            h6 { "Environment Variables" }
                                            div { class: "env-grid",
                                                {container.env.iter().map(|env| rsx! {
                                                    div { class: "env-item",
                                                        span { class: "env-name", "{env.name}" }
                                                        span { class: "env-value",
                                                            {if let Some(value) = &env.value {
                                                                value.clone()
                                                            } else if let Some(value_from) = &env.value_from {
                                                                format!("(from {})", value_from)
                                                            } else {
                                                                "".to_string()
                                                            }}
                                                        }
                                                    }
                                                })}
                                            }
                                        }
                                    })}

                                    // Resources
                                    div { class: "resources-section",
                                        h6 { "Resources" }
                                        div { class: "resource-grid",
                                            div { class: "resource-group",
                                                span { class: "resource-label", "Requests:" }
                                                {container.resources.requests.iter().map(|(key, value)| rsx! {
                                                    div { class: "resource-item",
                                                        span { class: "resource-key", "{key}" }
                                                        span { class: "resource-value", "{value}" }
                                                    }
                                                })}
                                            }
                                            div { class: "resource-group",
                                                span { class: "resource-label", "Limits:" }
                                                {container.resources.limits.iter().map(|(key, value)| rsx! {
                                                    div { class: "resource-item",
                                                        span { class: "resource-key", "{key}" }
                                                        span { class: "resource-value", "{value}" }
                                                    }
                                                })}
                                            }
                                        }
                                    }

                                    // Volume Mounts
                                    {(!container.volume_mounts.is_empty()).then(|| rsx! {
                                        div { class: "volume-mounts-section",
                                            h6 { "Volume Mounts" }
                                            div { class: "volume-mounts-grid",
                                                {container.volume_mounts.iter().map(|mount| rsx! {
                                                    div { class: "volume-mount-item",
                                                        span { class: "volume-name", "{mount.name}" }
                                                        span { class: "mount-path", "{mount.mount_path}" }
                                                        span { class: "read-only",
                                                            if mount.read_only { "(read-only)" } else { "" }
                                                        }
                                                    }
                                                })}
                                            }
                                        }
                                    })}
                                }
                            })}
                        }
                    }
                })}
            }
        }
    }
}
