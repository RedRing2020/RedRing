use cam_core::ValidationError;
use i18n_foundation::{UiMessage, UiMessageArg};

pub fn validation_error_to_ui_message(error: &ValidationError) -> UiMessage {
    let args = match error {
        ValidationError::ContourNotClosed {
            distance,
            tolerance,
        } => vec![
            arg("distance", format!("{distance:.6}")),
            arg("tolerance", format!("{tolerance:.6}")),
        ],
        ValidationError::InsufficientPoints { point_count } => {
            vec![arg("point_count", point_count.to_string())]
        }
        ValidationError::EmptyContour => Vec::new(),
        ValidationError::FeedRateLimitExceeded {
            segment_index,
            feed_rate,
            max_feed_rate,
        } => vec![
            arg("segment_index", segment_index.to_string()),
            arg("feed_rate", format!("{feed_rate:.6}")),
            arg("max_feed_rate", format!("{max_feed_rate:.6}")),
        ],
        ValidationError::LinearAccelerationLimitExceeded {
            segment_index,
            acceleration_mm_per_sec2,
            max_acceleration_mm_per_sec2,
        } => vec![
            arg("segment_index", segment_index.to_string()),
            arg(
                "acceleration_mm_per_sec2",
                format!("{acceleration_mm_per_sec2:.6}"),
            ),
            arg(
                "max_acceleration_mm_per_sec2",
                format!("{max_acceleration_mm_per_sec2:.6}"),
            ),
        ],
        ValidationError::RotaryAccelerationLimitExceeded {
            segment_index,
            acceleration_deg_per_sec2,
            max_acceleration_deg_per_sec2,
        } => vec![
            arg("segment_index", segment_index.to_string()),
            arg(
                "acceleration_deg_per_sec2",
                format!("{acceleration_deg_per_sec2:.6}"),
            ),
            arg(
                "max_acceleration_deg_per_sec2",
                format!("{max_acceleration_deg_per_sec2:.6}"),
            ),
        ],
        ValidationError::LinearAxisLimitExceeded {
            segment_index,
            pose_endpoint,
            axis_name,
            value_mm,
            min_mm,
            max_mm,
        } => vec![
            arg("segment_index", segment_index.to_string()),
            arg("pose_endpoint", pose_endpoint.to_string()),
            arg("axis_name", axis_name.clone()),
            arg("value_mm", format!("{value_mm:.6}")),
            arg("min_mm", format!("{min_mm:.6}")),
            arg("max_mm", format!("{max_mm:.6}")),
        ],
        ValidationError::RotaryAxisLimitExceeded {
            segment_index,
            pose_endpoint,
            axis_name,
            value_deg,
            min_deg,
            max_deg,
        } => vec![
            arg("segment_index", segment_index.to_string()),
            arg("pose_endpoint", pose_endpoint.to_string()),
            arg("axis_name", axis_name.clone()),
            arg("value_deg", format!("{value_deg:.6}")),
            arg("min_deg", format!("{min_deg:.6}")),
            arg("max_deg", format!("{max_deg:.6}")),
        ],
        ValidationError::UnsupportedMachineAxis {
            segment_index,
            pose_endpoint,
            axis_name,
            axis_kind,
        } => vec![
            arg("segment_index", segment_index.to_string()),
            arg("pose_endpoint", pose_endpoint.to_string()),
            arg("axis_name", axis_name.clone()),
            arg("axis_kind", format!("{axis_kind:?}")),
        ],
    };

    UiMessage {
        key: error.message_key().to_string(),
        args,
    }
}

fn arg(name: &str, value: String) -> UiMessageArg {
    UiMessageArg {
        name: name.to_string(),
        value,
    }
}

#[cfg(test)]
mod tests {
    use super::validation_error_to_ui_message;
    use cam_core::{MachineAxisKind, ValidationError};

    #[test]
    fn contour_not_closed_maps_to_stable_key_and_args() {
        let error = ValidationError::ContourNotClosed {
            distance: 1.0,
            tolerance: 0.01,
        };

        let message = validation_error_to_ui_message(&error);

        assert_eq!(message.key, "validation.contour.not_closed");
        assert_eq!(message.args.len(), 2);
        assert!(message.args.iter().any(|a| a.name == "distance"));
        assert!(message.args.iter().any(|a| a.name == "tolerance"));
    }

    #[test]
    fn unsupported_axis_maps_all_context_fields() {
        let error = ValidationError::UnsupportedMachineAxis {
            segment_index: 3,
            pose_endpoint: "start",
            axis_name: "A".to_string(),
            axis_kind: MachineAxisKind::Rotary,
        };

        let message = validation_error_to_ui_message(&error);

        assert_eq!(message.key, "validation.machine.unsupported_axis");
        assert!(message.args.iter().any(|a| a.name == "segment_index"));
        assert!(message.args.iter().any(|a| a.name == "pose_endpoint"));
        assert!(message.args.iter().any(|a| a.name == "axis_name"));
        assert!(message.args.iter().any(|a| a.name == "axis_kind"));
    }
}
