use crate::{sys, FontGlyph, Ui};
use std::{marker::PhantomData, ptr::NonNull};

/// A font baked for one size and rasterizer density, valid for the current frame.
/// The view cannot outlive its UI frame. Glyph queries return owned copies so
/// subsequent glyph loading cannot invalidate references held by application code.
///
/// ```compile_fail
/// let mut ctx = imgui::Context::create();
/// let font = ctx.frame().current_font_baked();
/// ctx.frame(); // A new frame invalidates baked-font views.
/// font.find_glyph('A');
/// ```
pub struct FontBaked<'ui> {
    raw: NonNull<sys::ImFontBaked>,
    _frame: PhantomData<&'ui Ui>,
}

impl<'ui> FontBaked<'ui> {
    pub(crate) fn current(_: &'ui Ui) -> Self {
        Self {
            raw: NonNull::new(unsafe { sys::igGetFontBaked() }).expect("no current baked font"),
            _frame: PhantomData,
        }
    }
    /// Font size in logical pixels.
    pub fn size(&self) -> f32 {
        unsafe { self.raw.as_ref().Size }
    }
    /// Density used for rasterization.
    pub fn rasterizer_density(&self) -> f32 {
        unsafe { self.raw.as_ref().RasterizerDensity }
    }
    /// Distance from baseline to the top of the font.
    pub fn ascent(&self) -> f32 {
        unsafe { self.raw.as_ref().Ascent }
    }
    /// Distance from baseline to the bottom of the font.
    pub fn descent(&self) -> f32 {
        unsafe { self.raw.as_ref().Descent }
    }
    /// Find or load a glyph, returning the fallback glyph if absent.
    pub fn find_glyph(&self, codepoint: char) -> Option<FontGlyph> {
        let glyph = unsafe { sys::ImFontBaked_FindGlyph(self.raw.as_ptr(), codepoint as u32) };
        Self::copy_glyph(glyph)
    }
    /// Find or load a glyph without substituting a fallback.
    pub fn find_glyph_no_fallback(&self, codepoint: char) -> Option<FontGlyph> {
        let glyph =
            unsafe { sys::ImFontBaked_FindGlyphNoFallback(self.raw.as_ptr(), codepoint as u32) };
        Self::copy_glyph(glyph)
    }
    /// Horizontal advance in pixels, loading the glyph if necessary.
    pub fn char_advance(&self, codepoint: char) -> f32 {
        unsafe { sys::ImFontBaked_GetCharAdvance(self.raw.as_ptr(), codepoint as u32) }
    }
    /// Whether a glyph has already been baked.
    pub fn is_glyph_loaded(&self, codepoint: char) -> bool {
        unsafe { sys::ImFontBaked_IsGlyphLoaded(self.raw.as_ptr(), codepoint as u32) }
    }
    /// Copy all glyphs currently baked into this font.
    pub fn glyphs(&self) -> Vec<FontGlyph> {
        let raw = unsafe { self.raw.as_ref() };
        if raw.Glyphs.Size <= 0 {
            return Vec::new();
        }
        unsafe {
            std::slice::from_raw_parts(
                raw.Glyphs.Data.cast::<FontGlyph>(),
                raw.Glyphs.Size as usize,
            )
        }
        .to_vec()
    }
    fn copy_glyph(glyph: *const sys::ImFontGlyph) -> Option<FontGlyph> {
        if glyph.is_null() {
            None
        } else {
            Some(unsafe { std::ptr::read(glyph.cast::<FontGlyph>()) })
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{FontId, FontSource};

    #[test]
    fn font_sources_remapping_sizes_and_copied_glyphs() {
        let (_guard, mut ctx) = crate::test::test_ctx_initialized();
        ctx.fonts().compact_cache(); // Also valid before the initial build.
        let vector = ctx
            .fonts()
            .add_font(&[FontSource::DefaultFontVector { config: None }]);
        let bitmap = ctx
            .fonts()
            .add_font(&[FontSource::DefaultFontBitmap { config: None }]);
        let invalid = FontId(std::ptr::null());
        let atlas = ctx.fonts();
        assert!(atlas.font_debug_name(vector).is_some());
        assert!(atlas.font_contains_glyph(vector, 'A'));
        assert!(atlas.remap_codepoint(vector, '\u{e000}', 'A'));
        let count = atlas.font_glyph_count(vector, 0).unwrap();
        assert!(count > 1);
        assert!(!atlas.remap_codepoint_to_glyph_index(vector, '\u{e001}', 0, count));
        assert!(!atlas.remap_codepoint_to_glyph_index(vector, '\u{e001}', 0, 0));
        assert!(!atlas.remap_codepoint_to_glyph_index(vector, '\u{e001}', 255, 1));
        assert!(atlas.remap_codepoint_to_glyph_index(vector, '\u{e001}', 0, 1));
        assert!(!atlas.remap_codepoint(invalid, 'x', 'y'));
        assert!(!atlas.font_contains_glyph(invalid, 'A'));
        assert!(atlas.remove_font(bitmap));
        assert!(!atlas.remove_font(bitmap));
        let ui = ctx.frame();
        let initial_size = ui.current_font_size();
        {
            let _font = ui.push_font_with_size(vector, 24.0);
            let baked = ui.current_font_baked();
            assert_eq!(baked.size(), 24.0);
            assert!(baked.rasterizer_density() > 0.0);
            assert!(baked.ascent() > baked.descent());
            let a = baked.find_glyph_no_fallback('A').unwrap();
            let mapped = baked.find_glyph_no_fallback('\u{e000}').unwrap();
            assert_eq!(a.advance_x, mapped.advance_x);
            assert_eq!(
                [a.x0, a.y0, a.x1, a.y1],
                [mapped.x0, mapped.y0, mapped.x1, mapped.y1]
            );
            assert!(baked.find_glyph_no_fallback('\u{e001}').is_some());
            assert_eq!(baked.char_advance('A'), a.advance_x);
            assert!(baked.is_glyph_loaded('A'));
            let glyphs = baked.glyphs();
            for ch in '!'..='~' {
                baked.find_glyph(ch);
            }
            assert!(glyphs.iter().any(|g| g.codepoint() == 'A' as u32));
            assert_eq!(a.codepoint(), 'A' as u32);
            let _size = ui.push_font_size(32.0);
            assert_eq!(ui.current_font_size(), 32.0);
        }
        assert_eq!(ui.current_font_size(), initial_size);
        ui.text("Font API smoke test");
        ctx.render();
        ctx.fonts().compact_cache();
    }
}
