//! エラーを、画面に出す失敗の理由（文言のキーと引数）へ変える。
//!
//! エラーの種類ごとにキーを決める。エラーの中の詳しい理由（英語の文）は画面に出さず、ログに残す。

use application::cam_simulation_workflow::CamSimulationWorkflowError;
use application::product_shape::ProductShapeError;
use application::space_model_file::SpaceModelFileError;
use i18n_foundation::{UiMessage, UiMessageArg};

use crate::cutting_simulation_session::CuttingSimulationSessionError;
use application::cutting_simulation_playback::CuttingSimulationPlaybackError;

fn message(key: &str) -> UiMessage {
    UiMessage {
        key: key.to_string(),
        args: Vec::new(),
    }
}

fn job_message(key: &str, job: u64) -> UiMessage {
    UiMessage {
        key: key.to_string(),
        args: vec![UiMessageArg {
            name: "job".to_string(),
            value: job.to_string(),
        }],
    }
}

/// メモリの大きさ `gb`（GB）の、画面に出す数（10 未満は小数 1 桁、それ以上は整数）
pub fn format_gb(gb: f64) -> String {
    if gb < 10.0 {
        format!("{gb:.1}")
    } else {
        format!("{gb:.0}")
    }
}

/// 投入・実行・取り込みの失敗 `error` の、画面に出す理由
pub fn cutting_simulation_failure_message(error: &CuttingSimulationSessionError) -> UiMessage {
    match error {
        CuttingSimulationSessionError::Workflow(error) => workflow_failure_message(error),
        CuttingSimulationSessionError::NoSucceededSimulation => {
            message("failure.no_succeeded_simulation")
        }
        CuttingSimulationSessionError::NoSucceededToolpath => {
            message("failure.no_succeeded_toolpath")
        }
        CuttingSimulationSessionError::ConditionsNotSet => message("failure.conditions_not_set"),
        CuttingSimulationSessionError::Playback(error) => match error {
            CuttingSimulationPlaybackError::Artifact(_) => message("failure.animation.artifact"),
            CuttingSimulationPlaybackError::NoSnapshot => message("failure.animation.no_snapshot"),
            CuttingSimulationPlaybackError::Replay(_) => message("failure.animation.replay"),
        },
    }
}

/// 計算の流れの失敗 `error` の、画面に出す理由
pub(crate) fn workflow_failure_message(error: &CamSimulationWorkflowError) -> UiMessage {
    match error {
        CamSimulationWorkflowError::InputRegistration(_) => message("failure.input_registration"),
        CamSimulationWorkflowError::NotSucceededSimulation(job) => {
            job_message("failure.not_succeeded_simulation", *job)
        }
        CamSimulationWorkflowError::NotSucceededToolpath(job) => {
            job_message("failure.not_succeeded_toolpath", *job)
        }
        CamSimulationWorkflowError::MissingArtifact(job) => {
            job_message("failure.missing_artifact", *job)
        }
        CamSimulationWorkflowError::MissingInput(job) => job_message("failure.missing_input", *job),
        CamSimulationWorkflowError::Sequence(_) => message("failure.sequence"),
        CamSimulationWorkflowError::ProductShape(error) => product_shape_failure_message(error),
        CamSimulationWorkflowError::EmptyProductShape => message("failure.empty_product_shape"),
        CamSimulationWorkflowError::Conditions(_) => message("failure.conditions"),
        CamSimulationWorkflowError::Orchestration(_) => message("failure.orchestration"),
        CamSimulationWorkflowError::InverseOffsetShape(_) => {
            message("failure.inverse_offset_shape")
        }
        CamSimulationWorkflowError::InverseOffsetDisplay(_) => {
            message("failure.inverse_offset_display")
        }
        CamSimulationWorkflowError::JobManagerNotFound => message("failure.job_manager_not_found"),
        CamSimulationWorkflowError::JobManagerOutdated => message("failure.job_manager_outdated"),
        CamSimulationWorkflowError::JobManagerNotRunning => {
            message("failure.job_manager_not_running")
        }
        CamSimulationWorkflowError::JobStore(_) => message("failure.job_store"),
        CamSimulationWorkflowError::NoCurrentImportedToolpath => {
            message("failure.no_current_imported_toolpath")
        }
        CamSimulationWorkflowError::ToolpathTooLarge(estimate) => UiMessage {
            key: "failure.toolpath_too_large".to_string(),
            args: vec![UiMessageArg {
                name: "gb".to_string(),
                value: format_gb(estimate.memory_gb()),
            }],
        },
    }
}

/// 製品形状の読み込み・解決の失敗 `error` の、画面に出す理由
pub fn product_shape_failure_message(error: &ProductShapeError) -> UiMessage {
    match error {
        ProductShapeError::StlFile { kind, .. } => file_failure_message(*kind),
        ProductShapeError::Stl(_) => message("failure.shape.stl"),
        ProductShapeError::UnknownEntity(_) | ProductShapeError::UnknownFace(_) => {
            message("failure.shape.unknown")
        }
        ProductShapeError::MixedFaceKinds => message("failure.shape.mixed_faces"),
        ProductShapeError::Tessellation(_) => message("failure.shape.tessellation"),
        ProductShapeError::NoFaces => message("failure.shape.no_faces"),
    }
}

/// モデルファイルの読み書きの失敗 `error` の、画面に出す理由
pub fn model_file_failure_message(error: &SpaceModelFileError) -> UiMessage {
    use application::space_model_file::{
        BinaryFormatError, FORMAT_VERSION_MAJOR_V1, SECTION_VERSION,
    };
    match error {
        SpaceModelFileError::Io(error) => file_failure_message(error.kind()),
        SpaceModelFileError::TooLarge => message("failure.model_file.too_large"),
        // 新しい版のアプリで保存したファイル（知らない必須の区画・知っているより新しい版）
        SpaceModelFileError::UnknownRequiredSection(_) => message("failure.model_file.newer"),
        SpaceModelFileError::UnsupportedSectionVersion { version, .. }
            if *version > SECTION_VERSION =>
        {
            message("failure.model_file.newer")
        }
        SpaceModelFileError::Format(BinaryFormatError::UnsupportedMinorVersion {
            found_minor,
            supported_minors,
            ..
        }) if supported_minors.iter().all(|minor| found_minor > minor) => {
            message("failure.model_file.newer")
        }
        SpaceModelFileError::Format(BinaryFormatError::UnsupportedMajorVersion {
            found,
            supported,
        }) if found > supported => message("failure.model_file.newer"),
        SpaceModelFileError::Format(BinaryFormatError::ConverterRequired {
            found_major, ..
        }) if *found_major > FORMAT_VERSION_MAJOR_V1 => message("failure.model_file.newer"),
        SpaceModelFileError::UnsupportedSectionVersion { .. }
        | SpaceModelFileError::Format(_)
        | SpaceModelFileError::MissingSection(_)
        | SpaceModelFileError::DuplicateSection(_)
        | SpaceModelFileError::InvalidSectionTable(_)
        | SpaceModelFileError::InvalidSection { .. }
        | SpaceModelFileError::ProductShape(_)
        | SpaceModelFileError::MachiningSequence(_)
        | SpaceModelFileError::ImportedToolpath(_) => message("failure.model_file.invalid"),
    }
}

/// ファイルの読み書きの失敗の種類 `kind` の、画面に出す理由（OS の文は OS の言語になるため、種類で決める）
pub fn file_failure_message(kind: std::io::ErrorKind) -> UiMessage {
    match kind {
        std::io::ErrorKind::NotFound => message("failure.file.not_found"),
        std::io::ErrorKind::PermissionDenied => message("failure.file.permission_denied"),
        std::io::ErrorKind::InvalidData | std::io::ErrorKind::UnexpectedEof => {
            message("failure.file.invalid_data")
        }
        _ => message("failure.file.other"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::failure_message_catalog::resolve_failure_message;
    use i18n_foundation::UiLocale;

    fn assert_resolved(message: &UiMessage) {
        for locale in [UiLocale::Ja, UiLocale::En] {
            let text = resolve_failure_message(locale, message);
            assert!(
                !text.contains('{')
                    && !text.contains("unexpected reason")
                    && !text.contains("思わぬ"),
                "{message:?}: {text}"
            );
        }
    }

    #[test]
    fn every_model_file_failure_has_its_own_text() {
        use application::space_model_file::BinaryFormatError;
        let errors = [
            SpaceModelFileError::Io(std::io::ErrorKind::NotFound.into()),
            SpaceModelFileError::TooLarge,
            SpaceModelFileError::UnknownRequiredSection(*b"FEAT"),
            SpaceModelFileError::Format(BinaryFormatError::UnknownMagic(*b"XXXX")),
            SpaceModelFileError::InvalidSectionTable("sections"),
            SpaceModelFileError::UnsupportedSectionVersion {
                kind: *b"FACE",
                version: 2,
            },
            // 知っている版より古い・壊れた版は、新しい版とは言わない
            SpaceModelFileError::UnsupportedSectionVersion {
                kind: *b"FACE",
                version: 0,
            },
            SpaceModelFileError::Format(BinaryFormatError::ConverterRequired {
                found_major: 1,
                found_minor: 0,
            }),
        ];
        let keys: Vec<String> = errors
            .iter()
            .map(|error| {
                let message = model_file_failure_message(error);
                assert_resolved(&message);
                message.key
            })
            .collect();
        assert_eq!(
            keys,
            [
                "failure.file.not_found",
                "failure.model_file.too_large",
                "failure.model_file.newer",
                "failure.model_file.invalid",
                "failure.model_file.invalid",
                "failure.model_file.newer",
                "failure.model_file.invalid",
                "failure.model_file.newer",
            ]
        );
    }

    #[test]
    fn every_workflow_failure_has_its_own_text() {
        let errors = [
            CamSimulationWorkflowError::InputRegistration("reason".to_string()),
            CamSimulationWorkflowError::NotSucceededSimulation(3),
            CamSimulationWorkflowError::NotSucceededToolpath(3),
            CamSimulationWorkflowError::MissingArtifact(3),
            CamSimulationWorkflowError::MissingInput(3),
            CamSimulationWorkflowError::EmptyProductShape,
            CamSimulationWorkflowError::Conditions(Vec::new()),
            CamSimulationWorkflowError::ProductShape(ProductShapeError::NoFaces),
            CamSimulationWorkflowError::JobManagerNotFound,
            CamSimulationWorkflowError::JobManagerNotRunning,
            CamSimulationWorkflowError::JobManagerOutdated,
            CamSimulationWorkflowError::JobStore("reason".to_string()),
            CamSimulationWorkflowError::NoCurrentImportedToolpath,
        ];
        for error in &errors {
            assert_resolved(&workflow_failure_message(error));
        }
        let job = workflow_failure_message(&CamSimulationWorkflowError::MissingInput(7));
        assert!(resolve_failure_message(UiLocale::En, &job).contains('7'));
    }

    #[test]
    fn every_session_and_shape_failure_has_its_own_text() {
        assert_resolved(&cutting_simulation_failure_message(
            &CuttingSimulationSessionError::NoSucceededSimulation,
        ));
        assert_resolved(&cutting_simulation_failure_message(
            &CuttingSimulationSessionError::NoSucceededToolpath,
        ));
        assert_resolved(&cutting_simulation_failure_message(
            &CuttingSimulationSessionError::Playback(CuttingSimulationPlaybackError::NoSnapshot),
        ));
        let missing = product_shape_failure_message(&ProductShapeError::StlFile {
            kind: std::io::ErrorKind::NotFound,
            reason: "reason".to_string(),
        });
        assert_eq!(missing.key, "failure.file.not_found");
        for error in [
            ProductShapeError::Stl("reason".to_string()),
            ProductShapeError::MixedFaceKinds,
            ProductShapeError::NoFaces,
        ] {
            assert_resolved(&product_shape_failure_message(&error));
        }
    }

    #[test]
    fn file_failures_are_told_by_kind() {
        for kind in [
            std::io::ErrorKind::NotFound,
            std::io::ErrorKind::PermissionDenied,
            std::io::ErrorKind::InvalidData,
            std::io::ErrorKind::Other,
        ] {
            assert_resolved(&file_failure_message(kind));
        }
    }

    /// このファイルが返すキーが、すべて文言の表にある（打ち間違えると、思わぬ失敗の文言になるため）
    #[test]
    fn every_key_in_this_mapper_is_in_the_catalog() {
        let source = include_str!("failure_message_mapper.rs");
        let body = &source[..source.find("#[cfg(test)]").unwrap()];
        let keys: Vec<&str> = body
            .split('"')
            .filter(|part| part.starts_with("failure."))
            .collect();
        assert!(keys.len() > 20, "{keys:?}");
        for key in keys {
            for locale in [UiLocale::Ja, UiLocale::En] {
                let text = resolve_failure_message(locale, &message(key));
                assert!(
                    !text.contains("unexpected reason") && !text.contains("思わぬ"),
                    "{key}: {text}"
                );
            }
        }
    }

    #[test]
    fn unknown_keys_fall_back_to_the_unexpected_text() {
        let text = resolve_failure_message(UiLocale::Ja, &message("failure.no_such_key"));
        assert!(text.contains("ログ"), "{text}");
    }
}
