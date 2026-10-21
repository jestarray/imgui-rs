//! Frame-borrowed texture requests for renderer backends.
use crate::{sys, TextureId};
use std::{marker::PhantomData, ptr::NonNull};

/// Pixel storage format requested by Dear ImGui.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TextureFormat {
    Rgba32,
    Alpha8,
}
/// Backends honor these requests before drawing the frame.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TextureStatus {
    Ok,
    Destroyed,
    WantCreate,
    WantUpdates,
    WantDestroy,
}
/// A pixel rectangle within a texture.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TextureRect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}
impl From<sys::ImTextureRect> for TextureRect {
    fn from(r: sys::ImTextureRect) -> Self {
        Self {
            x: r.x,
            y: r.y,
            width: r.w,
            height: r.h,
        }
    }
}

/// A borrowed texture request. Pixel slices cannot outlive the draw-data borrow.
/// Leave requests unacknowledged when an upload fails, so they can be retried.
///
/// ```compile_fail
/// let mut ctx = imgui::Context::create();
/// ctx.frame();
/// let texture = ctx.render().textures().next().unwrap();
/// ctx.frame(); // Texture requests borrow the rendered frame.
/// texture.pixels();
/// ```
pub struct TextureData<'frame> {
    raw: NonNull<sys::ImTextureData>,
    _frame: PhantomData<&'frame crate::DrawData>,
}
impl TextureData<'_> {
    /// Atlas-local identifier useful for diagnostics.
    pub fn unique_id(&self) -> i32 {
        unsafe { self.raw.as_ref().UniqueID }
    }
    /// Renderer identifier, or zero until uploaded.
    pub fn texture_id(&self) -> TextureId {
        TextureId::from_u64(unsafe { self.raw.as_ref().TexID })
    }
    /// Current request state.
    pub fn status(&self) -> TextureStatus {
        match unsafe { self.raw.as_ref().Status } {
            sys::ImTextureStatus_OK => TextureStatus::Ok,
            sys::ImTextureStatus_Destroyed => TextureStatus::Destroyed,
            sys::ImTextureStatus_WantCreate => TextureStatus::WantCreate,
            sys::ImTextureStatus_WantUpdates => TextureStatus::WantUpdates,
            sys::ImTextureStatus_WantDestroy => TextureStatus::WantDestroy,
            _ => unreachable!("unknown texture status"),
        }
    }
    /// Format of the CPU pixel buffer.
    pub fn format(&self) -> TextureFormat {
        match unsafe { self.raw.as_ref().Format } {
            sys::ImTextureFormat_RGBA32 => TextureFormat::Rgba32,
            sys::ImTextureFormat_Alpha8 => TextureFormat::Alpha8,
            _ => unreachable!("unknown texture format"),
        }
    }
    /// Texture dimensions in pixels.
    pub fn size(&self) -> [u32; 2] {
        let r = unsafe { self.raw.as_ref() };
        [
            r.Width.try_into().expect("negative width"),
            r.Height.try_into().expect("negative height"),
        ]
    }
    /// Row stride in bytes. Sub-rectangle uploads still use this full-image pitch.
    pub fn pitch(&self) -> usize {
        let r = unsafe { self.raw.as_ref() };
        usize::try_from(r.Width)
            .expect("negative width")
            .checked_mul(usize::try_from(r.BytesPerPixel).expect("negative pixel size"))
            .expect("texture pitch overflow")
    }
    /// Full CPU pixel buffer, or an empty slice when pixels have been discarded.
    pub fn pixels(&self) -> &[u8] {
        let r = unsafe { self.raw.as_ref() };
        let len = self
            .pitch()
            .checked_mul(usize::try_from(r.Height).expect("negative height"))
            .expect("texture size overflow");
        if r.Pixels.is_null() || len == 0 {
            return &[];
        }
        assert!(len <= isize::MAX as usize);
        unsafe { std::slice::from_raw_parts(r.Pixels, len) }
    }
    /// Copy the queued update rectangles, indexing the full pixel buffer.
    pub fn updates(&self) -> Vec<TextureRect> {
        let r = unsafe { self.raw.as_ref() };
        if r.Updates.Size <= 0 {
            return Vec::new();
        }
        unsafe { std::slice::from_raw_parts(r.Updates.Data, r.Updates.Size as usize) }
            .iter()
            .copied()
            .map(Into::into)
            .collect()
    }
    /// Bounding rectangle covering all queued updates.
    pub fn update_rect(&self) -> TextureRect {
        unsafe { self.raw.as_ref().UpdateRect }.into()
    }
    /// Consecutive unused frames, useful when releasing resources.
    pub fn unused_frames(&self) -> u32 {
        unsafe { self.raw.as_ref().UnusedFrames }.max(0) as u32
    }
    /// Atlas/context reference count, useful on renderer shutdown.
    pub fn ref_count(&self) -> u16 {
        unsafe { self.raw.as_ref().RefCount }
    }
    /// Acknowledge successful creation, storing the renderer's texture identifier.
    pub fn mark_created(&mut self, id: TextureId) {
        assert_eq!(
            self.status(),
            TextureStatus::WantCreate,
            "texture is not awaiting creation"
        );
        assert_ne!(
            id,
            TextureId::INVALID,
            "zero is the invalid managed texture identifier"
        );
        unsafe {
            sys::ImTextureData_SetTexID(self.raw.as_ptr(), id.id());
            sys::ImTextureData_SetStatus(self.raw.as_ptr(), sys::ImTextureStatus_OK);
        }
    }
    /// Acknowledge that all requested sub-rectangles have been uploaded.
    pub fn mark_updated(&mut self) {
        assert_eq!(
            self.status(),
            TextureStatus::WantUpdates,
            "texture is not awaiting updates"
        );
        unsafe {
            sys::ImTextureData_SetStatus(self.raw.as_ptr(), sys::ImTextureStatus_OK);
        }
    }
    /// Report resource destruction, also on shutdown. Upstream may request recreation.
    pub fn mark_destroyed(&mut self) {
        unsafe {
            sys::ImTextureData_SetTexID(self.raw.as_ptr(), 0);
            sys::ImTextureData_SetStatus(self.raw.as_ptr(), sys::ImTextureStatus_Destroyed);
        }
    }
}

/// Managed textures associated with a rendered frame.
pub struct TextureIterator<'frame> {
    iter: std::slice::Iter<'frame, *mut sys::ImTextureData>,
}
impl<'frame> TextureIterator<'frame> {
    pub(crate) unsafe fn new(list: *const sys::ImVector_ImTextureDataPtr) -> Self {
        let entries = if list.is_null() || (*list).Size <= 0 {
            &[][..]
        } else {
            std::slice::from_raw_parts((*list).Data, (*list).Size as usize)
        };
        Self {
            iter: entries.iter(),
        }
    }
}
impl<'frame> Iterator for TextureIterator<'frame> {
    type Item = TextureData<'frame>;
    fn next(&mut self) -> Option<Self::Item> {
        for &raw in self.iter.by_ref() {
            if let Some(raw) = NonNull::new(raw) {
                return Some(TextureData {
                    raw,
                    _frame: PhantomData,
                });
            }
        }
        None
    }
}

pub(crate) struct OwnedTextures {
    pub(crate) list: Box<sys::ImVector_ImTextureDataPtr>,
    _entries: Vec<OwnedTexture>,
    _pointers: Vec<*mut sys::ImTextureData>,
    pub(crate) remap: std::collections::HashMap<usize, *mut sys::ImTextureData>,
}
struct OwnedTexture {
    // Upload acknowledgement mutates metadata through shared DrawData views.
    _raw: Box<std::cell::UnsafeCell<sys::ImTextureData>>,
    _pixels: Vec<u8>,
    _updates: Vec<sys::ImTextureRect>,
}
impl OwnedTextures {
    pub(crate) unsafe fn copy(list: *const sys::ImVector_ImTextureDataPtr) -> Self {
        let mut entries = Vec::new();
        let mut pointers = Vec::new();
        let mut remap = std::collections::HashMap::new();
        for texture in TextureIterator::new(list) {
            let source = texture.raw.as_ptr();
            let ptr = *remap.entry(source as usize).or_insert_with(|| {
                let raw = Box::new(std::cell::UnsafeCell::new(*source));
                let metadata = &mut *raw.get();
                let mut pixels = texture.pixels().to_vec();
                let src = &*source;
                let mut updates = if src.Updates.Size <= 0 {
                    Vec::new()
                } else {
                    std::slice::from_raw_parts(src.Updates.Data, src.Updates.Size as usize).to_vec()
                };
                metadata.Pixels = if pixels.is_empty() {
                    std::ptr::null_mut()
                } else {
                    pixels.as_mut_ptr()
                };
                metadata.Updates = sys::ImVector_ImTextureRect {
                    Size: updates.len() as i32,
                    Capacity: updates.len() as i32,
                    Data: if updates.is_empty() {
                        std::ptr::null_mut()
                    } else {
                        updates.as_mut_ptr()
                    },
                };
                metadata.BackendUserData = std::ptr::null_mut();
                metadata.QueueUserData = std::ptr::null_mut();
                let ptr = raw.get();
                entries.push(OwnedTexture {
                    _raw: raw,
                    _pixels: pixels,
                    _updates: updates,
                });
                ptr
            });
            pointers.push(ptr);
        }
        let list = Box::new(sys::ImVector_ImTextureDataPtr {
            Size: pointers.len() as i32,
            Capacity: pointers.len() as i32,
            Data: if pointers.is_empty() {
                std::ptr::null_mut()
            } else {
                pointers.as_mut_ptr()
            },
        });
        Self {
            list,
            _entries: entries,
            _pointers: pointers,
            remap,
        }
    }
}
