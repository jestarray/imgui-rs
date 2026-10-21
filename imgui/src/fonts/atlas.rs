use bitflags::bitflags;
use std::f32;
use std::os::raw::{c_char, c_int, c_uchar, c_uint, c_void};
use std::rc::Rc;
use std::slice;

use crate::fonts::font::Font;
use crate::fonts::glyph_ranges::FontGlyphRanges;
use crate::internal::{ImVector, RawCast};
use crate::sys;

bitflags! {
    /// Font atlas configuration flags
    #[repr(transparent)]
    pub struct FontAtlasFlags: i32 {
        /// Don't round the height to next power of two
        const NO_POWER_OF_TWO_HEIGHT = sys::ImFontAtlasFlags_NoPowerOfTwoHeight;
        /// Don't build software mouse cursors into the atlas
        const NO_MOUSE_CURSORS = sys::ImFontAtlasFlags_NoMouseCursors;
        /// Do not bake line textures. This disables anti-aliased lines and line endings.
        const NO_BAKED_LINES = sys::ImFontAtlasFlags_NoBakedLines;
        /// Do not bake rounded-corner textures; use tessellation instead.
        const NO_BAKED_ROUND_CORNERS = sys::ImFontAtlasFlags_NoBakedRoundCorners;
    }
}

/// A font identifier
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct FontId(pub(crate) *const Font);

/// A font atlas that builds a single texture
#[repr(C)]
pub struct FontAtlas {
    /// Configuration flags
    pub flags: FontAtlasFlags,
    /// Texture format desired by user before building the atlas.
    pub tex_desired_format: sys::ImTextureFormat,
    /// Padding between glyphs within texture in pixels.
    ///
    /// Defaults to 1. If your rendering method doesn't rely on bilinear filtering, you may set
    /// this to 0.
    pub tex_glyph_padding: i32,
    /// Minimum texture width before building the atlas.
    pub tex_min_width: i32,
    /// Minimum texture height before building the atlas.
    pub tex_min_height: i32,
    /// Maximum texture width allowed by renderer (if any).
    pub tex_max_width: i32,
    /// Maximum texture height allowed by renderer (if any).
    pub tex_max_height: i32,

    user_data: *mut c_void,
    pub tex_ref: sys::ImTextureRef,
    tex_data: *mut sys::ImTextureData,
    tex_list: sys::ImVector_ImTextureDataPtr,
    locked: bool,
    renderer_has_textures: bool,
    tex_is_built: bool,
    tex_pixels_use_colors: bool,
    tex_uv_scale: sys::ImVec2,
    tex_uv_white_pixel: sys::ImVec2,
    fonts: ImVector<*mut Font>,
    sources: sys::ImVector_ImFontConfig,
    tex_next_unique_id: i32,
    font_next_unique_id: i32,
    draw_list_shared_datas: sys::ImVector_ImDrawListSharedDataPtr,
    builder: *mut sys::ImFontAtlasBuilder,
    font_loader: *const sys::ImFontLoader,
    font_loader_name: *const c_char,
    font_loader_data: *mut c_void,
    font_loader_flags: c_uint,
    ref_count: i32,
    owner_context: *mut sys::ImGuiContext,
}

unsafe impl RawCast<sys::ImFontAtlas> for FontAtlas {}

impl FontAtlas {
    pub(crate) fn font_ptr(&self, id: FontId) -> Option<*mut sys::ImFont> {
        self.fonts
            .as_slice()
            .iter()
            .copied()
            .find(|&font| font.cast_const() == id.0)
            .map(|font| font.cast())
    }
    #[doc(alias = "AddFontDefault", alias = "AddFont")]
    pub fn add_font(&mut self, font_sources: &[FontSource<'_>]) -> FontId {
        let (head, tail) = font_sources.split_first().unwrap();
        let font_id = self.add_font_internal(head, false);
        for font in tail {
            self.add_font_internal(font, true);
        }
        font_id
    }
    fn add_font_internal(&mut self, font_source: &FontSource<'_>, merge_mode: bool) -> FontId {
        let mut raw_config = sys_font_config_default();
        raw_config.MergeMode = merge_mode;
        let raw_font = match font_source {
            FontSource::DefaultFontData { config } => unsafe {
                if let Some(config) = config {
                    config.apply_to_raw_config(&mut raw_config, self.raw_mut());
                }
                sys::ImFontAtlas_AddFontDefault(self.raw_mut(), &raw_config)
            },
            FontSource::DefaultFontVector { config } => unsafe {
                if let Some(config) = config {
                    config.apply_to_raw_config(&mut raw_config, self.raw_mut());
                }
                sys::ImFontAtlas_AddFontDefaultVector(self.raw_mut(), &raw_config)
            },
            FontSource::DefaultFontBitmap { config } => unsafe {
                if let Some(config) = config {
                    config.apply_to_raw_config(&mut raw_config, self.raw_mut());
                }
                sys::ImFontAtlas_AddFontDefaultBitmap(self.raw_mut(), &raw_config)
            },
            FontSource::TtfData {
                data,
                size_pixels,
                config,
            } => {
                if let Some(config) = config {
                    unsafe {
                        config.apply_to_raw_config(&mut raw_config, self.raw_mut());
                    }
                }
                // We can't guarantee `data` is alive when the font atlas is built, so
                // make a copy and move ownership of the data to the atlas
                let data_copy = unsafe {
                    let ptr = sys::igMemAlloc(data.len()) as *mut u8;
                    assert!(!ptr.is_null());
                    slice::from_raw_parts_mut(ptr, data.len())
                };
                data_copy.copy_from_slice(data);
                raw_config.FontData = data_copy.as_mut_ptr() as *mut c_void;
                raw_config.FontDataSize = data_copy.len() as i32;
                raw_config.FontDataOwnedByAtlas = true;
                raw_config.SizePixels = *size_pixels;
                unsafe { sys::ImFontAtlas_AddFont(self.raw_mut(), &raw_config) }
            }
        };
        FontId(raw_font as *const _)
    }
    pub fn fonts(&self) -> Vec<FontId> {
        let mut result = Vec::new();
        unsafe {
            for &font in self.fonts.as_slice() {
                result.push((*font).id());
            }
        }
        result
    }
    pub fn get_font(&self, id: FontId) -> Option<&Font> {
        unsafe {
            for &font in self.fonts.as_slice() {
                if id == FontId(font) {
                    return Some(&*(font as *const Font));
                }
            }
        }
        None
    }
    /// Read a font's debug name as an owned string.
    pub fn font_debug_name(&self, id: FontId) -> Option<String> {
        let font = self.font_ptr(id)?;
        let name = unsafe { sys::ImFont_GetDebugName(font) };
        if name.is_null() {
            None
        } else {
            Some(
                unsafe { std::ffi::CStr::from_ptr(name) }
                    .to_string_lossy()
                    .into_owned(),
            )
        }
    }
    /// Test whether one of this font's sources contains the codepoint.
    pub fn font_contains_glyph(&self, id: FontId, codepoint: char) -> bool {
        self.font_ptr(id)
            .is_some_and(|font| unsafe { sys::ImFont_IsGlyphInFont(font, codepoint as u32) })
    }
    /// Map a codepoint to another codepoint. Apply remappings before the glyph is baked.
    /// Returns false if the font does not belong to this atlas.
    pub fn remap_codepoint(&mut self, id: FontId, from: char, to: char) -> bool {
        let Some(font) = self.font_ptr(id) else {
            return false;
        };
        unsafe { sys::ImFont_AddRemapCodepoint(font, from as u32, to as u32) };
        true
    }
    /// Map a codepoint to a glyph index in one of the font's sources.
    /// Returns false for unknown fonts/sources/formats, index zero, or an index
    /// outside the source's glyph count. Glyph indices use upstream's 16-bit
    /// representation. Apply remappings before the glyph is baked.
    pub fn remap_codepoint_to_glyph_index(
        &mut self,
        id: FontId,
        from: char,
        source: u8,
        glyph_index: u16,
    ) -> bool {
        let Some(font) = self.font_ptr(id) else {
            return false;
        };
        // stb_truetype's horizontal-metrics lookup does not range-check indices.
        // Validate the source's SFNT maxp table before handing an index to it.
        let Some(count) = self.font_glyph_count(id, source) else {
            return false;
        };
        if glyph_index == 0 || glyph_index >= count {
            return false;
        }
        unsafe {
            sys::ImFont_AddRemapCodepointToGlyphIndex(
                font,
                from as u32,
                source.into(),
                glyph_index.into(),
            )
        };
        true
    }
    /// Number of glyphs in a TrueType/OpenType source, or None for an unknown source/format.
    pub fn font_glyph_count(&self, id: FontId, source: u8) -> Option<u16> {
        let font = self.font_ptr(id)?;
        unsafe {
            let sources = &(*font).Sources;
            if i32::from(source) >= sources.Size {
                return None;
            }
            let source = &**sources.Data.add(usize::from(source));
            if source.FontData.is_null() || source.FontDataSize <= 0 {
                return None;
            }
            let data =
                slice::from_raw_parts(source.FontData.cast::<u8>(), source.FontDataSize as usize);
            sfnt_glyph_count(data, usize::try_from(source.FontNo).ok()?)
        }
    }
    /// Remove a font owned by this atlas. Existing FontIds for it become invalid.
    pub fn remove_font(&mut self, id: FontId) -> bool {
        assert!(!self.locked, "font atlas is locked during this frame");
        let Some(font) = self.font_ptr(id) else {
            return false;
        };
        unsafe { sys::ImFontAtlas_RemoveFont(self.raw_mut(), font) };
        true
    }
    /// Compact cached font/texture storage outside a locked frame.
    pub fn compact_cache(&mut self) {
        assert!(!self.locked, "font atlas is locked during this frame");
        if !self.tex_is_built || self.tex_data.is_null() {
            return;
        }
        unsafe { sys::ImFontAtlas_CompactCache(self.raw_mut()) };
    }
    /// Returns true if the font atlas has been built
    #[doc(alias = "IsBuilt")]
    pub fn is_built(&self) -> bool {
        self.tex_is_built && !self.fonts.as_slice().is_empty()
    }
    /// Builds a 1 byte per-pixel font atlas texture
    #[doc(alias = "GetTextDataAsAlpha8")]
    pub fn build_alpha8_texture(&mut self) -> FontAtlasTexture<'_> {
        self.build_texture_with_bytes_per_pixel(1)
    }
    /// Builds a 4 byte per-pixel font atlas texture
    #[doc(alias = "GetTextDataAsRGBA32")]
    pub fn build_rgba32_texture(&mut self) -> FontAtlasTexture<'_> {
        self.build_texture_with_bytes_per_pixel(4)
    }

    fn build_texture_with_bytes_per_pixel(&mut self, expected_bpp: c_int) -> FontAtlasTexture<'_> {
        unsafe {
            // Set desired format
            let format = if expected_bpp == 1 {
                sys::ImTextureFormat_Alpha8
            } else {
                sys::ImTextureFormat_RGBA32
            };

            // Build atlas on demand if not already built (matches Dear ImGui 1.92+ behavior)
            // This matches the logic in GetTexDataAsFormat() from imgui_draw.cpp
            let atlas_ptr = self as *mut FontAtlas;
            let needs_build = !(*atlas_ptr).tex_is_built
                || (*atlas_ptr).tex_data.is_null()
                || (*(*atlas_ptr).tex_data).Pixels.is_null()
                || (*atlas_ptr).raw_mut().TexDesiredFormat != format;

            if needs_build {
                let raw_atlas = (*atlas_ptr).raw_mut();
                raw_atlas.TexDesiredFormat = format;
                // Call ImFontAtlasBuildMain() - the non-obsolete build function
                // This is what GetTexDataAsFormat() calls internally
                sys::ImFontAtlas_BuildMain_Wrapper(raw_atlas);
            }

            let tex_data = (*atlas_ptr).tex_data;
            assert!(
                !tex_data.is_null(),
                "font texture data is not available after build"
            );
            let tex_data = &*tex_data;
            let width = tex_data.Width;
            let height = tex_data.Height;
            let bytes_per_pixel = tex_data.BytesPerPixel;
            let pixels = tex_data.Pixels as *const c_uchar;

            assert!(width >= 0, "font texture width must be positive");
            assert!(height >= 0, "font texture height must be positive");
            assert!(
                bytes_per_pixel >= 0,
                "font texture bytes per pixel must be positive"
            );
            assert_eq!(
                bytes_per_pixel, expected_bpp,
                "unexpected font texture format"
            );

            let height = height as usize;
            // Check multiplication to avoid constructing an invalid slice in case of overflow
            let pitch = width
                .checked_mul(bytes_per_pixel)
                .expect("Overflow in font texture pitch calculation")
                as usize;
            FontAtlasTexture {
                width: width as u32,
                height: height as u32,
                data: slice::from_raw_parts(pixels, pitch * height),
            }
        }
    }
    /// Clears the font atlas completely (both input and output data)
    #[doc(alias = "Clear")]
    pub fn clear(&mut self) {
        unsafe {
            sys::ImFontAtlas_Clear(self.raw_mut());
        }
    }
    /// Clears output font data (glyph storage, UV coordinates)
    #[doc(alias = "ClearFonts")]
    pub fn clear_fonts(&mut self) {
        unsafe {
            sys::ImFontAtlas_ClearFonts(self.raw_mut());
        }
    }
    /// Clears output texture data.
    ///
    /// Can be used to save RAM once the texture has been transferred to the GPU.
    #[doc(alias = "ClearTexData")]
    pub fn clear_tex_data(&mut self) {
        unsafe {
            sys::ImFontAtlas_ClearTexData(self.raw_mut());
        }
    }
    /// Clears all the data used to build the textures and fonts
    #[doc(alias = "ClearInputData")]
    pub fn clear_input_data(&mut self) {
        unsafe {
            sys::ImFontAtlas_ClearInputData(self.raw_mut());
        }
    }
}

fn sfnt_glyph_count(data: &[u8], font_index: usize) -> Option<u16> {
    fn u16_at(data: &[u8], pos: usize) -> Option<u16> {
        Some(u16::from_be_bytes(
            data.get(pos..pos.checked_add(2)?)?.try_into().ok()?,
        ))
    }
    fn u32_at(data: &[u8], pos: usize) -> Option<usize> {
        usize::try_from(u32::from_be_bytes(
            data.get(pos..pos.checked_add(4)?)?.try_into().ok()?,
        ))
        .ok()
    }
    let offset = if data.get(..4)? == b"ttcf" {
        if font_index >= u32_at(data, 8)? {
            return None;
        }
        u32_at(data, 12usize.checked_add(font_index.checked_mul(4)?)?)?
    } else {
        if font_index != 0 {
            return None;
        }
        0
    };
    let header = data.get(offset..offset.checked_add(12)?)?;
    if !matches!(&header[..4], b"\0\x01\0\0" | b"OTTO" | b"true" | b"typ1") {
        return None;
    }
    let table_start = offset.checked_add(12)?;
    for i in 0..usize::from(u16_at(header, 4)?) {
        let entry = table_start.checked_add(i.checked_mul(16)?)?;
        let table = data.get(entry..entry.checked_add(16)?)?;
        if &table[..4] == b"maxp" {
            let start = u32_at(table, 8)?;
            let len = u32_at(table, 12)?;
            if len < 6 {
                return None;
            }
            let maxp = data.get(start..start.checked_add(len)?)?;
            return u16_at(maxp, 4);
        }
    }
    None
}

#[test]
#[cfg(test)]
fn test_font_atlas_memory_layout() {
    use std::mem;
    assert_eq!(
        mem::size_of::<FontAtlas>(),
        mem::size_of::<sys::ImFontAtlas>()
    );
    assert_eq!(
        mem::align_of::<FontAtlas>(),
        mem::align_of::<sys::ImFontAtlas>()
    );
    use sys::ImFontAtlas;
    macro_rules! assert_field_offset {
        ($l:ident, $r:ident) => {
            assert_eq!(
                memoffset::offset_of!(FontAtlas, $l),
                memoffset::offset_of!(ImFontAtlas, $r)
            );
        };
    }
    assert_field_offset!(flags, Flags);
    assert_field_offset!(tex_desired_format, TexDesiredFormat);
    assert_field_offset!(tex_glyph_padding, TexGlyphPadding);
    assert_field_offset!(tex_min_width, TexMinWidth);
    assert_field_offset!(tex_min_height, TexMinHeight);
    assert_field_offset!(tex_max_width, TexMaxWidth);
    assert_field_offset!(tex_max_height, TexMaxHeight);
    assert_field_offset!(user_data, UserData);
    assert_field_offset!(tex_ref, TexRef);
    assert_field_offset!(tex_data, TexData);
    assert_field_offset!(tex_list, TexList);
    assert_field_offset!(locked, Locked);
    assert_field_offset!(renderer_has_textures, RendererHasTextures);
    assert_field_offset!(tex_is_built, TexIsBuilt);
    assert_field_offset!(tex_pixels_use_colors, TexPixelsUseColors);
    assert_field_offset!(tex_uv_scale, TexUvScale);
    assert_field_offset!(tex_uv_white_pixel, TexUvWhitePixel);
    assert_field_offset!(fonts, Fonts);
    assert_field_offset!(sources, Sources);
    assert_field_offset!(tex_next_unique_id, TexNextUniqueID);
    assert_field_offset!(font_next_unique_id, FontNextUniqueID);
    assert_field_offset!(draw_list_shared_datas, DrawListSharedDatas);
    assert_field_offset!(builder, Builder);
    assert_field_offset!(font_loader, FontLoader);
    assert_field_offset!(font_loader_name, FontLoaderName);
    assert_field_offset!(font_loader_data, FontLoaderData);
    assert_field_offset!(font_loader_flags, FontLoaderFlags);
    assert_field_offset!(ref_count, RefCount);
    assert_field_offset!(owner_context, OwnerContext);
}

/// A source for binary font data
#[derive(Clone, Debug)]
pub enum FontSource<'a> {
    /// Choose the default font using upstream size and display-density heuristics.
    DefaultFontData { config: Option<FontConfig> },
    /// Explicitly select the embedded scalable default font.
    DefaultFontVector { config: Option<FontConfig> },
    /// Explicitly select the embedded bitmap default font.
    DefaultFontBitmap { config: Option<FontConfig> },
    /// Binary TTF/OTF font data
    TtfData {
        data: &'a [u8],
        size_pixels: f32,
        config: Option<FontConfig>,
    },
}

/// Configuration settings for a font
#[derive(Clone, Debug)]
pub struct FontConfig {
    /// Size in pixels for the rasterizer
    pub size_pixels: f32,
    /// Horizontal oversampling
    pub oversample_h: i8,
    /// Vertical oversampling
    pub oversample_v: i8,
    /// Align every glyph to pixel boundary
    pub pixel_snap_h: bool,
    /// Offset for all glyphs in this font
    pub glyph_offset: [f32; 2],
    /// Unicode ranges to use from this font
    pub glyph_ranges: FontGlyphRanges,
    /// Unicode ranges to exclude from this font
    pub glyph_exclude_ranges: Option<FontGlyphRanges>,
    /// Minimum advance_x for glyphs
    pub glyph_min_advance_x: f32,
    /// Maximum advance_x for glyphs
    pub glyph_max_advance_x: f32,
    /// Extra advance between glyphs
    pub glyph_extra_advance_x: f32,
    /// Font index in the font file
    pub font_no: u32,
    /// Settings for a custom font loader if used
    pub font_loader_flags: u32,
    /// Brighten (>1.0) or darken (<1.0) font output
    pub rasterizer_multiply: f32,
    /// DPI scale for rasterization, not altering other font metrics:
    /// make it easy to swap between e.g. a 100% and a 400% fonts for a zooming display.
    /// IMPORTANT: If you increase this it is expected that you increase font scale
    /// accordingly, otherwise quality may look lowered.
    pub rasterizer_density: f32,
    /// Additional size scaling applied when rasterizing the font.
    pub extra_size_scale: f32,
    /// Font flags
    pub flags: sys::ImFontFlags,
    /// Explicitly specify the ellipsis character.
    ///
    /// With multiple font sources the first specified ellipsis is used.
    pub ellipsis_char: Option<char>,
    pub name: Option<String>,
}

impl Default for FontConfig {
    fn default() -> FontConfig {
        let sys_font_config = sys_font_config_default();
        FontConfig {
            size_pixels: sys_font_config.SizePixels,
            oversample_h: sys_font_config.OversampleH,
            oversample_v: sys_font_config.OversampleV,
            pixel_snap_h: sys_font_config.PixelSnapH,
            glyph_offset: [sys_font_config.GlyphOffset.x, sys_font_config.GlyphOffset.y],
            glyph_ranges: FontGlyphRanges::default(),
            glyph_exclude_ranges: None,
            glyph_min_advance_x: sys_font_config.GlyphMinAdvanceX,
            glyph_max_advance_x: sys_font_config.GlyphMaxAdvanceX,
            glyph_extra_advance_x: sys_font_config.GlyphExtraAdvanceX,
            font_no: sys_font_config.FontNo,
            font_loader_flags: sys_font_config.FontLoaderFlags,
            rasterizer_multiply: sys_font_config.RasterizerMultiply,
            rasterizer_density: sys_font_config.RasterizerDensity,
            extra_size_scale: sys_font_config.ExtraSizeScale,
            flags: sys_font_config.Flags,
            ellipsis_char: match sys_font_config.EllipsisChar {
                0 => None,
                value => std::char::from_u32(value),
            },
            name: None,
        }
    }
}

impl FontConfig {
    fn apply_to_raw_config(&self, raw: &mut sys::ImFontConfig, atlas: *mut sys::ImFontAtlas) {
        raw.SizePixels = self.size_pixels;
        raw.OversampleH = self.oversample_h;
        raw.OversampleV = self.oversample_v;
        raw.PixelSnapH = self.pixel_snap_h;
        raw.GlyphOffset = self.glyph_offset.into();
        raw.GlyphRanges = unsafe { self.glyph_ranges.to_ptr(atlas) };
        raw.GlyphExcludeRanges = match self.glyph_exclude_ranges.as_ref() {
            Some(ranges) => unsafe { ranges.to_ptr(atlas) },
            None => std::ptr::null(),
        };
        raw.GlyphMinAdvanceX = self.glyph_min_advance_x;
        raw.GlyphMaxAdvanceX = self.glyph_max_advance_x;
        raw.GlyphExtraAdvanceX = self.glyph_extra_advance_x;
        raw.FontNo = self.font_no;
        raw.FontLoaderFlags = self.font_loader_flags;
        raw.RasterizerMultiply = self.rasterizer_multiply;
        raw.RasterizerDensity = self.rasterizer_density;
        raw.ExtraSizeScale = self.extra_size_scale;
        raw.Flags = self.flags;
        raw.EllipsisChar = self.ellipsis_char.unwrap_or('\0') as sys::ImWchar;
        if let Some(name) = self.name.as_ref() {
            let bytes = name.as_bytes();
            let mut len = bytes.len().min(raw.Name.len() - 1);
            while !name.is_char_boundary(len) {
                len -= 1;
            }
            unsafe {
                bytes.as_ptr().copy_to(raw.Name.as_mut_ptr() as _, len);
                raw.Name[len] = 0;
            }
        }
    }
}

fn sys_font_config_default() -> sys::ImFontConfig {
    unsafe {
        let heap_allocated = sys::ImFontConfig_ImFontConfig();
        let copy = *heap_allocated;
        sys::ImFontConfig_destroy(heap_allocated);
        copy
    }
}

#[test]
fn test_font_config_default() {
    let sys_font_config = sys_font_config_default();
    let font_config = FontConfig::default();
    assert_eq!(font_config.size_pixels, sys_font_config.SizePixels);
    assert_eq!(font_config.oversample_h, sys_font_config.OversampleH);
    assert_eq!(font_config.oversample_v, sys_font_config.OversampleV);
    assert_eq!(font_config.pixel_snap_h, sys_font_config.PixelSnapH);
    assert_eq!(font_config.glyph_offset[0], sys_font_config.GlyphOffset.x);
    assert_eq!(font_config.glyph_offset[1], sys_font_config.GlyphOffset.y);
    assert_eq!(
        font_config.glyph_min_advance_x,
        sys_font_config.GlyphMinAdvanceX
    );
    assert_eq!(
        font_config.glyph_max_advance_x,
        sys_font_config.GlyphMaxAdvanceX
    );
    assert_eq!(
        font_config.glyph_extra_advance_x,
        sys_font_config.GlyphExtraAdvanceX
    );
    assert_eq!(font_config.font_no, sys_font_config.FontNo);
    assert_eq!(
        font_config.font_loader_flags,
        sys_font_config.FontLoaderFlags
    );
    assert_eq!(
        font_config.rasterizer_multiply,
        sys_font_config.RasterizerMultiply
    );
}

/// Handle to a font atlas texture
#[derive(Clone, Debug)]
pub struct FontAtlasTexture<'a> {
    /// Texture width (in pixels)
    pub width: u32,
    /// Texture height (in pixels)
    pub height: u32,
    /// Raw texture data (in bytes).
    ///
    /// The format depends on which function was called to obtain this data.
    pub data: &'a [u8],
}

/// A font atlas that can be shared between contexts
#[derive(Debug, Clone)]
pub struct SharedFontAtlas(pub(crate) Rc<*mut sys::ImFontAtlas>);

impl std::ops::Deref for SharedFontAtlas {
    type Target = Rc<*mut sys::ImFontAtlas>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for SharedFontAtlas {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl SharedFontAtlas {
    #[doc(alias = "ImFontAtlas", alias = "ImFontAtlas::ImFontAtlas")]
    pub fn create() -> SharedFontAtlas {
        SharedFontAtlas(unsafe { Rc::new(sys::ImFontAtlas_ImFontAtlas()) })
    }

    /// Gets a raw pointer to the underlying `ImFontAtlas`.
    pub fn as_ptr(&self) -> *const sys::ImFontAtlas {
        *self.0 as *const _
    }

    /// Gets a raw pointer to the underlying `ImFontAtlas`.
    pub fn as_ptr_mut(&mut self) -> *mut sys::ImFontAtlas {
        *self.0
    }
}

impl Drop for SharedFontAtlas {
    #[doc(alias = "ImFontAtlas::Destory")]
    fn drop(&mut self) {
        // ImGui owns shared font atlases and releases them when the last context shuts down.
    }
}

// /// An immutably borrowed reference to a (possibly shared) font atlas
// pub enum FontAtlasRef<'a> {
//     Owned(&'a FontAtlas),
//     Shared(&'a cell::RefMut<'a, SharedFontAtlas>),
// }

// impl<'a> Deref for FontAtlasRef<'a> {
//     type Target = FontAtlas;
//     fn deref(&self) -> &FontAtlas {
//         use self::FontAtlasRef::*;
//         match self {
//             Owned(atlas) => atlas,
//             Shared(cell) => {
//                 let font_atlas: &SharedFontAtlas = &cell;
//                 let font_atlas: &FontAtlas = &font_atlas;

//                 todo!()
//             }
//         }
//     }
// }

// /// A mutably borrowed reference to a (possibly shared) font atlas
// pub enum FontAtlasRefMut<'a> {
//     Owned(&'a mut FontAtlas),
//     Shared(cell::RefMut<'a, SharedFontAtlas>),
// }

// impl<'a> Deref for FontAtlasRefMut<'a> {
//     type Target = FontAtlas;
//     fn deref(&self) -> &FontAtlas {
//         use self::FontAtlasRefMut::*;
//         match self {
//             Owned(atlas) => atlas,
//             Shared(cell) => cell,
//         }
//     }
// }

// impl<'a> DerefMut for FontAtlasRefMut<'a> {
//     fn deref_mut(&mut self) -> &mut FontAtlas {
//         use self::FontAtlasRefMut::*;
//         match self {
//             Owned(atlas) => atlas,
//             Shared(cell) => cell,
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use super::FontAtlasFlags;
    use crate::FontSource;

    #[test]
    fn test_baking_options() {
        for flags in [
            FontAtlasFlags::NO_BAKED_ROUND_CORNERS,
            FontAtlasFlags::NO_BAKED_LINES,
        ] {
            let (_guard, mut ctx) = crate::test::test_ctx_initialized();
            ctx.fonts().flags.insert(flags);
            ctx.fonts()
                .add_font(&[FontSource::DefaultFontData { config: None }]);
            assert!(!ctx.fonts().build_rgba32_texture().data.is_empty());
            ctx.style_mut().anti_aliased_line_ends = true;
            let ui = ctx.frame();
            if flags.contains(FontAtlasFlags::NO_BAKED_LINES) {
                assert!(!ui.clone_style().anti_aliased_lines);
                assert!(!ui.clone_style().anti_aliased_line_ends);
            }
            ui.get_background_draw_list()
                .add_rect([10.0, 10.0], [50.0, 50.0], crate::ImColor32::WHITE)
                .rounding(8.0)
                .build();
            assert!(ctx.render().total_vtx_count > 0);
        }
    }

    #[test]
    fn test_build_texture_before_first_frame() {
        // Regression test for hudhook bug: build_rgba32_texture should work before NewFrame
        // by building the atlas on demand (like Dear ImGui 1.92+ GetTexDataAsRGBA32)

        // Use test serialization to avoid context conflicts in parallel tests
        let (_guard, mut ctx) = crate::test::test_ctx();

        // Add default font
        ctx.fonts()
            .add_font(&[FontSource::DefaultFontData { config: None }]);

        // Build texture before first frame - should not panic
        let texture = ctx.fonts().build_rgba32_texture();

        // Verify texture data
        assert!(texture.width > 0, "texture width should be positive");
        assert!(texture.height > 0, "texture height should be positive");
        assert!(!texture.data.is_empty(), "texture data should not be empty");
        assert_eq!(
            texture.data.len(),
            (texture.width as usize) * (texture.height as usize) * 4,
            "texture data size should match width * height * 4 (RGBA)"
        );

        // Context and guard dropped here, releasing the mutex
    }

    #[test]
    fn test_build_alpha8_texture_before_first_frame() {
        // Test alpha8 variant as well
        let (_guard, mut ctx) = crate::test::test_ctx();

        ctx.fonts()
            .add_font(&[FontSource::DefaultFontData { config: None }]);

        // Build alpha8 texture before first frame
        let texture = ctx.fonts().build_alpha8_texture();

        assert!(texture.width > 0);
        assert!(texture.height > 0);
        assert!(!texture.data.is_empty());
        assert_eq!(
            texture.data.len(),
            (texture.width as usize) * (texture.height as usize),
            "texture data size should match width * height (Alpha8)"
        );
    }

    #[test]
    fn test_rebuild_texture_after_format_change() {
        // Test that changing format triggers rebuild
        let (_guard, mut ctx) = crate::test::test_ctx();

        ctx.fonts()
            .add_font(&[FontSource::DefaultFontData { config: None }]);

        // Build as RGBA32 first
        let texture1 = ctx.fonts().build_rgba32_texture();
        let size1 = texture1.data.len();

        // Build as Alpha8 - should rebuild with different size
        let texture2 = ctx.fonts().build_alpha8_texture();
        let size2 = texture2.data.len();

        // Alpha8 should be 1/4 the size of RGBA32
        assert_eq!(size1, size2 * 4, "RGBA32 should be 4x larger than Alpha8");
    }
}
