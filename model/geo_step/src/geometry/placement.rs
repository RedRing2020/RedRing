//! 表現の間の変換（アセンブリの出現の配置）を、回転と移動の変換にする（設計: BREP_AND_STEP_IMPORT_DESIGN.md 10.4）

use analysis::linalg::matrix::Matrix4x4;
use geo_contracts::ToleranceSettings;
use geo_core::{Point3D, Vector3D};
use geo_primitives::SimilarityTransform3D;

use super::{Converter, Frame, GeometryError};
use crate::record::{Records, Transformation};

/// 表現の間の変換を、回転と移動の変換にできない理由
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementError {
    /// 配置・点・方向の記録がない、または座標系を作れない
    Geometry(GeometryError),
    /// 回転と移動でない（倍率が 1 でない・鏡映）
    NotRigid,
}

/// 変換 `transformation` の、関係の 1 つ目の表現の座標から 2 つ目の表現の座標への変換（`reversed` なら、
/// 2 つ目から 1 つ目への変換）
///
/// `ITEM_DEFINED_TRANSFORMATION` は、1 つ目の配置の座標系を 2 つ目の配置の座標系に重ねる変換とする。
/// `CARTESIAN_TRANSFORMATION_OPERATOR_3D` は、ISO 10303-42 の `base_axis` で軸を決めた変換とする（軸の決め方
/// と、座標系を作れないとみなす角度は、配置の座標系と同じ）。
///
/// # Errors
///
/// 配置・点・方向を読めない・座標系を作れないとき、倍率が 1 でない・鏡映のとき。
pub fn rigid_transformation(
    records: &Records,
    transformation: &Transformation,
    reversed: bool,
    tolerances: &ToleranceSettings<f64>,
) -> Result<SimilarityTransform3D<f64>, PlacementError> {
    let converter = Converter::new(records, tolerances);
    let geometry = PlacementError::Geometry;
    let (first, second) = match *transformation {
        Transformation::Placements { item_1, item_2 } => (
            converter.frame(item_1).map_err(geometry)?,
            converter.frame(item_2).map_err(geometry)?,
        ),
        Transformation::Operator {
            axis_1,
            axis_2,
            axis_3,
            origin,
            scale,
        } => {
            if scale.is_some_and(|scale| (scale - 1.0).abs() > tolerances.angle_tolerance) {
                return Err(PlacementError::NotRigid);
            }
            let identity = Frame {
                origin: Point3D::new(0.0, 0.0, 0.0),
                ..converter
                    .axes(None, None, Point3D::new(0.0, 0.0, 0.0))
                    .map_err(geometry)?
            };
            let origin = converter.point(origin).map_err(geometry)?;
            let operator = converter
                .operator_axes([axis_1, axis_2, axis_3], origin)
                .map_err(geometry)?;
            (identity, operator)
        }
    };
    let (from, to) = if reversed {
        (second, first)
    } else {
        (first, second)
    };
    SimilarityTransform3D::from_matrix(matrix(&to) * inverse(&from))
        .map_err(|_| PlacementError::NotRigid)
}

impl Converter<'_> {
    /// `CARTESIAN_TRANSFORMATION_OPERATOR_3D` の座標系（軸 3 がなければ Z、軸 1 は配置の基準の方向と同じに決め、
    /// 軸 2 がなければ 軸 3 × 軸 1。軸 2 から軸 3・軸 1 の成分を除く。右手系でなければ鏡映）
    fn operator_axes(
        &self,
        [axis_1, axis_2, axis_3]: [Option<u64>; 3],
        origin: Point3D<f64>,
    ) -> Result<Frame, GeometryError> {
        let base = self.axes(axis_3, axis_1, origin)?;
        let Some(axis_2) = axis_2 else {
            return Ok(base);
        };
        let second = self.direction(axis_2)?.as_vector();
        let (x, z) = (base.x.as_vector(), base.z.as_vector());
        let projected = second - z * second.dot(z) - x * second.dot(x);
        if projected.length() <= self.tolerances.angle_tolerance {
            return Err(GeometryError::DegeneratePlacement);
        }
        let y = geo_core::Direction3D::from_vector(projected)
            .ok_or(GeometryError::DegeneratePlacement)?;
        Ok(Frame { y, ..base })
    }
}

/// 座標系 `frame` の、局所の座標から外の座標への行列（軸が左手系なら、鏡映の行列）
fn matrix(frame: &Frame) -> Matrix4x4<f64> {
    let (x, y, z) = (
        frame.x.as_vector(),
        frame.y.as_vector(),
        frame.z.as_vector(),
    );
    let o = Vector3D::new(frame.origin.x(), frame.origin.y(), frame.origin.z());
    Matrix4x4::new(
        x.x(),
        y.x(),
        z.x(),
        o.x(),
        x.y(),
        y.y(),
        z.y(),
        o.y(),
        x.z(),
        y.z(),
        z.z(),
        o.z(),
        0.0,
        0.0,
        0.0,
        1.0,
    )
}

/// 座標系 `frame` の、外の座標から局所の座標への行列（軸が正規直交なので、転置で逆にする）
fn inverse(frame: &Frame) -> Matrix4x4<f64> {
    let (x, y, z) = (
        frame.x.as_vector(),
        frame.y.as_vector(),
        frame.z.as_vector(),
    );
    let o = Vector3D::new(frame.origin.x(), frame.origin.y(), frame.origin.z());
    Matrix4x4::new(
        x.x(),
        x.y(),
        x.z(),
        -x.dot(o),
        y.x(),
        y.y(),
        y.z(),
        -y.dot(o),
        z.x(),
        z.y(),
        z.z(),
        -z.dot(o),
        0.0,
        0.0,
        0.0,
        1.0,
    )
}
