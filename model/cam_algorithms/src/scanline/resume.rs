//! スキャン加工の区切りの記録（区切りは走査ライン）
//!
//! 走査ラインごとの計算は入力だけから決まる（前のラインの結果を使わない）ため、走査ラインを区切りにする。状態は、次に
//! 計算する走査ラインの位置・範囲を通った走査線の数（ジグザグの送る向き）・パスの点の数の合計とし、成果は、その走査
//! ラインのパス（走査線の向きの座標の点）とする。数は little endian で書く。

use geo_algorithms::Point3D;

use crate::checkpoint::Checkpoints;

/// 状態の頭（スキャン加工の記録であることの印）
const STATE_TAG: &[u8; 4] = b"SCAN";
/// 状態のバイト数（頭と、数 3 つ）
const STATE_LEN: usize = STATE_TAG.len() + 8 * 3;
/// 点 1 つのバイト数
const POINT_LEN: usize = 8 * 3;

/// 走査ラインの続き
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct ScanState {
    /// 次に計算する走査ラインの、走査ラインの並びの位置
    pub next: usize,
    /// 範囲を通った走査線の数（ジグザグの送る向きを決める）
    pub line: usize,
    /// パスの点の数の合計
    pub points: usize,
}

impl ScanState {
    pub(super) fn encode(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(STATE_LEN);
        bytes.extend_from_slice(STATE_TAG);
        for value in [self.next, self.line, self.points] {
            bytes.extend_from_slice(&(value as u64).to_le_bytes());
        }
        bytes
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != STATE_LEN || !bytes.starts_with(STATE_TAG) {
            return None;
        }
        let mut reader = Reader(&bytes[STATE_TAG.len()..]);
        Some(Self {
            next: usize::try_from(reader.u64()?).ok()?,
            line: usize::try_from(reader.u64()?).ok()?,
            points: usize::try_from(reader.u64()?).ok()?,
        })
    }
}

/// 走査ライン 1 本で増えたパス `passes` の成果のバイト列（パスの数と、パスごとの点の数と点）
pub(super) fn encode_passes(passes: &[Vec<Point3D<f64>>]) -> Vec<u8> {
    let points: usize = passes.iter().map(Vec::len).sum();
    let mut bytes = Vec::with_capacity(4 + passes.len() * 4 + points * POINT_LEN);
    bytes.extend_from_slice(&(passes.len() as u32).to_le_bytes());
    for pass in passes {
        bytes.extend_from_slice(&(pass.len() as u32).to_le_bytes());
        for point in pass {
            for value in [point.x(), point.y(), point.z()] {
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
    }
    bytes
}

/// 前の実行の記録 `checkpoints` から、走査ラインの数 `line_count`・点の数の上限 `point_limit` で、パスを `passes`
/// に読み戻し、続きの状態を返す
///
/// 記録を使えなければ、記録を捨て、パスを空にして、最初からの状態を返す。
pub(super) fn restore(
    checkpoints: &mut dyn Checkpoints,
    line_count: usize,
    point_limit: usize,
    passes: &mut Vec<Vec<Point3D<f64>>>,
) -> ScanState {
    match read_saved(checkpoints, line_count, point_limit, passes) {
        Ok(state) => state,
        Err(reason) => {
            passes.clear();
            checkpoints.discard(reason);
            ScanState::default()
        }
    }
}

fn read_saved(
    checkpoints: &mut dyn Checkpoints,
    line_count: usize,
    point_limit: usize,
    passes: &mut Vec<Vec<Point3D<f64>>>,
) -> Result<ScanState, &'static str> {
    let mut state = ScanState::default();
    while let Some(saved) = checkpoints.next_saved() {
        let next =
            ScanState::decode(&saved.state).ok_or("the state of a scanline record is broken")?;
        if next.next <= state.next || next.next > line_count || next.line > next.next {
            return Err("the scanline records are out of order");
        }
        let points = decode_passes(&saved.result, state.points, point_limit, passes)?;
        if points != next.points {
            return Err("the point count of a scanline record does not match its paths");
        }
        state = next;
    }
    Ok(state)
}

/// 成果のバイト列 `bytes` のパスを `passes` に足し、点の数の合計（前の合計 `points` に足したもの）を返す。合計が
/// 上限 `point_limit` を超えるなら、確保する前に誤りにする
fn decode_passes(
    bytes: &[u8],
    mut points: usize,
    point_limit: usize,
    passes: &mut Vec<Vec<Point3D<f64>>>,
) -> Result<usize, &'static str> {
    const BROKEN: &str = "the paths of a scanline record are broken";
    let mut reader = Reader(bytes);
    let count = reader.u32().ok_or(BROKEN)?;
    if count == 0 {
        return Err(BROKEN);
    }
    for _ in 0..count {
        let len = reader.u32().ok_or(BROKEN)? as usize;
        if len == 0 || len > reader.0.len() / POINT_LEN {
            return Err(BROKEN);
        }
        points = points
            .checked_add(len)
            .filter(|&points| points <= point_limit)
            .ok_or("the paths of the scanline records exceed the point limit")?;
        let mut pass = Vec::with_capacity(len);
        for _ in 0..len {
            let (x, y, z) = (reader.f64(), reader.f64(), reader.f64());
            let (Some(x), Some(y), Some(z)) = (x, y, z) else {
                return Err(BROKEN);
            };
            pass.push(Point3D::new(x, y, z));
        }
        passes.push(pass);
    }
    if !reader.0.is_empty() {
        return Err(BROKEN);
    }
    Ok(points)
}

/// バイト列を前から読む
struct Reader<'a>(&'a [u8]);

impl Reader<'_> {
    fn take<const N: usize>(&mut self) -> Option<[u8; N]> {
        let (head, rest) = self.0.split_first_chunk::<N>()?;
        self.0 = rest;
        Some(*head)
    }

    fn u32(&mut self) -> Option<u32> {
        self.take().map(u32::from_le_bytes)
    }

    fn u64(&mut self) -> Option<u64> {
        self.take().map(u64::from_le_bytes)
    }

    /// 有限の値だけを読む
    fn f64(&mut self) -> Option<f64> {
        self.take()
            .map(f64::from_le_bytes)
            .filter(|value| value.is_finite())
    }
}
