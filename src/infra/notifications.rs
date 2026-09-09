use notify_rust::{Hint, Notification};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::config::Settings;

static FAILURE_NOTIFIED: AtomicBool = AtomicBool::new(false);

pub fn notify_failure(settings: &Settings, summary: &str, body: Option<&str>) {
    if !settings.notifications.failure_notifications {
        return;
    }

    if FAILURE_NOTIFIED
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return;
    }

    let body = body.unwrap_or("Check the daemon logs for details.");

    let _ = Notification::new()
        .summary(summary)
        .body(body)
        .appname("INSS Watcher")
        .hint(Hint::Transient(true))
        .show();
}

pub fn notify_match(settings: &Settings, contributor: &str, reference_period: &str, docs_organized: usize) {
    if !settings.notifications.match_notifications {
        return;
    }

    let body = format!(
        "Contributor: {}\nReference period: {}\n\n{} documents organized",
        contributor, reference_period, docs_organized
    );

    let _ = Notification::new()
        .summary("Payment matched")
        .body(&body)
        .appname("INSS Watcher")
        .hint(Hint::Transient(true))
        .show();
}
