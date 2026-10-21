use super::*;
use std::panic::{catch_unwind, AssertUnwindSafe};

#[test]
fn primitives_emit_geometry() {
    for scale in [1.0, 2.0] {
        let (_guard, mut ctx) = crate::test::test_ctx_initialized();
        ctx.io_mut().display_framebuffer_scale = [scale, scale];
        let ui = ctx.frame();
        {
            let draw = ui.get_background_draw_list();
            let white = ImColor32::WHITE;
            let flags = DrawFlags::AA_LINE_ENDS | DrawFlags::STROKE_CENTER_BIASED;
            let mut count = 0;
            macro_rules! emitted {
                ($shape:expr) => {{
                    $shape.build();
                    let next = unsafe { (*draw.draw_list).VtxBuffer.Size };
                    assert!(next > count);
                    count = next;
                }};
            }
            emitted!(draw
                .add_line([10.0, 10.0], [80.0, 20.0], white)
                .thickness(3.0)
                .flags(flags));
            emitted!(draw
                .add_polyline(vec![[10.0, 30.0], [80.0, 30.0], [50.0, 60.0]], white)
                .thickness(4.0)
                .flags(DrawFlags::CLOSED | DrawFlags::JOIN_MITER));
            emitted!(draw
                .add_polyline(vec![[10.0, 70.0], [80.0, 70.0]], white)
                .flags(DrawFlags::CAP_SQUARE | flags));
            emitted!(draw
                .add_polyline(vec![[10.0, 80.0], [80.0, 80.0], [50.0, 100.0]], white)
                .filled(true)
                .flags(DrawFlags::JOIN_MITER));
            emitted!(draw
                .add_rect([100.0, 10.0], [180.0, 50.0], white)
                .rounding(8.0)
                .thickness(4.0)
                .flags(DrawFlags::STROKE_INSIDE | DrawFlags::ROUND_TOP));
            emitted!(draw
                .add_rect([100.25, 60.25], [180.25, 100.25], white)
                .rounding(8.0)
                .filled(true)
                .flags(DrawFlags::ROUND_ALL));
            emitted!(draw
                .add_triangle([200.0, 10.0], [250.0, 50.0], [190.0, 50.0], white)
                .thickness(3.0)
                .flags(DrawFlags::STROKE_OUTSIDE));
            emitted!(draw
                .add_triangle([200.0, 60.0], [250.0, 100.0], [190.0, 100.0], white)
                .filled(true));
            emitted!(draw
                .add_circle([300.0, 30.0], 20.0, white)
                .thickness(3.0)
                .flags(DrawFlags::STROKE_LEGACY));
            emitted!(draw.add_circle([300.0, 80.0], 20.0, white).filled(true));
            emitted!(draw
                .add_bezier_curve(
                    [350.0, 10.0],
                    [400.0, 10.0],
                    [350.0, 80.0],
                    [400.0, 80.0],
                    white
                )
                .thickness(3.0)
                .flags(flags));
            assert!(count > 0);
        }
        ui.show_demo_window(&mut true);
        let data = ctx.render();
        assert!(data.total_vtx_count > 0 && data.total_idx_count > 0);
        assert_eq!(data.framebuffer_scale, [scale, scale]);
    }
}

#[test]
fn scoped_flags_restore_after_return_panic_and_rejection() {
    let (_guard, mut ctx) = crate::test::test_ctx_initialized();
    let ui = ctx.frame();
    let draw = ui.get_background_draw_list();
    let original = unsafe { (*draw.draw_list).Flags };
    let result = draw.with_draw_flag(DrawFlags::STROKE_LEGACY, true, || {
        assert_eq!(
            unsafe { (*draw.draw_list).Flags } & DrawFlags::STROKE_LEGACY.bits(),
            DrawFlags::STROKE_LEGACY.bits()
        );
        draw.with_draw_flag(DrawFlags::AA_FILL, false, || {
            assert_eq!(
                unsafe { (*draw.draw_list).Flags } & DrawFlags::AA_FILL.bits(),
                0
            );
        });
        assert_ne!(
            unsafe { (*draw.draw_list).Flags } & DrawFlags::AA_FILL.bits(),
            0
        );
        42
    });
    assert_eq!(result, 42);
    assert_eq!(unsafe { (*draw.draw_list).Flags }, original);

    let panic = catch_unwind(AssertUnwindSafe(|| {
        draw.with_draw_flag(DrawFlags::AA_LINE_ENDS, true, || panic!("scope test"));
    }));
    assert!(panic.is_err());
    assert_eq!(unsafe { (*draw.draw_list).Flags }, original);
    for flags in [
        DrawFlags::CLOSED,
        DrawFlags::JOIN_MITER,
        DrawFlags::STROKE_INSIDE,
    ] {
        assert!(
            catch_unwind(AssertUnwindSafe(|| draw.with_draw_flag(flags, true, || ()))).is_err()
        );
        assert_eq!(unsafe { (*draw.draw_list).Flags }, original);
    }
    let overflow = catch_unwind(AssertUnwindSafe(|| {
        draw.with_draw_flag(DrawFlags::AA_LINE_ENDS, true, || {
            draw.with_draw_flag(DrawFlags::AA_FILL, false, || {
                draw.with_draw_flag(DrawFlags::AA_LINES, false, || {
                    draw.with_draw_flag(DrawFlags::STROKE_LEGACY, true, || ());
                });
            });
        });
    }));
    assert!(overflow.is_err());
    assert_eq!(unsafe { (*draw.draw_list).Flags }, original);
    assert_eq!(unsafe { (*draw.draw_list)._DrawFlagsStack.Size }, 0);
}

#[test]
fn legacy_strokes_and_inverted_rectangles_are_opt_in() {
    let (_guard, mut ctx) = crate::test::test_ctx_initialized();
    assert!(!ctx.io().config_debug_draw_list_defaults_to_stroke_legacy);
    {
        let ui = ctx.frame();
        let draw = ui.get_background_draw_list();
        let before = unsafe { (*draw.draw_list).VtxBuffer.Size };
        draw.add_rect([50.0, 50.0], [10.0, 10.0], ImColor32::WHITE)
            .filled(true)
            .build();
        assert_eq!(unsafe { (*draw.draw_list).VtxBuffer.Size }, before);
        draw.add_rect([50.0, 50.0], [10.0, 10.0], ImColor32::WHITE)
            .filled(true)
            .flags(DrawFlags::STROKE_LEGACY)
            .build();
        assert!(unsafe { (*draw.draw_list).VtxBuffer.Size } > before);
    }
    ctx.render();
    ctx.io_mut()
        .config_debug_draw_list_defaults_to_stroke_legacy = true;
    let draw = ctx.frame().get_background_draw_list();
    assert_eq!(
        unsafe { (*draw.draw_list).Flags } & DrawFlags::STROKE_LEGACY.bits(),
        DrawFlags::STROKE_LEGACY.bits()
    );
}
