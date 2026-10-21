use crate::{sys, DrawCmd};

/// Standard renderer operations registered in the current platform IO.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum RendererCallback {
    ResetRenderState,
    SetSamplerLinear,
    SetSamplerNearest,
    /// Restore the sampler associated with each texture, when supported.
    SetSamplerFromTexture,
}

/// A snapshot of the renderer's registered standard callbacks.
///
/// Take this snapshot before borrowing rendered draw data. Rust renderers can
/// classify commands and implement these operations without calling native
/// callback pointers. Unsupported operations are never enqueued by
/// [`DrawListMut::add_renderer_callback`](crate::DrawListMut::add_renderer_callback).
#[derive(Copy, Clone)]
pub struct RendererCallbacks {
    pub(crate) callbacks: [sys::ImDrawCallback; 4],
}

impl RendererCallbacks {
    pub(crate) fn get(self, callback: RendererCallback) -> sys::ImDrawCallback {
        self.callbacks[match callback {
            RendererCallback::ResetRenderState => 0,
            RendererCallback::SetSamplerLinear => 1,
            RendererCallback::SetSamplerNearest => 2,
            RendererCallback::SetSamplerFromTexture => 3,
        }]
    }

    /// Whether the renderer registered this operation.
    pub fn supports(self, callback: RendererCallback) -> bool {
        self.get(callback).is_some()
    }

    /// Identify a standard operation, leaving application callbacks unclassified.
    pub fn classify(self, command: &DrawCmd) -> Option<RendererCallback> {
        let address = match command {
            DrawCmd::ResetRenderState => return Some(RendererCallback::ResetRenderState),
            DrawCmd::RawCallback { callback, .. } => *callback as usize,
            DrawCmd::Elements { .. } => return None,
        };
        [
            RendererCallback::ResetRenderState,
            RendererCallback::SetSamplerLinear,
            RendererCallback::SetSamplerNearest,
            RendererCallback::SetSamplerFromTexture,
        ]
        .into_iter()
        .find(|&kind| {
            self.get(kind)
                .is_some_and(|callback| callback as usize == address)
        })
    }
}

impl crate::PlatformIo {
    /// Register standard command markers implemented by a Rust renderer.
    ///
    /// Call during backend initialization, then dispatch the selected operations
    /// using [`RendererCallbacks::classify`] while rendering. These markers have
    /// no native rendering behavior. This replaces any existing standard
    /// callbacks; native backends should keep the callbacks they register.
    pub fn register_renderer_callbacks(
        &mut self,
        supported: &[RendererCallback],
    ) -> RendererCallbacks {
        // Distinct observable constants prevent identical-code folding from
        // merging the four markers, whose addresses identify the operations.
        #[inline(never)]
        unsafe extern "C" fn marker<const KIND: u8>(
            _: *const sys::ImDrawList,
            _: *const sys::ImDrawCmd,
        ) {
            std::hint::black_box(KIND);
        }
        self.draw_callback_reset_render_state = supported
            .contains(&RendererCallback::ResetRenderState)
            .then_some(marker::<0>);
        self.draw_callback_set_sampler_linear = supported
            .contains(&RendererCallback::SetSamplerLinear)
            .then_some(marker::<1>);
        self.draw_callback_set_sampler_nearest = supported
            .contains(&RendererCallback::SetSamplerNearest)
            .then_some(marker::<2>);
        self.draw_callback_set_sampler_from_tex = supported
            .contains(&RendererCallback::SetSamplerFromTexture)
            .then_some(marker::<3>);
        self.renderer_callbacks()
    }

    /// Snapshot standard callback capabilities for command dispatch.
    pub fn renderer_callbacks(&self) -> RendererCallbacks {
        RendererCallbacks {
            callbacks: [
                self.draw_callback_reset_render_state,
                self.draw_callback_set_sampler_linear,
                self.draw_callback_set_sampler_nearest,
                self.draw_callback_set_sampler_from_tex,
            ],
        }
    }
}
