//! 段の進み具合と、途中で止めること
//!
//! 段の関数は、区切りごとに、済んだ数と全体の数を閉包に渡す。閉包が止めると返したら、段は途中で止まり、
//! 止めたことを誤りとして返す。

use std::fmt;
use std::ops::ControlFlow;

/// 段の進み具合を受け取り、続けるか止めるかを返す閉包（引数は、済んだ数と全体の数）
///
/// 数え方は段ごとに決める（段の関数の説明に書く）。済んだ数は、全体の数に届かずに終わることがある。
pub type Progress<'a> = dyn FnMut(usize, usize) -> ControlFlow<()> + 'a;

/// 進み具合の閉包に止めると言われて、段を途中で止めた
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cancelled;

impl fmt::Display for Cancelled {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("the stage was cancelled")
    }
}

impl std::error::Error for Cancelled {}

/// エンティティを数える段で、進み具合を知らせる区切り（エンティティの数）
pub(crate) const ENTITY_INTERVAL: usize = 4096;

/// 済んだ数を数え、区切りごとに進み具合を知らせる
pub(crate) struct Steps<'a, 'f> {
    progress: &'a mut Progress<'f>,
    done: usize,
    total: usize,
}

impl<'a, 'f> Steps<'a, 'f> {
    /// 全体の数が `total` の数え始め
    pub(crate) fn new(progress: &'a mut Progress<'f>, total: usize) -> Self {
        Self {
            progress,
            done: 0,
            total,
        }
    }

    /// 1 つ済んだと数える。区切り（[`ENTITY_INTERVAL`] 個ごと）なら、進み具合を知らせる
    ///
    /// # Errors
    ///
    /// 閉包が止めると返したとき。
    pub(crate) fn step(&mut self) -> Result<(), Cancelled> {
        self.done += 1;
        if !self.done.is_multiple_of(ENTITY_INTERVAL) {
            return Ok(());
        }
        match (self.progress)(self.done, self.total) {
            ControlFlow::Continue(()) => Ok(()),
            ControlFlow::Break(()) => Err(Cancelled),
        }
    }
}

/// 試験用: 進み具合を見ずに、いつも続けると返す
#[cfg(test)]
pub(crate) fn keep_going(_done: usize, _total: usize) -> ControlFlow<()> {
    ControlFlow::Continue(())
}
