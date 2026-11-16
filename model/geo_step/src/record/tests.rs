use super::*;
use crate::syntax::{Limits, parse};

/// エンティティの番号の印 `#` は `%` で書き、`step_file` で直して使う（ソースコードの検査で、別の番号と見分け
/// られないため）
fn step_file(data: &str) -> StepFile {
    let text = format!(
        "ISO-10303-21;\nHEADER;\nFILE_SCHEMA(('AUTOMOTIVE_DESIGN'));\nENDSEC;\nDATA;\n{}\nENDSEC;\nEND-ISO-10303-21;\n",
        data.replace('%', "#")
    );
    parse(text.as_bytes(), &Limits::default()).unwrap()
}

/// mm とラジアンの文脈（%5）と、平面と円錐の 2 つの面の立体（%27）の表現（%28）
const MM_RADIAN: &str = "
%1=(LENGTH_UNIT() NAMED_UNIT(*) SI_UNIT(.MILLI.,.METRE.));
%2=(NAMED_UNIT(*) PLANE_ANGLE_UNIT() SI_UNIT($,.RADIAN.));
%3=(NAMED_UNIT(*) SI_UNIT($,.STERADIAN.) SOLID_ANGLE_UNIT());
%4=UNCERTAINTY_MEASURE_WITH_UNIT(LENGTH_MEASURE(1.E-07),%1,'distance_accuracy_value','');
%5=(GEOMETRIC_REPRESENTATION_CONTEXT(3) GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT((%4)) GLOBAL_UNIT_ASSIGNED_CONTEXT((%1,%2,%3)) REPRESENTATION_CONTEXT('',''));
";

const SOLID: &str = "
%10=CARTESIAN_POINT('',(0.,0.,0.));
%11=DIRECTION('',(0.,0.,1.));
%12=DIRECTION('',(1.,0.,0.));
%13=AXIS2_PLACEMENT_3D('',%10,%11,%12);
%14=CIRCLE('',%13,10.);
%15=CARTESIAN_POINT('',(10.,0.,0.));
%16=VERTEX_POINT('',%15);
%17=EDGE_CURVE('',%16,%16,%14,.T.);
%18=ORIENTED_EDGE('',*,*,%17,.T.);
%19=EDGE_LOOP('',(%18));
%20=FACE_OUTER_BOUND('',%19,.T.);
%21=PLANE('',%13);
%22=ADVANCED_FACE('',(%20),%21,.T.);
%23=CONICAL_SURFACE('',%13,10.,ANGLE);
%25=FACE_BOUND('',%19,.F.);
%24=ADVANCED_FACE('',(%25),%23,.F.);
%26=CLOSED_SHELL('',(%22,%24));
%27=MANIFOLD_SOLID_BREP('solid',%26);
%28=ADVANCED_BREP_SHAPE_REPRESENTATION('part',(%27,%13),%5);
";

/// インチと度の文脈（%5）
const INCH_DEGREE: &str = "
%1=(CONVERSION_BASED_UNIT('INCH',%6) LENGTH_UNIT() NAMED_UNIT(%7));
%6=LENGTH_MEASURE_WITH_UNIT(LENGTH_MEASURE(25.4),%8);
%8=(LENGTH_UNIT() NAMED_UNIT(*) SI_UNIT(.MILLI.,.METRE.));
%7=DIMENSIONAL_EXPONENTS(1.,0.,0.,0.,0.,0.,0.);
%2=(CONVERSION_BASED_UNIT('DEGREE',%9) NAMED_UNIT(%7) PLANE_ANGLE_UNIT());
%9=PLANE_ANGLE_MEASURE_WITH_UNIT(PLANE_ANGLE_MEASURE(0.0174532925),%30);
%30=(NAMED_UNIT(*) PLANE_ANGLE_UNIT() SI_UNIT($,.RADIAN.));
%4=UNCERTAINTY_MEASURE_WITH_UNIT(LENGTH_MEASURE(1.E-05),%1,'distance_accuracy_value','');
%5=(GEOMETRIC_REPRESENTATION_CONTEXT(3) GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT((%4)) GLOBAL_UNIT_ASSIGNED_CONTEXT((%1,%2)) REPRESENTATION_CONTEXT('',''));
";

fn solid_in(context: &str, angle: &str) -> Records {
    read(&step_file(
        &(context.to_string() + &SOLID.replace("ANGLE", angle)),
    ))
}

#[test]
fn a_solid_becomes_geometry_and_topology_records() {
    let records = solid_in(MM_RADIAN, "0.5");

    assert_eq!(records.schemas, vec!["AUTOMOTIVE_DESIGN"]);
    let representation = &records.representations[&28];
    assert_eq!(representation.name, "part");
    assert_eq!(representation.context, 5);
    // 配置（%13）は製品構造で使う要素として、記録も飛ばしもしない
    assert_eq!(representation.items, vec![27]);
    assert_eq!(
        records.topology[&27],
        Topology::Solid {
            outer: 26,
            voids: vec![]
        }
    );
    assert_eq!(
        records.topology[&26],
        Topology::Shell {
            faces: vec![22, 24],
            closed: true
        }
    );
    assert_eq!(
        records.topology[&24],
        Topology::Face {
            bounds: vec![25],
            surface: Some(23),
            same_sense: Some(false)
        }
    );
    assert_eq!(
        records.topology[&20],
        Topology::FaceBound {
            bound: 19,
            orientation: Some(true),
            outer: true
        }
    );
    assert_eq!(
        records.topology[&17],
        Topology::Edge {
            start: 16,
            end: 16,
            curve: 14,
            same_sense: Some(true)
        }
    );
    // 配置はファイルの値のまま
    assert_eq!(
        records.geometry[&13],
        Geometry::Placement {
            location: 10,
            axis: Some(11),
            ref_direction: Some(12)
        }
    );
    assert_eq!(
        records.geometry[&14],
        Geometry::Circle {
            placement: 13,
            radius: 10.0
        }
    );
    assert!(records.skipped.is_empty(), "{:?}", records.skipped);
}

#[test]
fn units_are_read_from_each_context() {
    let mm = solid_in(MM_RADIAN, "0.5");
    let context = &mm.contexts[&5];
    assert_eq!(context.dimension, Some(3));
    assert_eq!(context.length_to_mm, 1.0);
    assert!((context.angle_to_degrees - 180.0 / std::f64::consts::PI).abs() < 1.0e-12);
    assert_eq!(context.uncertainty_mm, Some(1.0e-7));
    assert!(!context.default_units);
    // ラジアンの文脈の角度は、ラジアンのまま作る
    let Geometry::ConicalSurface { semi_angle, .. } = mm.geometry[&23] else {
        panic!("a conical surface");
    };
    assert_eq!(semi_angle.to_radians(), 0.5);

    // インチは、ファイルに書いた換算の値（25.4 mm）で mm に直す。度は度として読む
    let inch = solid_in(INCH_DEGREE, "45.");
    let context = &inch.contexts[&5];
    assert_eq!(context.length_to_mm, 25.4);
    assert_eq!(context.angle_to_degrees, 1.0);
    assert!((context.uncertainty_mm.unwrap() - 25.4e-5).abs() < 1.0e-15);
    assert_eq!(inch.geometry[&15], Geometry::Point([254.0, 0.0, 0.0]));
    assert_eq!(
        inch.geometry[&14],
        Geometry::Circle {
            placement: 13,
            radius: 254.0
        }
    );
    let Geometry::ConicalSurface { semi_angle, .. } = inch.geometry[&23] else {
        panic!("a conical surface");
    };
    assert_eq!(semi_angle.to_degrees(), 45.0);
}

#[test]
fn contexts_with_different_units_are_read_in_one_file() {
    // mm の文脈（%5）の点と、インチの文脈（%105）の点
    let inch = INCH_DEGREE.replace('%', "%10");
    let data = format!(
        "{MM_RADIAN}{inch}
%40=CARTESIAN_POINT('',(1.,2.,3.));
%41=SHAPE_REPRESENTATION('mm',(%42),%5);
%42=MANIFOLD_SOLID_BREP('',%43);
%43=CLOSED_SHELL('',());
%50=CARTESIAN_POINT('',(1.,2.,3.));
%51=SHAPE_REPRESENTATION('inch',(%52),%105);
%52=MANIFOLD_SOLID_BREP('',%53);
%53=OPEN_SHELL('',());
%60=VERTEX_POINT('',%40);
%61=VERTEX_POINT('',%50);
%62=CLOSED_SHELL('',());
"
    );
    let records = read(&step_file(&data));

    assert_eq!(records.contexts[&5].length_to_mm, 1.0);
    assert_eq!(records.contexts[&105].length_to_mm, 25.4);
    assert_eq!(records.representations.len(), 2);
    assert_eq!(
        records.topology[&53],
        Topology::Shell {
            faces: vec![],
            closed: false
        }
    );
    // どの表現からもたどれない要素（%40・%50・%60 など）は読まない
    assert!(!records.geometry.contains_key(&40));
    assert!(!records.topology.contains_key(&60));
}

#[test]
fn units_default_to_millimetres_and_degrees() {
    let data = "%5=(GEOMETRIC_REPRESENTATION_CONTEXT(3) REPRESENTATION_CONTEXT('',''));"
        .to_string()
        + &SOLID.replace("ANGLE", "45.");
    let records = read(&step_file(&data));
    let context = &records.contexts[&5];

    assert!(context.default_units);
    assert_eq!(context.length_to_mm, 1.0);
    assert_eq!(context.angle_to_degrees, 1.0);
    assert_eq!(context.uncertainty_mm, None);
}

#[test]
fn a_conversion_cycle_falls_back_to_the_default() {
    // インチの換算の単位が、自分を指して循環している
    let cycle = INCH_DEGREE.replace("LENGTH_MEASURE(25.4),%8", "LENGTH_MEASURE(25.4),%1");
    let records = solid_in(&cycle, "45.");

    assert_eq!(records.contexts[&5].length_to_mm, 1.0);
    assert!(records.contexts[&5].default_units);
}

#[test]
fn two_dimensional_and_wireframe_shapes_are_skipped_and_recorded() {
    let data = format!(
        "{MM_RADIAN}
%6=(GEOMETRIC_REPRESENTATION_CONTEXT(2) GLOBAL_UNIT_ASSIGNED_CONTEXT((%1,%2)) REPRESENTATION_CONTEXT('',''));
%70=CARTESIAN_POINT('',(1.,2.));
%71=GEOMETRIC_CURVE_SET('',(%70));
%72=GEOMETRICALLY_BOUNDED_2D_WIREFRAME_REPRESENTATION('drawing',(%71),%6);
%80=CARTESIAN_POINT('',(1.,2.,3.));
%81=GEOMETRIC_CURVE_SET('',(%80));
%82=GEOMETRICALLY_BOUNDED_WIREFRAME_SHAPE_REPRESENTATION('wires',(%81),%5);
"
    );
    let records = read(&step_file(&data));

    assert!(records.representations.is_empty());
    assert_eq!(
        records.skipped,
        vec![
            Skipped {
                id: 71,
                name: "GEOMETRIC_CURVE_SET".to_string(),
                reason: SkipReason::TwoDimensional
            },
            Skipped {
                id: 81,
                name: "GEOMETRIC_CURVE_SET".to_string(),
                reason: SkipReason::NotSolid
            },
        ]
    );
}

#[test]
fn rational_b_splines_are_read_from_complex_entities() {
    let data = format!(
        "{MM_RADIAN}
%10=CARTESIAN_POINT('',(0.,0.,0.));
%11=CARTESIAN_POINT('',(1.,1.,0.));
%12=CARTESIAN_POINT('',(2.,0.,0.));
%13=(BOUNDED_CURVE() B_SPLINE_CURVE(2,(%10,%11,%12),.UNSPECIFIED.,.F.,.F.) B_SPLINE_CURVE_WITH_KNOTS((3,3),(0.,1.),.UNSPECIFIED.) CURVE() GEOMETRIC_REPRESENTATION_ITEM() RATIONAL_B_SPLINE_CURVE((1.,0.5,1.)) REPRESENTATION_ITEM(''));
%14=B_SPLINE_SURFACE_WITH_KNOTS('',1,1,((%10,%11),(%12,%10)),.UNSPECIFIED.,.F.,.T.,.F.,(2,2),(2,2),(0.,1.),(0.,1.),.UNSPECIFIED.);
%15=VERTEX_POINT('',%10);
%16=VERTEX_POINT('',%12);
%17=EDGE_CURVE('',%15,%16,%13,.T.);
%18=ORIENTED_EDGE('',*,*,%17,.F.);
%19=EDGE_LOOP('',(%18));
%20=FACE_BOUND('',%19,.T.);
%21=ADVANCED_FACE('',(%20),%14,.T.);
%22=OPEN_SHELL('',(%21));
%23=SHELL_BASED_SURFACE_MODEL('',(%22));
%24=MANIFOLD_SURFACE_SHAPE_REPRESENTATION('surface',(%23),%5);
"
    );
    let records = read(&step_file(&data));

    assert_eq!(
        records.geometry[&13],
        Geometry::BSplineCurve(BSplineCurve {
            degree: 2,
            control_points: vec![10, 11, 12],
            closed: Some(false),
            multiplicities: vec![3, 3],
            knots: vec![0.0, 1.0],
            weights: Some(vec![1.0, 0.5, 1.0]),
        })
    );
    assert_eq!(
        records.geometry[&14],
        Geometry::BSplineSurface(BSplineSurface {
            u_degree: 1,
            v_degree: 1,
            control_points: vec![vec![10, 11], vec![12, 10]],
            u_closed: Some(false),
            v_closed: Some(true),
            u_multiplicities: vec![2, 2],
            v_multiplicities: vec![2, 2],
            u_knots: vec![0.0, 1.0],
            v_knots: vec![0.0, 1.0],
            weights: None,
        })
    );
    assert_eq!(
        records.topology[&23],
        Topology::SurfaceModel { shells: vec![22] }
    );
    assert!(records.skipped.is_empty(), "{:?}", records.skipped);
}

#[test]
fn broken_elements_are_skipped_and_the_rest_is_read() {
    let data = MM_RADIAN.to_string()
        + &SOLID
            .replace("ANGLE", "0.5")
            // 円の半径が数でない・平面の配置がない番号・扱わない種類の曲面・次数の高すぎる B-spline
            .replace("CIRCLE('',%13,10.)", "CIRCLE('',%13,'ten')")
            .replace("PLANE('',%13)", "PLANE('',%99)")
            .replace(
                "CONICAL_SURFACE('',%13,10.,0.5)",
                "B_SPLINE_SURFACE_WITH_KNOTS('',40,1,((%10,%15),(%15,%10)),.UNSPECIFIED.,.F.,.F.,.F.,(2,2),(2,2),(0.,1.),(0.,1.),.UNSPECIFIED.)",
            );
    let records = read(&step_file(&data));
    let reasons: BTreeMap<u64, SkipReason> = records
        .skipped
        .iter()
        .map(|skipped| (skipped.id, skipped.reason))
        .collect();

    assert_eq!(reasons[&14], SkipReason::Malformed);
    assert_eq!(reasons[&99], SkipReason::MissingReference);
    assert_eq!(reasons[&23], SkipReason::TooLarge);
    // ほかの要素は読む
    assert!(records.topology.contains_key(&27));
    assert!(records.geometry.contains_key(&21));
}

#[test]
fn an_element_shared_by_contexts_with_different_units_is_read_once() {
    // mm の表現（%41）とインチの表現（%51）が、同じ立体（%42）を指す
    let inch = INCH_DEGREE.replace('%', "%10");
    let data = format!(
        "{MM_RADIAN}{inch}
%40=CARTESIAN_POINT('',(1.,2.,3.));
%41=SHAPE_REPRESENTATION('mm',(%42),%5);
%42=MANIFOLD_SOLID_BREP('',%43);
%43=CLOSED_SHELL('',());
%51=SHAPE_REPRESENTATION('inch',(%42),%105);
"
    );
    let records = read(&step_file(&data));

    // 同じ要素の衝突は 1 回だけ記録する
    let conflicts: Vec<u64> = records
        .skipped
        .iter()
        .filter(|skipped| skipped.reason == SkipReason::UnitConflict)
        .map(|skipped| skipped.id)
        .collect();
    assert_eq!(conflicts, vec![42]);
    assert!(records.topology.contains_key(&42));
}

#[test]
fn changed_input_never_panics() {
    let data = MM_RADIAN.to_string() + &SOLID.replace("ANGLE", "0.5");
    let text = format!(
        "ISO-10303-21;
HEADER;
ENDSEC;
DATA;
{}
ENDSEC;
END-ISO-10303-21;
",
        data.replace('%', "#")
    );
    let bytes = text.as_bytes();
    // 1 バイトずつ、いろいろな値に替えて、構文として読めたものを中間データにする
    for position in 0..bytes.len() {
        for replacement in *b"09$*(),#.-" {
            let mut changed = bytes.to_vec();
            changed[position] = replacement;
            if let Ok(file) = parse(&changed, &Limits::default()) {
                let _ = read(&file);
            }
        }
    }
}

#[test]
fn degrees_must_be_at_least_one_in_each_direction() {
    let data = format!(
        "{MM_RADIAN}
%10=CARTESIAN_POINT('',(0.,0.,0.));
%14=B_SPLINE_SURFACE_WITH_KNOTS('',3,0,((%10,%10),(%10,%10)),.UNSPECIFIED.,.F.,.F.,.F.,(2,2),(2,2),(0.,1.),(0.,1.),.UNSPECIFIED.);
%21=ADVANCED_FACE('',(),%14,.T.);
%22=CONNECTED_FACE_SET('',(%21));
%23=FACE_BASED_SURFACE_MODEL('',(%22));
%24=MANIFOLD_SURFACE_SHAPE_REPRESENTATION('surface',(%23),%5);
"
    );
    let records = read(&step_file(&data));

    assert_eq!(
        records.skipped,
        vec![Skipped {
            id: 14,
            name: "B_SPLINE_SURFACE_WITH_KNOTS".to_string(),
            reason: SkipReason::Malformed
        }]
    );
    // 面の集まりの面のモデルも読む
    assert_eq!(
        records.topology[&22],
        Topology::Shell {
            faces: vec![21],
            closed: false
        }
    );
    assert_eq!(
        records.topology[&23],
        Topology::SurfaceModel { shells: vec![22] }
    );
}

#[test]
fn unknown_contexts_and_malformed_representations_are_recorded() {
    let data = format!(
        "{MM_RADIAN}
%6=REPRESENTATION_CONTEXT('','');
%42=MANIFOLD_SOLID_BREP('',%43);
%43=CLOSED_SHELL('',());
%44=SHAPE_REPRESENTATION('no dimension',(%42),%6);
%45=ADVANCED_BREP_SHAPE_REPRESENTATION('not a list',%42,%5);
"
    );
    let records = read(&step_file(&data));
    let reasons: Vec<(u64, SkipReason)> = records
        .skipped
        .iter()
        .map(|skipped| (skipped.id, skipped.reason))
        .collect();

    assert_eq!(
        reasons,
        vec![
            (42, SkipReason::UnknownContext),
            (45, SkipReason::Malformed)
        ]
    );
}

#[test]
fn measures_written_as_complex_entities_are_read() {
    // 換算の値と許容誤差を、引数のない部分が先に並ぶ複合のエンティティで書く
    let complex = INCH_DEGREE
        .replace(
            "%6=LENGTH_MEASURE_WITH_UNIT(LENGTH_MEASURE(25.4),%8);",
            "%6=(LENGTH_MEASURE_WITH_UNIT() MEASURE_WITH_UNIT(LENGTH_MEASURE(25.4),%8));",
        )
        .replace(
            "%4=UNCERTAINTY_MEASURE_WITH_UNIT(LENGTH_MEASURE(1.E-05),%1,'distance_accuracy_value','');",
            "%4=(LENGTH_MEASURE_WITH_UNIT() MEASURE_WITH_UNIT(LENGTH_MEASURE(1.E-05),%1) UNCERTAINTY_MEASURE_WITH_UNIT('distance_accuracy_value',''));",
        );
    let records = solid_in(&complex, "45.");
    let context = &records.contexts[&5];

    assert_eq!(context.length_to_mm, 25.4);
    assert!(!context.default_units);
    assert!((context.uncertainty_mm.unwrap() - 25.4e-5).abs() < 1.0e-15);
}
