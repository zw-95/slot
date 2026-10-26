use slot_ui::text::{coverage, fit, label_font};

fn ink_of(text: &str, max_w: f32, max_lines: usize, max_px: f32) -> (Vec<String>, u32) {
    let font = label_font().expect("label font loads");
    let layout = fit(font, text, max_w, max_lines, max_px, 8.0);
    let ink = coverage(240, 120, &layout)
        .iter()
        .filter(|a| **a > 0)
        .count() as u32;
    (layout.lines, ink)
}

#[test]
fn chinese_actually_draws_pixels() {
    let (_, ink) = ink_of("中文", 200.0, 1, 32.0);
    assert!(
        ink > 50,
        "汉字没有画出来：内置 label.ttf 缺字形时会是 0，说明中文字体没接上"
    );
}

#[test]
fn chinese_wraps_instead_of_being_dropped() {
    let (lines, _) = ink_of("塞尔达传说时之笛", 90.0, 3, 24.0);
    assert!(
        lines.len() > 1,
        "中文没有空格，若仍按空白断词就只能缩成一行：{lines:?}"
    );
    assert!(
        lines.join("").contains('笛'),
        "末尾的字被静默丢掉了：{lines:?}"
    );
}

#[test]
fn a_line_that_cannot_fit_ends_in_an_ellipsis() {
    let (lines, _) = ink_of("精灵宝可梦红宝石版", 60.0, 1, 24.0);
    assert_eq!(lines.len(), 1);
    assert!(
        lines[0].ends_with('\u{2026}'),
        "装不下时应以省略号收尾，而不是直接截断：{:?}",
        lines[0]
    );
}

#[test]
fn cjk_and_latin_sit_together_without_a_stray_space() {
    let (lines, _) = ink_of("中文 abc", 400.0, 1, 24.0);
    assert_eq!(
        lines,
        vec!["中文ABC".to_string()],
        "中日韩与拉丁之间不该多出空格"
    );
}

#[test]
fn latin_text_is_unchanged() {
    let (lines, ink) = ink_of("Fast Forward", 400.0, 1, 24.0);
    assert_eq!(lines, vec!["FAST FORWARD".to_string()]);
    assert!(ink > 50, "拉丁文案仍然要画得出来");
}
