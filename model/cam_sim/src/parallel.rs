//! 計算を並べるスレッド

use std::thread::{Scope, ScopedJoinHandle};

/// 呼んだスレッドと同じ優先度で、スコープのスレッドを作る
///
/// Windows では、作ったスレッドは呼んだスレッドの優先度を受け継がずに通常の優先度で始まるため、呼んだスレッドの
/// 優先度を移す（Linux では受け継がれる）。優先度を移せない場合は、通常の優先度のまま計算する。
pub(crate) fn spawn_scoped<'scope, F, T>(
    scope: &'scope Scope<'scope, '_>,
    f: F,
) -> ScopedJoinHandle<'scope, T>
where
    F: FnOnce() -> T + Send + 'scope,
    T: Send + 'scope,
{
    #[cfg(windows)]
    let priority = thread_priority::get_current_thread_priority().ok();
    scope.spawn(move || {
        #[cfg(windows)]
        if let Some(priority) = priority {
            // 移せなくても計算の結果は変わらない
            let _ = priority.set_for_current();
        }
        f()
    })
}

#[cfg(all(test, windows))]
mod tests {
    use thread_priority::{ThreadPriority, get_current_thread_priority};

    use super::spawn_scoped;

    #[test]
    fn scoped_threads_run_at_the_priority_of_the_calling_thread() {
        std::thread::spawn(|| {
            ThreadPriority::Min.set_for_current().unwrap();
            let caller = get_current_thread_priority().unwrap();
            let spawned = std::thread::scope(|scope| {
                spawn_scoped(scope, || get_current_thread_priority().unwrap())
                    .join()
                    .unwrap()
            });
            assert_eq!(spawned, caller);
            // 移さない場合は通常の優先度で始まる（呼んだスレッドと違う）
            let plain = std::thread::scope(|scope| {
                scope
                    .spawn(|| get_current_thread_priority().unwrap())
                    .join()
                    .unwrap()
            });
            assert_ne!(plain, caller);
        })
        .join()
        .unwrap();
    }
}
