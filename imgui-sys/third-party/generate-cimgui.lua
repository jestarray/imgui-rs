-- cimgui c4461a7 does not recognize ImGui 1.93.0's debug-stepping attribute.
-- Strip only that attribute from preprocessed input, leaving the vendored
-- ImGui headers and the pinned generator unchanged.
local cpp2ffi = require "cpp2ffi"
local location = cpp2ffi.location
cpp2ffi.location = function(...)
    local next_line = location(...)
    return function()
        local line, source, number = next_line()
        if line then
            return (line:gsub("%[%[gnu::artificial%]%]", "")), source, number
        end
    end
end
assert(loadfile("generator.lua"))(unpack(arg))
