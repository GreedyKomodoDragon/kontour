use dioxus::logger::tracing;
use k8s_openapi::{api::core::v1::Event, chrono::{DateTime, Utc}};
use kube::{
    api::{Api, ListParams},
    Client,
};

fn get_datetime(event: &Event) -> DateTime<Utc> {
    // Try last_timestamp first (Time)
    if let Some(ts) = &event.last_timestamp {
        return ts.0;
    }
    // Then try event_time (MicroTime)
    if let Some(ts) = &event.event_time {
        return ts.0;
    }
    // Finally try first_timestamp (Time)
    event.first_timestamp.as_ref().map(|ts| ts.0).unwrap_or_else(|| Utc::now())
}

pub async fn get_recent_events(client: Client) -> Vec<Event> {
    let events: Api<Event> = Api::all(client);
    let params = ListParams::default()
        .limit(5)  // Fetch at most 10 events
        .timeout(10);  // Add a reasonable timeout

    match events.list(&params).await {
        Ok(event_list) => {
            let mut events = event_list.items;
            // Sort by timestamp, most recent first
            events.sort_by(|a, b| {
                let a_time = get_datetime(a);
                let b_time = get_datetime(b);
                b_time.cmp(&a_time)
            });
            events
        }
        Err(e) => {
            tracing::error!("Failed to fetch events: {}", e);
            Vec::new()
        }
    }
}
