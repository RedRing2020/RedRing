//! 表現の文脈と、その単位（設計: BREP_AND_STEP_IMPORT_DESIGN.md 9.2）
//!
//! 長さは mm への倍率、角度は度への倍率として持つ。換算の単位は、ファイルに書いた換算の値を使う（度は、計算
//! した π を使わず、度として読む）。換算の連なりは 4 段までとし、循環・壊れた単位・単位がないときは、mm と度と
//! する。

use super::parameters::{at, enumeration, part, real, reference, references, text};
use crate::syntax::{Entity, Record, StepFile};

/// 換算の単位の連なりの上限
const MAX_CONVERSION_DEPTH: usize = 4;

/// ラジアンを度にする倍率（角度の単位がラジアンなら、ちょうどこの値になる）
pub(super) const DEGREES_PER_RADIAN: f64 = 180.0 / std::f64::consts::PI;

/// 表現の文脈
#[derive(Debug, Clone, PartialEq)]
pub struct Context {
    /// 座標の次元（2 か 3。分からなければ `None`）
    pub dimension: Option<i64>,
    /// 長さの単位を mm にする倍率
    pub length_to_mm: f64,
    /// 角度の単位を度にする倍率
    pub angle_to_degrees: f64,
    /// 文脈の許容誤差（mm。書かれていなければ `None`）
    pub uncertainty_mm: Option<f64>,
    /// 長さか角度の単位がない・読めないため、mm か度とした
    pub default_units: bool,
}

/// 番号 `id` の表現の文脈を読む（表現の文脈でなければ `None`）
pub(super) fn context(file: &StepFile, id: u64) -> Option<Context> {
    let entity = file.entities.get(&id)?;
    let geometric = part(&entity.records, "GEOMETRIC_REPRESENTATION_CONTEXT");
    if geometric.is_none() && part(&entity.records, "REPRESENTATION_CONTEXT").is_none() {
        return None;
    }
    let dimension = geometric
        .and_then(|record| at(record, 0))
        .and_then(super::parameters::integer)
        // 単純なエンティティの GEOMETRIC_REPRESENTATION_CONTEXT は、名前・種類の後ろに次元を持つ
        .or_else(|| {
            geometric
                .and_then(|record| at(record, 2))
                .and_then(super::parameters::integer)
        });
    let units = part(&entity.records, "GLOBAL_UNIT_ASSIGNED_CONTEXT")
        .and_then(|record| at(record, 0))
        .and_then(references)
        .unwrap_or_default();
    let mut length = None;
    let mut angle = None;
    for unit in units {
        match unit_kind(file, unit) {
            Some((UnitKind::Length, factor)) => length = length.or(factor),
            Some((UnitKind::PlaneAngle, factor)) => angle = angle.or(factor),
            _ => {}
        }
    }
    let uncertainty_mm = part(&entity.records, "GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT")
        .and_then(|record| at(record, 0))
        .and_then(references)
        .and_then(|uncertainties| {
            uncertainties
                .into_iter()
                .find_map(|uncertainty| uncertainty_mm(file, uncertainty))
        });
    Some(Context {
        dimension,
        length_to_mm: length.unwrap_or(1.0),
        angle_to_degrees: angle.unwrap_or(1.0),
        uncertainty_mm,
        default_units: length.is_none() || angle.is_none(),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnitKind {
    Length,
    PlaneAngle,
    Other,
}

/// 単位の種類と、その倍率（長さは mm へ、角度は度へ。読めなければ `None`）
fn unit_kind(file: &StepFile, id: u64) -> Option<(UnitKind, Option<f64>)> {
    let entity = file.entities.get(&id)?;
    let kind = if part(&entity.records, "LENGTH_UNIT").is_some() {
        UnitKind::Length
    } else if part(&entity.records, "PLANE_ANGLE_UNIT").is_some() {
        UnitKind::PlaneAngle
    } else {
        UnitKind::Other
    };
    Some((kind, unit_factor(file, entity, kind, 0)))
}

/// 単位 `entity` の倍率（長さは mm へ、角度は度へ）
fn unit_factor(file: &StepFile, entity: &Entity, kind: UnitKind, depth: usize) -> Option<f64> {
    if depth > MAX_CONVERSION_DEPTH {
        return None;
    }
    if let Some(si) = part(&entity.records, "SI_UNIT") {
        let prefix = match at(si, 0).and_then(enumeration) {
            Some(prefix) => si_prefix(prefix)?,
            None => 1.0,
        };
        let base = match (kind, at(si, 1).and_then(enumeration)?) {
            (UnitKind::Length, "METRE") => 1000.0,
            (UnitKind::PlaneAngle, "RADIAN") => DEGREES_PER_RADIAN,
            _ => return None,
        };
        return Some(prefix * base);
    }
    let conversion = part(&entity.records, "CONVERSION_BASED_UNIT")?;
    // 度は、計算した π を使わず、度として読む
    if kind == UnitKind::PlaneAngle
        && at(conversion, 0)
            .and_then(text)
            .is_some_and(|name| name.eq_ignore_ascii_case("DEGREE"))
    {
        return Some(1.0);
    }
    let measure = file.entities.get(&at(conversion, 1).and_then(reference)?)?;
    let measure = measure_part(&measure.records)?;
    let value = at(measure, 0).and_then(real)?;
    let base = file.entities.get(&at(measure, 1).and_then(reference)?)?;
    Some(value * unit_factor(file, base, kind, depth + 1)?)
}

/// SI の接頭語の倍率
fn si_prefix(prefix: &str) -> Option<f64> {
    Some(match prefix {
        "EXA" => 1e18,
        "PETA" => 1e15,
        "TERA" => 1e12,
        "GIGA" => 1e9,
        "MEGA" => 1e6,
        "KILO" => 1e3,
        "HECTO" => 1e2,
        "DECA" => 1e1,
        "DECI" => 1e-1,
        "CENTI" => 1e-2,
        "MILLI" => 1e-3,
        "MICRO" => 1e-6,
        "NANO" => 1e-9,
        "PICO" => 1e-12,
        "FEMTO" => 1e-15,
        "ATTO" => 1e-18,
        _ => return None,
    })
}

/// 長さの許容誤差（`UNCERTAINTY_MEASURE_WITH_UNIT`）を mm に直した値
fn uncertainty_mm(file: &StepFile, id: u64) -> Option<f64> {
    let records = &file.entities.get(&id)?.records;
    part(records, "UNCERTAINTY_MEASURE_WITH_UNIT")?;
    let record = measure_part(records)?;
    let value = at(record, 0).and_then(real)?;
    let unit = file.entities.get(&at(record, 1).and_then(reference)?)?;
    part(&unit.records, "LENGTH_UNIT")?;
    Some(value * unit_factor(file, unit, UnitKind::Length, 0)?)
}

/// 値と単位を持つ部分（単純なエンティティは、その部分。複合のエンティティは `MEASURE_WITH_UNIT` の部分。
/// 複合のエンティティでは、`LENGTH_MEASURE_WITH_UNIT()` のような引数のない部分が先に並ぶことがある）
fn measure_part(records: &[Record]) -> Option<&Record> {
    match records {
        [record] => Some(record),
        _ => part(records, "MEASURE_WITH_UNIT"),
    }
}
