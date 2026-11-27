//! コマンドが宣言すること

/// コマンドが触るものの種類
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessKind {
    /// 表示だけ
    ViewOnly,
    /// 読むだけ
    ReadOnly,
    /// 変える
    Modify,
    /// 入れ替える
    Replace,
}

/// コマンドが触るもの 1 つ（種類と対象）
///
/// `K` は対象の区分の型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Access<K> {
    /// 種類
    pub kind: AccessKind,
    /// 対象
    pub target: K,
}

/// 実行の形
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Execution {
    /// その場で終わる
    Synchronous,
    /// 始めた後、終わるまで実行中のまま残る
    Asynchronous,
}

/// 対象の集まり
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TargetSet<K> {
    /// すべての対象
    Any,
    /// 挙げた対象のほかのすべて
    AnyExcept(Vec<K>),
}

/// 非同期のコマンドが、自分の最中に実行してよいとする、触るものの集まり 1 つ（種類と、対象の集まり）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllowedInterrupt<K> {
    /// 種類
    pub kind: AccessKind,
    /// 対象の集まり
    pub targets: TargetSet<K>,
}

/// 実行中のコマンドに許されないときの扱い
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhenBlocked {
    /// 実行できるようになるまで待つ
    Wait,
    /// 受け付けない
    Reject,
    /// 許さないコマンドのうち取り消せるものを取り消し、許さないコマンドがなくなってから実行する
    CancelThenRun,
}

/// コマンドの宣言
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandDeclaration<K> {
    /// 実行の形
    pub execution: Execution,
    /// 触るもの
    pub touches: Vec<Access<K>>,
    /// 自分が実行中のまま残る間（非同期の処理の間、子のコマンドの間）に、実行してよいコマンドの触るもの
    ///
    /// その場で終わり、子を持たないコマンドは、空でよい。
    pub allowed_interrupts: Vec<AllowedInterrupt<K>>,
    /// 実行中のコマンドに許されないときの扱い
    pub when_blocked: WhenBlocked,
    /// 途中で取り消せるか
    pub cancellable: bool,
}

impl<K: PartialEq> CommandDeclaration<K> {
    /// 自分の最中に、宣言 `other` のコマンドを実行してよいか（`other` の触るものを、すべて許しているか）
    pub(super) fn allows(&self, other: &Self) -> bool {
        other.touches.iter().all(|access| {
            self.allowed_interrupts.iter().any(|allowed| {
                allowed.kind == access.kind
                    && match &allowed.targets {
                        TargetSet::Any => true,
                        TargetSet::AnyExcept(excluded) => !excluded.contains(&access.target),
                    }
            })
        })
    }
}
