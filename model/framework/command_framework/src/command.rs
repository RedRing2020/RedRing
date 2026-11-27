//! コマンドの宣言と、実行中のコマンドの最中に来たコマンドの受け付け方を決める仕組み
//!
//! - コマンドは、実行の形・触るもの・自分の最中に許す割り込み・許されないときの扱い・取り消せるかを宣言する
//!   （[`CommandDeclaration`]）
//! - [`CommandScheduler`] は、実行中のコマンドをスタックで、待たせたコマンドを待ち行列で持ち、来たコマンドを
//!   すぐ実行するか・待たせるか・受け付けないかを、宣言から決める
//!
//! コマンドを実行することと、取り消しを伝えることは、この仕組みの外で行う。コマンドの値の型と、対象の区分の型は、
//! 使う側が決める。

use std::collections::VecDeque;

mod declaration;
#[cfg(test)]
mod tests;

pub use declaration::{
    Access, AccessKind, AllowedInterrupt, CommandDeclaration, Execution, TargetSet, WhenBlocked,
};

/// 実行中のコマンドの識別子
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandId(u64);

/// ある宣言のコマンドを、今出したときの扱い
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Admission {
    /// すぐ実行する
    Run,
    /// 待たせる
    Wait,
    /// 許さないコマンドのうち取り消せるものを取り消し、許さないコマンドがなくなるまで待たせる
    CancelThenRun,
    /// 受け付けない
    Reject,
}

/// 実行を始めたコマンド
#[derive(Debug, PartialEq, Eq)]
pub struct Started<C> {
    /// 実行中のコマンドの識別子（終わったら [`CommandScheduler::finish`] に渡す）
    pub id: CommandId,
    /// コマンド
    pub command: C,
}

/// コマンドを出した結果
#[derive(Debug, PartialEq, Eq)]
pub enum Submission<C> {
    /// 実行を始めた（スタックに積んだ）
    Started(Started<C>),
    /// 待ち行列に入れた
    Queued,
    /// 受け付けなかった（コマンドを返す）
    Rejected(C),
}

/// 実行中のコマンドのスタックと、待たせたコマンドの待ち行列
///
/// `K` は対象の区分の型、`C` はコマンドの値の型。
///
/// - コマンドを出す（[`Self::submit`]）と、実行中のすべてのコマンドと互いに許すときだけ、実行を始める。
///   始めたコマンドは、使う側が実行し、終わったら [`Self::finish`] で下ろす（同期のコマンドは、実行してすぐ下ろす）
/// - 実行中のコマンドが変わったら、[`Self::take_ready`] で、待たせたコマンドのうち実行できるものを 1 つずつ受け取り、
///   [`Self::take_cancel_requests`] で、取り消すコマンドを受け取る
///
/// 実行中のコマンド `running` と、来たコマンド `incoming` が互いに許すとは、`running` が `incoming` の触るものを
/// 許し、`incoming` が非同期なら、`incoming` も `running` の触るものを許すことをいう。
#[derive(Debug)]
pub struct CommandScheduler<K, C> {
    /// 実行中のコマンド（先頭が、先に始めたもの）
    running: Vec<Running<K>>,
    /// 待たせたコマンド（先頭が、先に出したもの）
    waiting: VecDeque<Waiting<K, C>>,
    next_id: u64,
}

#[derive(Debug)]
struct Running<K> {
    id: CommandId,
    declaration: CommandDeclaration<K>,
    /// 取り消すコマンドとして、すでに渡したか
    cancel_requested: bool,
}

#[derive(Debug)]
struct Waiting<K, C> {
    declaration: CommandDeclaration<K>,
    command: C,
}

impl<K, C> Default for CommandScheduler<K, C> {
    fn default() -> Self {
        Self {
            running: Vec::new(),
            waiting: VecDeque::new(),
            next_id: 0,
        }
    }
}

impl<K: PartialEq, C> CommandScheduler<K, C> {
    /// 宣言 `declaration` のコマンドを、今出したときの扱い（何も変えない）
    pub fn admission(&self, declaration: &CommandDeclaration<K>) -> Admission {
        if !is_blocked(&self.running, declaration) {
            return Admission::Run;
        }
        match declaration.when_blocked {
            WhenBlocked::Wait => Admission::Wait,
            WhenBlocked::Reject => Admission::Reject,
            WhenBlocked::CancelThenRun => Admission::CancelThenRun,
        }
    }

    /// 宣言 `declaration` のコマンド `command` を出す
    ///
    /// 扱いは [`Self::admission`] と同じ。待たせるときは、前から待っているコマンドの後ろに入れる。
    /// すぐ実行できるコマンドは、前から待っているコマンドがあっても、実行を始める。
    pub fn submit(&mut self, declaration: CommandDeclaration<K>, command: C) -> Submission<C> {
        match self.admission(&declaration) {
            Admission::Run => Submission::Started(self.start(declaration, command)),
            Admission::Wait | Admission::CancelThenRun => {
                self.waiting.push_back(Waiting {
                    declaration,
                    command,
                });
                Submission::Queued
            }
            Admission::Reject => Submission::Rejected(command),
        }
    }

    /// 実行中のコマンド `parent` の中から、宣言 `declaration` のコマンド `command` を、子として出す
    ///
    /// 子は、`parent` とそれより先に始めたコマンドとは照らさず、`parent` より後に始めたコマンドとだけ照らす。
    /// 互いに許せば実行を始め、許さなければ受け付けない（子は待たせない。許されないときの扱いの宣言は使わない）。
    /// `parent` が実行中でないときも、受け付けない。
    pub fn submit_child(
        &mut self,
        parent: CommandId,
        declaration: CommandDeclaration<K>,
        command: C,
    ) -> Submission<C> {
        let Some(parent_index) = self.running.iter().position(|entry| entry.id == parent) else {
            return Submission::Rejected(command);
        };
        if is_blocked(&self.running[parent_index + 1..], &declaration) {
            return Submission::Rejected(command);
        }
        Submission::Started(self.start(declaration, command))
    }

    /// 待たせたコマンドのうち、今実行できるものを、先に出したものから 1 つ選び、実行を始める
    ///
    /// 実行できるものがなければ `None`。実行できないコマンドは、待たせたまま順を保つ。
    /// 始めたコマンドが実行中に加わると、残りのコマンドの扱いが変わるため、1 つずつ受け取る。
    pub fn take_ready(&mut self) -> Option<Started<C>> {
        let index = self
            .waiting
            .iter()
            .position(|entry| !is_blocked(&self.running, &entry.declaration))?;
        let entry = self.waiting.remove(index)?;
        Some(self.start(entry.declaration, entry.command))
    }

    /// 取り消すコマンドを受け取る
    ///
    /// 待たせたコマンドのうち、扱いが [`WhenBlocked::CancelThenRun`] のものと互いに許さない、実行中の取り消せる
    /// コマンドを返す。同じコマンドは 1 回だけ返す。
    pub fn take_cancel_requests(&mut self) -> Vec<CommandId> {
        let mut requests = Vec::new();
        for entry in &mut self.running {
            if entry.cancel_requested || !entry.declaration.cancellable {
                continue;
            }
            let must_cancel = self.waiting.iter().any(|waiting| {
                waiting.declaration.when_blocked == WhenBlocked::CancelThenRun
                    && blocks(&entry.declaration, &waiting.declaration)
            });
            if must_cancel {
                entry.cancel_requested = true;
                requests.push(entry.id);
            }
        }
        requests
    }

    /// 終わったコマンド `id` を、スタックから除く（始めた順と違う順に終わってよい）
    ///
    /// `id` が実行中でなければ、何もしない。
    pub fn finish(&mut self, id: CommandId) {
        self.running.retain(|entry| entry.id != id);
    }

    fn start(&mut self, declaration: CommandDeclaration<K>, command: C) -> Started<C> {
        let id = CommandId(self.next_id);
        self.next_id += 1;
        self.running.push(Running {
            id,
            declaration,
            cancel_requested: false,
        });
        Started { id, command }
    }
}

/// 実行中のコマンド `running` のどれかが、宣言 `incoming` のコマンドと互いに許さないか
fn is_blocked<K: PartialEq>(running: &[Running<K>], incoming: &CommandDeclaration<K>) -> bool {
    running
        .iter()
        .any(|entry| blocks(&entry.declaration, incoming))
}

/// 実行中のコマンドの宣言 `running` と、来たコマンドの宣言 `incoming` が、互いに許さないか
fn blocks<K: PartialEq>(running: &CommandDeclaration<K>, incoming: &CommandDeclaration<K>) -> bool {
    !running.allows(incoming)
        || (incoming.execution == Execution::Asynchronous && !incoming.allows(running))
}
