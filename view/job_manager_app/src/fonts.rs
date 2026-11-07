//! 画面の日本語の文字（OS にある CJK のフォントを使う）

use fontdb::{Database, Family, Query};

/// 日本語のフォントの文字を下げる量（文字の大きさに対する割合）
const CJK_Y_OFFSET_FACTOR: f32 = 0.25;

/// OS にある CJK のフォントを、egui の文字に加える（見つからなければ既定のまま）
pub(crate) fn install_cjk_font(ctx: &eframe::egui::Context) {
    let Some(bytes) = load_system_cjk_font_bytes() else {
        eprintln!("no CJK system font was found; non-ASCII text may not be shown");
        return;
    };
    let mut fonts = eframe::egui::FontDefinitions::default();
    let name = "system_cjk".to_owned();
    // 日本語のフォントは、行の高さに比べて文字が上に寄って描かれるため、少し下げて行の中央にそろえる
    let tweak = eframe::egui::FontTweak {
        y_offset_factor: CJK_Y_OFFSET_FACTOR,
        ..Default::default()
    };
    fonts.font_data.insert(
        name.clone(),
        eframe::egui::FontData::from_owned(bytes)
            .tweak(tweak)
            .into(),
    );
    if let Some(family) = fonts
        .families
        .get_mut(&eframe::egui::FontFamily::Proportional)
    {
        family.insert(0, name.clone());
    }
    if let Some(family) = fonts.families.get_mut(&eframe::egui::FontFamily::Monospace) {
        family.push(name);
    }
    ctx.set_fonts(fonts);
}

fn load_system_cjk_font_bytes() -> Option<Vec<u8>> {
    let mut db = Database::new();
    db.load_system_fonts();
    let families = [
        "Yu Gothic",
        "Meiryo",
        "MS Gothic",
        "Hiragino Sans",
        "Noto Sans CJK JP",
        "Noto Sans JP",
    ];
    families.iter().find_map(|name| {
        let query = Query {
            families: &[Family::Name(name)],
            ..Query::default()
        };
        db.query(&query)
            .and_then(|id| db.with_face_data(id, |data, _| data.to_vec()))
    })
}
