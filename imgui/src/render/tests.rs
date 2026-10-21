use crate::{DrawCmd, OwnedDrawData, RendererCallback, TextureId, TextureStatus};

#[test]
fn texture_requests_and_owned_snapshot_survive_context() {
    let (_guard, mut ctx) = crate::test::test_ctx_initialized();
    assert_eq!(ctx.textures().count(), 0);
    ctx.frame().get_background_draw_list().add_text(
        [10.0, 10.0],
        crate::ImColor32::WHITE,
        "Texture snapshot",
    );
    let data = ctx.render();
    let owned = OwnedDrawData::from(data);
    let snapshot = owned.draw_data().unwrap();
    let mut copied = snapshot.textures().next().unwrap();
    let other_view = snapshot.textures().next().unwrap();
    assert_eq!(copied.status(), TextureStatus::WantCreate);
    let pixels = copied.pixels().to_vec();
    assert!(!pixels.is_empty());
    assert_eq!(pixels.len(), copied.pitch() * copied.size()[1] as usize);
    let id = TextureId::from_u64(0x1234_5678_9876_5432);
    copied.mark_created(id);
    assert_eq!(copied.status(), TextureStatus::Ok);
    assert_eq!(copied.texture_id(), id);
    assert_eq!(other_view.texture_id(), id);
    assert!(snapshot.draw_lists().flat_map(|list| list.commands()).any(
        |cmd| matches!(cmd, DrawCmd::Elements { cmd_params, .. } if cmd_params.texture_id == id)
    ));
    assert_eq!(
        data.textures().next().unwrap().status(),
        TextureStatus::WantCreate
    );
    for mut texture in data.textures() {
        texture.mark_created(TextureId::new(1));
    }
    drop(ctx);
    assert_eq!(copied.pixels(), pixels);
    assert!(snapshot.total_vtx_count > 0);
    copied.mark_destroyed();
    assert_eq!(copied.texture_id(), TextureId::INVALID);
}

#[test]
fn texture_updates_are_acknowledged_after_upload() {
    let (_guard, mut ctx) = crate::test::test_ctx_initialized();
    ctx.frame().text("Initial glyphs");
    for mut texture in ctx.render().textures() {
        if texture.status() == TextureStatus::WantCreate {
            texture.mark_created(TextureId::new(1));
        }
    }
    let ui = ctx.frame();
    let _size = ui.push_font_size(27.0);
    ui.current_font_baked().find_glyph('Z');
    _size.pop();
    let mut updated = false;
    for mut texture in ctx.render().textures() {
        match texture.status() {
            TextureStatus::WantUpdates => {
                updated = true;
                assert!(!texture.updates().is_empty());
                let rect = texture.update_rect();
                assert!(u32::from(rect.x) + u32::from(rect.width) <= texture.size()[0]);
                assert!(u32::from(rect.y) + u32::from(rect.height) <= texture.size()[1]);
                texture.mark_updated();
                assert_eq!(texture.status(), TextureStatus::Ok);
            }
            TextureStatus::WantCreate => texture.mark_created(TextureId::new(2)),
            _ => (),
        }
    }
    assert!(updated);
    for mut texture in ctx.textures() {
        texture.mark_destroyed();
    }
}

#[test]
fn standard_renderer_callbacks_are_optional_and_classified() {
    let (_guard, mut ctx) = crate::test::test_ctx_initialized();
    let expected = [
        RendererCallback::ResetRenderState,
        RendererCallback::SetSamplerLinear,
        RendererCallback::SetSamplerNearest,
        RendererCallback::SetSamplerFromTexture,
    ];
    let callbacks = ctx.platform_io_mut().register_renderer_callbacks(&expected);
    {
        let draw = ctx.frame().get_background_draw_list();
        for kind in expected {
            assert!(callbacks.supports(kind));
            assert!(draw.add_renderer_callback(kind));
        }
    }
    let actual: Vec<_> = ctx
        .render()
        .draw_lists()
        .flat_map(|list| list.commands())
        .filter_map(|cmd| callbacks.classify(&cmd))
        .collect();
    assert_eq!(actual, expected);
    ctx.platform_io_mut().draw_callback_set_sampler_nearest = None;
    assert!(!ctx
        .platform_io()
        .renderer_callbacks()
        .supports(RendererCallback::SetSamplerNearest));
    assert!(!ctx
        .frame()
        .get_background_draw_list()
        .add_renderer_callback(RendererCallback::SetSamplerNearest));
}

#[test]
fn texture_registry_preserves_full_width_ids_and_reserves_zero() {
    let mut textures = crate::Textures::new();
    let id = textures.insert("first");
    assert_ne!(id, TextureId::INVALID);
    let wide = TextureId::from_u64(u64::MAX - 10);
    textures.replace(wide, "wide");
    assert_eq!(textures.get(wide), Some(&"wide"));
    assert_eq!(wide.id(), u64::MAX - 10);
}
