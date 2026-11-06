use crate::custom::{CustomPass, Placed};
use crate::lcd3x::ScreenEffect;
use crate::power::{screen_brightness, screen_rect_above};
use crate::quad::Quad;
use crate::shaders::{GAME_FRAG, RECT_VERT};
use crate::surface::{GfxError, OUT_H, OUT_W};

pub const SCALE: u32 = 3;
pub const SRC_W: u32 = OUT_W / SCALE;
pub const SRC_H: u32 = OUT_H / SCALE;

pub const WHOLE_TEXTURE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

pub struct GamePass {
    prog: gl::types::GLuint,
    game: gl::types::GLuint,
    /// The size the game texture was made at, which follows the frames uploaded into it.
    texels: (u32, u32),
    u_src: gl::types::GLint,
    /// How wide the picture should look against its height, or None to fill the screen rect
    /// the way the Game Boy family's 240x160 canvas does.
    aspect: Option<f32>,
    /// Quarter turns counter-clockwise the picture is drawn at.
    rotation: u32,
    u_rot: gl::types::GLint,
    /// The colour correction: a 3x3, a black point, and the gamma it runs under. Identity, zero
    /// and 1.0 until something asks for a look, which leaves the picture as the core drew it.
    u_cc: gl::types::GLint,
    u_cc_bias: gl::types::GLint,
    u_cc_gamma: gl::types::GLint,
    u_pal_on: gl::types::GLint,
    u_pix_on: gl::types::GLint,
    /// Two lookup tables: the machine's palette (256x1, read by the picture's own grey) and the
    /// pixel-art cube (1024x32). Both start as one clear texel, which is what keeps them off.
    pal: gl::types::GLuint,
    pix: gl::types::GLuint,
    /// What the shader is to do to the picture's colour: the 3x3, the black point, the gamma it
    /// runs under, and which of the two tables it is to read (never both). Defaults to nothing.
    cc: [[f32; 3]; 3],
    bias: [f32; 3],
    gamma: f32,
    pal_on: bool,
    pix_on: bool,
    u_rect: gl::types::GLint,
    u_bright: gl::types::GLint,
    u_uv: gl::types::GLint,
    u_mode: gl::types::GLint,
    u_paper_size: gl::types::GLint,
    u_fbo: gl::types::GLint,
    fbo_scale: [f32; 2],
    u_origin: gl::types::GLint,
    origin: [f32; 2],
    u_pic: gl::types::GLint,
    pic: [f32; 4],
    paper: gl::types::GLuint,
    paper_size: f32,
    effect: ScreenEffect,
    /// A shader from the card drawn in place of `effect`.
    custom: Option<CustomPass>,
    power: f32,
    fills: bool,
    /// Whether the picture is drawn a whole number of times its own size, and so is never
    /// resampled: smaller than the room it was given by whatever is left over, and sharp.
    integer: bool,
    /// The size of a still being drawn instead of the running picture (`Draw::Shot`), for as long
    /// as it is being drawn: a photograph is scaled by its own size, not by the core's frame.
    still: Option<(u32, u32)>,
    /// Pixels kept for the carousel footer. The picture scales in the space above them.
    bottom: f32,
    /// Whether the picture fills the viewport, as it does in an overlay's window.
    whole: bool,
    src: [f32; 4],
}

impl GamePass {
    pub fn new() -> Result<Self, GfxError> {
        let prog = crate::shaders::program(RECT_VERT, GAME_FRAG)?;
        let game = crate::gl::texture(SRC_W, SRC_H, gl::NEAREST, gl::CLAMP_TO_EDGE, gl::BGRA, None);
        let paper = crate::gl::texture(
            1,
            1,
            gl::NEAREST,
            gl::REPEAT,
            crate::gl::gray_format(),
            Some(&[255]),
        );
        let pal = crate::gl::texture(
            1,
            1,
            gl::NEAREST,
            gl::CLAMP_TO_EDGE,
            gl::RGBA,
            Some(&[0u8, 0, 0, 0]),
        );
        let pix = crate::gl::texture(
            1,
            1,
            gl::NEAREST,
            gl::CLAMP_TO_EDGE,
            gl::RGBA,
            Some(&[0u8, 0, 0, 0]),
        );
        let (
            u_rect,
            u_bright,
            u_uv,
            u_mode,
            u_paper_size,
            u_pic,
            u_fbo,
            u_origin,
            u_src,
            u_rot,
            u_cc,
            u_cc_bias,
            u_cc_gamma,
            u_pal_on,
            u_pix_on,
        );
        unsafe {
            gl::UseProgram(prog);
            gl::Uniform1i(crate::gl::uniform_location(prog, "u_game"), 0);
            gl::Uniform1i(crate::gl::uniform_location(prog, "u_pal"), 2);
            gl::Uniform1i(crate::gl::uniform_location(prog, "u_pix"), 3);
            u_src = crate::gl::uniform_location(prog, "u_src");
            u_rot = crate::gl::uniform_location(prog, "u_rot");
            u_cc = crate::gl::uniform_location(prog, "u_cc");
            u_cc_bias = crate::gl::uniform_location(prog, "u_cc_bias");
            u_cc_gamma = crate::gl::uniform_location(prog, "u_cc_gamma");
            u_pal_on = crate::gl::uniform_location(prog, "u_pal_on");
            u_pix_on = crate::gl::uniform_location(prog, "u_pix_on");
            gl::Uniform2f(
                crate::gl::uniform_location(prog, "u_target"),
                OUT_W as f32,
                OUT_H as f32,
            );
            u_rect = crate::gl::uniform_location(prog, "u_rect");
            u_bright = crate::gl::uniform_location(prog, "u_bright");
            u_uv = crate::gl::uniform_location(prog, "u_uv");
            gl::Uniform1i(crate::gl::uniform_location(prog, "u_paper"), 1);
            u_mode = crate::gl::uniform_location(prog, "u_mode");
            u_paper_size = crate::gl::uniform_location(prog, "u_paper_size");
            u_fbo = crate::gl::uniform_location(prog, "u_fbo");
            u_pic = crate::gl::uniform_location(prog, "u_pic");
            u_origin = crate::gl::uniform_location(prog, "u_origin");
        }
        Ok(GamePass {
            prog,
            game,
            texels: (SRC_W, SRC_H),
            u_src,
            aspect: None,
            rotation: 0,
            u_rot,
            u_cc,
            u_cc_bias,
            u_cc_gamma,
            u_pal_on,
            u_pix_on,
            pal,
            pix,
            cc: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            bias: [0.0; 3],
            gamma: 1.0,
            pal_on: false,
            pix_on: false,
            u_rect,
            u_bright,
            u_uv,
            u_mode,
            u_paper_size,
            u_fbo,
            fbo_scale: [1.0, 1.0],
            u_origin,
            origin: [0.0, 0.0],
            u_pic,
            pic: WHOLE_TEXTURE,
            paper,
            paper_size: 1.0,
            effect: ScreenEffect::Lcd3x,
            custom: None,
            power: 1.0,
            fills: false,
            integer: false,
            still: None,
            bottom: 0.0,
            whole: false,
            src: WHOLE_TEXTURE,
        })
    }

    pub fn set_power(&mut self, t: f32) {
        self.power = t.clamp(0.0, 1.0);
    }

    pub fn set_bottom(&mut self, px: f32) {
        self.bottom = px.max(0.0);
    }

    pub fn frame(&self) -> (f32, f32, f32, f32) {
        self.placed()
    }

    fn placed(&self) -> (f32, f32, f32, f32) {
        if self.whole {
            return (0.0, 0.0, OUT_W as f32, OUT_H as f32);
        }
        let foot = self.bottom;
        let rect = match self.fills {
            true => (0.0, 0.0, OUT_W as f32, OUT_H as f32 - foot),
            false => screen_rect_above(self.power, foot),
        };
        let rect = match self.aspect {
            Some(aspect) => fit_aspect(rect, aspect),
            None => rect,
        };
        match self.integer {
            true => whole_fit(rect, self.picture_size()),
            false => rect,
        }
    }

    /// How big the drawn part of the frame is in the screen's own pixels at one times: the part
    /// `src` names, not all the core handed us.
    fn picture_size(&self) -> (f32, f32) {
        match self.still {
            Some((w, h)) => (w as f32, h as f32),
            None => (
                self.src[2] * self.texels.0 as f32,
                self.src[3] * self.texels.1 as f32,
            ),
        }
    }

    pub fn set_rotation(&mut self, quarter_turns: u32) {
        self.rotation = quarter_turns % 4;
    }

    pub fn set_aspect(&mut self, aspect: Option<f32>) {
        self.aspect = aspect.filter(|a| a.is_finite() && *a > 0.0);
    }

    pub fn set_whole(&mut self, on: bool) {
        self.whole = on;
    }

    pub fn set_fills_frame(&mut self, on: bool) {
        self.fills = on;
    }

    /// Whether the picture is drawn a whole number of times its own size: sharp, and smaller than
    /// the room it was given by whatever is left over.
    pub fn set_integer(&mut self, on: bool) {
        self.integer = on;
    }

    /// What the picture is being scaled by, for the moment a still stands in for it.
    pub fn set_still(&mut self, at: Option<(u32, u32)>) {
        self.still = at;
    }

    /// What the picture is being held to, and whether it is held to a whole number of times its
    /// own size: both are read so that a still can put them back when it has been drawn.
    pub fn aspect(&self) -> Option<f32> {
        self.aspect
    }

    pub fn integer(&self) -> bool {
        self.integer
    }

    pub fn set_picture(&mut self, rect: [f32; 4]) {
        self.pic = rect;
    }

    pub fn set_fbo_scale(&mut self, x: f32, y: f32) {
        self.fbo_scale = [x, y];
    }

    pub fn set_origin(&mut self, x: f32, y: f32) {
        self.origin = [x, y];
    }

    pub fn set_effect(&mut self, effect: ScreenEffect) {
        self.effect = effect;
    }

    /// Draws the game with a RetroArch shader's source, or the built-in effect for None. A
    /// shader that does not build leaves the built-in effect.
    pub fn set_custom_shader(&mut self, src: Option<&str>, linear: bool) -> Result<(), GfxError> {
        self.custom = None;
        if let Some(src) = src {
            self.custom = Some(CustomPass::new(src, linear)?);
        }
        Ok(())
    }

    pub fn set_paper(&mut self, size: u32, gray: &[u8]) {
        let tex = crate::gl::texture(
            size,
            size,
            gl::NEAREST,
            gl::REPEAT,
            crate::gl::gray_format(),
            Some(gray),
        );
        unsafe { gl::DeleteTextures(1, &self.paper) };
        self.paper = tex;
        self.paper_size = size as f32;
    }

    pub fn set_source_rect(&mut self, rect: [f32; 4]) {
        self.src = rect;
    }

    pub fn upload(&mut self, xrgb8888: &[u8], (w, h): (u32, u32)) {
        if w == 0 || h == 0 || xrgb8888.len() < (w * h * 4) as usize {
            return;
        }
        if self.texels != (w, h) {
            let game = crate::gl::texture(w, h, gl::NEAREST, gl::CLAMP_TO_EDGE, gl::BGRA, None);
            unsafe { gl::DeleteTextures(1, &self.game) };
            self.game = game;
            self.texels = (w, h);
        }
        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, self.game);
            gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                0,
                0,
                w as i32,
                h as i32,
                gl::BGRA,
                gl::UNSIGNED_BYTE,
                xrgb8888.as_ptr() as *const std::ffi::c_void,
            );
        }
    }

    pub fn draw(&self, quad: &Quad) {
        let src = match self.effect {
            // The whole-texel fit pairs the texture's width with the screen's, which a quarter
            // turn swaps, so a turned picture keeps the plain source.
            ScreenEffect::Grid if self.rotation.is_multiple_of(2) => {
                whole_scale_source(self.src, self.fbo_scale, self.texels)
            }
            _ => self.src,
        };
        self.draw_source(self.game, quad, src);
    }

    pub fn draw_still(&self, tex: gl::types::GLuint, quad: &Quad) {
        self.draw_source(tex, quad, WHOLE_TEXTURE);
    }

    fn draw_source(&self, tex: gl::types::GLuint, quad: &Quad, src: [f32; 4]) {
        let (x, y, w, h) = self.placed();
        let bright = if self.fills {
            1.0
        } else {
            screen_brightness(self.power)
        };
        // The footer covers the bottom of a full-frame picture. Keep the part that sat above
        // it, so opening the scale does not squash the picture into the shorter rect.
        let mut src = src;
        if self.bottom > 0.0 && !self.whole {
            src[3] *= (OUT_H as f32 - self.bottom) / OUT_H as f32;
        }
        if let Some(custom) = &self.custom {
            let placed = Placed {
                rect: (x, y, w, h),
                src,
                texels: self.texels,
                rotation: self.rotation,
                fbo_scale: self.fbo_scale,
            };
            custom.draw(tex, &placed);
            return;
        }
        unsafe {
            gl::UseProgram(self.prog);
            gl::Uniform2f(self.u_src, self.texels.0 as f32, self.texels.1 as f32);
            gl::Uniform1f(self.u_rot, self.rotation as f32);
            gl::Uniform4f(self.u_rect, x, y, w, h);
            gl::Uniform4f(self.u_uv, src[0], src[1], src[2], src[3]);
            gl::Uniform1f(self.u_bright, bright);
            gl::Uniform1f(self.u_mode, self.effect.mode());
            gl::Uniform1f(self.u_paper_size, self.paper_size);
            gl::Uniform2f(self.u_fbo, self.fbo_scale[0], self.fbo_scale[1]);
            gl::Uniform2f(self.u_origin, self.origin[0], self.origin[1]);
            gl::Uniform4f(
                self.u_pic,
                self.pic[0],
                self.pic[1],
                self.pic[2],
                self.pic[3],
            );
            // Row-major in, and the driver transposes it: what the shader then multiplies with is
            // the matrix as written above.
            gl::UniformMatrix3fv(self.u_cc, 1, gl::TRUE, self.cc.as_ptr() as *const f32);
            gl::Uniform3f(self.u_cc_bias, self.bias[0], self.bias[1], self.bias[2]);
            gl::Uniform1f(self.u_cc_gamma, self.gamma);
            gl::Uniform1f(self.u_pal_on, if self.pal_on { 1.0 } else { 0.0 });
            gl::Uniform1f(self.u_pix_on, if self.pix_on { 1.0 } else { 0.0 });
            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, self.paper);
            gl::ActiveTexture(gl::TEXTURE2);
            gl::BindTexture(gl::TEXTURE_2D, self.pal);
            gl::ActiveTexture(gl::TEXTURE3);
            gl::BindTexture(gl::TEXTURE_2D, self.pix);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, tex);
        }
        quad.draw();
    }

    /// What the picture's colour goes through: a 3x3 in `gamma` space with a black point added
    /// after it, and neither table. Identity, zero and 1.0 is the default and changes nothing.
    pub fn set_colour(&mut self, cc: [[f32; 3]; 3], bias: [f32; 3], gamma: f32) {
        self.cc = cc;
        self.bias = bias;
        self.gamma = gamma.max(0.01);
        self.pal_on = false;
        self.pix_on = false;
    }

    /// A greyscale machine's palette, as 256 RGBA texels read by the picture's own grey. One row
    /// is all it takes: the four shades are interpolated across it on the CPU, so the shader is
    /// left with a single lookup and nothing to work out.
    pub fn set_palette_lut(&mut self, rgba: &[u8]) {
        if rgba.len() < 256 * 4 {
            return;
        }
        unsafe {
            gl::DeleteTextures(1, &self.pal);
            self.pal = crate::gl::texture(
                256,
                1,
                gl::NEAREST,
                gl::CLAMP_TO_EDGE,
                gl::RGBA,
                Some(&rgba[..256 * 4]),
            );
        }
        self.pal_on = true;
        self.pix_on = false;
    }

    /// The pixel-art cube: 1024x32 RGBA, `x = r + 32*g`, `y = b` — five bits a channel, which is
    /// every colour the machine can make.
    pub fn set_pix_lut(&mut self, rgba: &[u8]) {
        if rgba.len() < 1024 * 32 * 4 {
            return;
        }
        unsafe {
            gl::DeleteTextures(1, &self.pix);
            self.pix = crate::gl::texture(
                1024,
                32,
                gl::NEAREST,
                gl::CLAMP_TO_EDGE,
                gl::RGBA,
                Some(&rgba[..1024 * 32 * 4]),
            );
        }
        self.pix_on = true;
        self.pal_on = false;
    }
}

fn whole_scale_source(src: [f32; 4], fbo: [f32; 2], texels: (u32, u32)) -> [f32; 4] {
    let (x, w) = whole_scale(src[0], src[2], texels.0 as f32, OUT_W as f32 * fbo[0]);
    let (y, h) = whole_scale(src[1], src[3], texels.1 as f32, OUT_H as f32 * fbo[1]);
    [x, y, w, h]
}

/// The largest rect of `aspect` centred in `rect`, as (x, y, w, h).
pub fn fit_aspect((x, y, w, h): (f32, f32, f32, f32), aspect: f32) -> (f32, f32, f32, f32) {
    if h <= 0.0 || w <= 0.0 {
        return (x, y, w, h);
    }
    if w / h > aspect {
        let fitted = h * aspect;
        (x + (w - fitted) / 2.0, y, fitted, h)
    } else {
        let fitted = w / aspect;
        (x, y + (h - fitted) / 2.0, w, fitted)
    }
}

/// The picture at a whole number of times its own size, centred in the room it was given. No pixel
/// is drawn half a pixel wide, so nothing is resampled: what the picture loses is the odd pixel
/// or two of room that would have made it blurry.
fn whole_fit((x, y, w, h): (f32, f32, f32, f32), (pw, ph): (f32, f32)) -> (f32, f32, f32, f32) {
    if pw <= 0.0 || ph <= 0.0 {
        return (x, y, w, h);
    }
    let scale = (w / pw).min(h / ph).floor().max(1.0);
    let (nw, nh) = (pw * scale, ph * scale);
    (x + (w - nw) / 2.0, y + (h - nh) / 2.0, nw, nh)
}

fn whole_scale(at: f32, len: f32, texels: f32, out: f32) -> (f32, f32) {
    let shown = len * texels;
    let scale = (out / shown).ceil();
    let trim = scale * shown - out;
    if trim >= 0.1 * shown {
        return (at, len);
    }
    let top = (trim / 2.0).floor();
    (at + top / scale / texels, out / scale / texels)
}

impl Drop for GamePass {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.game);
            gl::DeleteTextures(1, &self.paper);
            gl::DeleteTextures(1, &self.pal);
            gl::DeleteTextures(1, &self.pix);
            gl::DeleteProgram(self.prog);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_four_by_three_picture_is_pillarboxed_on_the_three_by_two_screen() {
        let (x, y, w, h) = fit_aspect((0.0, 0.0, 720.0, 480.0), 4.0 / 3.0);
        assert_eq!((x, y, w, h), (40.0, 0.0, 640.0, 480.0));
    }

    #[test]
    fn a_wide_picture_is_letterboxed() {
        let (x, y, w, h) = fit_aspect((0.0, 0.0, 720.0, 480.0), 2.0);
        assert_eq!((x, y, w, h), (0.0, 60.0, 720.0, 360.0));
    }

    #[test]
    fn a_matching_aspect_fills_the_rect() {
        assert_eq!(
            fit_aspect((10.0, 20.0, 720.0, 480.0), 1.5),
            (10.0, 20.0, 720.0, 480.0)
        );
    }

    #[test]
    fn a_collapsed_rect_is_left_alone() {
        assert_eq!(fit_aspect((5.0, 5.0, 0.0, 0.0), 1.5), (5.0, 5.0, 0.0, 0.0));
    }
}
