//! 素材のファイル（切削シミュレーションの区間の終わりの素材。設計: CAM_PROCESS_ENTITY_DESIGN.md 6.3）
//!
//! 共通のヘッダ（種別 `ArtifactKind::Stock`・版・単位・座標系）は cam_core の成果物の形式を使い、中身は本モジュールが
//! 定義する。中身は成果物と同じく節の列で、各節は種類 `u16` と長さ `u64` に続く中身からなる（数値はすべて
//! リトルエンディアン）。読み込みは、知らない種類の節を読み飛ばす。
//! 1. 格子: 切削シミュレーションの成果物の格子の節と同じ形
//! 2. 素材: 成果物のキーフレーム 1 つと同じ形で、素材の範囲（格子の初期の区間）から変わったデクセルの区間（格子の
//!    番号・デクセルの番号の昇順）

#[cfg(test)]
mod tests;

use std::io::{Read, Write};

use cam_core::{
    ArtifactHeaderV1, ArtifactKind, ArtifactPayload, BinaryFormatError, CoordinateFrame,
    FORMAT_VERSION_MINOR_V1, LengthUnit, read_artifact_v1,
};

use crate::simulation_artifact::{
    DexelGridLayout, StockDexels, grids_bytes, read_grids, read_sections, read_stock_dexels,
    write_dexel_states, write_section,
};

const SUPPORTED_MINORS: &[u16] = &[FORMAT_VERSION_MINOR_V1];

/// 節の種類
const SECTION_GRIDS: u16 = 1;
const SECTION_DEXELS: u16 = 2;

/// 素材の状態（格子と、素材の範囲から変わったデクセルの区間）
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct StockState {
    pub(crate) grids: [DexelGridLayout; 3],
    pub(crate) dexels: StockDexels,
}

/// 素材の状態を、素材のファイルの形式で書く
pub(crate) fn write_stock_state_v1<W: Write>(
    writer: &mut W,
    state: &StockState,
    unit: LengthUnit,
    frame: CoordinateFrame,
) -> Result<(), BinaryFormatError> {
    let mut dexels = Vec::new();
    write_dexel_states(&mut dexels, state.dexels.iter())?;
    let mut payload = Vec::new();
    write_section(&mut payload, SECTION_GRIDS, &grids_bytes(&state.grids));
    write_section(&mut payload, SECTION_DEXELS, &dexels);

    let mut header = ArtifactHeaderV1::new(ArtifactKind::Stock, payload.len() as u64);
    header.unit = unit;
    header.frame = frame;
    header.write_to(writer)?;
    writer.write_all(&payload)?;
    Ok(())
}

/// 素材のファイルを読む（種別・版・破損を検査する）
///
/// # Errors
///
/// 種別が素材でない、版に対応していない、途中で切れている、値が契約に違反している場合。
pub(crate) fn read_stock_state_v1<R: Read>(
    reader: &mut R,
) -> Result<(ArtifactHeaderV1, StockState), BinaryFormatError> {
    let (header, payload) = read_artifact_v1(reader)?;
    let ArtifactPayload::Stock(bytes) = payload else {
        return Err(BinaryFormatError::ArtifactKindMismatch {
            expected: ArtifactKind::Stock,
            found: header.kind,
        });
    };
    if !SUPPORTED_MINORS.contains(&header.version_minor) {
        return Err(BinaryFormatError::UnsupportedMinorVersion {
            major: header.version_major,
            found_minor: header.version_minor,
            supported_minors: SUPPORTED_MINORS,
        });
    }
    let [Some(grids), Some(dexels)] = read_sections(&bytes, [SECTION_GRIDS, SECTION_DEXELS])?
    else {
        return Err(BinaryFormatError::InvalidPayload("section.missing"));
    };
    let grids = read_grids(grids)?;
    let dexels = read_stock_dexels(dexels, &grids)?;
    Ok((header, StockState { grids, dexels }))
}
