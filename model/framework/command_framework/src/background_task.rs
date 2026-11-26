//! 時間のかかる処理 1 つを別のスレッドで行い、始めた側から、進み具合と結果を待たずに問い合わせる仕組み
//!
//! - 処理は、[`TaskContext`] を受け取り、区切りごとに取り消しを確かめ、進み具合を知らせる
//! - 始めた側は、[`BackgroundTask`] に、進み具合・結果を問い合わせ、取り消しを頼む。どれも待たない
//! - 進み具合は、最も新しい値だけを持つ（途中の値は捨てる）
//! - 処理の中の panic は、処理のスレッドで捕まえ、結果（[`TaskOutcome::Panicked`]）として返す
//!
//! 結果の型・誤りの型・進み具合の型は、処理ごとに決める。

use std::any::Any;
use std::error::Error;
use std::fmt;
use std::io;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

#[cfg(test)]
mod tests;

/// 処理が、取り消しを確かめて進み具合を知らせる口
///
/// `P` は進み具合の型。
pub struct TaskContext<P> {
    signals: Arc<Signals<P>>,
}

impl<P> TaskContext<P> {
    /// 取り消しを頼まれているか
    ///
    /// 処理は、区切りごとにこれを確かめ、頼まれていれば止めて [`TaskFailure::Cancelled`] を返す。
    pub fn is_cancel_requested(&self) -> bool {
        self.signals.cancel_requested.load(Ordering::Relaxed)
    }

    /// 進み具合を知らせる
    ///
    /// 前に知らせた値は、問い合わせられていなくても置き換える。
    pub fn report_progress(&self, progress: P) {
        *lock(&self.signals.progress) = Some(progress);
    }
}

/// 処理が、結果を出さずに終わった理由
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskFailure<E> {
    /// 取り消しを頼まれて止めた
    Cancelled,
    /// 誤りで終わった
    Failed(E),
}

/// 処理の結果
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskOutcome<T, E> {
    /// 成功した
    Succeeded(T),
    /// 誤りで終わった
    Failed(E),
    /// 取り消しを頼まれて止めた
    Cancelled,
    /// 処理の中で panic した（panic の文。文字列でない panic は空）
    Panicked(String),
}

/// 処理のスレッドを作れなかった誤り
///
/// 始めなかった処理を持つ。[`Self::into_work`] で取り出し、[`BackgroundTask::run_on_current_thread`] で
/// 今のスレッドで行える。
pub struct SpawnError<W> {
    source: io::Error,
    work: W,
}

impl<W> SpawnError<W> {
    /// 始めなかった処理を取り出す
    pub fn into_work(self) -> W {
        self.work
    }
}

impl<W> fmt::Debug for SpawnError<W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SpawnError")
            .field("source", &self.source)
            .finish_non_exhaustive()
    }
}

impl<W> fmt::Display for SpawnError<W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "cannot start a thread for the background task: {}",
            self.source
        )
    }
}

impl<W> Error for SpawnError<W> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

/// 別のスレッドで行っている処理 1 つ
///
/// `T` は結果の型、`E` は誤りの型、`P` は進み具合の型。
///
/// 結果を受け取らずに手放すと、取り消しを頼む（処理のスレッドは、処理が止まるまで残る）。
pub struct BackgroundTask<T, E, P> {
    signals: Arc<Signals<P>>,
    outcome: Arc<Mutex<Option<TaskOutcome<T, E>>>>,
}

impl<T, E, P> BackgroundTask<T, E, P>
where
    T: Send + 'static,
    E: Send + 'static,
    P: Send + 'static,
{
    /// 処理 `work` を、名前 `name` を付けた新しいスレッドで始める
    ///
    /// # Errors
    ///
    /// スレッドを作れない場合。誤りは、始めなかった処理を持つ。
    ///
    /// # Panics
    ///
    /// `name` が 0 のバイトを含む場合。
    pub fn spawn<W>(name: &str, work: W) -> Result<Self, SpawnError<W>>
    where
        W: FnOnce(&TaskContext<P>) -> Result<T, TaskFailure<E>> + Send + 'static,
    {
        Self::spawn_with(work, |body| {
            std::thread::Builder::new()
                .name(name.to_string())
                .spawn(body)
                .map(drop)
        })
    }

    /// 処理 `work` を、`start_thread` で作ったスレッドで始める
    ///
    /// `start_thread` は、受け取った閉包を新しいスレッドで呼ぶか、呼ばずに誤りを返す。
    fn spawn_with<W>(
        work: W,
        start_thread: impl FnOnce(Box<dyn FnOnce() + Send>) -> io::Result<()>,
    ) -> Result<Self, SpawnError<W>>
    where
        W: FnOnce(&TaskContext<P>) -> Result<T, TaskFailure<E>> + Send + 'static,
    {
        let task = Self::new();
        let context = task.context();
        let outcome = Arc::clone(&task.outcome);
        // スレッドを作れなかったときに処理を取り戻せるよう、処理は共有の置き場から取り出して始める
        let pending = Arc::new(Mutex::new(Some(work)));
        let taken = Arc::clone(&pending);
        let body = move || {
            let work = lock(&taken).take();
            if let Some(work) = work {
                run(work, &context, &outcome);
            }
        };
        match start_thread(Box::new(body)) {
            Ok(()) => Ok(task),
            Err(source) => match lock(&pending).take() {
                Some(work) => Err(SpawnError { source, work }),
                None => unreachable!("a thread that failed to start cannot have taken the work"),
            },
        }
    }

    /// 処理 `work` を、今のスレッドで終わりまで行う
    ///
    /// 戻り値は、終わった処理になる（[`Self::take_outcome`] で結果を受け取る）。処理の中の panic は、
    /// 別のスレッドで行うときと同じく、結果として返す。
    pub fn run_on_current_thread<W>(work: W) -> Self
    where
        W: FnOnce(&TaskContext<P>) -> Result<T, TaskFailure<E>>,
    {
        let task = Self::new();
        run(work, &task.context(), &task.outcome);
        task
    }

    fn new() -> Self {
        Self {
            signals: Arc::new(Signals {
                cancel_requested: AtomicBool::new(false),
                progress: Mutex::new(None),
            }),
            outcome: Arc::new(Mutex::new(None)),
        }
    }

    fn context(&self) -> TaskContext<P> {
        TaskContext {
            signals: Arc::clone(&self.signals),
        }
    }
}

impl<T, E, P> BackgroundTask<T, E, P> {
    /// 最も新しい進み具合（処理がまだ知らせていなければ `None`）
    ///
    /// 処理が終わった後も、最後に知らせた値を返す。
    pub fn progress(&self) -> Option<P>
    where
        P: Clone,
    {
        lock(&self.signals.progress).clone()
    }

    /// 取り消しを頼む（処理が止まるのを待たない）
    ///
    /// 処理が止まるまでは、[`Self::take_outcome`] は `None` を返す。処理が取り消しを確かめる前に終われば、
    /// 結果は取り消しにならない。
    pub fn request_cancel(&self) {
        self.signals.cancel_requested.store(true, Ordering::Relaxed);
    }

    /// 取り消しを頼んであるか
    pub fn is_cancel_requested(&self) -> bool {
        self.signals.cancel_requested.load(Ordering::Relaxed)
    }

    /// 処理が終わっていれば、結果を受け取る
    ///
    /// 処理が終わっていないとき、結果を受け取った後は `None`（結果は 1 回だけ受け取れる）。結果を受け取った後は、
    /// 問い合わせを続けずに手放す。
    pub fn take_outcome(&mut self) -> Option<TaskOutcome<T, E>> {
        lock(&self.outcome).take()
    }
}

impl<T, E, P> Drop for BackgroundTask<T, E, P> {
    fn drop(&mut self) {
        self.request_cancel();
    }
}

/// 始めた側と処理で共有する印と進み具合
struct Signals<P> {
    cancel_requested: AtomicBool,
    progress: Mutex<Option<P>>,
}

/// 処理 `work` を行い、結果を `outcome` に置く（処理の中の panic は捕まえる）
fn run<T, E, P, W>(work: W, context: &TaskContext<P>, outcome: &Mutex<Option<TaskOutcome<T, E>>>)
where
    W: FnOnce(&TaskContext<P>) -> Result<T, TaskFailure<E>>,
{
    let finished = match catch_unwind(AssertUnwindSafe(|| work(context))) {
        Ok(Ok(value)) => TaskOutcome::Succeeded(value),
        Ok(Err(TaskFailure::Failed(error))) => TaskOutcome::Failed(error),
        Ok(Err(TaskFailure::Cancelled)) => TaskOutcome::Cancelled,
        Err(payload) => {
            let message = panic_message(payload.as_ref());
            // panic の値を手放すときの panic で、結果を置かずに終わらないよう、値は手放さずに残す
            std::mem::forget(payload);
            TaskOutcome::Panicked(message)
        }
    };
    *lock(outcome) = Some(finished);
}

/// panic の文（文字列でなければ空）
fn panic_message(payload: &(dyn Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        String::new()
    }
}

/// 鍵を取る（鍵を持ったまま panic した後も、値を使う。値は 1 回の代入で置き換えるだけで、途中の状態にならない）
fn lock<V>(mutex: &Mutex<V>) -> MutexGuard<'_, V> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}
