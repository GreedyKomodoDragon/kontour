use k8s_openapi::api::core::v1::Pod;

#[derive(Default, Debug, Clone)]
pub struct ClusterStats {
    pub crashloop_count: usize,
    pub restart_count: usize,
    pub evicted_count: usize,
}

impl ClusterStats {
    pub fn compute_from_pods(pods: &[Pod]) -> Self {
        let mut stats = ClusterStats::default();
        
        for pod in pods {
            if let Some(container_statuses) = pod.status.as_ref().and_then(|s| s.container_statuses.as_ref()) {
                for container in container_statuses.iter() {
                    if let Some(waiting) = container.state.as_ref().and_then(|s| s.waiting.as_ref()) {
                        if waiting.reason.as_deref() == Some("CrashLoopBackOff") {
                            stats.crashloop_count += 1;
                        }
                    }
                    if container.restart_count > 5 {
                        stats.restart_count += 1;
                    }
                }
            }
            if let Some(reason) = &pod.status.as_ref().and_then(|s| s.reason.as_ref()) {
                if *reason == "Evicted" {
                    stats.evicted_count += 1;
                }
            }
        }
        
        stats
    }
}
