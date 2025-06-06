use k8s_openapi::api::core::v1::Pod;

#[derive(Debug, Clone)]
pub struct ProblemPod {
    pub name: String,
    pub namespace: String,
    pub issue_type: String,
    pub details: String,
    pub severity: String,
}

pub fn check_pod_status(pod: &Pod) -> Option<ProblemPod> {
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
