use std::time::{SystemTime, UNIX_EPOCH};
use k8s_openapi::{apimachinery::pkg::apis::meta::v1::Time, chrono::{DateTime, Utc}};

/// Formats a duration in seconds into a human-readable string (e.g., "2d", "5h", "30m")
pub fn format_duration(seconds: i64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m", seconds / 60)
    } else if seconds < 86400 {
        format!("{}h", seconds / 3600)
    } else {
        format!("{}d", seconds / 86400)
    }
}

/// Calculates the age of a resource based on its creation timestamp
pub fn calculate_age(creation_time: Option<&Time>) -> String {
    if let Some(creation) = creation_time {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let creation_secs = creation.0
            .signed_duration_since(DateTime::<Utc>::from(UNIX_EPOCH))
            .num_seconds();
        
        let age_secs = now - creation_secs;
        format_duration(age_secs)
    } else {
        "Unknown".to_string()
    }
}
