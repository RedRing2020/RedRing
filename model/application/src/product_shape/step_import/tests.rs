use super::*;

mod broken_files;
mod conformance;
mod large_files;

/// STEP のバイト列を、進み具合を見ずに終わりまで読む
fn read_shells(
    bytes: &[u8],
    file_name: &str,
) -> Result<(Vec<ImportedShell>, StepImportReport), StepImportError> {
    read_step(bytes, file_name, &mut |_| ControlFlow::Continue(())).map_err(|failure| match failure
    {
        TaskFailure::Failed(error) => error,
        TaskFailure::Cancelled => panic!("the reading is not cancelled"),
    })
}

/// エンティティの番号の印 `#` は `%` で書き、ここで直す（ソースコードの検査で、別の番号と見分けられないため）
pub(crate) fn step(data: &str) -> Vec<u8> {
    format!(
        "ISO-10303-21;\nHEADER;\nFILE_SCHEMA(('AUTOMOTIVE_DESIGN'));\nENDSEC;\nDATA;\n{}\nENDSEC;\nEND-ISO-10303-21;\n",
        data.replace('%', "#")
    )
    .into_bytes()
}

/// mm とラジアンの文脈（%5）と、底の円と円錐の面の立体（%27）の表現（%28）
pub(crate) const PART: &str = "
%1=(LENGTH_UNIT() NAMED_UNIT(*) SI_UNIT(.MILLI.,.METRE.));
%2=(NAMED_UNIT(*) PLANE_ANGLE_UNIT() SI_UNIT($,.RADIAN.));
%3=(NAMED_UNIT(*) SI_UNIT($,.STERADIAN.) SOLID_ANGLE_UNIT());
%4=UNCERTAINTY_MEASURE_WITH_UNIT(LENGTH_MEASURE(1.E-07),%1,'distance_accuracy_value','');
%5=(GEOMETRIC_REPRESENTATION_CONTEXT(3) GLOBAL_UNCERTAINTY_ASSIGNED_CONTEXT((%4)) GLOBAL_UNIT_ASSIGNED_CONTEXT((%1,%2,%3)) REPRESENTATION_CONTEXT('',''));
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
%23=CONICAL_SURFACE('',%13,10.,0.5);
%25=FACE_BOUND('',%19,.F.);
%24=ADVANCED_FACE('',(%25),%23,.F.);
%26=CLOSED_SHELL('',(%22,%24));
%27=MANIFOLD_SOLID_BREP('cone solid',%26);
%28=ADVANCED_BREP_SHAPE_REPRESENTATION('part',(%27,%13),%5);
%40=PRODUCT('P1','cone part','',(%41));
%41=PRODUCT_CONTEXT('',%42,'mechanical');
%42=APPLICATION_CONTEXT('automotive design');
%43=PRODUCT_DEFINITION_FORMATION('','',%40);
%44=PRODUCT_DEFINITION('design','',%43,%45);
%45=PRODUCT_DEFINITION_CONTEXT('part definition',%42,'design');
%46=PRODUCT_DEFINITION_SHAPE('','',%44);
%47=SHAPE_DEFINITION_REPRESENTATION(%46,%48);
%48=SHAPE_REPRESENTATION('part',(%13),%5);
%49=SHAPE_REPRESENTATION_RELATIONSHIP('','',%48,%28);
";

/// 部品（%44）を 3 回使うアセンブリ（%52）: 項目で決める配置で x に 100、変換の演算子で y に 50、倍率 2 の
/// 変換の演算子（外す）
pub(crate) const ASSEMBLY: &str = "
%50=PRODUCT('A1','assembly','',(%41));
%51=PRODUCT_DEFINITION_FORMATION('','',%50);
%52=PRODUCT_DEFINITION('design','',%51,%45);
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
%74=CARTESIAN_TRANSFORMATION_OPERATOR_3D('',%12,$,%75,1.,%11);
%75=CARTESIAN_POINT('',(0.,50.,0.));
%80=NEXT_ASSEMBLY_USAGE_OCCURRENCE('NAUO3','instance 3','',%52,%44,$);
%81=PRODUCT_DEFINITION_SHAPE('','',%80);
%82=CONTEXT_DEPENDENT_SHAPE_REPRESENTATION(%83,%81);
%83=(REPRESENTATION_RELATIONSHIP('','',%48,%56) REPRESENTATION_RELATIONSHIP_WITH_TRANSFORMATION(%84) SHAPE_REPRESENTATION_RELATIONSHIP());
%84=CARTESIAN_TRANSFORMATION_OPERATOR_3D('',%12,$,%75,2.,%11);
";

fn origin_of(shell: &ImportedShell) -> geo_algorithms::Point3D<f64> {
    shell
        .placement
        .apply_point(geo_algorithms::Point3D::new(0.0, 0.0, 0.0))
}

#[test]
fn occurrences_share_the_brep_of_their_part_and_are_placed() {
    let (shells, report) =
        read_shells(&step(&(PART.to_string() + ASSEMBLY)), "assembly.step").unwrap();
    // 倍率 2 の出現は外す
    assert_eq!(report.shells, 2);
    assert_eq!(report.solids, 1);
    assert_eq!(report.skipped_occurrences, 1);
    let [first, second] = shells.as_slice() else {
        panic!("two occurrences are read");
    };
    assert!(Arc::ptr_eq(&first.brep, &second.brep));
    assert!(origin_of(first).distance_to(geo_algorithms::Point3D::new(100.0, 0.0, 0.0)) < 1.0e-9);
    assert!(origin_of(second).distance_to(geo_algorithms::Point3D::new(0.0, 50.0, 0.0)) < 1.0e-9);
    // 名前は立体の名前。製品と出現の名前は属性に残す
    assert_eq!(first.metadata.name.as_deref(), Some("cone solid"));
    assert_eq!(first.metadata.custom[PRODUCT_NAME_KEY], "cone part");
    assert_eq!(first.metadata.custom[OCCURRENCE_NAMES_KEY], "instance 1");
    assert_eq!(second.metadata.custom[OCCURRENCE_NAMES_KEY], "instance 2");
    // 元の要素: ファイルの名前・出現の並び・B-rep の面の順の元の面
    assert_eq!(first.source.file_name, "assembly.step");
    assert_eq!(first.source.occurrences, vec![62]);
    let mut faces = first.source.faces.clone();
    faces.sort_unstable();
    assert_eq!(faces, vec![22, 24]);
}

#[test]
fn a_part_without_an_assembly_is_read_in_place_with_its_names() {
    let (shells, report) = read_shells(&step(PART), "part.step").unwrap();
    assert_eq!(report.shells, 1);
    let [shell] = shells.as_slice() else {
        panic!("one shell is read");
    };
    assert!(origin_of(shell).distance_to(geo_algorithms::Point3D::new(0.0, 0.0, 0.0)) < 1.0e-9);
    assert!(shell.source.occurrences.is_empty());
    assert_eq!(shell.metadata.custom[SOLID_NAME_KEY], "cone solid");
    assert_eq!(shell.metadata.custom[PRODUCT_NAME_KEY], "cone part");
}

#[test]
fn a_shape_outside_the_product_structure_is_read_in_place() {
    // 製品のない立体だけのファイル（名前は立体の名前）
    let part = &PART[..PART.find("%40=").unwrap()];
    let (shells, _) = read_shells(&step(part), "solid.step").unwrap();
    assert_eq!(shells.len(), 1);
    assert_eq!(shells[0].metadata.name.as_deref(), Some("cone solid"));
}

/// 製品 `name`（製品の定義は `definition`、番号は `first` から 3 つ）
fn product(first: u64, definition: u64, name: &str) -> String {
    format!(
        "%{first}=PRODUCT('{name}','{name}','',(%41));\n%{}=PRODUCT_DEFINITION_FORMATION('','',%{first});\n%{definition}=PRODUCT_DEFINITION('design','',%{},%45);\n",
        first + 1,
        first + 1
    )
}

#[test]
fn a_cyclic_assembly_stops_at_the_depth_limit() {
    // 根 R（%102）→ A（%112）→ B（%122）→ A ... の循環
    let data = PART.to_string()
        + &product(100, 102, "R")
        + &product(110, 112, "A")
        + &product(120, 122, "B")
        + "%130=NEXT_ASSEMBLY_USAGE_OCCURRENCE('1','r-a','',%102,%112,$);\n"
        + "%131=NEXT_ASSEMBLY_USAGE_OCCURRENCE('2','a-b','',%112,%122,$);\n"
        + "%132=NEXT_ASSEMBLY_USAGE_OCCURRENCE('3','b-a','',%122,%112,$);\n";
    assert_eq!(
        read_shells(&step(&data), "cycle.step").err(),
        Some(StepImportError::Limit(StepImportLimit::AssemblyDepth))
    );
}

#[test]
fn broken_files_and_files_without_shapes_are_refused() {
    assert!(matches!(
        read_shells(b"ISO-10303-21;\nHEADER;", "broken.step").err(),
        Some(StepImportError::Syntax(_))
    ));
    let context = &PART[..PART.find("%10=").unwrap()];
    assert_eq!(
        read_shells(&step(context), "empty.step").err(),
        Some(StepImportError::NoShapes)
    );
}

/// 部品（%44）を 2 回使うアセンブリ（%152）: 関係の 1 つ目がアセンブリの表現で、z のまわりに 90 度回して x に 100
/// 動かす配置と、鏡映の変換の演算子（外す）
const ROTATED: &str = "
%150=PRODUCT('A2','rotated assembly','',(%41));
%151=PRODUCT_DEFINITION_FORMATION('','',%150);
%152=PRODUCT_DEFINITION('design','',%151,%45);
%154=PRODUCT_DEFINITION_SHAPE('','',%152);
%155=SHAPE_DEFINITION_REPRESENTATION(%154,%156);
%156=SHAPE_REPRESENTATION('assembly',(%13,%160),%5);
%160=AXIS2_PLACEMENT_3D('',%161,%11,%162);
%161=CARTESIAN_POINT('',(100.,0.,0.));
%162=DIRECTION('',(0.,1.,0.));
%170=NEXT_ASSEMBLY_USAGE_OCCURRENCE('NAUO4','rotated','',%152,%44,$);
%171=PRODUCT_DEFINITION_SHAPE('','',%170);
%172=CONTEXT_DEPENDENT_SHAPE_REPRESENTATION(%173,%171);
%173=(REPRESENTATION_RELATIONSHIP('','',%156,%48) REPRESENTATION_RELATIONSHIP_WITH_TRANSFORMATION(%174) SHAPE_REPRESENTATION_RELATIONSHIP());
%174=ITEM_DEFINED_TRANSFORMATION('','',%160,%13);
%180=NEXT_ASSEMBLY_USAGE_OCCURRENCE('NAUO5','mirrored','',%152,%44,$);
%181=PRODUCT_DEFINITION_SHAPE('','',%180);
%182=CONTEXT_DEPENDENT_SHAPE_REPRESENTATION(%183,%181);
%183=(REPRESENTATION_RELATIONSHIP('','',%48,%156) REPRESENTATION_RELATIONSHIP_WITH_TRANSFORMATION(%184) SHAPE_REPRESENTATION_RELATIONSHIP());
%184=CARTESIAN_TRANSFORMATION_OPERATOR_3D('',%12,%185,%10,1.,%11);
%185=DIRECTION('',(0.,-1.,0.));
";

#[test]
fn a_placement_written_from_the_assembly_side_is_inverted_and_a_mirror_is_skipped() {
    let (shells, report) =
        read_shells(&step(&(PART.to_string() + ROTATED)), "rotated.step").unwrap();
    assert_eq!(report.skipped_occurrences, 1);
    let [shell] = shells.as_slice() else {
        panic!("one occurrence is read");
    };
    // 部品の (10, 0, 0) は、z のまわりに 90 度回して x に 100 動かした (100, 10, 0)
    let moved = shell
        .placement
        .apply_point(geo_algorithms::Point3D::new(10.0, 0.0, 0.0));
    assert!(moved.distance_to(geo_algorithms::Point3D::new(100.0, 10.0, 0.0)) < 1.0e-9);
}

/// 部品（%44）を、軸が X で基準の方向のない配置（%190）に置くアセンブリ（%202）と、鏡映の出現でだけ部品を使う
/// アセンブリ（%212）
const AXIS_X: &str = "
%190=AXIS2_PLACEMENT_3D('',%10,%12,$);
%200=PRODUCT('A3','axis assembly','',(%41));
%201=PRODUCT_DEFINITION_FORMATION('','',%200);
%202=PRODUCT_DEFINITION('design','',%201,%45);
%204=PRODUCT_DEFINITION_SHAPE('','',%202);
%205=SHAPE_DEFINITION_REPRESENTATION(%204,%206);
%206=SHAPE_REPRESENTATION('assembly',(%13,%190),%5);
%207=NEXT_ASSEMBLY_USAGE_OCCURRENCE('NAUO6','axis x','',%202,%44,$);
%208=PRODUCT_DEFINITION_SHAPE('','',%207);
%209=CONTEXT_DEPENDENT_SHAPE_REPRESENTATION(%210,%208);
%210=(REPRESENTATION_RELATIONSHIP('','',%48,%206) REPRESENTATION_RELATIONSHIP_WITH_TRANSFORMATION(%211) SHAPE_REPRESENTATION_RELATIONSHIP());
%211=ITEM_DEFINED_TRANSFORMATION('','',%13,%190);
";

#[test]
fn a_placement_along_x_without_a_reference_direction_follows_iso() {
    // 軸が X で基準の方向がなければ、X 軸の向きは Y（ISO 10303-42）。部品の (10, 0, 0) は (0, 10, 0)、(0, 0, 10) は
    // (10, 0, 0) に置く
    let (shells, _) = read_shells(&step(&(PART.to_string() + AXIS_X)), "axis.step").unwrap();
    let [shell] = shells.as_slice() else {
        panic!("one occurrence is read");
    };
    let place = |x: f64, y: f64, z: f64| {
        shell
            .placement
            .apply_point(geo_algorithms::Point3D::new(x, y, z))
    };
    assert!(
        place(10.0, 0.0, 0.0).distance_to(geo_algorithms::Point3D::new(0.0, 10.0, 0.0)) < 1.0e-9
    );
    assert!(
        place(0.0, 0.0, 10.0).distance_to(geo_algorithms::Point3D::new(10.0, 0.0, 0.0)) < 1.0e-9
    );
}

#[test]
fn a_part_used_only_by_a_skipped_occurrence_is_not_read_in_place() {
    // 鏡映の出現（NAUO5）だけで部品を使うアセンブリ。部品を配置なしで読み直さない
    let mirrored_only: String = ROTATED
        .lines()
        .filter(|line| !line.starts_with("%17"))
        .collect::<Vec<_>>()
        .join("\n");
    let data = PART.to_string() + &mirrored_only;
    assert_eq!(
        read_shells(&step(&data), "mirrored.step").err(),
        Some(StepImportError::NoShapes)
    );
}

/// 読み込みの進み具合を集め、`stop_at` 回目の知らせで止めると返す閉包で読む（読めなかった理由と、知らせた進み具合を
/// 返す）
fn read_reporting(
    bytes: &[u8],
    stop_at: Option<usize>,
) -> (
    Option<TaskFailure<StepImportError>>,
    Vec<StepImportProgress>,
) {
    let mut reports = Vec::new();
    let result = read_step(bytes, "part.step", &mut |progress| {
        reports.push(progress);
        if Some(reports.len()) == stop_at {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    });
    (result.err(), reports)
}

#[test]
fn progress_is_reported_at_the_start_of_each_stage_in_order() {
    let (failure, reports) = read_reporting(&step(PART), None);

    assert_eq!(failure, None);
    // 小さなファイルは区切りに届かないので、段の初めと、B-rep を作る段の表現の要素ごとに知らせる
    let stages: Vec<StepImportStage> = reports.iter().map(|progress| progress.stage).collect();
    assert_eq!(
        stages,
        [
            StepImportStage::Syntax,
            StepImportStage::Records,
            StepImportStage::Geometry,
            StepImportStage::Topology,
            StepImportStage::Topology,
        ]
    );
    // 全体の量は変わらず、済んだ量は減らない。読む処理は、表示用の三角形の段の量を残して終わる
    assert!(
        reports
            .iter()
            .all(|progress| progress.total == PROGRESS_TOTAL)
    );
    assert!(reports.is_sorted_by_key(|progress| progress.done));
    assert_eq!(reports[0].done, 0);
    let (display_start, display_span) = StepImportStage::DisplayMeshes.span();
    assert!(reports.iter().all(|progress| progress.done < display_start));
    assert_eq!(display_start + display_span, PROGRESS_TOTAL);
}

#[test]
fn reading_stops_at_the_stage_where_the_progress_closure_breaks() {
    let bytes = step(PART);
    let (_, all) = read_reporting(&bytes, None);
    for stop_at in 1..=all.len() {
        let (failure, reports) = read_reporting(&bytes, Some(stop_at));

        assert_eq!(
            failure,
            Some(TaskFailure::Cancelled),
            "stopped at report {stop_at}"
        );
        // 止めると返した後は知らせず、止めるまでの知らせは、止めないときと同じ
        assert_eq!(reports, all[..stop_at]);
    }
}

#[test]
fn the_overall_progress_of_a_stage_stays_within_the_span_of_the_stage() {
    let (start, span) = StepImportStage::Topology.span();
    let at =
        |done, total| StepImportProgress::in_stage(StepImportStage::Topology, done, total).done;

    assert_eq!(at(0, 4), start);
    assert_eq!(at(2, 4), start + span / 2);
    assert_eq!(at(4, 4), start + span);
    // 全体の数が 0 のとき・済んだ数が全体の数を超えるときも、段の量の中に収める
    assert_eq!(at(0, 0), start);
    assert_eq!(at(9, 4), start + span);
}
