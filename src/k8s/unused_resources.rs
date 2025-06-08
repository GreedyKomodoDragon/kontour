use k8s_openapi::api::core::v1::{ConfigMap, PersistentVolumeClaim, Pod};
use kube::{api::ListParams, Api, Client};

/// Find unused ConfigMaps in the cluster
pub async fn find_unused_configmaps(client: Client) -> Vec<(ConfigMap, String)> {
    let pods: Api<Pod> = Api::all(client.clone());
    let configmaps: Api<ConfigMap> = Api::all(client);
    let mut unused_configmaps = Vec::new();

    // Get all pods first
    let pod_list = match pods.list(&ListParams::default()).await {
        Ok(list) => list.items,
        Err(_) => return Vec::new(),
    };

    // Build a set of used ConfigMaps for efficient lookup
    let mut used_configmaps = std::collections::HashSet::new();
    for pod in &pod_list {
        let pod_namespace = pod.metadata.namespace.as_deref().unwrap_or_default();

        if let Some(spec) = &pod.spec {
            // Check volumes
            if let Some(volumes) = &spec.volumes {
                for volume in volumes {
                    if let Some(config_map) = &volume.config_map {
                        used_configmaps.insert(format!("{}:{}", pod_namespace, config_map.name));
                    }
                }
            }

            // Check containers
            for container in &spec.containers {
                // Check envFrom
                if let Some(env_from) = &container.env_from {
                    for env_source in env_from {
                        if let Some(config_map_ref) = &env_source.config_map_ref {
                            used_configmaps
                                .insert(format!("{}:{}", pod_namespace, config_map_ref.name));
                        }
                    }
                }

                // Check individual env vars
                if let Some(env) = &container.env {
                    for env_var in env {
                        if let Some(value_from) = &env_var.value_from {
                            if let Some(config_map_key_ref) = &value_from.config_map_key_ref {
                                used_configmaps.insert(format!(
                                    "{}:{}",
                                    pod_namespace, config_map_key_ref.name
                                ));
                            }
                        }
                    }
                }
            }
        }
    }

    // Get and check all configmaps
    if let Ok(configmap_list) = configmaps.list(&ListParams::default()).await {
        for configmap in configmap_list.items {
            let name = configmap.metadata.name.clone().unwrap_or_default();
            let namespace = configmap.metadata.namespace.clone().unwrap_or_default();

            // Skip kube-root-ca.crt and system namespaces
            if name == "kube-root-ca.crt"
                || namespace == "kube-system"
                || namespace == "kube-public"
            {
                continue;
            }

            let key = format!("{}:{}", namespace, name);
            if !used_configmaps.contains(&key) {
                unused_configmaps.push((
                    configmap,
                    format!(
                        "ConfigMap '{}' in namespace '{}' is not mounted by any pods",
                        name, namespace
                    ),
                ));
            }
        }
    }

    unused_configmaps
}

/// Find unused PersistentVolumeClaims (PVCs) in the cluster
pub async fn find_unused_pvcs(client: Client) -> Vec<(PersistentVolumeClaim, String)> {
    let pvcs: Api<PersistentVolumeClaim> = Api::all(client.clone());
    let pods: Api<Pod> = Api::all(client);
    let mut unused_pvcs = Vec::new();

    // Get all resources
    let pvc_list = match pvcs.list(&ListParams::default()).await {
        Ok(list) => list.items,
        Err(_) => return Vec::new(),
    };

    let pod_list = match pods.list(&ListParams::default()).await {
        Ok(list) => list.items,
        Err(_) => return Vec::new(),
    };

    // Build a set of used PVCs for efficient lookup
    let mut used_pvcs = std::collections::HashSet::new();
    for pod in &pod_list {
        if let Some(spec) = &pod.spec {
            if let Some(volumes) = &spec.volumes {
                for volume in volumes {
                    if let Some(pvc_source) = &volume.persistent_volume_claim {
                        if let Some(namespace) = &pod.metadata.namespace {
                            // Store namespace:name as the key
                            used_pvcs.insert(format!("{}:{}", namespace, &pvc_source.claim_name));
                        }
                    }
                }
            }
        }
    }

    // Check each PVC against the set of used PVCs
    for pvc in pvc_list {
        let pvc_name = pvc.metadata.name.clone().unwrap_or_default();
        let pvc_namespace = pvc.metadata.namespace.clone().unwrap_or_default();
        let key = format!("{}:{}", pvc_namespace, pvc_name);

        // If PVC is not in the used set, it's unused
        if !used_pvcs.contains(&key) {
            let reason = format!(
                "PVC '{}' in namespace '{}' is not used by any pod",
                pvc_name, pvc_namespace
            );
            unused_pvcs.push((pvc, reason));
        }
    }

    unused_pvcs
}
