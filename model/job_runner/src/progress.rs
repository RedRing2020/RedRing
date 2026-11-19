//! 進み具合のファイル（`--progress`）の中身と、計算の途中の進み具合を一定の間隔で書くこと

use std::path::Path;
use std::time::{Duration, Instant};

/// 計算の途中で、進み具合のファイルを書き直す間隔
const WRITE_INTERVAL: Duration = Duration::from_secs(2);
/// 残りの目安を出すのに要る、この実行で進んだ割合（進み始めは速さが定まらないため）
const MIN_ESTIMATE_ADVANCE: f64 = 0.01;
/// 残りの目安を出すのに要る、この実行で進み始めてからの時間
const MIN_ESTIMATE_TIME: Duration = Duration::from_secs(4);

/// 進み具合のファイルの中身（状態 `state`・割合 `fraction`・残りの目安の秒数 `remaining_seconds`（分からなければ
/// 書かない））
///
/// 割合は千分の一で切り捨てる（終わる前に 1 と書かないように）。
pub(crate) fn progress_json(state: &str, fraction: f64, remaining_seconds: Option<u64>) -> String {
    let fraction = (fraction.clamp(0.0, 1.0) * 1000.0).floor() / 1000.0;
    match remaining_seconds {
        Some(seconds) => format!(
            "{{\"state\":\"{state}\",\"fraction\":{fraction:.3},\"remaining_seconds\":{seconds}}}\n"
        ),
        None => format!("{{\"state\":\"{state}\",\"fraction\":{fraction:.3}}}\n"),
    }
}

/// 計算の途中の進み具合を受け取り、間隔 [`WRITE_INTERVAL`] ごとに、割合と残りの目安を書く
pub(crate) struct ProgressReporter<'a> {
    /// 書き先（`None` なら書かない）
    path: Option<&'a Path>,
    /// この実行で最初に知らされた時刻と割合（続きから計算するときは、読み戻した分を速さに含めないため）
    start: Option<(Instant, f64)>,
    /// 前に書いた時刻
    written_at: Instant,
    /// 書く処理
    write: fn(&Path, &str) -> std::io::Result<()>,
    /// 書けなかった最初の理由（英語）
    error: Option<String>,
}

impl<'a> ProgressReporter<'a> {
    /// 時刻 `now` に始まりの進み具合を書いた後の、`path` へ `write` で書く進み具合
    pub(crate) fn new(
        path: Option<&'a Path>,
        now: Instant,
        write: fn(&Path, &str) -> std::io::Result<()>,
    ) -> Self {
        Self {
            path,
            start: None,
            written_at: now,
            write,
            error: None,
        }
    }

    /// 時刻 `now` に、全体 `total` のうち `done` を終えたことを受け取る（前に書いてから間隔が過ぎていれば書く。全体が
    /// 0 なら何もしない）
    pub(crate) fn report(&mut self, done: usize, total: usize, now: Instant) {
        let Some(path) = self.path else {
            return;
        };
        if total == 0 {
            return;
        }
        let fraction = (done as f64 / total as f64).min(1.0);
        let (started_at, start_fraction) = *self.start.get_or_insert((now, fraction));
        if now.saturating_duration_since(self.written_at) < WRITE_INTERVAL {
            return;
        }
        self.written_at = now;
        let remaining = remaining_seconds(
            now.saturating_duration_since(started_at),
            fraction - start_fraction,
            1.0 - fraction,
        );
        if let Err(error) = (self.write)(path, &progress_json("running", fraction, remaining)) {
            self.error.get_or_insert_with(|| {
                format!("cannot write the progress {}: {error}", path.display())
            });
        }
    }

    /// 書けなかった最初の理由（英語。書けたなら `None`）
    pub(crate) fn take_error(&mut self) -> Option<String> {
        self.error.take()
    }
}

/// 時間 `elapsed` で割合 `advanced` だけ進んだときの、残りの割合 `rest` の秒数の目安（進みが少ない・時間が短いと
/// `None`）
fn remaining_seconds(elapsed: Duration, advanced: f64, rest: f64) -> Option<u64> {
    if advanced < MIN_ESTIMATE_ADVANCE || elapsed < MIN_ESTIMATE_TIME {
        return None;
    }
    Some((elapsed.as_secs_f64() * rest.max(0.0) / advanced).round() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::path::PathBuf;

    thread_local! {
        static WRITTEN: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
    }

    fn remember(_: &Path, json: &str) -> std::io::Result<()> {
        WRITTEN.with(|written| written.borrow_mut().push(json.to_string()));
        Ok(())
    }

    fn fail(_: &Path, _: &str) -> std::io::Result<()> {
        Err(std::io::Error::other("disk full"))
    }

    fn written() -> Vec<String> {
        WRITTEN.with(|written| written.take())
    }

    #[test]
    fn json_has_the_remaining_seconds_only_when_known() {
        assert_eq!(
            progress_json("running", 0.4219, Some(95)),
            "{\"state\":\"running\",\"fraction\":0.421,\"remaining_seconds\":95}\n"
        );
        // 終わる前に 1 と書かない
        assert_eq!(
            progress_json("running", 0.9999, None),
            "{\"state\":\"running\",\"fraction\":0.999}\n"
        );
        assert_eq!(
            progress_json("succeeded", 1.0, None),
            "{\"state\":\"succeeded\",\"fraction\":1.000}\n"
        );
    }

    #[test]
    fn progress_is_written_at_intervals_with_the_remaining_time() {
        let path = PathBuf::from("progress.json");
        let start = Instant::now();
        let at = |seconds: u64| start + Duration::from_secs(seconds);
        let mut reporter = ProgressReporter::new(Some(&path), start, remember);
        // 始まりの進み具合を書いてから間隔が過ぎるまでは書かない
        reporter.report(0, 100, at(1));
        assert!(written().is_empty());
        // 進み始めて間もないうちは、残りの目安を書かない
        reporter.report(1, 100, at(3));
        assert_eq!(written(), ["{\"state\":\"running\",\"fraction\":0.010}\n"]);
        reporter.report(2, 100, at(4));
        assert!(written().is_empty());
        // 1 から 11 秒で 25% 進んだ。残りの 75% は 30 秒
        reporter.report(25, 100, at(11));
        assert_eq!(
            written(),
            ["{\"state\":\"running\",\"fraction\":0.250,\"remaining_seconds\":30}\n"]
        );
        assert_eq!(reporter.take_error(), None);
    }

    #[test]
    fn resumed_progress_does_not_count_the_restored_part_in_the_speed() {
        let path = PathBuf::from("progress.json");
        let start = Instant::now();
        let at = |seconds: u64| start + Duration::from_secs(seconds);
        let mut reporter = ProgressReporter::new(Some(&path), start, remember);
        // 半分を読み戻して、10 秒後から計算を続けた
        reporter.report(50, 100, at(10));
        assert_eq!(written(), ["{\"state\":\"running\",\"fraction\":0.500}\n"]);
        // 10 秒で 10% 進んだ。残りの 40% は 40 秒
        reporter.report(60, 100, at(20));
        assert_eq!(
            written(),
            ["{\"state\":\"running\",\"fraction\":0.600,\"remaining_seconds\":40}\n"]
        );
    }

    #[test]
    fn nothing_is_written_without_a_file_or_a_total() {
        let start = Instant::now();
        let mut reporter = ProgressReporter::new(None, start, remember);
        reporter.report(1, 2, start + Duration::from_secs(10));
        let path = PathBuf::from("progress.json");
        let mut reporter = ProgressReporter::new(Some(&path), start, remember);
        reporter.report(0, 0, start + Duration::from_secs(10));
        assert!(written().is_empty());
    }

    #[test]
    fn the_first_write_error_is_kept_and_the_calculation_goes_on() {
        let path = PathBuf::from("progress.json");
        let start = Instant::now();
        let mut reporter = ProgressReporter::new(Some(&path), start, fail);
        reporter.report(1, 10, start + Duration::from_secs(3));
        reporter.report(2, 10, start + Duration::from_secs(6));
        assert_eq!(
            reporter.take_error().as_deref(),
            Some("cannot write the progress progress.json: disk full")
        );
        assert_eq!(reporter.take_error(), None);
    }
}
