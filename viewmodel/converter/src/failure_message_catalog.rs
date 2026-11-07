//! 失敗の理由の文言（日本語・英語）
//!
//! キーはエラーの種類ごとに決める。表にないキーは、思わぬ失敗として「詳しくはログを確かめてください」の文言にする。
//! 文言は、ほかの文言の括弧の中に入れて出すため、括弧を使わない。

use i18n_foundation::{resolve_message, TableMessageCatalog, UiLocale, UiMessage};

const FAILURE_TEMPLATE_JA: &[(&str, &str)] = &[
    ("failure.input_registration", "計算の入力を登録できません"),
    (
        "failure.not_succeeded_simulation",
        "ジョブ {job} は、正常に終わった切削シミュレーションではありません",
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
    ("failure.orchestration", "ジョブを投入・実行できません"),
    (
        "failure.job_manager_not_found",
        "ジョブマネージャーがインストールされていないため、計算を投入できません",
    ),
    (
        "failure.job_store",
        "ジョブの保存先を使えないか、ジョブマネージャーを起動できません",
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
    (
        "failure.shape.tessellation",
        "面を表示用の三角形に分けられません",
    ),
    ("failure.shape.no_faces", "面がありません"),
    ("failure.file.not_found", "ファイルが見つかりません"),
    (
        "failure.file.permission_denied",
        "ファイルを読み書きする許可がありません",
    ),
    ("failure.file.invalid_data", "ファイルの内容が壊れています"),
    ("failure.file.other", "ファイルを読み書きできません"),
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
    ("failure.orchestration", "Cannot submit or run the job"),
    (
        "failure.job_manager_not_found",
        "Cannot submit the calculation because the job manager is not installed",
    ),
    (
        "failure.job_store",
        "Cannot use the job store or start the job manager",
    ),
    (
        "failure.inverse_offset_shape",
        "Cannot build the inverse offset shape",
    ),
    (
        "failure.no_succeeded_simulation",
        "No succeeded cutting simulation for the current conditions",
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
        "Cannot split a face into triangles for display",
    ),
    ("failure.shape.no_faces", "There are no faces"),
    ("failure.file.not_found", "The file is not found"),
    (
        "failure.file.permission_denied",
        "No permission to read or write the file",
    ),
    ("failure.file.invalid_data", "The file content is broken"),
    ("failure.file.other", "Cannot read or write the file"),
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
