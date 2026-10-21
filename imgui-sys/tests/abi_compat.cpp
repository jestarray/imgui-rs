// Defines are set by build.rs via cc::Build - no need to hardcode
#include "imgui.h"
#include <stddef.h>

extern "C" {
    // Core types
    size_t cpp_sizeof_ImWchar() { return sizeof(ImWchar); }
    size_t cpp_sizeof_ImTextureID() { return sizeof(ImTextureID); }
    size_t cpp_sizeof_ImDrawIdx() { return sizeof(ImDrawIdx); }
    
    // Major structures
    size_t cpp_sizeof_ImGuiStyle() { return sizeof(ImGuiStyle); }
    size_t cpp_sizeof_ImGuiIO() { return sizeof(ImGuiIO); }
    size_t cpp_sizeof_ImGuiPlatformIO() { return sizeof(ImGuiPlatformIO); }
    size_t cpp_sizeof_ImGuiViewport() { return sizeof(ImGuiViewport); }
    size_t cpp_sizeof_ImDrawData() { return sizeof(ImDrawData); }
    size_t cpp_sizeof_ImDrawList() { return sizeof(ImDrawList); }
    size_t cpp_sizeof_ImDrawCmd() { return sizeof(ImDrawCmd); }
    size_t cpp_sizeof_ImDrawVert() { return sizeof(ImDrawVert); }
    size_t cpp_sizeof_ImFontAtlas() { return sizeof(ImFontAtlas); }
    size_t cpp_sizeof_ImFont() { return sizeof(ImFont); }
    size_t cpp_sizeof_ImFontConfig() { return sizeof(ImFontConfig); }
    size_t cpp_sizeof_ImFontGlyph() { return sizeof(ImFontGlyph); }
    size_t cpp_sizeof_ImGuiInputTextCallbackData() { return sizeof(ImGuiInputTextCallbackData); }
    size_t cpp_sizeof_ImGuiListClipper() { return sizeof(ImGuiListClipper); }
    size_t cpp_sizeof_ImGuiTableSortSpecs() { return sizeof(ImGuiTableSortSpecs); }
    size_t cpp_sizeof_ImGuiPayload() { return sizeof(ImGuiPayload); }
    size_t cpp_sizeof_ImTextureData() { return sizeof(ImTextureData); }
    
    // Alignment
    size_t cpp_alignof_ImGuiStyle() { return alignof(ImGuiStyle); }
    size_t cpp_alignof_ImGuiIO() { return alignof(ImGuiIO); }
    size_t cpp_alignof_ImDrawData() { return alignof(ImDrawData); }
    size_t cpp_alignof_ImFont() { return alignof(ImFont); }
    
    // Key field offsets - ImFontConfig
    size_t cpp_offset_ImFontConfig_FontData() { return offsetof(ImFontConfig, FontData); }
    size_t cpp_offset_ImFontConfig_PixelSnapH() { return offsetof(ImFontConfig, PixelSnapH); }
    size_t cpp_offset_ImFontConfig_OversampleH() { return offsetof(ImFontConfig, OversampleH); }
    size_t cpp_offset_ImFontConfig_EllipsisChar() { return offsetof(ImFontConfig, EllipsisChar); }
    
    // Key field offsets - ImGuiIO
    size_t cpp_offset_ImGuiIO_DisplaySize() { return offsetof(ImGuiIO, DisplaySize); }
    size_t cpp_offset_ImGuiIO_DeltaTime() { return offsetof(ImGuiIO, DeltaTime); }
    size_t cpp_offset_ImGuiIO_MousePos() { return offsetof(ImGuiIO, MousePos); }
    size_t cpp_offset_ImGuiIO_InputQueueCharacters() { return offsetof(ImGuiIO, InputQueueCharacters); }
    
    // Key field offsets - ImFont (ImWchar fields)
    size_t cpp_offset_ImFont_EllipsisChar() { return offsetof(ImFont, EllipsisChar); }
    size_t cpp_offset_ImFont_FallbackChar() { return offsetof(ImFont, FallbackChar); }
    
    // Key field offsets - ImGuiPlatformIO
    size_t cpp_offset_ImGuiPlatformIO_Platform_LocaleDecimalPoint() { 
        return offsetof(ImGuiPlatformIO, Platform_LocaleDecimalPoint); 
    }
    
    // Key field offsets - ImGuiInputTextCallbackData
    size_t cpp_offset_ImGuiInputTextCallbackData_EventChar() { 
        return offsetof(ImGuiInputTextCallbackData, EventChar); 
    }
    
    // Key field offsets - ImDrawData
    size_t cpp_offset_ImDrawData_Valid() { return offsetof(ImDrawData, Valid); }
    size_t cpp_offset_ImDrawData_CmdLists() { return offsetof(ImDrawData, CmdLists); }
    
    // Key field offsets - ImDrawVert
    size_t cpp_offset_ImDrawVert_pos() { return offsetof(ImDrawVert, pos); }
    size_t cpp_offset_ImDrawVert_uv() { return offsetof(ImDrawVert, uv); }
    size_t cpp_offset_ImDrawVert_col() { return offsetof(ImDrawVert, col); }
    
    // Key field offsets - ImDrawCmd
    size_t cpp_offset_ImDrawCmd_ClipRect() { return offsetof(ImDrawCmd, ClipRect); }
    size_t cpp_offset_ImDrawCmd_TexRef() { return offsetof(ImDrawCmd, TexRef); }
    size_t cpp_offset_ImDrawCmd_ElemCount() { return offsetof(ImDrawCmd, ElemCount); }
}

// Offsets affected by the 1.93.0 rendering overhaul.
extern "C" {
    size_t cpp_sizeof_DrawFlagsStack() { return sizeof(ImSmallStack_<ImDrawFlags, 3>); }
    size_t cpp_offset_ImGuiStyle_AntiAliasedLineEnds() { return offsetof(ImGuiStyle, AntiAliasedLineEnds); }
    size_t cpp_offset_ImGuiStyle_CurveTessellationMaxError() { return offsetof(ImGuiStyle, CurveTessellationMaxError); }
    size_t cpp_offset_ImGuiStyle_Colors() { return offsetof(ImGuiStyle, Colors); }
    size_t cpp_offset_ImGuiIO_ConfigDebugDrawListDefaultsToStrokeLegacy() { return offsetof(ImGuiIO, ConfigDebugDrawListDefaultsToStrokeLegacy); }
    size_t cpp_offset_ImGuiIO_BackendPlatformName() { return offsetof(ImGuiIO, BackendPlatformName); }
    size_t cpp_offset_ImGuiPlatformIO_DrawCallback_SetSamplerFromTex() { return offsetof(ImGuiPlatformIO, DrawCallback_SetSamplerFromTex); }
    size_t cpp_offset_ImGuiPlatformIO_Textures() { return offsetof(ImGuiPlatformIO, Textures); }
    size_t cpp_offset_ImFontAtlas_TexNextUniqueID() { return offsetof(ImFontAtlas, TexNextUniqueID); }
    size_t cpp_offset_ImFontAtlas_Builder() { return offsetof(ImFontAtlas, Builder); }
    size_t cpp_offset_ImFontAtlas_OwnerContext() { return offsetof(ImFontAtlas, OwnerContext); }
    size_t cpp_offset_ImDrawList_Flags() { return offsetof(ImDrawList, Flags); }
    size_t cpp_offset_ImDrawList__DrawFlagsStack() { return offsetof(ImDrawList, _DrawFlagsStack); }
    size_t cpp_offset_ImDrawList__InvFringeScale() { return offsetof(ImDrawList, _InvFringeScale); }
    size_t cpp_offset_ImDrawList__OwnerName() { return offsetof(ImDrawList, _OwnerName); }
}
