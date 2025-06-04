//! The views module contains the components for all Layouts and Routes for our app.
//! Each submodule corresponds to a specific section of the application,
//! encapsulating the layout and routing logic for that section.

mod deployments;
pub use deployments::Deployments;

mod statefulsets;
pub use statefulsets::StatefulSets;

mod daemonsets;
pub use daemonsets::DaemonSets;

mod services;
pub use services::Services;

mod ingresses;
pub use ingresses::Ingresses;

mod pvcs; // Add module
pub use pvcs::Pvcs; // Add export

mod configmaps; // Add module
pub use configmaps::ConfigMaps; // Add export

mod secrets; // Add module
pub use secrets::Secrets; // Add export

mod jobs;
pub use jobs::Jobs;

mod cronjobs;
pub use cronjobs::CronJobs;

mod home;
pub use home::Home;

mod namespaces;
pub use namespaces::Namespaces;

mod navbar;
pub use navbar::Navbar;

mod nodes;
pub use nodes::Nodes;

mod pods;
pub use pods::Pods;

mod create_pod;
pub use create_pod::CreatePod;

mod create_namespace;
pub use create_namespace::CreateNamespace;

mod create_deployment;
pub use create_deployment::CreateDeployment;

mod create_statefulset;
pub use create_statefulset::CreateStatefulSet;

mod create_daemonset;
pub use create_daemonset::CreateDaemonSet;

mod create_cronjob;
pub use create_cronjob::CreateCronJob;