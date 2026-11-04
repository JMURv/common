use sqlx::{PgPool, postgres::PgListener};
use std::time::Duration;
use tokio::{sync::mpsc, task::JoinHandle, time::Instant};

/// PostgreSQL notifications are wake-up hints, never the source of truth.
/// Callers must subscribe before their first drain and re-drain after every wait.
pub struct PostgresWakeup {
    pool: PgPool,
    channel: &'static str,
    wake_tx: mpsc::Sender<()>,
    wake_rx: mpsc::Receiver<()>,
    listener_task: Option<JoinHandle<()>>,
    safety_interval: Duration,
}

impl PostgresWakeup {
    pub fn new(pool: PgPool, channel: &'static str) -> Self {
        let (wake_tx, wake_rx) = mpsc::channel(1);
        Self { pool, channel, wake_tx, wake_rx, listener_task: None, safety_interval: Duration::from_secs(60) }
    }

    /// Establishes LISTEN before returning, then leaves the connection in a
    /// dedicated task. Retry timers therefore cannot cancel an in-flight
    /// reconnect and leave the worker subscribed only nominally.
    pub async fn subscribe(&mut self) {
        if self.listener_task.as_ref().is_some_and(|task| !task.is_finished()) {
            return;
        }
        if let Some(task) = self.listener_task.take() {
            task.abort();
        }

        match PgListener::connect_with(&self.pool).await {
            Ok(mut listener) => match listener.listen(self.channel).await {
                Ok(()) => {
                    let channel = self.channel;
                    let wake_tx = self.wake_tx.clone();
                    self.listener_task = Some(tokio::spawn(async move {
                        listen(listener, wake_tx, channel).await;
                    }));
                }
                Err(error) => tracing::warn!(?error, channel = self.channel, "failed to subscribe to database wakeup"),
            },
            Err(error) => tracing::warn!(?error, channel = self.channel, "failed to connect database wakeup"),
        }
    }

    /// Sleeps until a committed notification, the persisted retry deadline, or
    /// a rare safety sweep that recovers notifications lost during reconnects.
    pub async fn wait(&mut self, retry_deadline: Option<Instant>) {
        self.subscribe().await;
        let deadline = effective_deadline(Instant::now(), self.safety_interval, retry_deadline);
        tokio::select! {
            _ = self.wake_rx.recv() => {}
            _ = tokio::time::sleep_until(deadline) => {}
        }
    }

    /// Convenience for workers whose persisted state cannot expose an exact
    /// retry deadline yet. Prefer [`Self::wait`] with the database deadline for
    /// retryable work; this method is mainly useful for scheduled maintenance.
    pub async fn wait_for(&mut self, delay: Duration) {
        self.wait(Some(Instant::now() + delay)).await;
    }
}

impl Drop for PostgresWakeup {
    fn drop(&mut self) {
        if let Some(task) = self.listener_task.take() {
            task.abort();
        }
    }
}

async fn listen(mut listener: PgListener, wake_tx: mpsc::Sender<()>, channel: &'static str) {
    loop {
        match listener.try_recv().await {
            Ok(Some(_)) => signal(&wake_tx),
            Ok(None) => {
                // PgListener has eagerly reconnected and restored LISTEN. Work
                // committed in the disconnected window is recovered by drain.
                signal(&wake_tx);
            }
            Err(error) => {
                tracing::warn!(?error, channel, "database wakeup listener disconnected");
                signal(&wake_tx);
                return;
            }
        }
    }
}

fn signal(wake_tx: &mpsc::Sender<()>) {
    // Wakeups are hints and may be coalesced: one drain sees all committed rows.
    let _ = wake_tx.try_send(());
}

fn effective_deadline(now: Instant, safety_interval: Duration, retry_deadline: Option<Instant>) -> Instant {
    let safety_deadline = now + safety_interval;
    retry_deadline.map_or(safety_deadline, |value| value.min(safety_deadline))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_deadline_wins_only_when_earlier_than_safety_sweep() {
        let now = Instant::now();
        assert_eq!(effective_deadline(now, Duration::from_secs(60), None), now + Duration::from_secs(60));
        assert_eq!(
            effective_deadline(now, Duration::from_secs(60), Some(now + Duration::from_secs(5))),
            now + Duration::from_secs(5),
        );
        assert_eq!(
            effective_deadline(now, Duration::from_secs(60), Some(now + Duration::from_secs(90))),
            now + Duration::from_secs(60),
        );
    }
}
