//! 失敗の理由の文言（日本語・英語）
//!
//! キーはエラーの種類ごとに決める。表にないキーは、思わぬ失敗として「詳しくはログを確かめてください」の文言にする。
//! 文言は、ほかの文言の括弧の中に入れて出すため、括弧を使わない。

use i18n_foundation::{TableMessageCatalog, UiLocale, UiMessage, resolve_message};

const FAILURE_TEMPLATE_JA: &[(&str, &str)] = &[
    ("failure.input_registration", "計算の入力を登録できません"),
    (
        "failure.not_succeeded_simulation",
        "ジョブ {job} は、正常に終わった切削シミュレーションではありません",
    ),
    (
        "failure.not_succeeded_toolpath",
        "ジョブ {job} は、正常に終わったツールパス生成ではありません",
    ),
    (
        "failure.missing_artifact",
        "ジョブ {job} の計算結果のファイルがありません",
    ),
    (
        "failure.missing_input",
        "ジョブ {job} の計算の入力がありません",
    ),
    ("failure.sequence", "加工工程が見つかりません"),
    (
        "failure.empty_product_shape",
        "加工の対象の形状がありません",
    ),
    (
        "failure.conditions",
        "計算条件に誤りがあります。各欄のメッセージを確かめてください",
    ),
    ("failure.orchestration", "計算のジョブを扱えません"),
    (
        "failure.job_manager_not_found",
        "ジョブマネージャーがインストールされていないため、計算できません",
    ),
    (
        "failure.job_manager_outdated",
        "動いているジョブマネージャーの版が、このアプリと違います。ジョブマネージャーを起動し直してください",
    ),
    (
        "failure.job_manager_not_running",
        "ジョブマネージャーが動いていません",
    ),
    (
        "failure.job_store",
        "ジョブマネージャーにつなげないか、起動できません",
    ),
    (
        "failure.no_current_imported_toolpath",
        "空間モデルに今の条件の CL がありません",
    ),
    ("failure.conditions_not_set", "工程の条件を設定していません"),
    (
        "failure.toolpath_too_large",
        "ツールパスが約 {gb} GB になり、計算に使えるメモリの上限を超えます",
    ),
    (
        "failure.inverse_offset_shape",
        "逆オフセット形状を作れません",
    ),
    (
        "failure.no_succeeded_simulation",
        "今の条件で正常に終わった切削シミュレーションがありません",
    ),
    (
        "failure.no_succeeded_toolpath",
        "今の条件で正常に終わったツールパス生成がありません",
    ),
    (
        "failure.animation.artifact",
        "計算結果のファイルを読めません",
    ),
    (
        "failure.animation.no_snapshot",
        "計算結果にスナップショットがありません",
    ),
    ("failure.animation.replay", "計算結果を再生できません"),
    (
        "failure.inverse_offset_display",
        "逆オフセット形状を表示できません",
    ),
    ("failure.shape.stl", "壊れているか、形式の違うファイルです"),
    ("failure.shape.unknown", "形状が見つかりません"),
    (
        "failure.shape.mixed_faces",
        "NURBS の面と三角形の面が混ざっています",
    ),
    ("failure.shape.tessellation", "面を三角形に分けられません"),
    ("failure.shape.no_faces", "面がありません"),
    (
        "failure.shape.step_syntax",
        "壊れているか、形式の違うファイルです",
    ),
    (
        "failure.shape.step_limit",
        "ファイルの形状が、読み込める大きさを超えています",
    ),
    ("failure.shape.step_no_shapes", "読める形状がありません"),
    (
        "failure.shape.source_faces",
        "元のファイルの面の数が形状の面の数と合いません",
    ),
    ("failure.file.not_found", "ファイルが見つかりません"),
    (
        "failure.file.permission_denied",
        "ファイルを読み書きする許可がありません",
    ),
    ("failure.file.invalid_data", "ファイルの内容が壊れています"),
    (
        "failure.file.already_exists",
        "同じ名前のファイルがすでにあります",
    ),
    (
        "failure.tool_library.locked",
        "{owner} がこのライブラリに書いています。しばらく待ってから操作し直してください",
    ),
    (
        "failure.tool_library.read_only",
        "このライブラリに書く許可がないため、読むだけで使います",
    ),
    (
        "failure.tool_library.unavailable",
        "読めないか、同じライブラリがほかにあるため、このライブラリには書けません",
    ),
    (
        "failure.tool_library.toolset_not_found",
        "ツールセットがありません。ほかの PC で消された可能性があります",
    ),
    (
        "failure.tool_library.empty_name",
        "ライブラリの名前が空です",
    ),
    (
        "failure.tool_library.empty_toolset_name",
        "ツールセットの名前が空です",
    ),
    (
        "failure.tool_library.invalid_tool",
        "工具の値に誤りがあります",
    ),
    (
        "failure.tool_library.already_registered",
        "この工具はすでにライブラリにあります",
    ),
    ("failure.file.other", "ファイルを読み書きできません"),
    ("failure.model_file.too_large", "ファイルが大きすぎます"),
    (
        "failure.model_file.newer",
        "新しい版のアプリで保存されたファイルです",
    ),
    (
        "failure.model_file.invalid",
        "壊れているか、形式の違うファイルです",
    ),
    (
        "failure.job.dependency_not_earlier",
        "依存するジョブ {job} が、前のジョブではありません",
    ),
    (
        "failure.job.dependency_cannot_run",
        "依存するジョブ {job} を計算できません",
    ),
    (
        "failure.job.dependency_not_found",
        "依存するジョブ {job} がありません",
    ),
    (
        "failure.job.dependency_failed",
        "依存するジョブ {job} が失敗しました",
    ),
    (
        "failure.job.dependency_canceled",
        "依存するジョブ {job} が取り消されました",
    ),
    ("failure.job.runner_not_started", "ランナーを起動できません"),
    (
        "failure.job.runner_wrote_no_result",
        "ランナーが結果を書かずに終わりました",
    ),
    (
        "failure.job.runner_invalid_arguments",
        "ランナーの引数が正しくありません",
    ),
    (
        "failure.job.runner_cannot_read_input",
        "ランナーが入力を読めません",
    ),
    ("failure.job.invalid_input", "入力が正しくありません"),
    ("failure.job.calculation_failed", "計算に失敗しました"),
    (
        "failure.job.runner_cannot_write_result",
        "ランナーが結果を書けません",
    ),
    (
        "failure.job.toolpath_too_large",
        "ツールパスの点が多すぎて、メモリの上限を超えるため、計算を止めました",
    ),
    (
        "failure.job.runner_exit_code",
        "ランナーが終了の番号 {code} で終わりました",
    ),
    ("failure.job.runner_stopped", "ランナーが止められました"),
    (
        "failure.job.manager_stopped",
        "計算中にジョブマネージャーが止まりました",
    ),
    (
        "failure.job.manager_quit",
        "ジョブマネージャーを終了しました",
    ),
    (
        "failure.job.unknown",
        "計算に失敗しました。詳しくはログを確かめてください",
    ),
];

const FAILURE_TEMPLATE_EN: &[(&str, &str)] = &[
    (
        "failure.input_registration",
        "Cannot register the calculation input",
    ),
    (
        "failure.not_succeeded_simulation",
        "Job {job} is not a succeeded cutting simulation",
    ),
    (
        "failure.not_succeeded_toolpath",
        "Job {job} is not a succeeded toolpath generation",
    ),
    ("failure.missing_artifact", "Job {job} has no result file"),
    (
        "failure.missing_input",
        "Job {job} has no calculation input",
    ),
    (
        "failure.sequence",
        "The machining operation cannot be found",
    ),
    (
        "failure.empty_product_shape",
        "There is no shape to machine",
    ),
    (
        "failure.conditions",
        "Some calculation conditions are invalid; see the messages under each field",
    ),
    ("failure.orchestration", "Cannot handle the calculation job"),
    (
        "failure.job_manager_not_found",
        "Cannot calculate because the job manager is not installed",
    ),
    (
        "failure.job_manager_outdated",
        "The running job manager is a different version from this app. Restart the job manager",
    ),
    (
        "failure.job_manager_not_running",
        "The job manager is not running",
    ),
    (
        "failure.job_store",
        "Cannot connect to or start the job manager",
    ),
    (
        "failure.no_current_imported_toolpath",
        "The space model has no CL of the current conditions",
    ),
    (
        "failure.conditions_not_set",
        "The conditions of the operation are not set",
    ),
    (
        "failure.toolpath_too_large",
        "The toolpath would need about {gb} GB, more than the memory limit for the calculation",
    ),
    (
        "failure.inverse_offset_shape",
        "Cannot build the inverse offset shape",
    ),
    (
        "failure.no_succeeded_simulation",
        "No succeeded cutting simulation for the current conditions",
    ),
    (
        "failure.no_succeeded_toolpath",
        "No succeeded toolpath generation for the current conditions",
    ),
    ("failure.animation.artifact", "Cannot read the result file"),
    (
        "failure.animation.no_snapshot",
        "The result has no snapshot",
    ),
    ("failure.animation.replay", "Cannot replay the result"),
    (
        "failure.inverse_offset_display",
        "Cannot display the inverse offset shape",
    ),
    (
        "failure.shape.stl",
        "The file is broken or in another format",
    ),
    ("failure.shape.unknown", "The shape cannot be found"),
    (
        "failure.shape.mixed_faces",
        "NURBS faces and triangle faces are mixed",
    ),
    (
        "failure.shape.tessellation",
        "Cannot split a face into triangles",
    ),
    ("failure.shape.no_faces", "There are no faces"),
    (
        "failure.shape.step_syntax",
        "The file is broken or in another format",
    ),
    (
        "failure.shape.step_limit",
        "The shapes in the file are larger than can be loaded",
    ),
    (
        "failure.shape.step_no_shapes",
        "There are no shapes that can be loaded",
    ),
    (
        "failure.shape.source_faces",
        "The faces of the source file do not match the faces of the shape",
    ),
    ("failure.file.not_found", "The file is not found"),
    (
        "failure.file.permission_denied",
        "No permission to read or write the file",
    ),
    ("failure.file.invalid_data", "The file content is broken"),
    (
        "failure.file.already_exists",
        "A file with the same name already exists",
    ),
    (
        "failure.tool_library.locked",
        "{owner} is writing to this library. Wait a moment and try again",
    ),
    (
        "failure.tool_library.read_only",
        "No permission to write to this library, so it is used read-only",
    ),
    (
        "failure.tool_library.unavailable",
        "This library cannot be written because it cannot be read or the same library is listed",
    ),
    (
        "failure.tool_library.toolset_not_found",
        "The tool set is not found. It may have been removed on another PC",
    ),
    (
        "failure.tool_library.empty_name",
        "The library name is empty",
    ),
    (
        "failure.tool_library.empty_toolset_name",
        "The tool set name is empty",
    ),
    (
        "failure.tool_library.invalid_tool",
        "The tool values are invalid",
    ),
    (
        "failure.tool_library.already_registered",
        "This tool is already in a library",
    ),
    ("failure.file.other", "Cannot read or write the file"),
    ("failure.model_file.too_large", "The file is too large"),
    (
        "failure.model_file.newer",
        "The file was saved by a newer version of the app",
    ),
    (
        "failure.model_file.invalid",
        "The file is broken or in another format",
    ),
    (
        "failure.job.dependency_not_earlier",
        "Dependency job {job} is not an earlier job",
    ),
    (
        "failure.job.dependency_cannot_run",
        "Dependency job {job} cannot be calculated",
    ),
    (
        "failure.job.dependency_not_found",
        "Dependency job {job} does not exist",
    ),
    (
        "failure.job.dependency_failed",
        "Dependency job {job} failed",
    ),
    (
        "failure.job.dependency_canceled",
        "Dependency job {job} was canceled",
    ),
    ("failure.job.runner_not_started", "Cannot start the runner"),
    (
        "failure.job.runner_wrote_no_result",
        "The runner ended without writing a result",
    ),
    (
        "failure.job.runner_invalid_arguments",
        "The runner arguments are invalid",
    ),
    (
        "failure.job.runner_cannot_read_input",
        "The runner cannot read the input",
    ),
    ("failure.job.invalid_input", "The input is invalid"),
    ("failure.job.calculation_failed", "The calculation failed"),
    (
        "failure.job.runner_cannot_write_result",
        "The runner cannot write the result",
    ),
    (
        "failure.job.toolpath_too_large",
        "The calculation was stopped because the toolpath has too many points and exceeds the memory limit",
    ),
    (
        "failure.job.runner_exit_code",
        "The runner ended with exit code {code}",
    ),
    ("failure.job.runner_stopped", "The runner was stopped"),
    (
        "failure.job.manager_stopped",
        "The job manager stopped during the calculation",
    ),
    ("failure.job.manager_quit", "The job manager was quit"),
    (
        "failure.job.unknown",
        "The calculation failed. See the log for details",
    ),
];

const FAILURE_MESSAGE_CATALOG: TableMessageCatalog = TableMessageCatalog::new(
    FAILURE_TEMPLATE_JA,
    FAILURE_TEMPLATE_EN,
    "思わぬ理由で失敗しました。詳しくはログを確かめてください",
    "Failed for an unexpected reason. See the log for details",
);

/// 失敗の理由 `message` を、言語 `locale` の文言にする
pub fn resolve_failure_message(locale: UiLocale, message: &UiMessage) -> String {
    resolve_message(&FAILURE_MESSAGE_CATALOG, locale, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_key_has_both_japanese_and_english_texts() {
        let ja: Vec<&str> = FAILURE_TEMPLATE_JA.iter().map(|(key, _)| *key).collect();
        let en: Vec<&str> = FAILURE_TEMPLATE_EN.iter().map(|(key, _)| *key).collect();
        assert_eq!(ja, en);
    }

    #[test]
    fn texts_have_no_parentheses() {
        for (key, text) in FAILURE_TEMPLATE_JA.iter().chain(FAILURE_TEMPLATE_EN) {
            assert!(!text.contains(['（', '）', '(', ')']), "{key}: {text}");
        }
    }
}
