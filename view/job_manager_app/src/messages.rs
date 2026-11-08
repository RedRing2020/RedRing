//! ジョブマネージャーの画面の文言（言語ごとの文言の表）

use crate::locale::Language;

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
    ("language", "言語"),
    (
        "language.save_failed",
        "選んだ言語を保存できません（{reason}）",
    ),
    ("tray.open", "ウィンドウを開く"),
    ("notify.succeeded", "{kind}（ジョブ {job}）が完了しました"),
    ("notify.failed", "{kind}（ジョブ {job}）が失敗しました"),
    ("tray.autostart", "Windows の起動のときに起動する"),
    ("tray.quit", "終了"),
    (
        "tray.quit_confirm",
        "計算中のジョブがあります。終了すると計算を止め、そのジョブは失敗になります。終了しますか？",
    ),
    (
        "tray.autostart_failed",
        "Windows の起動のときに起動する設定を変えられませんでした（{reason}）",
    ),
    (
        "startup.no_data_folder",
        "利用者ごとのデータのフォルダが分からないため、起動できません",
    ),
    (
        "startup.already_running",
        "ジョブマネージャーはすでに動いています",
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
    ("language", "Language"),
    (
        "language.save_failed",
        "Cannot save the selected language ({reason})",
    ),
    ("tray.open", "Open window"),
    ("notify.succeeded", "{kind} (job {job}) has completed"),
    ("notify.failed", "{kind} (job {job}) has failed"),
    ("tray.autostart", "Start when Windows starts"),
    ("tray.quit", "Quit"),
    (
        "tray.quit_confirm",
        "A job is being calculated. Quitting stops the calculation, and the job fails. Quit?",
    ),
    (
        "tray.autostart_failed",
        "Cannot change the setting to start when Windows starts ({reason})",
    ),
    (
        "startup.no_data_folder",
        "Cannot start because the per-user data folder is not found",
    ),
    (
        "startup.already_running",
        "The job manager is already running",
    ),
    ("startup.failed", "Cannot start the job manager ({reason})"),
];

/// 言語 `language` の文言の表
fn table(language: Language) -> &'static [(&'static str, &'static str)] {
    match language {
        Language::Ja => JOB_MANAGER_TEMPLATE_JA,
        Language::En => JOB_MANAGER_TEMPLATE_EN,
    }
}

/// 文言のキー `key` を、言語 `language` の文言にする（`args` の名前の `{名前}` を値に替える。差し込んだ値の中の
/// `{名前}` は替えない。キーがなければ `__missing__`）
pub(crate) fn text(language: Language, key: &str, args: &[(&str, String)]) -> String {
    let template = table(language)
        .iter()
        .find(|(candidate, _)| *candidate == key)
        .map_or("__missing__", |(_, template)| *template);
    let mut text = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        text.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let value = after.find('}').and_then(|end| {
            let name = &after[..end];
            args.iter()
                .find(|(candidate, _)| *candidate == name)
                .map(|(_, value)| (value, end))
        });
        match value {
            Some((value, end)) => {
                text.push_str(value);
                rest = &after[end + 1..];
            }
            None => {
                text.push('{');
                rest = after;
            }
        }
    }
    text.push_str(rest);
    text
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
            text(Language::En, "store.size", &[("size", "1 GB".to_string())]),
            "Job store: 1 GB"
        );
        // 差し込んだ値の中の名前は、替えない
        assert!(
            text(
                Language::En,
                "notify.succeeded",
                &[("kind", "{job}".to_string()), ("job", "7".to_string())]
            )
            .contains("{job}")
        );
        assert_eq!(text(Language::Ja, "no.such.key", &[]), "__missing__");
    }
}
