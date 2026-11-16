//! エンティティの引数を、型ごとに取り出す（形が合わなければ `None`）

use crate::syntax::{Parameter, Record};

/// エンティティの部分 `records` のうち、名前が `name` のもの（単純なエンティティは、その 1 つ）
pub(super) fn part<'a>(records: &'a [Record], name: &str) -> Option<&'a Record> {
    records.iter().find(|record| record.name == name)
}

/// `index` 番目の引数
pub(super) fn at(record: &Record, index: usize) -> Option<&Parameter> {
    record.parameters.get(index)
}

/// 実数（整数と、型つきの値の中の数も読む）
pub(super) fn real(parameter: &Parameter) -> Option<f64> {
    match parameter {
        Parameter::Real(value) => Some(*value),
        Parameter::Integer(value) => Some(*value as f64),
        Parameter::Typed { value, .. } => real(value),
        _ => None,
    }
}

pub(super) fn integer(parameter: &Parameter) -> Option<i64> {
    match parameter {
        Parameter::Integer(value) => Some(*value),
        _ => None,
    }
}

pub(super) fn reference(parameter: &Parameter) -> Option<u64> {
    match parameter {
        Parameter::Reference(id) => Some(*id),
        _ => None,
    }
}

/// 参照か、なし（`$`）
pub(super) fn optional_reference(parameter: &Parameter) -> Option<Option<u64>> {
    match parameter {
        Parameter::Reference(id) => Some(Some(*id)),
        Parameter::Unset => Some(None),
        _ => None,
    }
}

/// 論理値（`.T.`・`.F.`。`.U.`（分からない）は `None`）
pub(super) fn boolean(parameter: &Parameter) -> Option<bool> {
    match parameter {
        Parameter::Enumeration(value) if value == "T" => Some(true),
        Parameter::Enumeration(value) if value == "F" => Some(false),
        _ => None,
    }
}

pub(super) fn enumeration(parameter: &Parameter) -> Option<&str> {
    match parameter {
        Parameter::Enumeration(value) => Some(value),
        _ => None,
    }
}

pub(super) fn text(parameter: &Parameter) -> Option<&str> {
    match parameter {
        Parameter::String(value) => Some(value),
        _ => None,
    }
}

pub(super) fn list(parameter: &Parameter) -> Option<&[Parameter]> {
    match parameter {
        Parameter::List(items) => Some(items),
        _ => None,
    }
}

pub(super) fn references(parameter: &Parameter) -> Option<Vec<u64>> {
    list(parameter)?.iter().map(reference).collect()
}

pub(super) fn reals(parameter: &Parameter) -> Option<Vec<f64>> {
    list(parameter)?.iter().map(real).collect()
}

pub(super) fn integers(parameter: &Parameter) -> Option<Vec<i64>> {
    list(parameter)?.iter().map(integer).collect()
}
