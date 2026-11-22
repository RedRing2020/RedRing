use cam_core::{ArtifactHeaderV1, ArtifactKind};
use geo_algorithms::{Aabb3D, Point3D};

use super::*;
use crate::dexel_normal::DexelNormal;
use crate::tri_dexel::{grid_layouts, initial_normals};

fn state() -> StockState {
    let stock = Aabb3D::new(Point3D::new(0.0, 0.0, 0.0), Point3D::new(4.0, 3.0, 2.0));
    let mut dexels = StockDexels::default();
    let cut = [
        initial_normals(2)[0],
        DexelNormal::from_unit([0.0, 0.6, 0.8]),
    ];
    dexels.push(0, 1, &[(0.0, 1.5), (2.5, 4.0)], &[cut, cut]);
    dexels.push(2, 0, &[(0.0, 1.25)], &[cut]);
    dexels.push(2, 3, &[], &[]);
    StockState {
        grids: grid_layouts(&stock, 1.0),
        dexels,
    }
}

fn bytes(state: &StockState) -> Vec<u8> {
    let mut bytes = Vec::new();
    write_stock_state_v1(
        &mut bytes,
        state,
        LengthUnit::Millimeter,
        CoordinateFrame::WorldRightHandedZUp,
    )
    .unwrap();
    bytes
}

/// 中身 `payload` に、素材の共通のヘッダを付けたバイト列
fn with_header(payload: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    ArtifactHeaderV1::new(ArtifactKind::Stock, payload.len() as u64)
        .write_to(&mut bytes)
        .unwrap();
    bytes.extend_from_slice(payload);
    bytes
}

#[test]
fn stock_states_round_trip() {
    let state = state();
    let written = bytes(&state);
    let (header, read) = read_stock_state_v1(&mut written.as_slice()).unwrap();
    assert_eq!(header.kind, ArtifactKind::Stock);
    assert_eq!(read, state);
    let empty = StockState {
        dexels: StockDexels::default(),
        ..state
    };
    let (_, read) = read_stock_state_v1(&mut bytes(&empty).as_slice()).unwrap();
    assert_eq!(read, empty);
}

#[test]
fn dexels_out_of_order_or_outside_the_grid_are_rejected() {
    let state = state();
    let file = |entries: &[(u8, u32)]| {
        let mut section = (entries.len() as u32).to_le_bytes().to_vec();
        for &(grid, index) in entries {
            section.push(grid);
            section.extend_from_slice(&index.to_le_bytes());
            section.extend_from_slice(&1_u32.to_le_bytes());
            for value in [0.0_f64, 1.0] {
                section.extend_from_slice(&value.to_le_bytes());
            }
            section.extend_from_slice(&[0; 8]);
        }
        let mut payload = Vec::new();
        write_section(&mut payload, SECTION_GRIDS, &grids_bytes(&state.grids));
        write_section(&mut payload, SECTION_DEXELS, &section);
        with_header(&payload)
    };
    assert!(read_stock_state_v1(&mut file(&[(0, 1), (1, 0)]).as_slice()).is_ok());
    for entries in [
        &[(1, 0), (0, 1)][..],
        &[(0, 1), (0, 1)],
        &[(0, 99)],
        &[(3, 0)],
    ] {
        assert!(
            matches!(
                read_stock_state_v1(&mut file(entries).as_slice()),
                Err(BinaryFormatError::InvalidPayload(_))
            ),
            "{entries:?}"
        );
    }
}

#[test]
fn intervals_out_of_order_or_outside_the_stock_are_rejected() {
    let normals = [initial_normals(0); 2];
    for intervals in [
        &[(1.0, 0.5)][..],
        &[(0.0, 1.5), (1.0, 2.0)],
        &[(-0.5, 1.0)],
        &[(0.0, 4.5)],
        &[(f64::NAN, 1.0)],
    ] {
        let mut dexels = StockDexels::default();
        dexels.push(0, 0, intervals, &normals[..intervals.len()]);
        let written = bytes(&StockState { dexels, ..state() });
        assert!(
            matches!(
                read_stock_state_v1(&mut written.as_slice()),
                Err(BinaryFormatError::InvalidPayload("stock.intervals"))
            ),
            "{intervals:?}"
        );
    }
}

#[test]
fn other_kinds_missing_sections_and_cut_off_files_are_rejected() {
    let bytes = bytes(&state());
    let mut other = bytes.clone();
    other[..4].copy_from_slice(b"RRCS");
    assert!(matches!(
        read_stock_state_v1(&mut other.as_slice()),
        Err(BinaryFormatError::ArtifactKindMismatch { .. })
    ));
    let mut grids_only = Vec::new();
    write_section(&mut grids_only, SECTION_GRIDS, &grids_bytes(&state().grids));
    assert!(matches!(
        read_stock_state_v1(&mut with_header(&grids_only).as_slice()),
        Err(BinaryFormatError::InvalidPayload("section.missing"))
    ));
    for length in 0..bytes.len() {
        assert!(read_stock_state_v1(&mut &bytes[..length]).is_err());
    }
}
