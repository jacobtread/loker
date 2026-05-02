use crate::database::secrets::{delete_excess_secret_versions, delete_scheduled_secrets};
use chrono::Utc;
use futures::StreamExt;
use tokio_rusqlite::Connection;
use tokio_simple_fixed_scheduler::{SchedulerEventStream, SchedulerQueueEvent};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum BackgroundEvent {
    /// Task to purge scheduled secrets
    PurgeDeletedSecrets,

    /// Task to prune the secrets with versions in excess of 100 versions that are
    /// over 24h old
    PurgeExcessSecrets,
}

pub async fn perform_background_tasks(db: Connection) {
    let events = vec![
        SchedulerQueueEvent {
            event: BackgroundEvent::PurgeDeletedSecrets,
            interval: 60 * 60,
        },
        SchedulerQueueEvent {
            event: BackgroundEvent::PurgeExcessSecrets,
            interval: 60 * 60,
        },
    ];

    let mut events = SchedulerEventStream::new(events);

    while let Some(event) = events.next().await {
        match event {
            BackgroundEvent::PurgeDeletedSecrets => {
                tracing::debug!("performing background purge for presigned tasks");
                let now = Utc::now();

                if let Err(error) = db.call(move |db| delete_scheduled_secrets(db, now)).await {
                    tracing::error!(?error, "failed to performed scheduled secrets deletion")
                }
            }

            BackgroundEvent::PurgeExcessSecrets => {
                tracing::debug!("performing background deletion for secret version limits");

                if let Err(error) = db.call(move |db| delete_excess_secret_versions(db)).await {
                    tracing::error!(
                        ?error,
                        "failed to performed background deletion for secret version limits"
                    )
                }
            }
        }
    }
}
