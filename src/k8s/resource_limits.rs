use k8s_openapi::api::core::v1::Pod;

#[derive(Clone)]
pub struct PodResourceIssue {
    pub name: String,
    pub namespace: String,
    pub issue_type: String,
    pub details: String,
}

pub fn check_pod_resource_limits(pod: &Pod) -> Option<PodResourceIssue> {
    let spec = pod.spec.as_ref()?;
    let name = pod.metadata.name.clone()?;
    let namespace = pod.metadata.namespace.clone()?;
    
    // Skip system namespaces
    if namespace == "kube-system" || namespace == "kube-public" {
        return None;
    }
    
    // Early return if pod has no containers
    if spec.containers.is_empty() {
        return None;
    }
    
    let mut has_partial_limits = false;
    let mut has_memory_limit = false;
    let mut has_cpu_limit = false;
    
    // Check each container until we find one with all limits or confirm none have limits
    for container in &spec.containers {
        if let Some(limits) = container.resources.as_ref().and_then(|r| r.limits.as_ref()) {
            has_memory_limit |= limits.contains_key("memory");
            has_cpu_limit |= limits.contains_key("cpu");
            
            // Early return if we found a container with no limits at all
            if limits.is_empty() {
                return Some(PodResourceIssue {
                    name,
                    namespace,
                    issue_type: "No Resource Limits".to_string(),
                    details: format!("Container '{}' is running without any resource limits, which could lead to resource contention", 
                        container.name),
                });
            }
            
            has_partial_limits |= limits.contains_key("memory") != limits.contains_key("cpu");
        } else {
            // No resources section or no limits section for this container
            return Some(PodResourceIssue {
                name,
                namespace,
                issue_type: "No Resource Limits".to_string(),
                details: format!("Container '{}' has no resource limits defined", container.name),
            });
        }
    }
    
    // After checking all containers, determine the overall status
    if !has_memory_limit && !has_cpu_limit {
        Some(PodResourceIssue {
            name,
            namespace,
            issue_type: "No Resource Limits".to_string(),
            details: "Pod is running without CPU or memory limits, which could lead to resource contention".to_string(),
        })
    } else if has_partial_limits {
        Some(PodResourceIssue {
            name,
            namespace,
            issue_type: "Partial Resource Limits".to_string(),
            details: if !has_cpu_limit {
                "Pod has memory limits but no CPU limits defined".to_string()
            } else {
                "Pod has CPU limits but no memory limits defined".to_string()
            },
        })
    } else {
        None
    }
}

pub async fn find_pods_without_limits(client: kube::Client) -> Vec<PodResourceIssue> {
    let pods: kube::Api<Pod> = kube::Api::all(client);
    let mut pods_without_limits = Vec::new();
    let mut params = kube::api::ListParams::default().limit(100); // Fetch pods in batches of 100

    loop {
        match pods.list(&params).await {
            Ok(pod_list) => {
                // Process this batch of pods
                pods_without_limits.extend(
                    pod_list.items
                        .iter()
                        .filter_map(check_pod_resource_limits)
                );

                // Check if we need to fetch more pods
                if let Some(continue_token) = pod_list.metadata.continue_ {
                    params = params.continue_token(&continue_token);
                } else {
                    break;
                }
            }
            Err(_e) => {
                break;
            }
        }
    }

    pods_without_limits
}
