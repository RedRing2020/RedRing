//! 画面の日本語の文字（OS にある CJK のフォントを使う）

use fontdb::{Database, Family, Query};

/// OS にある CJK のフォントを、egui の文字に加える（見つからなければ既定のまま）
pub(crate) fn install_cjk_font(ctx: &eframe::egui::Context) {
    let Some(bytes) = load_system_cjk_font_bytes() else {
        eprintln!("no CJK system font was found; non-ASCII text may not be shown");
        return;
    };
    let mut fonts = eframe::egui::FontDefinitions::default();
    let name = "system_cjk".to_owned();
    fonts.font_data.insert(
        name.clone(),
        eframe::egui::FontData::from_owned(bytes).into(),
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
