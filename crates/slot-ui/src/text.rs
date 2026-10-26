use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use fontdue::{Font, FontSettings, LineMetrics, Metrics};

const LABEL_TTF: &[u8] = include_bytes!("../assets/label.ttf");
const LABEL_CJK_TTF: &[u8] = include_bytes!("../assets/label_cjk.ttf");

/// 字形缓存上限，超出后整表清空：中文长文本下不至于无上限增长。
const GLYPH_CACHE_LIMIT: usize = 8192;

/// 栅格化结果（度量 + 覆盖图）。缓存里存 `Arc`，取用时不复制像素。
type Glyph = Arc<(Metrics, Vec<u8>)>;
type GlyphCache = Mutex<HashMap<(usize, char, u32), Glyph>>;

pub struct Layout {
    pub lines: Vec<String>,
    pub px: f32,
    pub tracking: f32,
}

pub fn label_font() -> Option<&'static Font> {
    static FONT: OnceLock<Option<Font>> = OnceLock::new();
    FONT.get_or_init(|| Font::from_bytes(LABEL_TTF, FontSettings::default()).ok())
        .as_ref()
}

/// 中文字体。`label.ttf` 只有拉丁字形，汉字、假名和全角标点都由它补上。
pub fn cjk_font() -> Option<&'static Font> {
    static FONT: OnceLock<Option<Font>> = OnceLock::new();
    FONT.get_or_init(|| Font::from_bytes(LABEL_CJK_TTF, FontSettings::default()).ok())
        .as_ref()
}

/// 挑能画出 `ch` 的字体：主字体缺这个字形就退到中文字体。
fn pick(primary: &Font, ch: char) -> &Font {
    if primary.lookup_glyph_index(ch) != 0 {
        return primary;
    }
    match cjk_font() {
        Some(f) if f.lookup_glyph_index(ch) != 0 => f,
        _ => primary,
    }
}

/// 逐字栅格化在中文下很贵，同一个（字体, 字符, 字号）只算一次。
fn glyph_cache() -> &'static GlyphCache {
    static CACHE: OnceLock<GlyphCache> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn rasterized(font: &Font, ch: char, px: f32) -> Glyph {
    let key = (font as *const Font as usize, ch, px.to_bits());
    if let Some(hit) = glyph_cache().lock().unwrap().get(&key) {
        return hit.clone();
    }
    let value = Arc::new(font.rasterize(ch, px));
    let mut cache = glyph_cache().lock().unwrap();
    if cache.len() >= GLYPH_CACHE_LIMIT {
        cache.clear();
    }
    cache.insert(key, value.clone());
    value
}

/// 汉字、假名、全角之类：没有词间空格，得逐字断行。
fn is_cjk(ch: char) -> bool {
    matches!(ch,
        '\u{1100}'..='\u{11FF}'   // 谚文字母
        | '\u{2E80}'..='\u{2EFF}' // CJK 部首补充
        | '\u{2F00}'..='\u{2FDF}' // 康熙部首
        | '\u{3000}'..='\u{303F}' // CJK 标点
        | '\u{3040}'..='\u{30FF}' // 假名
        | '\u{3100}'..='\u{312F}' // 注音
        | '\u{3130}'..='\u{318F}' // 谚文兼容字母
        | '\u{31C0}'..='\u{31EF}'
        | '\u{31F0}'..='\u{31FF}'
        | '\u{3400}'..='\u{4DBF}' // CJK 扩展 A
        | '\u{4E00}'..='\u{9FFF}' // 主汉字区
        | '\u{AC00}'..='\u{D7A3}' // 谚文音节
        | '\u{F900}'..='\u{FAFF}' // 兼容汉字
        | '\u{FE30}'..='\u{FE4F}' // 兼容形式
        | '\u{FF01}'..='\u{FF60}' // 全角形式
        | '\u{FFE0}'..='\u{FFE6}')
}

/// 只把 ASCII 字母转大写。原来的 `to_uppercase()` 会改写带声调的拉丁字母、也可能改变长度。
fn upper_ascii(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        out.push(if ch.is_ascii() {
            ch.to_ascii_uppercase()
        } else {
            ch
        });
    }
    out
}

pub fn fit(
    font: &Font,
    text: &str,
    max_w: f32,
    max_lines: usize,
    max_px: f32,
    min_px: f32,
) -> Layout {
    fit_box(font, text, max_w, f32::INFINITY, max_lines, max_px, min_px)
}

#[allow(clippy::too_many_arguments)]
pub fn fit_box(
    font: &Font,
    text: &str,
    max_w: f32,
    max_h: f32,
    max_lines: usize,
    max_px: f32,
    min_px: f32,
) -> Layout {
    let upper = upper_ascii(text);
    let mut px = max_px;
    while px > min_px {
        let tracking = tracking_for(px);
        let lines = wrap(font, &upper, px, tracking, max_w, false);
        if lines.len() <= max_lines
            && lines
                .iter()
                .all(|l| line_width(font, l, px, tracking) <= max_w)
            && block_height(font, px, &lines) <= max_h
        {
            return Layout {
                lines,
                px,
                tracking,
            };
        }
        px -= 1.0;
    }
    let tracking = tracking_for(min_px);
    let mut lines = wrap(font, &upper, min_px, tracking, max_w, true);
    if lines.len() > max_lines {
        // 装不下就在末行加省略号，而不是静默丢掉后面的行。
        lines.truncate(max_lines);
        if let Some(last) = lines.last_mut() {
            *last = ellipsize(font, last, min_px, tracking, max_w);
        }
    }
    Layout {
        lines,
        px: min_px,
        tracking,
    }
}

fn ellipsize(font: &Font, line: &str, px: f32, tracking: f32, max_w: f32) -> String {
    let mut out = format!("{line}\u{2026}");
    while out.chars().count() > 1 && line_width(font, &out, px, tracking) > max_w {
        let keep = out.chars().count() - 2;
        let mut trimmed: String = out.chars().take(keep).collect();
        trimmed.push('\u{2026}');
        out = trimmed;
    }
    out
}

/// One line of `text` as written, for where case matters: what someone typed. Shrinks from
/// `max_px` until it fits `max_w`, down to `min_px`.
pub fn fit_as_written(font: &Font, text: &str, max_w: f32, max_px: f32, min_px: f32) -> Layout {
    let mut px = max_px;
    while px > min_px && line_width(font, text, px, tracking_for(px)) > max_w {
        px -= 1.0;
    }
    Layout {
        lines: vec![text.to_string()],
        px,
        tracking: tracking_for(px),
    }
}

pub fn draw_centred(dst: &mut [u8], dst_w: u32, dst_h: u32, layout: &Layout, colour: [u8; 3]) {
    for (i, a) in coverage(dst_w, dst_h, layout).into_iter().enumerate() {
        if a > 0 {
            blend(&mut dst[i * 4..i * 4 + 4], a as u32, colour);
        }
    }
}

pub fn coverage(dst_w: u32, dst_h: u32, layout: &Layout) -> Vec<u8> {
    let mut out = vec![0u8; (dst_w * dst_h) as usize];
    let Some(font) = label_font() else { return out };
    let block_h = block_height(font, layout.px, &layout.lines);
    let mut top = (dst_h as f32 - block_h) / 2.0;

    for line in &layout.lines {
        let Some(vm) = line_metrics(font, line, layout.px) else {
            continue;
        };
        let baseline = top + vm.ascent;
        top += vm.new_line_size;
        let mut pen = (dst_w as f32 - line_width(font, line, layout.px, layout.tracking)) / 2.0;
        for ch in line.chars() {
            let f = pick(font, ch);
            let (m, cov) = &*rasterized(f, ch, layout.px);
            stamp(
                &mut out,
                dst_w,
                dst_h,
                (pen + m.xmin as f32).round() as i32,
                (baseline - (m.height as f32 + m.ymin as f32)).round() as i32,
                cov,
                m.width as u32,
                m.height as u32,
            );
            pen += m.advance_width + layout.tracking;
        }
    }
    out
}

/// 一行的行高取自实际用上的字体：含中日韩字符时用两套字体里更大的那个，避免汉字被切。
fn line_metrics(font: &Font, line: &str, px: f32) -> Option<LineMetrics> {
    let primary = font.horizontal_line_metrics(px)?;
    if !line.chars().any(is_cjk) {
        return Some(primary);
    }
    let Some(cjk) = cjk_font().and_then(|f| f.horizontal_line_metrics(px)) else {
        return Some(primary);
    };
    let ascent = primary.ascent.max(cjk.ascent);
    let descent = primary.descent.min(cjk.descent);
    let line_gap = primary.line_gap.max(cjk.line_gap);
    Some(LineMetrics {
        ascent,
        descent,
        line_gap,
        new_line_size: ascent - descent + line_gap,
    })
}

fn block_height(font: &Font, px: f32, lines: &[String]) -> f32 {
    lines
        .iter()
        .map(|l| line_metrics(font, l, px).map_or(0.0, |vm| vm.new_line_size))
        .sum()
}

pub fn line_width(font: &Font, line: &str, px: f32, tracking: f32) -> f32 {
    let mut w = 0.0;
    let mut n = 0;
    for ch in line.chars() {
        w += pick(font, ch).metrics(ch, px).advance_width;
        n += 1;
    }
    w + tracking * (n as f32 - 1.0).max(0.0)
}

fn tracking_for(px: f32) -> f32 {
    (px * 0.10).round()
}

fn wrap(
    font: &Font,
    text: &str,
    px: f32,
    tracking: f32,
    max_w: f32,
    break_words: bool,
) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();
    for word in tokens(text) {
        let parts = if break_words {
            break_word(font, &word, px, tracking, max_w)
        } else {
            vec![word]
        };
        for part in parts {
            if line.is_empty() {
                line = part;
                continue;
            }
            let joined = if needs_space(&line, &part) {
                format!("{line} {part}")
            } else {
                format!("{line}{part}")
            };
            // 行首禁则：收尾标点不另起一行，宁可略微出界。
            if line_width(font, &joined, px, tracking) <= max_w || starts_forbidden(&part) {
                line = joined;
            } else {
                lines.push(std::mem::take(&mut line));
                line = part;
            }
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

/// 把文本切成可断行的块：拉丁词整块，中日韩字符各成一块（这样它们之间不会被塞空格）。
fn tokens(text: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut word = String::new();
    for ch in text.chars() {
        if ch.is_whitespace() {
            if !word.is_empty() {
                out.push(std::mem::take(&mut word));
            }
        } else if is_cjk(ch) {
            if !word.is_empty() {
                out.push(std::mem::take(&mut word));
            }
            out.push(ch.to_string());
        } else {
            word.push(ch);
        }
    }
    if !word.is_empty() {
        out.push(word);
    }
    out
}

fn needs_space(left: &str, right: &str) -> bool {
    match (left.chars().next_back(), right.chars().next()) {
        (Some(a), Some(b)) => !is_cjk(a) && !is_cjk(b) && !a.is_whitespace() && !b.is_whitespace(),
        _ => false,
    }
}

fn starts_forbidden(part: &str) -> bool {
    matches!(
        part.chars().next(),
        Some(
            '，' | '。'
                | '、'
                | '；'
                | '：'
                | '？'
                | '！'
                | '）'
                | '】'
                | '》'
                | '」'
                | '』'
                | '”'
                | '’'
                | '…'
                | '·'
                | '%'
                | ','
                | '.'
                | ';'
                | ':'
                | '?'
                | '!'
                | ')'
                | ']'
        )
    )
}

fn break_word(font: &Font, word: &str, px: f32, tracking: f32, max_w: f32) -> Vec<String> {
    if line_width(font, word, px, tracking) <= max_w {
        return vec![word.to_string()];
    }
    let mut parts = Vec::new();
    let mut part = String::new();
    for ch in word.chars() {
        part.push(ch);
        if line_width(font, &part, px, tracking) > max_w && part.chars().count() > 1 {
            part.pop();
            parts.push(std::mem::take(&mut part));
            part.push(ch);
        }
    }
    if !part.is_empty() {
        parts.push(part);
    }
    parts
}

#[allow(clippy::too_many_arguments)]
fn stamp(dst: &mut [u8], dst_w: u32, dst_h: u32, x: i32, y: i32, cov: &[u8], gw: u32, gh: u32) {
    for gy in 0..gh {
        let dy = y + gy as i32;
        if dy < 0 || dy >= dst_h as i32 {
            continue;
        }
        for gx in 0..gw {
            let dx = x + gx as i32;
            if dx < 0 || dx >= dst_w as i32 {
                continue;
            }
            let i = (dy as u32 * dst_w + dx as u32) as usize;
            dst[i] = dst[i].max(cov[(gy * gw + gx) as usize]);
        }
    }
}

fn blend(px: &mut [u8], a: u32, colour: [u8; 3]) {
    let under = px[3] as u32 * (255 - a) / 255;
    let out = a + under;
    for c in 0..3 {
        px[c] = ((colour[c] as u32 * a + px[c] as u32 * under + out / 2) / out) as u8;
    }
    px[3] = out as u8;
}
