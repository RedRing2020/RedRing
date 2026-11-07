//! ジョブマネージャーの画面の文言（日本語・英語）

use i18n_foundation::{TableMessageCatalog, UiLocale, UiMessage, UiMessageArg, resolve_message};

const JOB_MANAGER_TEMPLATE_JA: &[(&str, &str)] = &[
    ("window.title", "RedRing ジョブマネージャー"),
    ("filter.header", "絞り込み"),
    ("filter.all", "すべて"),
    ("status.queued", "計算待ち"),
    ("status.running", "計算中"),
    ("status.succeeded", "計算完了"),
    ("status.failed", "失敗"),
    ("status.canceled", "取り消し"),
    ("column.id", "番号"),
    ("column.source", "投入元の工程"),
    ("column.kind", "種類"),
    ("column.status", "状態"),
    ("column.priority", "優先順位"),
    ("column.elapsed", "経過"),
    ("column.submitted", "投入した時刻"),
    ("kind.cam", "CAM の計算"),
    ("kind.sim", "切削シミュレーション"),
    ("store.size", "保存先: {size}"),
    ("store.unreadable", "保存先を読めません（{reason}）"),
    ("store.unreadable_jobs", "読めないジョブの記録: {count} 件"),
    ("details.tab", "詳細"),
    ("details.log_tab", "ログ"),
    ("details.none", "ジョブを選ぶと、詳細を出します"),
    ("details.status", "状態: {status}"),
    ("details.error", "理由: {reason}"),
    ("details.input", "入力: {path}（{size}）"),
    ("details.result", "成果物: {path}（{size}）"),
    ("details.no_result", "成果物: なし"),
    ("details.depends_on", "待つジョブ: {jobs}"),
    ("details.attempts", "計算した回数: {count}"),
    ("details.no_log", "ログはまだありません"),
    ("source.none", "（なし）"),
    ("toolbar.cancel", "取り消し"),
    ("toolbar.retry", "やり直し"),
    ("toolbar.delete", "削除"),
    ("toolbar.raise", "▲ 上げる"),
    ("toolbar.lower", "▼ 下げる"),
    ("command.failed", "操作できませんでした（{reason}）"),
    (
        "delete.confirm",
        "ジョブ {job} の成果物を、後のジョブ（{jobs}）が使います。消すと、計算待ちの後のジョブは計算できなくなり、計算の済んだ後のジョブの結果も取り込めなくなります。消しますか？",
    ),
    ("delete.confirm_yes", "消す"),
    ("delete.confirm_no", "やめる"),
    (
        "startup.no_data_folder",
        "利用者ごとのデータのフォルダが分からないため、起動できません",
    ),
    (
        "startup.already_running",
        "ジョブマネージャーはすでに動いています（今は、動いているジョブマネージャーを終了してから起動すると、ウィンドウを開けます）",
    ),
    (
        "startup.failed",
        "ジョブマネージャーを起動できません（{reason}）",
    ),
];

const JOB_MANAGER_TEMPLATE_EN: &[(&str, &str)] = &[
    ("window.title", "RedRing Job Manager"),
    ("filter.header", "Filter"),
    ("filter.all", "All"),
    ("status.queued", "Waiting"),
    ("status.running", "Calculating"),
    ("status.succeeded", "Completed"),
    ("status.failed", "Failed"),
    ("status.canceled", "Canceled"),
    ("column.id", "No."),
    ("column.source", "Source process"),
    ("column.kind", "Kind"),
    ("column.status", "Status"),
    ("column.priority", "Priority"),
    ("column.elapsed", "Elapsed"),
    ("column.submitted", "Submitted"),
    ("kind.cam", "CAM calculation"),
    ("kind.sim", "Cutting simulation"),
    ("store.size", "Job store: {size}"),
    ("store.unreadable", "Cannot read the job store ({reason})"),
    ("store.unreadable_jobs", "Unreadable job records: {count}"),
    ("details.tab", "Details"),
    ("details.log_tab", "Log"),
    ("details.none", "Select a job to see its details"),
    ("details.status", "Status: {status}"),
    ("details.error", "Reason: {reason}"),
    ("details.input", "Input: {path} ({size})"),
    ("details.result", "Result: {path} ({size})"),
    ("details.no_result", "Result: none"),
    ("details.depends_on", "Waits for jobs: {jobs}"),
    ("details.attempts", "Calculated: {count} times"),
    ("details.no_log", "No log yet"),
    ("source.none", "(none)"),
    ("toolbar.cancel", "Cancel"),
    ("toolbar.retry", "Retry"),
    ("toolbar.delete", "Delete"),
    ("toolbar.raise", "▲ Raise"),
    ("toolbar.lower", "▼ Lower"),
    ("command.failed", "The operation failed ({reason})"),
    (
        "delete.confirm",
        "Later jobs ({jobs}) use the result of job {job}. If you delete it, the waiting later jobs cannot be calculated, and the results of the completed later jobs cannot be imported. Delete it?",
    ),
    ("delete.confirm_yes", "Delete"),
    ("delete.confirm_no", "Keep"),
    (
        "startup.no_data_folder",
        "Cannot start because the per-user data folder is not found",
    ),
    (
        "startup.already_running",
        "The job manager is already running (for now, end the running job manager and start again to open the window)",
    ),
    ("startup.failed", "Cannot start the job manager ({reason})"),
];

const JOB_MANAGER_MESSAGE_CATALOG: TableMessageCatalog = TableMessageCatalog::new(
    JOB_MANAGER_TEMPLATE_JA,
    JOB_MANAGER_TEMPLATE_EN,
    "__missing__",
    "__missing__",
);

/// 文言のキー `key` を、言語 `locale` の文言にする（`args` は差し込む値）
pub(crate) fn text(locale: UiLocale, key: &str, args: &[(&str, String)]) -> String {
    let message = UiMessage {
        key: key.to_string(),
        args: args
            .iter()
            .map(|(name, value)| UiMessageArg {
                name: (*name).to_string(),
                value: value.clone(),
            })
            .collect(),
    };
    resolve_message(&JOB_MANAGER_MESSAGE_CATALOG, locale, &message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_key_has_both_languages() {
        for (key, _) in JOB_MANAGER_TEMPLATE_JA {
            assert!(
                JOB_MANAGER_TEMPLATE_EN
                    .iter()
                    .any(|(other, _)| other == key),
                "{key}"
            );
        }
        assert_eq!(JOB_MANAGER_TEMPLATE_JA.len(), JOB_MANAGER_TEMPLATE_EN.len());
        assert_eq!(
            text(UiLocale::En, "store.size", &[("size", "1 GB".to_string())]),
            "Job store: 1 GB"
        );
    }
}
