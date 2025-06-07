use k8s_openapi::api::core::v1::{ConfigMap, Pod};
use kube::{api::ListParams, Api, Client};

/// Check if a configmap is referenced by a pod's volumes or environment variables
pub fn is_configmap_used_by_pod(
    configmap_name: String,
    configmap_namespace: String,
    pod: &Pod,
) -> bool {
    let pod_namespace = pod
        .metadata
        .namespace
        .as_ref()
        .map(String::as_str)
        .unwrap_or_default();

    // Only check pods in the same namespace as the configmap
    if pod_namespace != configmap_namespace {
        return false;
    }

    let pod_spec = match &pod.spec {
        Some(spec) => spec,
        None => return false,
    };

    // Check volumes
    if let Some(volumes) = &pod_spec.volumes {
        for volume in volumes {
            if let Some(config_map) = &volume.config_map {
                if config_map.name == configmap_name {
                    return true;
                }
            }
        }
    }

    // Check containers for environment variables
    for container in &pod_spec.containers {
        // Check envFrom
        if let Some(env_from) = &container.env_from {
            for env_source in env_from {
                if let Some(config_map) = &env_source.config_map_ref {
                    if config_map.name == configmap_name {
                        return true;
                    }
                }
            }
        }

        // Check individual env vars
        if let Some(env) = &container.env {
            for env_var in env {
                if let Some(value_from) = &env_var.value_from {
                    if let Some(config_map_key) = &value_from.config_map_key_ref {
                        if config_map_key.name == configmap_name {
                            return true;
                        }
                    }
                }
            }
        }
    }

    false
}

/// Find unused ConfigMaps in the cluster
pub async fn find_unused_configmaps(client: Client) -> Vec<(ConfigMap, String)> {
    let pods: Api<Pod> = Api::all(client.clone());
    let configmaps: Api<ConfigMap> = Api::all(client);

    let mut unused_configmaps = Vec::new();

    // Get all pods first
    let pod_list = match pods.list(&ListParams::default()).await {
        Ok(list) => list.items,
        Err(e) => {
            return Vec::new();
        }
    };

    // Get all configmaps
    match configmaps.list(&ListParams::default()).await {
        Ok(configmap_list) => {
            for configmap in configmap_list.items {
                // skip kube-root-ca.crt 
                if configmap.metadata.name.as_deref() == Some("kube-root-ca.crt") {
                    continue;
                }

                let name = configmap.metadata.name.clone().unwrap_or_default();
                let namespace = configmap.metadata.namespace.clone().unwrap_or_default();

                // Skip system configmaps
                if namespace == "kube-system" || namespace == "kube-public" {
                    continue;
                }

                // Check if the configmap is used by any pod
                let is_used = pod_list
                    .iter()
                    .any(|pod| is_configmap_used_by_pod(name.clone(), namespace.clone(), pod));

                if !is_used {
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
        Err(_e) => {}
    }

    unused_configmaps
}
