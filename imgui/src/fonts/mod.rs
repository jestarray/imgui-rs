use crate::fonts::font::Font;
use crate::internal::RawCast;
use crate::Ui;

pub mod atlas;
mod baked;
pub mod font;
pub mod glyph;
pub mod glyph_ranges;
pub use baked::FontBaked;

/// # Fonts
impl Ui {
    /// Borrow the current frame's baked font. Glyphs returned by this view are copies.
    pub fn current_font_baked(&self) -> FontBaked<'_> {
        FontBaked::current(self)
    }
    /// Returns the current font
    #[doc(alias = "GetFont")]
    pub fn current_font(&self) -> &Font {
        unsafe { Font::from_raw(&*sys::igGetFont()) }
    }
    /// Returns the current font size (= height in pixels) with font scale applied
    #[doc(alias = "GetFontSize")]
    pub fn current_font_size(&self) -> f32 {
        unsafe { sys::igGetFontSize() }
    }
    /// Returns the UV coordinate for a white pixel.
    ///
    /// Useful for drawing custom shapes with the draw list API.
    #[doc(alias = "FontTexUvWhitePixel")]
    pub fn font_tex_uv_white_pixel(&self) -> [f32; 2] {
        unsafe { sys::igGetFontTexUvWhitePixel() }.into()
    }
}
