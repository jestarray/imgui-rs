use std::mem::{align_of, offset_of, size_of};

extern "C" {
    // Core types
    fn cpp_sizeof_ImWchar() -> usize;
    fn cpp_sizeof_ImTextureID() -> usize;
    fn cpp_sizeof_ImDrawIdx() -> usize;

    // Major structures
    fn cpp_sizeof_ImGuiStyle() -> usize;
    fn cpp_sizeof_ImGuiIO() -> usize;
    fn cpp_sizeof_ImGuiPlatformIO() -> usize;
    fn cpp_sizeof_ImGuiViewport() -> usize;
    fn cpp_sizeof_ImDrawData() -> usize;
    fn cpp_sizeof_ImDrawList() -> usize;
    fn cpp_sizeof_ImDrawCmd() -> usize;
    fn cpp_sizeof_ImDrawVert() -> usize;
    fn cpp_sizeof_ImFontAtlas() -> usize;
    fn cpp_sizeof_ImFont() -> usize;
    fn cpp_sizeof_ImFontConfig() -> usize;
    fn cpp_sizeof_ImFontGlyph() -> usize;
    fn cpp_sizeof_ImGuiInputTextCallbackData() -> usize;
    fn cpp_sizeof_ImGuiListClipper() -> usize;
    fn cpp_sizeof_ImGuiTableSortSpecs() -> usize;
    fn cpp_sizeof_ImGuiPayload() -> usize;
    fn cpp_sizeof_ImTextureData() -> usize;

    // Alignment
    fn cpp_alignof_ImGuiStyle() -> usize;
    fn cpp_alignof_ImGuiIO() -> usize;
    fn cpp_alignof_ImDrawData() -> usize;
    fn cpp_alignof_ImFont() -> usize;

    // Key field offsets
    fn cpp_offset_ImFontConfig_FontData() -> usize;
    fn cpp_offset_ImFontConfig_PixelSnapH() -> usize;
    fn cpp_offset_ImFontConfig_OversampleH() -> usize;
    fn cpp_offset_ImFontConfig_EllipsisChar() -> usize;

    fn cpp_offset_ImGuiIO_DisplaySize() -> usize;
    fn cpp_offset_ImGuiIO_DeltaTime() -> usize;
    fn cpp_offset_ImGuiIO_MousePos() -> usize;
    fn cpp_offset_ImGuiIO_InputQueueCharacters() -> usize;

    fn cpp_offset_ImFont_EllipsisChar() -> usize;
    fn cpp_offset_ImFont_FallbackChar() -> usize;

    fn cpp_offset_ImGuiPlatformIO_Platform_LocaleDecimalPoint() -> usize;

    fn cpp_offset_ImGuiInputTextCallbackData_EventChar() -> usize;

    fn cpp_offset_ImDrawData_Valid() -> usize;
    fn cpp_offset_ImDrawData_CmdLists() -> usize;

    fn cpp_offset_ImDrawVert_pos() -> usize;
    fn cpp_offset_ImDrawVert_uv() -> usize;
    fn cpp_offset_ImDrawVert_col() -> usize;

    fn cpp_offset_ImDrawCmd_ClipRect() -> usize;
    fn cpp_offset_ImDrawCmd_TexRef() -> usize;
    fn cpp_offset_ImDrawCmd_ElemCount() -> usize;
}

#[test]
fn test_abi_compatibility() {
    unsafe {
        // Core types - verify WCHAR32, ImTextureID, ImDrawIdx
        assert_eq!(
            size_of::<imgui_sys::ImWchar>(),
            cpp_sizeof_ImWchar(),
            "ImWchar size mismatch (should be 4 bytes with WCHAR32)"
        );
        assert_eq!(
            size_of::<imgui_sys::ImTextureID>(),
            cpp_sizeof_ImTextureID(),
            "ImTextureID size mismatch"
        );
        assert_eq!(
            size_of::<imgui_sys::ImDrawIdx>(),
            cpp_sizeof_ImDrawIdx(),
            "ImDrawIdx size mismatch"
        );

        // Major structures - sizeof
        assert_eq!(
            size_of::<imgui_sys::ImGuiStyle>(),
            cpp_sizeof_ImGuiStyle(),
            "ImGuiStyle size mismatch"
        );
        assert_eq!(
            size_of::<imgui_sys::ImGuiIO>(),
            cpp_sizeof_ImGuiIO(),
            "ImGuiIO size mismatch"
        );
        assert_eq!(
            size_of::<imgui_sys::ImGuiPlatformIO>(),
            cpp_sizeof_ImGuiPlatformIO(),
            "ImGuiPlatformIO size mismatch"
        );
        assert_eq!(
            size_of::<imgui_sys::ImGuiViewport>(),
            cpp_sizeof_ImGuiViewport(),
            "ImGuiViewport size mismatch"
        );
        assert_eq!(
            size_of::<imgui_sys::ImDrawData>(),
            cpp_sizeof_ImDrawData(),
            "ImDrawData size mismatch"
        );
        assert_eq!(
            size_of::<imgui_sys::ImDrawList>(),
            cpp_sizeof_ImDrawList(),
            "ImDrawList size mismatch"
        );
        assert_eq!(
            size_of::<imgui_sys::ImDrawCmd>(),
            cpp_sizeof_ImDrawCmd(),
            "ImDrawCmd size mismatch"
        );
        assert_eq!(
            size_of::<imgui_sys::ImDrawVert>(),
            cpp_sizeof_ImDrawVert(),
            "ImDrawVert size mismatch"
        );
        assert_eq!(
            size_of::<imgui_sys::ImFontAtlas>(),
            cpp_sizeof_ImFontAtlas(),
            "ImFontAtlas size mismatch"
        );
        assert_eq!(
            size_of::<imgui_sys::ImFont>(),
            cpp_sizeof_ImFont(),
            "ImFont size mismatch"
        );
        assert_eq!(
            size_of::<imgui_sys::ImFontConfig>(),
            cpp_sizeof_ImFontConfig(),
            "ImFontConfig size mismatch (check IMGUI_DISABLE_OBSOLETE_FUNCTIONS)"
        );
        assert_eq!(
            size_of::<imgui_sys::ImFontGlyph>(),
            cpp_sizeof_ImFontGlyph(),
            "ImFontGlyph size mismatch"
        );
        assert_eq!(
            size_of::<imgui_sys::ImGuiInputTextCallbackData>(),
            cpp_sizeof_ImGuiInputTextCallbackData(),
            "ImGuiInputTextCallbackData size mismatch"
        );
        assert_eq!(
            size_of::<imgui_sys::ImGuiListClipper>(),
            cpp_sizeof_ImGuiListClipper(),
            "ImGuiListClipper size mismatch"
        );
        assert_eq!(
            size_of::<imgui_sys::ImGuiTableSortSpecs>(),
            cpp_sizeof_ImGuiTableSortSpecs(),
            "ImGuiTableSortSpecs size mismatch"
        );
        assert_eq!(
            size_of::<imgui_sys::ImGuiPayload>(),
            cpp_sizeof_ImGuiPayload(),
            "ImGuiPayload size mismatch"
        );
        assert_eq!(
            size_of::<imgui_sys::ImTextureData>(),
            cpp_sizeof_ImTextureData(),
            "ImTextureData size mismatch"
        );

        // Alignment
        assert_eq!(
            align_of::<imgui_sys::ImGuiStyle>(),
            cpp_alignof_ImGuiStyle(),
            "ImGuiStyle alignment mismatch"
        );
        assert_eq!(
            align_of::<imgui_sys::ImGuiIO>(),
            cpp_alignof_ImGuiIO(),
            "ImGuiIO alignment mismatch"
        );
        assert_eq!(
            align_of::<imgui_sys::ImDrawData>(),
            cpp_alignof_ImDrawData(),
            "ImDrawData alignment mismatch"
        );
        assert_eq!(
            align_of::<imgui_sys::ImFont>(),
            cpp_alignof_ImFont(),
            "ImFont alignment mismatch"
        );

        // Key field offsets for ImFontConfig
        assert_eq!(
            offset_of!(imgui_sys::ImFontConfig, FontData),
            cpp_offset_ImFontConfig_FontData(),
            "ImFontConfig::FontData offset mismatch"
        );
        assert_eq!(
            offset_of!(imgui_sys::ImFontConfig, PixelSnapH),
            cpp_offset_ImFontConfig_PixelSnapH(),
            "ImFontConfig::PixelSnapH offset mismatch"
        );
        assert_eq!(
            offset_of!(imgui_sys::ImFontConfig, OversampleH),
            cpp_offset_ImFontConfig_OversampleH(),
            "ImFontConfig::OversampleH offset mismatch"
        );
        assert_eq!(
            offset_of!(imgui_sys::ImFontConfig, EllipsisChar),
            cpp_offset_ImFontConfig_EllipsisChar(),
            "ImFontConfig::EllipsisChar offset mismatch (ImWchar field)"
        );

        // Key field offsets for ImGuiIO
        assert_eq!(
            offset_of!(imgui_sys::ImGuiIO, DisplaySize),
            cpp_offset_ImGuiIO_DisplaySize(),
            "ImGuiIO::DisplaySize offset mismatch"
        );
        assert_eq!(
            offset_of!(imgui_sys::ImGuiIO, DeltaTime),
            cpp_offset_ImGuiIO_DeltaTime(),
            "ImGuiIO::DeltaTime offset mismatch"
        );
        assert_eq!(
            offset_of!(imgui_sys::ImGuiIO, MousePos),
            cpp_offset_ImGuiIO_MousePos(),
            "ImGuiIO::MousePos offset mismatch"
        );
        assert_eq!(
            offset_of!(imgui_sys::ImGuiIO, InputQueueCharacters),
            cpp_offset_ImGuiIO_InputQueueCharacters(),
            "ImGuiIO::InputQueueCharacters offset mismatch"
        );

        // Key field offsets for ImFont (verify ImWchar fields)
        assert_eq!(
            offset_of!(imgui_sys::ImFont, EllipsisChar),
            cpp_offset_ImFont_EllipsisChar(),
            "ImFont::EllipsisChar offset mismatch (ImWchar field)"
        );
        assert_eq!(
            offset_of!(imgui_sys::ImFont, FallbackChar),
            cpp_offset_ImFont_FallbackChar(),
            "ImFont::FallbackChar offset mismatch (ImWchar field)"
        );

        // Key field offsets for ImGuiPlatformIO
        assert_eq!(
            offset_of!(imgui_sys::ImGuiPlatformIO, Platform_LocaleDecimalPoint),
            cpp_offset_ImGuiPlatformIO_Platform_LocaleDecimalPoint(),
            "ImGuiPlatformIO::Platform_LocaleDecimalPoint offset mismatch (ImWchar field)"
        );

        // Key field offsets for ImGuiInputTextCallbackData
        assert_eq!(
            offset_of!(imgui_sys::ImGuiInputTextCallbackData, EventChar),
            cpp_offset_ImGuiInputTextCallbackData_EventChar(),
            "ImGuiInputTextCallbackData::EventChar offset mismatch (ImWchar field)"
        );

        // Key field offsets for ImDrawData
        assert_eq!(
            offset_of!(imgui_sys::ImDrawData, Valid),
            cpp_offset_ImDrawData_Valid(),
            "ImDrawData::Valid offset mismatch"
        );
        assert_eq!(
            offset_of!(imgui_sys::ImDrawData, CmdLists),
            cpp_offset_ImDrawData_CmdLists(),
            "ImDrawData::CmdLists offset mismatch"
        );

        // Key field offsets for ImDrawVert
        assert_eq!(
            offset_of!(imgui_sys::ImDrawVert, pos),
            cpp_offset_ImDrawVert_pos(),
            "ImDrawVert::pos offset mismatch"
        );
        assert_eq!(
            offset_of!(imgui_sys::ImDrawVert, uv),
            cpp_offset_ImDrawVert_uv(),
            "ImDrawVert::uv offset mismatch"
        );
        assert_eq!(
            offset_of!(imgui_sys::ImDrawVert, col),
            cpp_offset_ImDrawVert_col(),
            "ImDrawVert::col offset mismatch"
        );

        // Key field offsets for ImDrawCmd
        assert_eq!(
            offset_of!(imgui_sys::ImDrawCmd, ClipRect),
            cpp_offset_ImDrawCmd_ClipRect(),
            "ImDrawCmd::ClipRect offset mismatch"
        );
        assert_eq!(
            offset_of!(imgui_sys::ImDrawCmd, TexRef),
            cpp_offset_ImDrawCmd_TexRef(),
            "ImDrawCmd::TexRef offset mismatch"
        );
        assert_eq!(
            offset_of!(imgui_sys::ImDrawCmd, ElemCount),
            cpp_offset_ImDrawCmd_ElemCount(),
            "ImDrawCmd::ElemCount offset mismatch"
        );
    }
}

#[test]
fn test_abi_1930_rendering_changes() {
    extern "C" {
        fn cpp_sizeof_DrawFlagsStack() -> usize;
        fn cpp_offset_ImGuiStyle_AntiAliasedLineEnds() -> usize;
        fn cpp_offset_ImGuiStyle_CurveTessellationMaxError() -> usize;
        fn cpp_offset_ImGuiStyle_Colors() -> usize;
        fn cpp_offset_ImGuiIO_ConfigDebugDrawListDefaultsToStrokeLegacy() -> usize;
        fn cpp_offset_ImGuiIO_BackendPlatformName() -> usize;
        fn cpp_offset_ImGuiPlatformIO_DrawCallback_SetSamplerFromTex() -> usize;
        fn cpp_offset_ImGuiPlatformIO_Textures() -> usize;
        fn cpp_offset_ImFontAtlas_TexNextUniqueID() -> usize;
        fn cpp_offset_ImFontAtlas_Builder() -> usize;
        fn cpp_offset_ImFontAtlas_OwnerContext() -> usize;
        fn cpp_offset_ImDrawList_Flags() -> usize;
        fn cpp_offset_ImDrawList__DrawFlagsStack() -> usize;
        fn cpp_offset_ImDrawList__InvFringeScale() -> usize;
        fn cpp_offset_ImDrawList__OwnerName() -> usize;
    }
    assert_eq!(
        unsafe { std::ffi::CStr::from_ptr(imgui_sys::igGetVersion()) }.to_bytes(),
        b"1.93.0"
    );
    unsafe {
        assert_eq!(
            size_of::<imgui_sys::ImSmallStack__ImDrawFlags__3>(),
            cpp_sizeof_DrawFlagsStack()
        );
        assert_eq!(
            offset_of!(imgui_sys::ImGuiStyle, AntiAliasedLineEnds),
            cpp_offset_ImGuiStyle_AntiAliasedLineEnds(),
            "ImGuiStyle::AntiAliasedLineEnds"
        );
        assert_eq!(
            offset_of!(imgui_sys::ImGuiStyle, CurveTessellationMaxError),
            cpp_offset_ImGuiStyle_CurveTessellationMaxError(),
            "ImGuiStyle::CurveTessellationMaxError"
        );
        assert_eq!(
            offset_of!(imgui_sys::ImGuiStyle, Colors),
            cpp_offset_ImGuiStyle_Colors(),
            "ImGuiStyle::Colors"
        );
        assert_eq!(
            offset_of!(
                imgui_sys::ImGuiIO,
                ConfigDebugDrawListDefaultsToStrokeLegacy
            ),
            cpp_offset_ImGuiIO_ConfigDebugDrawListDefaultsToStrokeLegacy(),
            "ImGuiIO::ConfigDebugDrawListDefaultsToStrokeLegacy"
        );
        assert_eq!(
            offset_of!(imgui_sys::ImGuiIO, BackendPlatformName),
            cpp_offset_ImGuiIO_BackendPlatformName(),
            "ImGuiIO::BackendPlatformName"
        );
        assert_eq!(
            offset_of!(imgui_sys::ImGuiPlatformIO, DrawCallback_SetSamplerFromTex),
            cpp_offset_ImGuiPlatformIO_DrawCallback_SetSamplerFromTex(),
            "ImGuiPlatformIO::DrawCallback_SetSamplerFromTex"
        );
        assert_eq!(
            offset_of!(imgui_sys::ImGuiPlatformIO, Textures),
            cpp_offset_ImGuiPlatformIO_Textures(),
            "ImGuiPlatformIO::Textures"
        );
        assert_eq!(
            offset_of!(imgui_sys::ImFontAtlas, TexNextUniqueID),
            cpp_offset_ImFontAtlas_TexNextUniqueID(),
            "ImFontAtlas::TexNextUniqueID"
        );
        assert_eq!(
            offset_of!(imgui_sys::ImFontAtlas, Builder),
            cpp_offset_ImFontAtlas_Builder(),
            "ImFontAtlas::Builder"
        );
        assert_eq!(
            offset_of!(imgui_sys::ImFontAtlas, OwnerContext),
            cpp_offset_ImFontAtlas_OwnerContext(),
            "ImFontAtlas::OwnerContext"
        );
        assert_eq!(
            offset_of!(imgui_sys::ImDrawList, Flags),
            cpp_offset_ImDrawList_Flags(),
            "ImDrawList::Flags"
        );
        assert_eq!(
            offset_of!(imgui_sys::ImDrawList, _DrawFlagsStack),
            cpp_offset_ImDrawList__DrawFlagsStack(),
            "ImDrawList::_DrawFlagsStack"
        );
        assert_eq!(
            offset_of!(imgui_sys::ImDrawList, _InvFringeScale),
            cpp_offset_ImDrawList__InvFringeScale(),
            "ImDrawList::_InvFringeScale"
        );
        assert_eq!(
            offset_of!(imgui_sys::ImDrawList, _OwnerName),
            cpp_offset_ImDrawList__OwnerName(),
            "ImDrawList::_OwnerName"
        );
    }
}
