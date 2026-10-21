use super::*;

#[test]
fn matching_syntax_and_exclusion_precedence() {
    let (_guard, _ctx) = crate::test::test_ctx();
    for (pattern, yes, no) in [
        ("hello world", "WORLD then hello", "hello alone"),
        ("hello,world", "world alone", "neither word"),
        (
            "\"hello world\"",
            "a hello world phrase",
            "hello wide world",
        ),
        ("hello,-world", "hello alone", "hello world"),
        ("-world,hello", "hello alone", "hello world"),
        ("-world", "anything else", "world"),
    ] {
        let filter = TextFilter::new_with_filter("Filter".into(), pattern.into());
        filter.build();
        assert!(filter.is_active());
        assert!(filter.pass_filter(yes), "{pattern}: {yes}");
        assert!(!filter.pass_filter(no), "{pattern}: {no}");
        filter.clear();
        assert!(!filter.is_active());
        assert!(filter.pass_filter(no));
    }
}

#[test]
fn draw_widths_and_hint() {
    let (_guard, mut ctx) = crate::test::test_ctx_initialized();
    let filter = TextFilter::new("##filter".into());
    let ui = ctx.frame();
    ui.window("Filters")
        .size([600.0, 300.0], crate::Condition::Always)
        .build(|| {
            filter.draw_with_size(123.0);
            assert_eq!(ui.item_rect_size()[0], 123.0);
            ui.set_next_item_width(145.0);
            filter.draw_with_size(0.0);
            assert_eq!(ui.item_rect_size()[0], 145.0);
            ui.set_next_item_width(167.0);
            assert!(!filter.draw_with_hint("Search phrases and -exclusions"));
            assert_eq!(ui.item_rect_size()[0], 167.0);
        });
    assert!(!filter.is_active());
    ctx.render();
}

#[test]
fn mixed_value_item_flags_are_scoped() {
    let (_guard, mut ctx) = crate::test::test_ctx_initialized();
    let ui = ctx.frame();
    let mut value = false;
    ui.window("Mixed values").build(|| {
        {
            let _mixed = ui.push_item_flag(crate::ItemFlag::MIXED_VALUE, true);
            let _live = ui.push_item_flag(crate::ItemFlag::LIVE_EDIT_ON_INPUT, false);
            ui.checkbox("Mixed", &mut value);
            let flags = unsafe { sys::igGetItemFlags() };
            assert_ne!(flags & crate::ItemFlag::MIXED_VALUE.bits(), 0);
            assert_eq!(flags & crate::ItemFlag::LIVE_EDIT_ON_INPUT.bits(), 0);
        }
        ui.checkbox("Normal", &mut value);
        assert_eq!(
            unsafe { sys::igGetItemFlags() } & crate::ItemFlag::MIXED_VALUE.bits(),
            0
        );
    });
    ctx.render();
}
