//! macOS 键盘模拟须在主线程执行（HIToolbox/TSM 会 dispatch_assert_queue）。

use std::sync::mpsc;

use dispatch::Queue;
use objc2::MainThreadMarker;

pub fn run_on_main_thread<R, F>(f: F) -> R
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    if MainThreadMarker::new().is_some() {
        return f();
    }

    let (tx, rx) = mpsc::sync_channel(1);
    Queue::main().exec_async(move || {
        let _ = tx.send(f());
    });
    rx.recv().expect("main thread keyboard dispatch")
}
