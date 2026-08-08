use notify_rust::{Notification, Hint};
use std::sync::atomic::{AtomicBool, Ordering};

static NOTIFIED: AtomicBool = AtomicBool::new(false);

pub fn notify_failure(summary: &str, body: Option<&str>) {
    if NOTIFIED
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
