//! 中間データの製品構造（設計: BREP_AND_STEP_IMPORT_DESIGN.md 9.2・10.2・10.4）
//!
//! 製品・製品と形状の表現のつながり・アセンブリの出現と配置・名前・色・非表示を、ファイルの値のまま記録する。
//! 配置は、それぞれの表現の文脈の単位で、幾何の記録（配置・点・方向）として読む。計算はしない（段 2 の決まり）。
//! 製品の文脈の書き方の違い（AP203 の第 1 版の `DESIGN_CONTEXT`・AP214 の `PRODUCT_DEFINITION_CONTEXT` など）
//! は、文脈を読まないので、どれも受け付ける。

use super::parameters::{at, list, optional_reference, part, real, reference, references, text};
use super::{Reader, SkipReason};
use crate::progress::Cancelled;
use crate::syntax::{Parameter, Record};
use std::collections::{BTreeMap, BTreeSet, HashSet};

/// 色を探すときに、参照をたどる深さの上限（スタイルの割り当て → 面のスタイル → 塗りつぶし → 色）
const MAX_STYLE_DEPTH: usize = 8;

/// 製品（`PRODUCT_DEFINITION` ごと）
#[derive(Debug, Clone, PartialEq)]
pub struct Product {
    /// 製品の識別子（`PRODUCT` の `id`）
    pub id: String,
    /// 製品の名前（`PRODUCT` の `name`）
    pub name: String,
    /// 製品の形状の表現（`SHAPE_DEFINITION_REPRESENTATION` で結ばれたもの。ファイルの順）
    pub representations: Vec<u64>,
}

/// アセンブリの出現（`NEXT_ASSEMBLY_USAGE_OCCURRENCE`）
#[derive(Debug, Clone, PartialEq)]
pub struct Occurrence {
    /// 出現の識別子（`id`）
    pub id: String,
    pub name: String,
    /// 親の製品（アセンブリ）と、子の製品（部品か、下のアセンブリ）の `PRODUCT_DEFINITION`
    pub parent: u64,
    pub child: u64,
    /// 配置（`CONTEXT_DEPENDENT_SHAPE_REPRESENTATION` がなければ `None`）
    pub placement: Option<OccurrencePlacement>,
}

/// 出現の配置（`REPRESENTATION_RELATIONSHIP_WITH_TRANSFORMATION`。ファイルの値のまま）
#[derive(Debug, Clone, PartialEq)]
pub struct OccurrencePlacement {
    /// 関係の 1 つ目と 2 つ目の表現（`rep_1`・`rep_2`）
    pub representation_1: u64,
    pub representation_2: u64,
    pub transformation: Transformation,
}

/// 表現の間の変換
#[derive(Debug, Clone, PartialEq)]
pub enum Transformation {
    /// 2 つの配置（`ITEM_DEFINED_TRANSFORMATION` の `transform_item_1`・`transform_item_2`）。1 つ目は
    /// `representation_1` の文脈、2 つ目は `representation_2` の文脈の単位で読んだ配置の記録
    Placements { item_1: u64, item_2: u64 },
    /// `CARTESIAN_TRANSFORMATION_OPERATOR_3D`（方向・原点の記録と、倍率。原点は `representation_2` の
    /// 文脈の単位で読む）
    Operator {
        axis_1: Option<u64>,
        axis_2: Option<u64>,
        axis_3: Option<u64>,
        origin: u64,
        scale: Option<f64>,
    },
}

/// 色（0 から 1 の RGB。ファイルの値のまま）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub red: f64,
    pub green: f64,
    pub blue: f64,
}

/// 製品の定義・出現から、形状の表現と配置への索引（ファイルを 1 回たどって作る）
#[derive(Default)]
struct Index {
    /// 製品の定義から、`SHAPE_DEFINITION_REPRESENTATION` で結ばれた形状の表現
    representations: BTreeMap<u64, Vec<u64>>,
    /// 出現から、`CONTEXT_DEPENDENT_SHAPE_REPRESENTATION` の番号と、その変換つきの表現の関係
    placements: BTreeMap<u64, (u64, u64)>,
}

/// 色を探して見つけたもの（面の塗りつぶしの下で見つけたか）
struct Found {
    color: Color,
    surface: bool,
}

/// 色の優先の順（上書きのスタイルの割り当て、面の塗りつぶしの色の順に優先する）
type ColorPriority = (bool, bool);

impl Reader<'_, '_> {
    /// 製品構造を読む（形状の表現を読んだ後に呼ぶ）
    pub(super) fn product_structure(&mut self) -> Result<(), Cancelled> {
        let file = self.file;
        let index = self.index()?;
        let mut priorities: BTreeMap<u64, ColorPriority> = BTreeMap::new();
        let mut transformed = Vec::new();
        // 出現の配置として読んだ変換つきの関係
        let mut used = BTreeSet::new();
        for (&id, entity) in &file.entities {
            self.steps.step()?;
            match entity.records.as_slice() {
                [record] => self.product_record(id, record, &index, &mut priorities, &mut used),
                records => {
                    if self.product_complex(id, records) {
                        transformed.push(id);
                    }
                }
            }
        }
        // 出現の配置として読まなかった変換つきの関係は、飛ばしたと記録する
        for relation in transformed {
            if !used.contains(&relation) {
                self.skip(relation, SkipReason::Unsupported);
            }
        }
        let names: Vec<(u64, String)> = self
            .records
            .representations
            .values()
            .flat_map(|representation| representation.items.iter().copied())
            .filter_map(|item| {
                let record = file.entities.get(&item)?.records.first()?;
                Some((item, at(record, 0).and_then(text)?.to_string()))
            })
            .collect();
        self.records.item_names.extend(names);
        Ok(())
    }

    /// 製品の定義・出現から、形状の表現と配置への索引を作る（ファイルを 1 回たどり、後で突き合わせる）
    fn index(&mut self) -> Result<Index, Cancelled> {
        let file = self.file;
        // 製品の定義の形状から、その定義（製品の定義か出現）
        let mut definitions: BTreeMap<u64, u64> = BTreeMap::new();
        // 形状の定義と表現の組（製品の定義の形状、形状の表現）
        let mut shape_representations = Vec::new();
        // 文脈による形状の表現（番号、変換つきの関係、製品の定義の形状）
        let mut dependent = Vec::new();
        for (&id, entity) in &file.entities {
            self.steps.step()?;
            let [record] = entity.records.as_slice() else {
                continue;
            };
            let reference_at = |index: usize| at(record, index).and_then(reference);
            match record.name.as_str() {
                "PRODUCT_DEFINITION_SHAPE" => match reference_at(2) {
                    Some(definition) => {
                        definitions.insert(id, definition);
                    }
                    None => self.skip(id, SkipReason::Malformed),
                },
                "SHAPE_DEFINITION_REPRESENTATION" => match (reference_at(0), reference_at(1)) {
                    (Some(shape), Some(representation)) => {
                        shape_representations.push((shape, representation))
                    }
                    _ => self.skip(id, SkipReason::Malformed),
                },
                "CONTEXT_DEPENDENT_SHAPE_REPRESENTATION" => {
                    match (reference_at(0), reference_at(1)) {
                        (Some(relation), Some(shape)) => dependent.push((id, relation, shape)),
                        _ => self.skip(id, SkipReason::Malformed),
                    }
                }
                _ => {}
            }
        }
        let mut index = Index::default();
        for (shape, representation) in shape_representations {
            if let Some(&definition) = definitions.get(&shape) {
                index
                    .representations
                    .entry(definition)
                    .or_default()
                    .push(representation);
            }
        }
        for (id, relation, shape) in dependent {
            match definitions.get(&shape) {
                Some(&occurrence) => {
                    index.placements.entry(occurrence).or_insert((id, relation));
                }
                None => self.skip(id, SkipReason::MissingReference),
            }
        }
        Ok(index)
    }

    /// 単純なエンティティのうち、製品構造のもの
    fn product_record(
        &mut self,
        id: u64,
        record: &Record,
        index: &Index,
        priorities: &mut BTreeMap<u64, ColorPriority>,
        used: &mut BTreeSet<u64>,
    ) {
        let file = self.file;
        let malformed = |reader: &mut Self| reader.skip(id, SkipReason::Malformed);
        match record.name.as_str() {
            "PRODUCT_DEFINITION" | "PRODUCT_DEFINITION_WITH_ASSOCIATED_DOCUMENTS" => {
                // 製品の定義 → 版（`PRODUCT_DEFINITION_FORMATION` など） → 製品
                let product = at(record, 2)
                    .and_then(reference)
                    .and_then(|formation| file.entities.get(&formation))
                    .and_then(|formation| formation.records.first())
                    .and_then(|formation| at(formation, 2).and_then(reference))
                    .and_then(|product| file.entities.get(&product))
                    .and_then(|product| product.records.first())
                    .filter(|product| product.name == "PRODUCT");
                let Some(product) = product else {
                    malformed(self);
                    return;
                };
                let representations = index.representations.get(&id).cloned().unwrap_or_default();
                self.records.products.insert(
                    id,
                    Product {
                        id: string(product, 0),
                        name: string(product, 1),
                        representations,
                    },
                );
            }
            "NEXT_ASSEMBLY_USAGE_OCCURRENCE" => {
                let (Some(parent), Some(child)) = (
                    at(record, 3).and_then(reference),
                    at(record, 4).and_then(reference),
                ) else {
                    malformed(self);
                    return;
                };
                let placement = index.placements.get(&id).and_then(|&(cdsr, relation)| {
                    // 読めなかったときは、文脈による形状の表現に理由を残す（関係は、読んだ側に数える）
                    used.insert(relation);
                    match self.transformation(relation) {
                        Ok(placement) => Some(placement),
                        Err(reason) => {
                            self.skip(cdsr, reason);
                            None
                        }
                    }
                });
                self.records.occurrences.insert(
                    id,
                    Occurrence {
                        id: string(record, 0),
                        name: string(record, 1),
                        parent,
                        child,
                        placement,
                    },
                );
            }
            "SHAPE_REPRESENTATION_RELATIONSHIP" => {
                match (
                    at(record, 2).and_then(reference),
                    at(record, 3).and_then(reference),
                ) {
                    (Some(first), Some(second)) => {
                        self.records.representation_links.push((first, second))
                    }
                    _ => malformed(self),
                }
            }
            name @ ("STYLED_ITEM" | "OVER_RIDING_STYLED_ITEM") => {
                let (Some(styles), Some(item)) = (
                    at(record, 1).and_then(references),
                    at(record, 2).and_then(reference),
                ) else {
                    malformed(self);
                    return;
                };
                let mut found = Vec::new();
                let mut visited = HashSet::new();
                for style in styles {
                    self.find_colors(style, 0, false, &mut visited, &mut found);
                }
                let chosen = found
                    .iter()
                    .find(|found| found.surface)
                    .or_else(|| found.first());
                if let Some(chosen) = chosen {
                    // 同じ要素に複数の割り当てがあれば、上書きの割り当て、面の塗りつぶしの色の順に優先し、
                    // 同じ順なら初めのものを使う
                    let priority = (name == "OVER_RIDING_STYLED_ITEM", chosen.surface);
                    if priorities
                        .get(&item)
                        .is_none_or(|&current| priority > current)
                    {
                        priorities.insert(item, priority);
                        self.records.colors.insert(item, chosen.color);
                    }
                }
            }
            "INVISIBILITY" => {
                let Some(items) = at(record, 0).and_then(references) else {
                    malformed(self);
                    return;
                };
                for item in items {
                    let targets = self.invisible_targets(item);
                    self.records.invisible.extend(targets);
                }
            }
            _ => {}
        }
    }

    /// 非表示の要素 `item` の対象（スタイルの割り当てはその対象、レイヤーはそこに割り当てた要素）
    fn invisible_targets(&self, item: u64) -> Vec<u64> {
        let styled_target = |id: u64| {
            self.file
                .entities
                .get(&id)
                .and_then(|entity| entity.records.first())
                .filter(|record| {
                    matches!(
                        record.name.as_str(),
                        "STYLED_ITEM" | "OVER_RIDING_STYLED_ITEM"
                    )
                })
                .and_then(|record| at(record, 2).and_then(reference))
                .unwrap_or(id)
        };
        let layer = self
            .file
            .entities
            .get(&item)
            .and_then(|entity| entity.records.first())
            .filter(|record| record.name == "PRESENTATION_LAYER_ASSIGNMENT")
            .and_then(|record| at(record, 2).and_then(references));
        match layer {
            Some(assigned) => assigned.into_iter().map(styled_target).collect(),
            None => vec![styled_target(item)],
        }
    }

    /// 複合のエンティティのうち、製品構造のもの（変換のない形状の表現の間の関係を記録する）。変換つきの関係か
    fn product_complex(&mut self, id: u64, records: &[Record]) -> bool {
        let Some(relation) = part(records, "REPRESENTATION_RELATIONSHIP") else {
            return false;
        };
        if part(records, "REPRESENTATION_RELATIONSHIP_WITH_TRANSFORMATION").is_some() {
            return true;
        }
        if part(records, "SHAPE_REPRESENTATION_RELATIONSHIP").is_none() {
            return false;
        }
        match (
            at(relation, 2).and_then(reference),
            at(relation, 3).and_then(reference),
        ) {
            (Some(first), Some(second)) => self.records.representation_links.push((first, second)),
            _ => self.skip(id, SkipReason::Malformed),
        }
        false
    }

    /// 変換つきの表現の関係 `relation` の配置（変換の配置・点・方向は、表現の文脈の単位で読む）
    fn transformation(&mut self, relation: u64) -> Result<OccurrencePlacement, SkipReason> {
        let file = self.file;
        let records = &file
            .entities
            .get(&relation)
            .ok_or(SkipReason::MissingReference)?
            .records;
        let base = part(records, "REPRESENTATION_RELATIONSHIP").ok_or(SkipReason::Malformed)?;
        let with = part(records, "REPRESENTATION_RELATIONSHIP_WITH_TRANSFORMATION")
            .ok_or(SkipReason::Malformed)?;
        let reference_at = |record: &Record, index: usize| {
            at(record, index)
                .and_then(reference)
                .ok_or(SkipReason::Malformed)
        };
        let representation_1 = reference_at(base, 2)?;
        let representation_2 = reference_at(base, 3)?;
        let transformation = reference_at(with, 0)?;
        let record = file
            .entities
            .get(&transformation)
            .and_then(|entity| entity.records.first())
            .ok_or(SkipReason::MissingReference)?;
        let transformation = match record.name.as_str() {
            "ITEM_DEFINED_TRANSFORMATION" => {
                let item_1 = reference_at(record, 2)?;
                let item_2 = reference_at(record, 3)?;
                let (length_1, angle_1) = self.representation_units(representation_1)?;
                let (length_2, angle_2) = self.representation_units(representation_2)?;
                self.enqueue(item_1, length_1, angle_1);
                self.enqueue(item_2, length_2, angle_2);
                Transformation::Placements { item_1, item_2 }
            }
            "CARTESIAN_TRANSFORMATION_OPERATOR_3D" => {
                // 終わりの 5 つの引数が、軸 1・軸 2・原点・倍率・軸 3（前の名前と説明の数は、ファイルで違う）
                let offset = record
                    .parameters
                    .len()
                    .checked_sub(5)
                    .ok_or(SkipReason::Malformed)?;
                let optional = |index: usize| {
                    at(record, offset + index)
                        .and_then(optional_reference)
                        .ok_or(SkipReason::Malformed)
                };
                let (axis_1, axis_2, axis_3) = (optional(0)?, optional(1)?, optional(4)?);
                let origin = reference_at(record, offset + 2)?;
                let scale = at(record, offset + 3).and_then(real);
                let (length, angle) = self.representation_units(representation_2)?;
                for axis in [axis_1, axis_2, axis_3].into_iter().flatten() {
                    self.enqueue(axis, length, angle);
                }
                self.enqueue(origin, length, angle);
                Transformation::Operator {
                    axis_1,
                    axis_2,
                    axis_3,
                    origin,
                    scale,
                }
            }
            _ => return Err(SkipReason::Unsupported),
        };
        self.drain_uncounted();
        Ok(OccurrencePlacement {
            representation_1,
            representation_2,
            transformation,
        })
    }

    /// 表現 `representation` の文脈の、長さと角度の倍率
    fn representation_units(&mut self, representation: u64) -> Result<(f64, f64), SkipReason> {
        let record = self
            .file
            .entities
            .get(&representation)
            .and_then(|entity| entity.records.first())
            .ok_or(SkipReason::MissingReference)?;
        let context = at(record, 2)
            .and_then(reference)
            .ok_or(SkipReason::Malformed)?;
        let context = self.context(context)?;
        Ok((context.length_to_mm, context.angle_to_degrees))
    }

    /// スタイル `id` から参照をたどり、色を集める（`surface` は、面の塗りつぶしの下か。同じ要素は 1 回だけ
    /// たどる）
    fn find_colors(
        &self,
        id: u64,
        depth: usize,
        surface: bool,
        visited: &mut HashSet<u64>,
        found: &mut Vec<Found>,
    ) {
        if depth > MAX_STYLE_DEPTH || !visited.insert(id) {
            return;
        }
        let Some(record) = self
            .file
            .entities
            .get(&id)
            .and_then(|entity| entity.records.first())
        else {
            return;
        };
        match record.name.as_str() {
            "COLOUR_RGB" => {
                if let (Some(red), Some(green), Some(blue)) = (
                    at(record, 1).and_then(real),
                    at(record, 2).and_then(real),
                    at(record, 3).and_then(real),
                ) {
                    found.push(Found {
                        color: Color { red, green, blue },
                        surface,
                    });
                }
            }
            "DRAUGHTING_PRE_DEFINED_COLOUR" => {
                if let Some(color) = at(record, 0).and_then(text).and_then(predefined_color) {
                    found.push(Found { color, surface });
                }
            }
            name => {
                // 面の境界・輪郭の線の色は、面の色にしない
                let surface = surface || name == "SURFACE_STYLE_FILL_AREA";
                let mut targets = Vec::new();
                for parameter in &record.parameters {
                    collect_references(parameter, &mut targets);
                }
                for target in targets {
                    self.find_colors(target, depth + 1, surface, visited, found);
                }
            }
        }
    }
}

/// 引数 `index` の文字列（なければ空）
fn string(record: &Record, index: usize) -> String {
    at(record, index)
        .and_then(text)
        .unwrap_or_default()
        .to_string()
}

/// 引数の中の参照（リストの中も）
fn collect_references(parameter: &Parameter, targets: &mut Vec<u64>) {
    match parameter {
        Parameter::Reference(id) => targets.push(*id),
        Parameter::List(_) => {
            for item in list(parameter).unwrap_or_default() {
                collect_references(item, targets);
            }
        }
        Parameter::Typed { value, .. } => collect_references(value, targets),
        _ => {}
    }
}

/// 規格で決まった名前の色（`DRAUGHTING_PRE_DEFINED_COLOUR`）
fn predefined_color(name: &str) -> Option<Color> {
    let (red, green, blue) = match name {
        "red" => (1.0, 0.0, 0.0),
        "green" => (0.0, 1.0, 0.0),
        "blue" => (0.0, 0.0, 1.0),
        "yellow" => (1.0, 1.0, 0.0),
        "magenta" => (1.0, 0.0, 1.0),
        "cyan" => (0.0, 1.0, 1.0),
        "black" => (0.0, 0.0, 0.0),
        "white" => (1.0, 1.0, 1.0),
        _ => return None,
    };
    Some(Color { red, green, blue })
}
