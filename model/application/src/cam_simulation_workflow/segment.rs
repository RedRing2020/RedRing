//! 加工手順の切削シミュレーションの区間（設計: CAM_PROCESS_ENTITY_DESIGN.md 6.3）
//!
//! 切削シミュレーションは、加工手順の工程ごとの区間に分け、加工の順に前の区間の終わりの素材から削る。区間の結果は、
//! その区間の前提（区間の前提の鍵）が今の手順と同じときだけ、今の結果とする。

use std::fmt::Write as _;

use cam_entity::{MachiningOperation, MachiningOperationId, MachiningSequence};

/// 加工手順 `sequence` の切削シミュレーションの格子を決める工具の半径（計算の条件を設定した工程の工具のうち、最も
/// 小さい半径。手順のすべての区間で同じ格子にするため。設定した工程がなければ `fallback` の工程の工具の半径）
pub(super) fn grid_tool_radius(sequence: &MachiningSequence, fallback: &MachiningOperation) -> f64 {
    sequence
        .operations()
        .iter()
        .filter(|operation| operation.conditions_set())
        .map(tool_radius)
        .filter(|radius| radius.is_finite() && *radius > 0.0)
        .min_by(f64::total_cmp)
        .unwrap_or_else(|| tool_radius(fallback))
}

fn tool_radius(operation: &MachiningOperation) -> f64 {
    operation.content().conditions.tool.radius()
}

/// 加工手順 `sequence` の工程ごとの区間の前提の鍵（加工の順）
///
/// 区間の前提は、その工程と手順の中で前にある工程の ID と改訂の鍵の列（加工の順）と、格子（素材の余裕・精度・格子を
/// 決める工具の半径）とする。工程を変える・並べ替える・格子が変わると、その工程から後の区間の鍵が変わる。鍵は前提を
/// まとめた文字列の要約（ジョブの投入元の長さの上限に収めるため）とする。
pub(super) fn segment_keys(sequence: &MachiningSequence) -> Vec<(MachiningOperationId, String)> {
    let Some(first) = sequence.operations().first() else {
        return Vec::new();
    };
    let stock = sequence.stock();
    let margins = stock.margins;
    let mut premise = String::new();
    for value in [
        margins.minus_x,
        margins.plus_x,
        margins.minus_y,
        margins.plus_y,
        margins.bottom,
        margins.top,
        stock.precision,
        grid_tool_radius(sequence, first),
    ] {
        let _ = write!(premise, "{:016x};", value.to_bits());
    }
    sequence
        .operations()
        .iter()
        .map(|operation| {
            let _ = write!(premise, "{}:{};", operation.id(), operation.revision_key());
            (operation.id(), digest(&premise))
        })
        .collect()
}

/// 文字列 `text` の要約（FNV-1a の 64 ビットを、種を替えて 2 つ。32 桁の 16 進数）
fn digest(text: &str) -> String {
    const PRIME: u64 = 0x0000_0100_0000_01b3;
    let hash = |seed: u64| {
        text.bytes().fold(seed, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(PRIME)
        })
    };
    format!(
        "{:016x}{:016x}",
        hash(0xcbf2_9ce4_8422_2325),
        hash(0x6c62_272e_07bb_0142)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digests_differ_for_different_premises_and_are_stable() {
        assert_eq!(digest("a:1;"), digest("a:1;"));
        assert_ne!(digest("a:1;"), digest("a:2;"));
        assert_eq!(digest("").len(), 32);
    }
}
