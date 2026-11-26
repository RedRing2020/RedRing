use std::io;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use super::{BackgroundTask, TaskContext, TaskFailure, TaskOutcome};

/// 試験が止まったままにならないための、待つ時間の上限
const TIMEOUT: Duration = Duration::from_secs(30);

/// 結果が出るまで、短い間をおいて問い合わせる
fn wait_outcome<T, E, P>(task: &mut BackgroundTask<T, E, P>) -> TaskOutcome<T, E> {
    let started = Instant::now();
    loop {
        if let Some(outcome) = task.take_outcome() {
            return outcome;
        }
        assert!(started.elapsed() < TIMEOUT, "the task did not finish");
        std::thread::sleep(Duration::from_millis(1));
    }
}

/// 条件 `satisfied` が成り立つまで、短い間をおいて確かめる
fn wait_until(mut satisfied: impl FnMut() -> bool) {
    let started = Instant::now();
    while !satisfied() {
        assert!(started.elapsed() < TIMEOUT, "the condition did not hold");
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn a_successful_outcome_is_taken_only_once() {
    let mut task =
        BackgroundTask::<i32, String, ()>::spawn("test-task", |_| Ok(42)).expect("spawn");

    assert_eq!(wait_outcome(&mut task), TaskOutcome::Succeeded(42));
    assert_eq!(task.take_outcome(), None);
}

#[test]
fn a_failure_is_returned_as_the_outcome() {
    let mut task = BackgroundTask::<i32, String, ()>::spawn("test-task", |_| {
        Err(TaskFailure::Failed("broken".to_string()))
    })
    .expect("spawn");

    assert_eq!(
        wait_outcome(&mut task),
        TaskOutcome::Failed("broken".to_string())
    );
}

#[test]
fn the_outcome_is_none_without_waiting_while_the_work_runs() {
    let (release, released) = mpsc::channel::<()>();
    let mut task = BackgroundTask::<(), String, ()>::spawn("test-task", move |_| {
        released.recv().expect("release");
        Ok(())
    })
    .expect("spawn");

    assert_eq!(task.take_outcome(), None);

    release.send(()).expect("send");
    assert_eq!(wait_outcome(&mut task), TaskOutcome::Succeeded(()));
}

#[test]
fn only_the_latest_progress_is_kept() {
    let (release, released) = mpsc::channel::<()>();
    let (reported_sender, reported) = mpsc::channel::<()>();
    let mut task =
        BackgroundTask::<(), String, (usize, usize)>::spawn("test-task", move |context| {
            released.recv().expect("release");
            for done in 1..=3 {
                context.report_progress((done, 3));
            }
            reported_sender.send(()).expect("reported");
            released.recv().expect("release");
            Ok(())
        })
        .expect("spawn");

    // 処理が知らせる前
    assert_eq!(task.progress(), None);

    release.send(()).expect("send");
    reported.recv_timeout(TIMEOUT).expect("reported");
    // 途中の値は捨て、何度問い合わせても最も新しい値を返す
    assert_eq!(task.progress(), Some((3, 3)));
    assert_eq!(task.progress(), Some((3, 3)));
    assert_eq!(task.take_outcome(), None);

    release.send(()).expect("send");
    assert_eq!(wait_outcome(&mut task), TaskOutcome::Succeeded(()));
    // 終わった後も、最後に知らせた値を返す
    assert_eq!(task.progress(), Some((3, 3)));
}

#[test]
fn a_cancel_request_stops_the_work_at_its_next_check() {
    let (started_sender, started) = mpsc::channel::<()>();
    let mut task = BackgroundTask::<(), String, ()>::spawn("test-task", move |context| {
        started_sender.send(()).expect("started");
        loop {
            if context.is_cancel_requested() {
                return Err(TaskFailure::Cancelled);
            }
            std::thread::sleep(Duration::from_millis(1));
        }
    })
    .expect("spawn");
    started.recv_timeout(TIMEOUT).expect("started");

    assert!(!task.is_cancel_requested());
    assert_eq!(task.take_outcome(), None);

    task.request_cancel();

    assert!(task.is_cancel_requested());
    assert_eq!(wait_outcome(&mut task), TaskOutcome::Cancelled);
}

#[test]
fn work_that_finishes_without_checking_the_request_is_not_cancelled() {
    let (release, released) = mpsc::channel::<()>();
    let mut task = BackgroundTask::<i32, String, ()>::spawn("test-task", move |_| {
        released.recv().expect("release");
        Ok(7)
    })
    .expect("spawn");

    task.request_cancel();
    release.send(()).expect("send");

    assert_eq!(wait_outcome(&mut task), TaskOutcome::Succeeded(7));
}

#[test]
fn a_panic_in_the_work_is_returned_as_the_outcome() {
    let mut literal = BackgroundTask::<(), String, ()>::spawn("test-task", |_| {
        panic!("broken in the task");
    })
    .expect("spawn");
    let mut formatted = BackgroundTask::<(), String, ()>::spawn("test-task", |_| {
        let index = 3;
        panic!("broken at {index}");
    })
    .expect("spawn");
    let mut not_text = BackgroundTask::<(), String, ()>::spawn("test-task", |_| {
        std::panic::panic_any(5_i32);
    })
    .expect("spawn");

    assert_eq!(
        wait_outcome(&mut literal),
        TaskOutcome::Panicked("broken in the task".to_string())
    );
    assert_eq!(
        wait_outcome(&mut formatted),
        TaskOutcome::Panicked("broken at 3".to_string())
    );
    assert_eq!(
        wait_outcome(&mut not_text),
        TaskOutcome::Panicked(String::new())
    );
}

#[test]
fn progress_and_outcome_survive_a_panic_after_a_report() {
    let mut task = BackgroundTask::<(), String, u32>::spawn("test-task", |context| {
        context.report_progress(1);
        panic!("broken after the progress");
    })
    .expect("spawn");

    assert_eq!(
        wait_outcome(&mut task),
        TaskOutcome::Panicked("broken after the progress".to_string())
    );
    assert_eq!(task.progress(), Some(1));
}

#[test]
fn the_work_runs_on_another_named_thread() {
    let caller = std::thread::current().id();
    let mut task = BackgroundTask::<_, String, ()>::spawn("test-task-name", |_| {
        let thread = std::thread::current();
        Ok((thread.id(), thread.name().map(str::to_string)))
    })
    .expect("spawn");

    let TaskOutcome::Succeeded((id, name)) = wait_outcome(&mut task) else {
        panic!("the task did not succeed");
    };
    assert_ne!(id, caller);
    assert_eq!(name.as_deref(), Some("test-task-name"));
}

#[test]
fn dropping_the_task_requests_cancel() {
    let (started_sender, started) = mpsc::channel::<()>();
    let (cancelled_sender, cancelled) = mpsc::channel::<()>();
    let task = BackgroundTask::<(), String, ()>::spawn("test-task", move |context| {
        started_sender.send(()).expect("started");
        wait_until(|| context.is_cancel_requested());
        cancelled_sender.send(()).expect("cancelled");
        Err(TaskFailure::Cancelled)
    })
    .expect("spawn");
    started.recv_timeout(TIMEOUT).expect("started");

    drop(task);

    cancelled.recv_timeout(TIMEOUT).expect("cancelled");
}

#[test]
fn a_spawn_failure_returns_the_work_to_run_on_the_current_thread() {
    let work = |context: &TaskContext<u32>| {
        context.report_progress(1);
        Ok::<_, TaskFailure<String>>(std::thread::current().id())
    };

    let error = BackgroundTask::spawn_with(work, |_| Err(io::Error::other("no thread")))
        .err()
        .expect("spawn must fail");
    assert_eq!(
        error.to_string(),
        "cannot start a thread for the background task: no thread"
    );

    let mut task = BackgroundTask::run_on_current_thread(error.into_work());

    assert_eq!(task.progress(), Some(1));
    assert_eq!(
        task.take_outcome(),
        Some(TaskOutcome::Succeeded(std::thread::current().id()))
    );
    assert_eq!(task.take_outcome(), None);
}

#[test]
fn a_panic_on_the_current_thread_is_returned_as_the_outcome() {
    let mut task = BackgroundTask::<(), String, ()>::run_on_current_thread(|_| {
        panic!("broken on the current thread");
    });

    assert_eq!(
        task.take_outcome(),
        Some(TaskOutcome::Panicked(
            "broken on the current thread".to_string()
        ))
    );
}

/// 手放すときに panic する値
struct PanicsOnDrop;

impl Drop for PanicsOnDrop {
    fn drop(&mut self) {
        panic!("broken while dropping the panic payload");
    }
}

#[test]
fn a_panic_payload_that_panics_on_drop_still_gives_an_outcome() {
    let mut task = BackgroundTask::<(), String, ()>::spawn("test-task", |_| {
        std::panic::panic_any(PanicsOnDrop);
    })
    .expect("spawn");

    assert_eq!(
        wait_outcome(&mut task),
        TaskOutcome::Panicked(String::new())
    );
}

/// 手放されたことを知らせる値
struct ReportsDrop(mpsc::Sender<()>);

impl Drop for ReportsDrop {
    fn drop(&mut self) {
        // 受け手が先に手放されていても、試験の結果には関わらない
        let _ = self.0.send(());
    }
}

#[test]
fn a_panic_payload_is_dropped_after_the_panic_is_caught() {
    let (dropped_sender, dropped) = mpsc::channel::<()>();
    let mut task = BackgroundTask::<(), String, ()>::spawn("test-task", move |_| {
        std::panic::panic_any(ReportsDrop(dropped_sender));
    })
    .expect("spawn");

    assert_eq!(
        wait_outcome(&mut task),
        TaskOutcome::Panicked(String::new())
    );
    dropped
        .recv_timeout(TIMEOUT)
        .expect("the payload is dropped");
}

/// 手放している間、合図を待つ進み具合
struct SlowToDrop {
    value: u32,
    dropping: Option<mpsc::Sender<()>>,
    release: Option<mpsc::Receiver<()>>,
}

impl Clone for SlowToDrop {
    fn clone(&self) -> Self {
        Self {
            value: self.value,
            dropping: None,
            release: None,
        }
    }
}

impl Drop for SlowToDrop {
    fn drop(&mut self) {
        if let (Some(dropping), Some(release)) = (&self.dropping, &self.release) {
            dropping.send(()).expect("dropping");
            release.recv_timeout(TIMEOUT).expect("release");
        }
    }
}

#[test]
fn a_replaced_progress_is_dropped_without_blocking_the_query() {
    let (dropping_sender, dropping) = mpsc::channel::<()>();
    let (release, released) = mpsc::channel::<()>();
    let mut task = BackgroundTask::<(), String, SlowToDrop>::spawn("test-task", move |context| {
        context.report_progress(SlowToDrop {
            value: 1,
            dropping: Some(dropping_sender),
            release: Some(released),
        });
        // 前の値を手放し終えるまで、ここから戻らない
        context.report_progress(SlowToDrop {
            value: 2,
            dropping: None,
            release: None,
        });
        Ok(())
    })
    .expect("spawn");

    // 処理が前の値を手放している間も、新しい値を待たずに受け取れる
    dropping.recv_timeout(TIMEOUT).expect("dropping");
    assert_eq!(task.progress().map(|progress| progress.value), Some(2));
    assert_eq!(task.take_outcome(), None);

    release.send(()).expect("send");
    assert_eq!(wait_outcome(&mut task), TaskOutcome::Succeeded(()));
}
