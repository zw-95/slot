//! The menu in Chinese. One test on purpose: the language is one global every test shares, so
//! nothing here may run beside a test that wants English.

use slot_i18n::{set_lang, Lang};
use slot_ui::{quick_label_face, QuickRow, QuickValue, Toast};

fn ink(f: &slot_ui::UndoFace) -> usize {
    f.rgba.chunks(4).filter(|p| p[3] > 0).count()
}

#[test]
fn the_menu_speaks_the_language_the_card_asked_for() {
    // English is what unslot shipped.
    set_lang(Lang::En);
    assert_eq!(QuickRow::About.label(), "About");
    assert_eq!(QuickValue::On.text(), "On");
    assert_eq!(Toast::StateSaved.text(), "State Saved");

    set_lang(Lang::ZhCn);
    assert_eq!(QuickRow::About.label(), "关于");
    assert_eq!(QuickRow::Language.label(), "语言");
    assert_eq!(QuickRow::Network.label(), "网络");
    assert_eq!(QuickValue::On.text(), "开");
    assert_eq!(QuickValue::LangEn.text(), "English");
    assert_eq!(QuickValue::LangZh.text(), "简体中文");
    assert_eq!(Toast::StateSaved.text(), "存档已保存");
    assert_eq!(
        Toast::CardUnwritable.detail(),
        Some("进度不会被保存"),
        "the alarm's second line is translated too"
    );

    // The Chinese rows are drawn, not rastered to nothing: the font has these glyphs.
    for row in [QuickRow::General, QuickRow::Network, QuickRow::Language] {
        let face = quick_label_face(row, true);
        assert!(ink(&face) > 0, "{row:?} 一个字都没画出来");
    }

    set_lang(Lang::En);
}
