use super::*;
use crate::progress::{ENTITY_INTERVAL, keep_going};
use crate::syntax::{Limits, parse};
use std::ops::ControlFlow;

/// エンティティの表を、進み具合を見ずに終わりまで読む
fn read(file: StepFile) -> Records {
    super::read(file, &mut keep_going).unwrap()
}

/// エンティティの番号の印 `#` は `%` で書き、`step_file` で直して使う（ソースコードの検査で、別の番号と見分け
/// られないため）
fn step_file(data: &str) -> StepFile {
    let text = format!(
        "ISO-10303-21;\nHEADER;\nFILE_SCHEMA(('AUTOMOTIVE_DESIGN'));\nENDSEC;\nDATA;\n{}\nENDSEC;\nEND-ISO-10303-21;\n",
        data.replace('%', "#")
    );
    parse(text.as_bytes(), &Limits::default(), &mut keep_going).unwrap()
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
    read(step_file(
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
    let records = read(step_file(&data));

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
    let records = read(step_file(&data));
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
    let records = read(step_file(&data));

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
    let records = read(step_file(&data));

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
    let records = read(step_file(&data));
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
    let records = read(step_file(&data));

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
            if let Ok(file) = parse(&changed, &Limits::default(), &mut keep_going) {
                let _ = read(file.clone());
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
    let records = read(step_file(&data));

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
    let records = read(step_file(&data));
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

/// 部品（%44。立体の表現 %28 と、形状の表現 %48）と、それを 2 回配置したアセンブリ（%52。AP203 の第 1 版の
/// 書き方の文脈）。1 つ目の出現（%62）は 2 つの配置、2 つ目（%70）は変換の演算子で配置する。立体に面の色、
/// 平面の面に線の色と非表示を割り当てる
const ASSEMBLY: &str = "
%40=PRODUCT('P1','part name','',(%41));
%41=PRODUCT_CONTEXT('',%42,'mechanical');
%42=APPLICATION_CONTEXT('automotive design');
%43=PRODUCT_DEFINITION_FORMATION('','',%40);
%44=PRODUCT_DEFINITION('design','',%43,%45);
%45=PRODUCT_DEFINITION_CONTEXT('part definition',%42,'design');
%46=PRODUCT_DEFINITION_SHAPE('','',%44);
%47=SHAPE_DEFINITION_REPRESENTATION(%46,%48);
%48=SHAPE_REPRESENTATION('part',(%13),%5);
%49=SHAPE_REPRESENTATION_RELATIONSHIP('','',%48,%28);
%50=PRODUCT('A1','assembly','',(%41));
%51=PRODUCT_DEFINITION_FORMATION_WITH_SPECIFIED_SOURCE('','',%50,.NOT_KNOWN.);
%52=PRODUCT_DEFINITION('design','',%51,%53);
%53=DESIGN_CONTEXT('',%42,'design');
%54=PRODUCT_DEFINITION_SHAPE('','',%52);
%55=SHAPE_DEFINITION_REPRESENTATION(%54,%56);
%56=SHAPE_REPRESENTATION('assembly',(%13,%60),%5);
%60=AXIS2_PLACEMENT_3D('',%61,%11,%12);
%61=CARTESIAN_POINT('',(100.,0.,0.));
%62=NEXT_ASSEMBLY_USAGE_OCCURRENCE('NAUO1','instance 1','',%52,%44,$);
%63=PRODUCT_DEFINITION_SHAPE('','',%62);
%64=CONTEXT_DEPENDENT_SHAPE_REPRESENTATION(%65,%63);
%65=(REPRESENTATION_RELATIONSHIP('','',%48,%56) REPRESENTATION_RELATIONSHIP_WITH_TRANSFORMATION(%66) SHAPE_REPRESENTATION_RELATIONSHIP());
%66=ITEM_DEFINED_TRANSFORMATION('','',%13,%60);
%70=NEXT_ASSEMBLY_USAGE_OCCURRENCE('NAUO2','instance 2','',%52,%44,$);
%71=PRODUCT_DEFINITION_SHAPE('','',%70);
%72=CONTEXT_DEPENDENT_SHAPE_REPRESENTATION(%73,%71);
%73=(REPRESENTATION_RELATIONSHIP('','',%48,%56) REPRESENTATION_RELATIONSHIP_WITH_TRANSFORMATION(%74) SHAPE_REPRESENTATION_RELATIONSHIP());
%74=CARTESIAN_TRANSFORMATION_OPERATOR_3D('',%11,%12,%75,2.,$);
%75=CARTESIAN_POINT('',(0.,50.,0.));
%80=COLOUR_RGB('',0.2,0.4,0.6);
%81=FILL_AREA_STYLE_COLOUR('',%80);
%82=FILL_AREA_STYLE('',(%81));
%83=SURFACE_STYLE_FILL_AREA(%82);
%84=SURFACE_SIDE_STYLE('',(%83));
%85=SURFACE_STYLE_USAGE(.BOTH.,%84);
%86=PRESENTATION_STYLE_ASSIGNMENT((%85));
%87=STYLED_ITEM('color',(%86),%27);
%88=DRAUGHTING_PRE_DEFINED_COLOUR('red');
%89=CURVE_STYLE('',$,$,%88);
%90=PRESENTATION_STYLE_ASSIGNMENT((%89));
%91=STYLED_ITEM('',(%90),%22);
%92=INVISIBILITY((%91));
";

fn assembly_in(context: &str) -> Records {
    read(step_file(
        &(context.to_string() + &SOLID.replace("ANGLE", "0.5") + ASSEMBLY),
    ))
}

#[test]
fn products_and_occurrences_are_recorded() {
    let records = assembly_in(MM_RADIAN);
    assert!(records.skipped.is_empty(), "{:?}", records.skipped);

    assert_eq!(
        records.products[&44],
        Product {
            id: "P1".to_string(),
            name: "part name".to_string(),
            representations: vec![48],
        }
    );
    // AP203 の第 1 版の文脈（DESIGN_CONTEXT）と、版の書き方の違いも受け付ける
    assert_eq!(records.products[&52].name, "assembly");
    assert_eq!(records.products[&52].representations, vec![56]);
    // 部品の形状の表現から、立体の表現へ
    assert_eq!(records.representation_links, vec![(48, 28)]);
    assert_eq!(records.item_names[&27], "solid");

    let first = &records.occurrences[&62];
    assert_eq!(
        (first.id.as_str(), first.name.as_str()),
        ("NAUO1", "instance 1")
    );
    assert_eq!((first.parent, first.child), (52, 44));
    assert_eq!(
        first.placement,
        Some(OccurrencePlacement {
            representation_1: 48,
            representation_2: 56,
            transformation: Transformation::Placements {
                item_1: 13,
                item_2: 60
            },
        })
    );
    // 配置は、幾何の記録として読む
    assert!(matches!(
        records.geometry[&60],
        Geometry::Placement { location: 61, .. }
    ));
    assert_eq!(records.geometry[&61], Geometry::Point([100.0, 0.0, 0.0]));

    let second = &records.occurrences[&70];
    assert_eq!(
        second
            .placement
            .as_ref()
            .map(|placement| &placement.transformation),
        Some(&Transformation::Operator {
            axis_1: Some(11),
            axis_2: Some(12),
            axis_3: None,
            origin: 75,
            scale: Some(2.0),
        })
    );
    assert_eq!(records.geometry[&75], Geometry::Point([0.0, 50.0, 0.0]));
}

#[test]
fn colors_and_invisibility_are_recorded() {
    let records = assembly_in(MM_RADIAN);

    // 面のスタイルの色
    assert_eq!(
        records.colors[&27],
        Color {
            red: 0.2,
            green: 0.4,
            blue: 0.6
        }
    );
    // 名前で決まった色（線のスタイルしかなければ、それを使う）
    assert_eq!(
        records.colors[&22],
        Color {
            red: 1.0,
            green: 0.0,
            blue: 0.0
        }
    );
    // スタイルの割り当ての非表示は、その対象の非表示
    assert_eq!(records.invisible, BTreeSet::from([22]));
}

#[test]
fn placements_are_read_in_the_units_of_their_representation() {
    // インチの文脈では、配置の点を mm に直す
    let records = assembly_in(INCH_DEGREE);
    assert_eq!(records.geometry[&61], Geometry::Point([2540.0, 0.0, 0.0]));
    assert_eq!(records.geometry[&75], Geometry::Point([0.0, 1270.0, 0.0]));
}

#[test]
fn colors_follow_their_priority() {
    // 平面の面（%22）には線の色（赤）があり、別の割り当てで面の塗りつぶしの色（緑）を足す。面の境界の線の色（黒）は
    // 面の色にしない。円錐の面（%24）は、上書きの割り当て（黄）が、前の面の色（緑）より優先する
    let extra = "
%100=COLOUR_RGB('',0.,0.,0.);
%101=CURVE_STYLE('',$,$,%100);
%102=SURFACE_STYLE_BOUNDARY(%101);
%103=COLOUR_RGB('',0.,1.,0.);
%104=FILL_AREA_STYLE_COLOUR('',%103);
%105=FILL_AREA_STYLE('',(%104));
%106=SURFACE_STYLE_FILL_AREA(%105);
%107=SURFACE_SIDE_STYLE('',(%102,%106));
%108=SURFACE_STYLE_USAGE(.BOTH.,%107);
%109=PRESENTATION_STYLE_ASSIGNMENT((%108));
%110=STYLED_ITEM('',(%109),%22);
%111=STYLED_ITEM('',(%109),%24);
%112=DRAUGHTING_PRE_DEFINED_COLOUR('yellow');
%113=CURVE_STYLE('',$,$,%112);
%114=PRESENTATION_STYLE_ASSIGNMENT((%113));
%115=OVER_RIDING_STYLED_ITEM('',(%114),%24,%111);
";
    let records = read(step_file(
        &(MM_RADIAN.to_string() + &SOLID.replace("ANGLE", "0.5") + ASSEMBLY + extra),
    ));
    let green = Color {
        red: 0.0,
        green: 1.0,
        blue: 0.0,
    };
    assert_eq!(records.colors[&22], green);
    assert_eq!(
        records.colors[&24],
        Color {
            red: 1.0,
            green: 1.0,
            blue: 0.0
        }
    );
}

#[test]
fn a_style_that_refers_to_itself_is_followed_once() {
    let records = read(step_file(
        &(MM_RADIAN.to_string()
            + &SOLID.replace("ANGLE", "0.5")
            + "%100=PRESENTATION_STYLE_ASSIGNMENT((%100,%100,%100,%100,%100,%100,%100,%100));
%101=STYLED_ITEM('',(%100),%27);"),
    ));
    assert!(records.colors.is_empty());
}

#[test]
fn a_hidden_layer_hides_its_items() {
    let records = read(step_file(
        &(MM_RADIAN.to_string()
            + &SOLID.replace("ANGLE", "0.5")
            + "%100=PRESENTATION_LAYER_ASSIGNMENT('hidden','',(%27,%24));
%101=INVISIBILITY((%100));"),
    ));
    assert_eq!(records.invisible, BTreeSet::from([24, 27]));
}

#[test]
fn an_operator_with_a_name_and_a_description_is_read() {
    // 2 つの上位型の名前と説明を、すべて書いた形（引数が 8 つ）
    let data = (MM_RADIAN.to_string() + &SOLID.replace("ANGLE", "0.5") + ASSEMBLY).replace(
        "CARTESIAN_TRANSFORMATION_OPERATOR_3D('',%11,%12,%75,2.,$)",
        "CARTESIAN_TRANSFORMATION_OPERATOR_3D('','','',%11,%12,%75,2.,$)",
    );
    let records = read(step_file(&data));
    assert!(matches!(
        records.occurrences[&70]
            .placement
            .as_ref()
            .map(|placement| &placement.transformation),
        Some(Transformation::Operator { origin: 75, .. })
    ));
}

#[test]
fn a_transformed_relation_outside_occurrences_is_recorded_as_skipped() {
    let records = read(step_file(
        &(MM_RADIAN.to_string()
            + &SOLID.replace("ANGLE", "0.5")
            + ASSEMBLY
            + "%120=(REPRESENTATION_RELATIONSHIP('','',%48,%56) REPRESENTATION_RELATIONSHIP_WITH_TRANSFORMATION(%66) SHAPE_REPRESENTATION_RELATIONSHIP());"),
    ));
    let skipped: Vec<(u64, SkipReason)> = records
        .skipped
        .iter()
        .map(|skipped| (skipped.id, skipped.reason))
        .collect();
    assert_eq!(skipped, vec![(120, SkipReason::Unsupported)]);

    // 文脈による形状の表現が、出現ではなく製品の定義の形状（%46）を指すときも、関係は飛ばしたと記録する
    let records = read(step_file(
        &(MM_RADIAN.to_string()
            + &SOLID.replace("ANGLE", "0.5")
            + ASSEMBLY
            + "%120=(REPRESENTATION_RELATIONSHIP('','',%48,%56) REPRESENTATION_RELATIONSHIP_WITH_TRANSFORMATION(%66) SHAPE_REPRESENTATION_RELATIONSHIP());
%121=CONTEXT_DEPENDENT_SHAPE_REPRESENTATION(%120,%46);"),
    ));
    let skipped: Vec<(u64, SkipReason)> = records
        .skipped
        .iter()
        .map(|skipped| (skipped.id, skipped.reason))
        .collect();
    assert_eq!(skipped, vec![(120, SkipReason::Unsupported)]);
}

/// 記録 `data` のエンティティの名前のうち、短い名前の表にあるものを短い名前に書き換えたもの（書き換えた数と）
fn with_short_names(data: &str) -> (String, usize) {
    let mut text = data.to_string();
    let mut count = 0;
    // 名前の前の区切りと後ろの `(` で挟んで探すので、ほかの名前の一部は書き換えない
    for &(short, long) in short_names::pairs() {
        for prefix in ["=", "(", " "] {
            let from = format!("{prefix}{long}(");
            count += text.matches(&from).count();
            text = text.replace(&from, &format!("{prefix}{short}("));
        }
    }
    (text, count)
}

#[test]
fn short_names_give_the_same_records_as_full_names() {
    let full = MM_RADIAN.to_string() + &SOLID.replace("ANGLE", "0.5") + ASSEMBLY;
    let (short, count) = with_short_names(&full);
    assert!(count >= 40, "{count}");
    assert!(short.contains("ADVFC("));
    assert_eq!(read(step_file(&short)), read(step_file(&full)));
}

#[test]
fn the_short_name_table_is_sorted_and_has_no_duplicates() {
    let pairs = short_names::pairs();
    assert!(pairs.windows(2).all(|pair| pair[0].0 < pair[1].0));
    let longs: BTreeSet<&str> = pairs.iter().map(|&(_, long)| long).collect();
    assert_eq!(longs.len(), pairs.len());
    // 短い名前が別の組の正式な名前と同じなら、正しく書かれた正式な名前まで書き換わってしまう
    assert!(pairs.iter().all(|(short, _)| !longs.contains(short)));
    assert_eq!(short_names::long_name("ADVFC"), Some("ADVANCED_FACE"));
    assert_eq!(short_names::long_name("ADVANCED_FACE"), None);
}

#[test]
fn unknown_short_names_are_kept_as_written() {
    // 表にある短い名前（平面の面）は直して読み、表にない短い名前（円錐の面）は書かれたとおりの名前で、扱わない
    // 種類として飛ばす
    let data = MM_RADIAN.to_string()
        + &SOLID
            .replace("ANGLE", "0.5")
            .replace("%22=ADVANCED_FACE(", "%22=ADVFC(")
            .replace("%24=ADVANCED_FACE(", "%24=QQQQQ(");
    let records = read(step_file(&data));
    assert!(matches!(
        records.topology.get(&22),
        Some(Topology::Face { .. })
    ));
    assert!(!records.topology.contains_key(&24));
    let skipped: Vec<(u64, &str)> = records
        .skipped
        .iter()
        .map(|skipped| (skipped.id, skipped.name.as_str()))
        .collect();
    assert!(skipped.contains(&(24, "QQQQQ")), "{skipped:?}");
}

#[test]
fn every_entity_name_the_reader_matches_has_its_short_name() {
    // 中間データの段が名前で見分ける語は、短い名前の表の正式な名前か、短い名前と正式な名前が同じエンティティか、
    // エンティティでない語（単位の接頭語・単位の名前など）のどれか。新しく名前で見分けるエンティティを足したら、
    // ISO の短い名前の表から組を写す
    const SAME_SHORT_NAME: [&str; 5] = ["CIRCLE", "FACE", "LINE", "PLANE", "VECTOR"];
    const NOT_ENTITIES: [&str; 20] = [
        "ATTO",
        "CENTI",
        "DECA",
        "DECI",
        "DEGREE",
        "EXA",
        "FEMTO",
        "GIGA",
        "HECTO",
        "KILO",
        "MEGA",
        "METRE",
        "MICRO",
        "MILLI",
        "NANO",
        "PETA",
        "PICO",
        "PRESENTATION_",
        "RADIAN",
        "TERA",
    ];
    let longs: BTreeSet<&str> = short_names::pairs().iter().map(|&(_, long)| long).collect();
    let sources = [
        include_str!("../record.rs"),
        include_str!("product.rs"),
        include_str!("units.rs"),
    ];
    let mut missing = Vec::new();
    for source in sources {
        for word in source.split('"').skip(1).step_by(2) {
            let looks_like_name = word.len() >= 3
                && word.starts_with(|c: char| c.is_ascii_uppercase())
                && word
                    .chars()
                    .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
            if looks_like_name
                && !longs.contains(word)
                && !SAME_SHORT_NAME.contains(&word)
                && !NOT_ENTITIES.contains(&word)
            {
                missing.push(word);
            }
        }
    }
    assert!(missing.is_empty(), "{missing:?}");
}

/// 立体の表現（[`MM_RADIAN`]・[`SOLID`]）に、どこからも指されない点 `extra` 個を足したエンティティの表
fn solid_with_extra_points(extra: usize) -> StepFile {
    let points: String = (0..extra)
        .map(|index| {
            format!(
                "%{}=CARTESIAN_POINT('',(0.,0.,0.));
",
                1000 + index
            )
        })
        .collect();
    step_file(&(MM_RADIAN.to_string() + &SOLID.replace("ANGLE", "0.5") + &points))
}

#[test]
fn progress_is_reported_at_each_interval_of_entities() {
    let file = solid_with_extra_points(2 * ENTITY_INTERVAL);
    let entities = file.entities.len();
    let mut reports = Vec::new();
    let records = super::read(file.clone(), &mut |done, total| {
        reports.push((done, total));
        ControlFlow::Continue(())
    })
    .unwrap();

    // 進み具合を見ないときと、同じ記録になる
    assert_eq!(records, read(file));
    assert_eq!(records.representations.len(), 1);
    // 表を 3 回たどり、立体から指される要素を読む。全体の数は、表のエンティティの数に、たどる回数の上限を掛けた数
    assert!(reports.len() >= 6, "{reports:?}");
    assert!(reports.iter().all(|&(_, total)| total == entities * PASSES));
    assert!(
        reports
            .iter()
            .zip(1..)
            .all(|(&(done, total), report)| done == report * ENTITY_INTERVAL && done <= total)
    );
}

#[test]
fn reading_stops_when_the_progress_closure_breaks() {
    let file = solid_with_extra_points(2 * ENTITY_INTERVAL);
    for stop_at in [1, 3, 5] {
        let mut reports = 0;
        let result = super::read(file.clone(), &mut |_, _| {
            reports += 1;
            if reports == stop_at {
                ControlFlow::Break(())
            } else {
                ControlFlow::Continue(())
            }
        });

        assert_eq!(result, Err(Cancelled));
        // 止めると返した後は、知らせない
        assert_eq!(reports, stop_at);
    }
}
