use cam_core::ValidationError;
use i18n_foundation::{render_template, TableMessageCatalog, UiLocale, UiMessage};

use crate::validation_message_mapper::validation_error_to_ui_message;

const VALIDATION_TEMPLATE_JA: &[(&str, &str)] = &[
    (
        "validation.contour.not_closed",
        "輪郭が閉じていません: distance={distance}, tolerance={tolerance}",
    ),
    (
        "validation.contour.insufficient_points",
        "閉曲線に必要な点数が不足しています: point_count={point_count}",
    ),
    ("validation.contour.empty", "輪郭が空です"),
    (
        "validation.machine.feed_rate_limit_exceeded",
        "送り速度が上限を超えています: segment={segment_index}, feed_rate={feed_rate}, max={max_feed_rate}",
    ),
    (
        "validation.machine.linear_acceleration_limit_exceeded",
        "線形加速度が上限を超えています: segment={segment_index}, acceleration={acceleration_mm_per_sec2}, max={max_acceleration_mm_per_sec2}",
    ),
    (
        "validation.machine.rotary_acceleration_limit_exceeded",
        "回転加速度が上限を超えています: segment={segment_index}, acceleration={acceleration_deg_per_sec2}, max={max_acceleration_deg_per_sec2}",
    ),
    (
        "validation.machine.linear_axis_limit_exceeded",
        "直動軸が範囲外です: segment={segment_index}, pose={pose_endpoint}, axis={axis_name}, value={value_mm}, range=[{min_mm}, {max_mm}]",
    ),
    (
        "validation.machine.rotary_axis_limit_exceeded",
        "回転軸が範囲外です: segment={segment_index}, pose={pose_endpoint}, axis={axis_name}, value={value_deg}, range=[{min_deg}, {max_deg}]",
    ),
    (
        "validation.machine.unsupported_axis",
        "未対応の機械軸です: segment={segment_index}, pose={pose_endpoint}, axis={axis_name}, kind={axis_kind}",
    ),
];

const VALIDATION_TEMPLATE_EN: &[(&str, &str)] = &[
    (
        "validation.contour.not_closed",
        "Contour is not closed: distance={distance}, tolerance={tolerance}",
    ),
    (
        "validation.contour.insufficient_points",
        "Insufficient points for closed contour: point_count={point_count}",
    ),
    ("validation.contour.empty", "Contour is empty"),
    (
        "validation.machine.feed_rate_limit_exceeded",
        "Feed rate exceeds machine limit: segment={segment_index}, feed_rate={feed_rate}, max={max_feed_rate}",
    ),
    (
        "validation.machine.linear_acceleration_limit_exceeded",
        "Linear acceleration exceeds machine limit: segment={segment_index}, acceleration={acceleration_mm_per_sec2}, max={max_acceleration_mm_per_sec2}",
    ),
    (
        "validation.machine.rotary_acceleration_limit_exceeded",
        "Rotary acceleration exceeds machine limit: segment={segment_index}, acceleration={acceleration_deg_per_sec2}, max={max_acceleration_deg_per_sec2}",
    ),
    (
        "validation.machine.linear_axis_limit_exceeded",
        "Linear axis is out of range: segment={segment_index}, pose={pose_endpoint}, axis={axis_name}, value={value_mm}, range=[{min_mm}, {max_mm}]",
    ),
    (
        "validation.machine.rotary_axis_limit_exceeded",
        "Rotary axis is out of range: segment={segment_index}, pose={pose_endpoint}, axis={axis_name}, value={value_deg}, range=[{min_deg}, {max_deg}]",
    ),
    (
        "validation.machine.unsupported_axis",
        "Unsupported machine axis: segment={segment_index}, pose={pose_endpoint}, axis={axis_name}, kind={axis_kind}",
    ),
];

pub const VALIDATION_MESSAGE_CATALOG: TableMessageCatalog = TableMessageCatalog::new(
    VALIDATION_TEMPLATE_JA,
    VALIDATION_TEMPLATE_EN,
    "__missing__",
    "__missing__",
);

pub fn resolve_validation_error(locale: UiLocale, error: &ValidationError) -> String {
    let message = validation_error_to_ui_message(error);
    resolve_validation_message(locale, &message)
}

pub fn resolve_validation_message(locale: UiLocale, message: &UiMessage) -> String {
    resolve_with_default_locale(&VALIDATION_MESSAGE_CATALOG, locale, UiLocale::En, message)
}

fn resolve_with_default_locale(
    catalog: &TableMessageCatalog,
    locale: UiLocale,
    default_locale: UiLocale,
    message: &UiMessage,
) -> String {
    if let Some(template) = catalog.template_or_none(locale, &message.key) {
        return render_template(template, &message.args);
    }

    if let Some(template) = catalog.template_or_none(default_locale, &message.key) {
        return render_template(template, &message.args);
    }

    message.key.clone()
}

#[cfg(test)]
mod tests {
    use super::{
        resolve_validation_message, resolve_with_default_locale, TableMessageCatalog, UiLocale,
    };
    use i18n_foundation::{UiMessage, UiMessageArg};

    #[test]
    fn resolves_validation_message_in_ja_and_en() {
        let message = UiMessage {
            key: "validation.contour.not_closed".to_string(),
            args: vec![
                UiMessageArg {
                    name: "distance".to_string(),
                    value: "1.000000".to_string(),
                },
                UiMessageArg {
                    name: "tolerance".to_string(),
                    value: "0.010000".to_string(),
                },
            ],
        };

        let ja = resolve_validation_message(UiLocale::Ja, &message);
        let en = resolve_validation_message(UiLocale::En, &message);

        assert!(ja.contains("輪郭が閉じていません"));
        assert!(en.contains("Contour is not closed"));
    }

    #[test]
    fn fallback_order_is_locale_then_en_then_key() {
        let ja_only_missing_catalog = TableMessageCatalog::new(
            &[("validation.only.ja", "JA")],
            &[("validation.only.en", "EN")],
            "__missing__",
            "__missing__",
        );

        let en_fallback_message = UiMessage {
            key: "validation.only.en".to_string(),
            args: Vec::new(),
        };
        let key_fallback_message = UiMessage {
            key: "validation.unknown".to_string(),
            args: Vec::new(),
        };

        let resolved_by_en = resolve_with_default_locale(
            &ja_only_missing_catalog,
            UiLocale::Ja,
            UiLocale::En,
            &en_fallback_message,
        );
        let resolved_by_key = resolve_with_default_locale(
            &ja_only_missing_catalog,
            UiLocale::Ja,
            UiLocale::En,
            &key_fallback_message,
        );

        assert_eq!(resolved_by_en, "EN");
        assert_eq!(resolved_by_key, "validation.unknown");
    }

    #[test]
    fn all_validation_keys_have_templates_for_ja_and_en() {
        let required_keys = [
            "validation.contour.not_closed",
            "validation.contour.insufficient_points",
            "validation.contour.empty",
            "validation.machine.feed_rate_limit_exceeded",
            "validation.machine.linear_acceleration_limit_exceeded",
            "validation.machine.rotary_acceleration_limit_exceeded",
            "validation.machine.linear_axis_limit_exceeded",
            "validation.machine.rotary_axis_limit_exceeded",
            "validation.machine.unsupported_axis",
        ];

        for key in required_keys {
            let message = UiMessage {
                key: key.to_string(),
                args: Vec::new(),
            };

            let ja = resolve_validation_message(UiLocale::Ja, &message);
            let en = resolve_validation_message(UiLocale::En, &message);

            assert_ne!(ja, key);
            assert_ne!(en, key);
        }
    }
}
