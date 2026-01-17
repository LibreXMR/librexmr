use xmr_wallet::{LockWatcher, WatcherConfig, WatcherEvent};

#[test]
fn reports_no_lock_without_observation() {
    let watcher = LockWatcher::new(WatcherConfig::default());
    let event = watcher.evaluate(100);
    assert_eq!(event, WatcherEvent::NoLockObserved);
}

#[test]
fn waits_for_confirmations_then_confirms() {
    let config = WatcherConfig {
        confirmations_required: 10,
        reorg_buffer: 5,
    };
    let mut watcher = LockWatcher::new(config);
    watcher.observe_lock(50);

    let awaiting = watcher.evaluate(55);
    assert_eq!(
        awaiting,
        WatcherEvent::AwaitingConfirmations {
            observed_height: 50,
            current_height: 55,
            remaining: 5,
        }
    );

    let confirmed = watcher.evaluate(60);
    assert_eq!(
        confirmed,
        WatcherEvent::Confirmed {
            observed_height: 50,
            confirmations: 10,
        }
    );
}

#[test]
fn detects_reorg_and_clears_lock() {
    let config = WatcherConfig {
        confirmations_required: 10,
        reorg_buffer: 5,
    };
    let mut watcher = LockWatcher::new(config);
    watcher.observe_lock(50);
    watcher.update_height(100);

    let reorg = watcher.update_height(90);
    assert_eq!(
        reorg,
        Some(WatcherEvent::ReorgDetected {
            previous_height: 100,
            current_height: 90,
        })
    );

    let after = watcher.evaluate(90);
    assert_eq!(after, WatcherEvent::NoLockObserved);
}
