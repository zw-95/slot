use crate::surface::GfxError;

const VERT_PREAMBLE: &str = "#version 330 core\n#define attribute in\n#define varying out\n";

const FRAG_PREAMBLE: &str = "#version 330 core\n#define varying in\n\
                             #define texture2D texture\nout vec4 FRAG_COLOR;\n";

const VERT_PREAMBLE_ES: &str = "";
const FRAG_PREAMBLE_ES: &str = "#define FRAG_COLOR gl_FragColor\n";

pub fn program(vert: &str, frag: &str) -> Result<gl::types::GLuint, GfxError> {
    let (vp, fp) = match crate::gl::es() {
        true => (VERT_PREAMBLE_ES, FRAG_PREAMBLE_ES),
        false => (VERT_PREAMBLE, FRAG_PREAMBLE),
    };
    crate::gl::program(&format!("{vp}{vert}"), &format!("{fp}{frag}"))
}

pub const RECT_VERT: &str = r#"
attribute vec2 a_pos;
uniform vec4 u_rect;
uniform vec2 u_target;
varying vec2 v_uv;
void main() {
    v_uv = a_pos;
    vec2 p = (u_rect.xy + a_pos * u_rect.zw) / u_target;
    gl_Position = vec4(p.x * 2.0 - 1.0, 1.0 - p.y * 2.0, 0.0, 1.0);
}
"#;

pub const SPRITE_VERT: &str = r#"
attribute vec2 a_pos;
uniform vec4 u_rect;
uniform vec2 u_target;
uniform vec2 u_turn;
uniform vec4 u_src;
varying vec2 v_uv;
void main() {
    v_uv = u_src.xy + a_pos * u_src.zw;
    vec2 mid = u_rect.zw * 0.5;
    vec2 local = a_pos * u_rect.zw - mid;
    vec2 turned = vec2(u_turn.x * local.x - u_turn.y * local.y,
                       u_turn.y * local.x + u_turn.x * local.y);
    vec2 p = (u_rect.xy + a_pos * u_rect.zw + (turned - local)) / u_target;
    gl_Position = vec4(p.x * 2.0 - 1.0, 1.0 - p.y * 2.0, 0.0, 1.0);
}
"#;

pub const GAME_FRAG: &str = r#"
#ifdef GL_FRAGMENT_PRECISION_HIGH
precision highp float;
#else
precision mediump float;
#endif
uniform sampler2D u_game;
uniform sampler2D u_paper;
uniform vec2 u_src;
uniform vec4 u_uv;
uniform float u_bright;
uniform float u_mode;
uniform float u_paper_size;
uniform vec2 u_origin;
uniform vec4 u_rect;
uniform vec2 u_fbo;
uniform vec4 u_pic;
uniform float u_rot;
// 色彩校正：一个 3x3 加一个黑点偏移，在 `u_cc_gamma` 指定的空间里做。
// 黑点是**实机色板需要的**——DMG 最亮档不是白、最暗档也不是黑（#B8F878 到 #102000），
// 只靠一个 3x3 只能做「黑 → 某色」的斜率，做不出两点映射。有了它，一个色板就是
// 「白点 - 黑点」的斜率加一个偏移，四个灰阶能落在实机那四个颜色上。
uniform mat3 u_cc;
uniform vec3 u_cc_bias;
uniform float u_cc_gamma;
// 实机色板查找表：256×1 的 RGBA，按画面灰度直接取色（见 `palette_lut`）。
uniform sampler2D u_pal;
uniform float u_pal_on;
// 像素画色板/色调映射：32^3 的立方平铺成 1024×32，x = r + 32*g，y = b。
uniform sampler2D u_pix;
uniform float u_pix_on;
varying vec2 v_uv;
// Where on the texture a point of the screen rect reads, for a picture turned u_rot quarter
// turns counter-clockwise. Both run top down.
vec2 turned(vec2 s) {
    if (u_rot > 2.5) return vec2(s.y, 1.0 - s.x);
    if (u_rot > 1.5) return vec2(1.0 - s.x, 1.0 - s.y);
    if (u_rot > 0.5) return vec2(1.0 - s.y, s.x);
    return s;
}
// 4×4 有序抖动（Bayer）。两次 2×2 用算术叠出来，避开 GLSL ES 1.00 没有的位运算与数组索引。
float Bayer2(vec2 a) {
    vec2 f = floor(a);
    return fract(f.x * 0.5 + f.y * f.y * 0.75);
}
float Bayer4(vec2 a) {
    return Bayer2(a * 0.5) * 0.25 + Bayer2(a);
}
const vec3 offsets = vec3(3.141592654) * vec3(1.0 / 2.0, 1.0 / 2.0 - 2.0 / 3.0, 1.0 / 2.0 - 4.0 / 3.0);
const vec3 luma_coeff = vec3(0.2126, 0.7152, 0.0722);
vec2 grid_band(vec2 x) {
    return floor(x) / 3.0 + min(fract(x), vec2(1.0 / 3.0));
}
float dot_weight(vec2 f, vec2 o, float bloom) {
    return exp(-4.0 * length(f - (o + vec2(0.5))) * bloom);
}
void main() {
    vec2 uv = u_uv.xy + turned(v_uv) * u_uv.zw;
    vec2 f = fract(uv * u_src);
    vec3 c = texture2D(u_game, uv).rgb;
    // 色彩，三选一，在网格与纸纹之前做：那两者是面板自己的东西，不该被画面的调色带着走。
    // 默认值（gamma 1.0、单位矩阵、零偏移、两条查找表都关）合起来是恒等运算，画面一字不改。
    if (u_pix_on > 0.5) {
        // 抖动 ±半个 5-bit 步，吸附到 32 级，再查立方。
        float bayer = Bayer4(floor(gl_FragCoord.xy)) - 0.5;
        vec3 q = clamp(c + bayer / 31.0, 0.0, 1.0);
        vec3 lv = floor(q * 31.0 + 0.5);
        c = texture2D(u_pix, vec2((lv.x + 32.0 * lv.y + 0.5) / 1024.0, (lv.z + 0.5) / 32.0)).rgb;
    } else if (u_pal_on > 0.5) {
        // 画面是四阶灰，色板按灰阶取色。表在 CPU 上按四档插好了，这里一次取样。
        c = texture2D(u_pal, vec2(dot(c, vec3(0.299, 0.587, 0.114)), 0.5)).rgb;
    } else {
        c = pow(c, vec3(u_cc_gamma));
        c = u_cc * c + u_cc_bias;
        c = pow(max(c, vec3(0.0)), vec3(1.0 / u_cc_gamma));
    }
    bool inside = all(greaterThanEqual(uv, u_pic.xy)) && all(lessThan(uv, u_pic.zw));
    if (!inside) {
    } else if (u_mode > 3.5) {
        c = pow(c, vec3(1.05));
        vec2 d = abs(f - 0.5);
        float x2 = max(d.x, d.y);
        x2 = x2 * x2;
        float x4 = x2 * x2;
        float w = 48.0 * (x4 - (8.0 / 3.0) * x4 * x2);
        w = w * (w + (1.0 - w) * 0.65) * 0.65;
        float luma = dot(c, luma_coeff);
        w = w * luma;
        c = mix(c, vec3(1.0), w);
        vec3 bg = vec3(texture2D(u_paper, (floor(gl_FragCoord.xy - u_origin) + 0.5) / u_paper_size).r) * c;
        c = mix(c, bg, dot(c, luma_coeff));
    } else if (u_mode > 2.5) {
        float bloom = mix(1.05, 0.95, dot(c, vec3(0.30, 0.59, 0.11)));
        float mid = dot_weight(f, vec2(0.0), bloom);
        float sum = 0.0;
        for (int j = -1; j <= 1; j++) {
            for (int i = -1; i <= 1; i++) {
                sum += dot_weight(f, vec2(float(i), float(j)), bloom);
            }
        }
        c = mix(c, mix(1.2 * c * mid, c * sum, 0.85), 0.85);
    } else if (u_mode > 1.5) {
        vec2 span = abs(u_rot - 1.0) < 0.5 || abs(u_rot - 3.0) < 0.5 ? u_rect.wz : u_rect.zw;
        vec2 hp = 0.5 * u_uv.zw * u_src / (span * u_fbo);
        vec2 edge = (grid_band(f + hp) - grid_band(f - hp)) / (2.0 * hp);
        c *= 1.0 - 0.75 * max(edge.x, edge.y);
    } else if (u_mode > 0.5) {
        vec2 angle = f * 6.283185307;
        float yfactor = (16.0 + sin(angle.y)) / 17.0;
        vec3 xfactors = (4.0 + sin(angle.x + offsets)) / 5.0;
        c *= yfactor * xfactors;
    }
    FRAG_COLOR = vec4(c * u_bright, 1.0);
}
"#;

pub const SPRITE_FRAG: &str = r#"
precision mediump float;
uniform sampler2D u_tex;
uniform vec4 u_colour;
varying vec2 v_uv;
void main() {
    FRAG_COLOR = texture2D(u_tex, v_uv) * u_colour;
}
"#;

pub const BLIT_VERT: &str = r#"
attribute vec2 a_pos;
uniform vec4 u_uv;
varying vec2 v_uv;
void main() {
    v_uv = u_uv.xy + a_pos * u_uv.zw;
    gl_Position = vec4(a_pos * 2.0 - 1.0, 0.0, 1.0);
}
"#;

pub const TURN_VERT: &str = r#"
attribute vec2 a_pos;
uniform float u_ccw;
varying vec2 v_uv;
void main() {
    vec2 cw = vec2(1.0 - a_pos.y, a_pos.x);
    vec2 ccw = vec2(a_pos.y, 1.0 - a_pos.x);
    v_uv = mix(cw, ccw, u_ccw);
    gl_Position = vec4(a_pos * 2.0 - 1.0, 0.0, 1.0);
}
"#;

pub const BLIT_FRAG: &str = r#"
precision mediump float;
uniform sampler2D u_tex;
uniform vec3 u_gain;
varying vec2 v_uv;
void main() {
    FRAG_COLOR = vec4(texture2D(u_tex, v_uv).rgb * u_gain, 1.0);
}
"#;

pub const BEZEL_FRAG: &str = r#"
precision mediump float;
uniform sampler2D u_tex;
uniform vec3 u_gain;
uniform float u_alpha;
varying vec2 v_uv;
void main() {
    vec4 c = texture2D(u_tex, v_uv);
    FRAG_COLOR = vec4(c.rgb * u_gain, c.a * u_alpha);
}
"#;
