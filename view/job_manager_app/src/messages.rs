//! ジョブマネージャーの画面の文言（言語ごとの文言の表）

use job_runtime::store::StoredJobError;

use crate::locale::Language;
use crate::service::StoreFailure;
use crate::store_change::ChangeFailure;

const JOB_MANAGER_TEMPLATE_JA: &[(&str, &str)] = &[
    ("window.title", "RedRing ジョブマネージャー"),
    ("filter.header", "絞り込み"),
    ("filter.all", "すべて"),
    ("status.queued", "計算待ち"),
    ("status.running", "計算中"),
    ("status.succeeded", "計算完了"),
    ("status.failed", "計算失敗"),
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
    ("details.imported", "CL を空間モデルに取り込みました"),
    ("details.depends_on", "待つジョブ: {jobs}"),
    ("details.attempts", "計算した回数: {count}"),
    ("details.no_log", "ログはまだありません"),
    ("source.none", "（なし）"),
    ("toolbar.cancel", "取り消し"),
    ("toolbar.retry", "再計算"),
    ("toolbar.delete", "削除"),
    ("toolbar.raise", "▲ 上げる"),
    ("toolbar.lower", "▼ 下げる"),
    ("toolbar.bulk_delete", "まとめて削除…"),
    ("bulk.title", "まとめて削除"),
    (
        "bulk.superseded",
        "前の改訂の結果（同じ加工工程に、別の条件で後から投入して正常に終わったジョブがあるもの）: {count} 件",
    ),
    (
        "bulk.imported",
        "取り込み済みの CL の元（CL を空間モデルに取り込んだツールパス生成）: {count} 件",
    ),
    ("bulk.old", "古いジョブ:"),
    ("bulk.days_suffix", " 日"),
    ("bulk.old_count", "より前に投入したもの（{count} 件）"),
    ("bulk.rejected", "取り込めなかった投入: {count} 件"),
    (
        "bulk.summary",
        "削除するジョブ: {count} 件（その結果を使う後のジョブを含む。終わったジョブだけ）",
    ),
    (
        "bulk.kept",
        "後のジョブが計算待ち・計算中のため、残すジョブ: {jobs}",
    ),
    (
        "bulk.note",
        "削除したジョブの結果は、取り込めなくなります。",
    ),
    ("bulk.yes", "削除"),
    ("bulk.no", "やめる"),
    ("command.failed", "操作できませんでした（{reason}）"),
    ("delete.confirm", "選んだジョブ（{jobs}）を削除します。"),
    (
        "delete.confirm_later",
        "その結果を使う後のジョブ（{jobs}）も、一緒に消えます。",
    ),
    (
        "delete.confirm_kept",
        "ジョブ {jobs} は、それか後のジョブが計算待ち・計算中のため、削除しません。",
    ),
    ("delete.confirm_question", "削除しますか？"),
    (
        "delete.kept",
        "計算待ち・計算中のジョブがあるため、消せません（ジョブ {jobs}）",
    ),
    ("delete.confirm_yes", "削除"),
    ("delete.confirm_no", "やめる"),
    ("store.change", "保存先を変更…"),
    (
        "store.change_busy",
        "計算待ち・計算中のジョブがある間は、保存先を変更できません",
    ),
    ("store.choose", "ジョブの保存先のフォルダを選ぶ"),
    (
        "store.change_confirm",
        "保存先を替えて、ジョブマネージャーを起動し直します。今の保存先のジョブは一覧に出なくなります（ファイルは消しません）。\n\n今の保存先: {current}\n新しい保存先: {chosen}",
    ),
    (
        "store.change_existing",
        "新しい保存先にあるジョブを、一覧に出します。",
    ),
    (
        "store.change_other_files",
        "新しい保存先のフォルダには、ジョブの保存先でないファイルがあります。",
    ),
    ("store.change_question", "保存先を替えますか？"),
    ("store.change_yes", "替える"),
    ("store.change_no", "やめる"),
    ("store.change_failed", "保存先を替えられません（{reason}）"),
    (
        "store.change_restart_failed",
        "保存先は替えました。ジョブマネージャーを起動し直してください",
    ),
    ("change_failure.same", "今の保存先と同じフォルダです"),
    (
        "change_failure.nested",
        "今の保存先と入れ子になるフォルダです",
    ),
    (
        "change_failure.not_local",
        "この PC のフォルダではありません",
    ),
    (
        "change_failure.inaccessible",
        "フォルダを読み書きできません",
    ),
    (
        "change_failure.name_conflict",
        "中のフォルダの名前が保存先と重なるため、使えません",
    ),
    (
        "change_failure.busy",
        "計算待ち・計算中のジョブ、または取り込みを待つ投入があります",
    ),
    ("change_failure.setting_not_saved", "設定を保存できません"),
    ("language", "言語"),
    (
        "error.dependency_not_earlier",
        "依存するジョブ {job} が、前のジョブではありません",
    ),
    (
        "error.dependency_cannot_run",
        "依存するジョブ {job} を計算できません",
    ),
    (
        "error.dependency_not_found",
        "依存するジョブ {job} がありません",
    ),
    (
        "error.dependency_failed",
        "依存するジョブ {job} が失敗しました",
    ),
    (
        "error.dependency_canceled",
        "依存するジョブ {job} が取り消されました",
    ),
    ("error.runner_not_started", "ランナーを起動できません"),
    (
        "error.runner_wrote_no_result",
        "ランナーが結果を書かずに終わりました",
    ),
    (
        "error.runner_invalid_arguments",
        "ランナーの引数が正しくありません",
    ),
    (
        "error.runner_cannot_read_input",
        "ランナーが入力を読めません",
    ),
    ("error.invalid_input", "入力が正しくありません"),
    ("error.calculation_failed", "計算に失敗しました"),
    (
        "error.runner_cannot_write_result",
        "ランナーが結果を書けません",
    ),
    (
        "error.toolpath_too_large",
        "ツールパスの点が多すぎるため、計算を止めました（メモリの上限を超えます）",
    ),
    (
        "error.runner_exit_code",
        "ランナーが終了の番号 {code} で終わりました",
    ),
    ("error.runner_stopped", "ランナーが止められました"),
    (
        "error.manager_stopped",
        "計算中にジョブマネージャーが止まりました",
    ),
    ("error.manager_quit", "ジョブマネージャーを終了しました"),
    (
        "store_failure.already_locked",
        "ほかのジョブマネージャーが保存先を使っています",
    ),
    (
        "store_failure.not_found",
        "保存先のファイルが見つかりません",
    ),
    (
        "store_failure.permission_denied",
        "保存先のファイルを読み書きする権限がありません",
    ),
    ("store_failure.io", "保存先のファイルを読み書きできません"),
    (
        "store_failure.invalid_ref",
        "ジョブの記録の参照が正しくありません",
    ),
    (
        "store_failure.service_stopped",
        "ジョブの管理が止まっています",
    ),
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
        "計算中のジョブがあります。終了すると計算を止め、そのジョブは計算失敗になります。終了しますか？",
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
        "startup.invalid_store_setting",
        "設定のファイルの保存先（store）が、絶対のパスではありません",
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
    ("status.failed", "Calculation failed"),
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
    ("details.imported", "The CL was imported into a space model"),
    ("details.depends_on", "Waits for jobs: {jobs}"),
    ("details.attempts", "Calculated: {count} times"),
    ("details.no_log", "No log yet"),
    ("source.none", "(none)"),
    ("toolbar.cancel", "Cancel"),
    ("toolbar.retry", "Recalculate"),
    ("toolbar.delete", "Delete"),
    ("toolbar.raise", "▲ Raise"),
    ("toolbar.lower", "▼ Lower"),
    ("toolbar.bulk_delete", "Bulk delete…"),
    ("bulk.title", "Bulk delete"),
    (
        "bulk.superseded",
        "Results of previous revisions (a later job with other conditions completed for the same machining operation): {count}",
    ),
    (
        "bulk.imported",
        "Sources of imported CLs (toolpath generations whose CL was imported into a space model): {count}",
    ),
    ("bulk.old", "Old jobs:"),
    ("bulk.days_suffix", " days"),
    ("bulk.old_count", "or older ({count})"),
    (
        "bulk.rejected",
        "Submissions that could not be taken in: {count}",
    ),
    (
        "bulk.summary",
        "Jobs to delete: {count} (including later jobs that use their results; finished jobs only)",
    ),
    (
        "bulk.kept",
        "Jobs kept because their later jobs are waiting or calculating: {jobs}",
    ),
    (
        "bulk.note",
        "The results of deleted jobs can no longer be imported.",
    ),
    ("bulk.yes", "Delete"),
    ("bulk.no", "Cancel"),
    ("command.failed", "The operation failed ({reason})"),
    (
        "delete.confirm",
        "The selected jobs ({jobs}) will be deleted.",
    ),
    (
        "delete.confirm_later",
        "The later jobs that use their results ({jobs}) will also be deleted.",
    ),
    (
        "delete.confirm_kept",
        "Jobs {jobs} will not be deleted because they or their later jobs are waiting or calculating.",
    ),
    ("delete.confirm_question", "Delete them?"),
    (
        "delete.kept",
        "Cannot delete because jobs are waiting or calculating (jobs {jobs})",
    ),
    ("delete.confirm_yes", "Delete"),
    ("delete.confirm_no", "Keep"),
    ("store.change", "Change job store…"),
    (
        "store.change_busy",
        "The job store cannot be changed while jobs are waiting or calculating",
    ),
    ("store.choose", "Choose a folder for the job store"),
    (
        "store.change_confirm",
        "The job manager will restart with the new job store. Jobs in the current job store will no longer be listed (their files are kept).\n\nCurrent job store: {current}\nNew job store: {chosen}",
    ),
    (
        "store.change_existing",
        "The jobs in the new job store will be listed.",
    ),
    (
        "store.change_other_files",
        "The new folder contains files that are not part of a job store.",
    ),
    ("store.change_question", "Change the job store?"),
    ("store.change_yes", "Change"),
    ("store.change_no", "Cancel"),
    (
        "store.change_failed",
        "Cannot change the job store ({reason})",
    ),
    (
        "store.change_restart_failed",
        "The job store was changed. Restart the job manager.",
    ),
    ("change_failure.same", "It is the current job store folder"),
    (
        "change_failure.nested",
        "The folder is inside the current job store or contains it",
    ),
    ("change_failure.not_local", "The folder is not on this PC"),
    (
        "change_failure.inaccessible",
        "Cannot read or write the folder",
    ),
    (
        "change_failure.name_conflict",
        "The folder cannot be used because folders in it have the same names as job store folders",
    ),
    (
        "change_failure.busy",
        "Jobs are waiting or calculating, or submissions are waiting to be taken in",
    ),
    (
        "change_failure.setting_not_saved",
        "Cannot save the setting",
    ),
    ("language", "Language"),
    (
        "error.dependency_not_earlier",
        "Dependency job {job} is not an earlier job",
    ),
    (
        "error.dependency_cannot_run",
        "Dependency job {job} cannot be calculated",
    ),
    (
        "error.dependency_not_found",
        "Dependency job {job} does not exist",
    ),
    ("error.dependency_failed", "Dependency job {job} failed"),
    (
        "error.dependency_canceled",
        "Dependency job {job} was canceled",
    ),
    ("error.runner_not_started", "Cannot start the runner"),
    (
        "error.runner_wrote_no_result",
        "The runner ended without writing a result",
    ),
    (
        "error.runner_invalid_arguments",
        "The runner arguments are invalid",
    ),
    (
        "error.runner_cannot_read_input",
        "The runner cannot read the input",
    ),
    ("error.invalid_input", "The input is invalid"),
    ("error.calculation_failed", "The calculation failed"),
    (
        "error.runner_cannot_write_result",
        "The runner cannot write the result",
    ),
    (
        "error.toolpath_too_large",
        "The calculation was stopped because the toolpath has too many points (it exceeds the memory limit)",
    ),
    (
        "error.runner_exit_code",
        "The runner ended with exit code {code}",
    ),
    ("error.runner_stopped", "The runner was stopped"),
    (
        "error.manager_stopped",
        "The job manager stopped during the calculation",
    ),
    ("error.manager_quit", "The job manager was quit"),
    (
        "store_failure.already_locked",
        "Another job manager is using the job store",
    ),
    ("store_failure.not_found", "A job store file is not found"),
    (
        "store_failure.permission_denied",
        "No permission to read or write the job store files",
    ),
    (
        "store_failure.io",
        "Cannot read or write the job store files",
    ),
    (
        "store_failure.invalid_ref",
        "A reference in a job record is invalid",
    ),
    (
        "store_failure.service_stopped",
        "Job management has stopped",
    ),
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
        "startup.invalid_store_setting",
        "The job store (store) in the settings file is not an absolute path",
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

/// ジョブの失敗の理由の種類 `error` の、言語 `language` の文言（知らない種類は `None`）
pub(crate) fn job_error_text(language: Language, error: &StoredJobError) -> Option<String> {
    use StoredJobError as E;
    let (key, args): (&str, Vec<(&str, String)>) = match error {
        E::DependencyNotEarlier { job } => (
            "error.dependency_not_earlier",
            vec![("job", job.to_string())],
        ),
        E::DependencyCannotRun { job } => (
            "error.dependency_cannot_run",
            vec![("job", job.to_string())],
        ),
        E::DependencyNotFound { job } => {
            ("error.dependency_not_found", vec![("job", job.to_string())])
        }
        E::DependencyFailed { job } => ("error.dependency_failed", vec![("job", job.to_string())]),
        E::DependencyCanceled { job } => {
            ("error.dependency_canceled", vec![("job", job.to_string())])
        }
        E::RunnerNotStarted => ("error.runner_not_started", Vec::new()),
        E::RunnerWroteNoResult => ("error.runner_wrote_no_result", Vec::new()),
        E::RunnerInvalidArguments => ("error.runner_invalid_arguments", Vec::new()),
        E::RunnerCannotReadInput => ("error.runner_cannot_read_input", Vec::new()),
        E::InvalidInput => ("error.invalid_input", Vec::new()),
        E::CalculationFailed => ("error.calculation_failed", Vec::new()),
        E::RunnerCannotWriteResult => ("error.runner_cannot_write_result", Vec::new()),
        E::ToolpathTooLarge => ("error.toolpath_too_large", Vec::new()),
        E::RunnerExitCode { code } => ("error.runner_exit_code", vec![("code", code.to_string())]),
        E::RunnerStopped => ("error.runner_stopped", Vec::new()),
        E::ManagerStopped => ("error.manager_stopped", Vec::new()),
        E::ManagerQuit => ("error.manager_quit", Vec::new()),
        E::Unknown => return None,
    };
    Some(text(language, key, &args))
}

/// 保存先を替えられない理由 `failure` の、言語 `language` の文言
pub(crate) fn change_failure_text(language: Language, failure: ChangeFailure) -> String {
    let reason = match failure {
        ChangeFailure::Same => "change_failure.same",
        ChangeFailure::Nested => "change_failure.nested",
        ChangeFailure::NotLocal => "change_failure.not_local",
        ChangeFailure::Inaccessible => "change_failure.inaccessible",
        ChangeFailure::NameConflict => "change_failure.name_conflict",
        ChangeFailure::Busy => "change_failure.busy",
        ChangeFailure::SettingNotSaved => "change_failure.setting_not_saved",
        // 設定は保存したため、替えられなかったとは書かない
        ChangeFailure::RestartFailed => return text(language, "store.change_restart_failed", &[]),
    };
    text(
        language,
        "store.change_failed",
        &[("reason", text(language, reason, &[]))],
    )
}

/// 保存先の誤りの種類 `failure` の文言のキー
pub(crate) fn store_failure_key(failure: StoreFailure) -> &'static str {
    match failure {
        StoreFailure::AlreadyLocked => "store_failure.already_locked",
        StoreFailure::NotFound => "store_failure.not_found",
        StoreFailure::PermissionDenied => "store_failure.permission_denied",
        StoreFailure::Io => "store_failure.io",
        StoreFailure::InvalidRef => "store_failure.invalid_ref",
        StoreFailure::ServiceStopped => "store_failure.service_stopped",
    }
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

    #[test]
    fn every_error_kind_and_store_failure_has_a_text_in_both_languages() {
        use StoredJobError as E;
        let errors = [
            E::DependencyNotEarlier { job: 1 },
            E::DependencyCannotRun { job: 1 },
            E::DependencyNotFound { job: 1 },
            E::DependencyFailed { job: 1 },
            E::DependencyCanceled { job: 1 },
            E::RunnerNotStarted,
            E::RunnerWroteNoResult,
            E::RunnerInvalidArguments,
            E::RunnerCannotReadInput,
            E::InvalidInput,
            E::CalculationFailed,
            E::RunnerCannotWriteResult,
            E::ToolpathTooLarge,
            E::RunnerExitCode { code: 9 },
            E::RunnerStopped,
            E::ManagerStopped,
            E::ManagerQuit,
        ];
        for language in Language::ALL {
            for error in &errors {
                let text = job_error_text(language, error).unwrap();
                assert!(
                    !text.contains("__missing__") && !text.contains('{'),
                    "{error:?}: {text}"
                );
            }
            for failure in [
                StoreFailure::AlreadyLocked,
                StoreFailure::NotFound,
                StoreFailure::PermissionDenied,
                StoreFailure::Io,
                StoreFailure::InvalidRef,
                StoreFailure::ServiceStopped,
            ] {
                assert!(!text(language, store_failure_key(failure), &[]).contains("__missing__"));
            }
        }
        assert!(
            job_error_text(Language::Ja, &E::DependencyFailed { job: 3 })
                .unwrap()
                .contains('3')
        );
        assert_eq!(job_error_text(Language::Ja, &E::Unknown), None);
    }

    #[test]
    fn every_change_failure_has_a_text_in_both_languages() {
        for language in Language::ALL {
            for failure in [
                ChangeFailure::Same,
                ChangeFailure::Nested,
                ChangeFailure::NotLocal,
                ChangeFailure::Inaccessible,
                ChangeFailure::NameConflict,
                ChangeFailure::Busy,
                ChangeFailure::SettingNotSaved,
                ChangeFailure::RestartFailed,
            ] {
                let text = change_failure_text(language, failure);
                assert!(
                    !text.contains("__missing__") && !text.contains('{'),
                    "{failure:?}: {text}"
                );
            }
        }
    }
}
