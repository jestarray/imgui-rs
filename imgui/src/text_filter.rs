use crate::sys;
use crate::Ui;
use std::ptr;

#[cfg(test)]
mod tests;

/// Helper to parse and apply text filters.
pub struct TextFilter {
    id: String,
    raw: *mut sys::ImGuiTextFilter,
}

impl TextFilter {
    /// Creates a new TextFilter with an empty filter.
    ///
    /// This is equivalent of [new_with_filter](Self::new_with_filter) with `filter` set to `""`.
    pub fn new(label: String) -> Self {
        Self::new_with_filter(label, String::new())
    }

    /// Creates a new TextFilter with a custom filter.
    pub fn new_with_filter(label: String, mut filter: String) -> Self {
        filter.push('\0');
        let ptr = filter.as_mut_ptr();
        Self {
            id: label,
            raw: unsafe { sys::ImGuiTextFilter_ImGuiTextFilter(ptr as *mut core::ffi::c_char) },
        }
    }

    /// Builds the TextFilter with its filter attribute. You can use
    /// [`pass_filter()`](Self::pass_filter) after it.
    ///
    /// If you want control the filter with an InputText, check [`draw()`](Self::draw).
    pub fn build(&self) {
        unsafe {
            sys::ImGuiTextFilter_Build(self.raw);
        }
    }

    /// Draws an [InputText](crate::input_widget::InputText) to control the filter of the TextFilter.
    ///
    /// This is equivalent of [draw_with_size](Self::draw_with_size) with `size` set to `0.0`.
    pub fn draw(&self) {
        self.draw_with_size(0.0);
    }

    /// Draws an [InputText](crate::input_widget::InputText) to control the filter of the TextFilter.
    ///
    /// The InputText has the size passed in parameters.
    pub fn draw_with_size(&self, size: f32) {
        unsafe {
            if size != 0.0 {
                sys::igSetNextItemWidth(size);
            }
            let mut id = self.id.clone();
            id.push('\0');
            let ptr = id.as_mut_ptr();
            sys::ImGuiTextFilter_Draw(self.raw, ptr as *mut core::ffi::c_char);
        }
    }

    /// Draws the filter with a custom hint. Returns true when the filter changes.
    /// Use [`Ui::set_next_item_width`] before this call to set its width.
    pub fn draw_with_hint(&self, hint: &str) -> bool {
        let mut id = self.id.clone();
        id.push('\0');
        let mut hint = hint.to_owned();
        hint.push('\0');
        unsafe {
            sys::ImGuiTextFilter_DrawWithHint(self.raw, id.as_ptr().cast(), hint.as_ptr().cast())
        }
    }

    /// Returns true if the filter is not empty (`""`).
    pub fn is_active(&self) -> bool {
        unsafe { sys::ImGuiTextFilter_IsActive(self.raw) }
    }

    /// Returns true if the buffer matches the filter.
    ///
    /// Spaces combine terms with AND, commas combine alternatives with OR,
    /// quotes match an exact phrase, and `-term` excludes matching text.
    /// Exclusions take precedence over inclusions.
    ///
    /// [`draw()`](Self::draw) or [`build()`](Self::build) mut be called **before** this function.
    pub fn pass_filter(&self, buf: &str) -> bool {
        let mut buf = String::from(buf);
        buf.push('\0');
        let ptr = buf.as_mut_ptr();
        unsafe {
            sys::ImGuiTextFilter_PassFilter(self.raw, ptr as *mut core::ffi::c_char, ptr::null())
        }
    }

    pub fn pass_filter_with_end(&self, start: &str, end: &str) -> bool {
        let (mut start, mut end) = (String::from(start), String::from(end));
        start.push('\0');
        end.push('\0');
        let b_ptr = start.as_mut_ptr();
        let e_ptr = end.as_mut_ptr();
        unsafe {
            sys::ImGuiTextFilter_PassFilter(
                self.raw,
                b_ptr as *mut core::ffi::c_char,
                e_ptr as *mut core::ffi::c_char,
            )
        }
    }

    /// Clears the filter.
    pub fn clear(&self) {
        unsafe {
            sys::ImGuiTextFilter_Clear(self.raw);
        }
    }
}

impl Drop for TextFilter {
    fn drop(&mut self) {
        unsafe { sys::ImGuiTextFilter_destroy(self.raw) };
    }
}

impl Ui {
    pub fn text_filter(label: String) -> TextFilter {
        TextFilter::new(label)
    }

    pub fn text_filter_with_filter(label: String, filter: String) -> TextFilter {
        TextFilter::new_with_filter(label, filter)
    }
}
