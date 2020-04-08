-- vim: noexpandtab ts=4 sw=4
require "tundra.syntax.rust-cargo"
require "tundra.syntax.dotnet"
local native = require "tundra.native"
local path = require "tundra.path"

local function get_src(dir, recursive)
	return FGlob {
		Dir = dir,
		Extensions = { ".cpp", ".c", ".h", ".s", ".m" },
		Filters = {
			{ Pattern = "[/\\]_unittest[/\\]"; Config = "win32-*" },
			{ Pattern = "[/\\]_win[/\\]"; Config = "win32-*" },
			{ Pattern = "[/\\]_macos[/\\]"; Config = "mac*-*" },
			{ Pattern = "[/\\]_linux[/\\]"; Config = "linux-*" },
			{ Pattern = "[/\\]_posix[/\\]"; Config = { "mac*-*", "linux-*" } },
		},
		Recursive = recursive and true or false,
	}
end

local angle_dir = "external/angle/"
local common_dir = angle_dir .. "src/common/"

-- Common sources/headers
local common = {
	common_dir .. "aligned_memory.cpp",
	common_dir .. "aligned_memory.h",
	common_dir .. "android_util.cpp",
	common_dir .. "android_util.h",
	common_dir .. "angleutils.cpp",
	common_dir .. "angleutils.h",
	common_dir .. "apple_platform_utils.h",
	common_dir .. "bitset_utils.h",
	common_dir .. "Color.h",
	common_dir .. "debug.cpp",
	common_dir .. "debug.h",
	common_dir .. "event_tracer.cpp",
	common_dir .. "event_tracer.h",
	common_dir .. "FastVector.h",
	common_dir .. "FixedVector.h",
	common_dir .. "Float16ToFloat32.cpp",
	common_dir .. "hash_utils.h",
	common_dir .. "mathutil.cpp",
	common_dir .. "mathutil.h",
	common_dir .. "matrix_utils.cpp",
	common_dir .. "matrix_utils.h",
	common_dir .. "MemoryBuffer.cpp",
	common_dir .. "MemoryBuffer.h",
	common_dir .. "Optional.h",
	common_dir .. "PackedEGLEnums_autogen.cpp",
	common_dir .. "PackedEGLEnums_autogen.h",
	common_dir .. "PackedEnums.cpp",
	common_dir .. "PackedEnums.h",
	common_dir .. "PackedGLEnums_autogen.cpp",
	common_dir .. "PackedGLEnums_autogen.h",
	common_dir .. "platform.h",
	common_dir .. "PoolAlloc.cpp",
	common_dir .. "PoolAlloc.h",
	common_dir .. "string_utils.cpp",
	common_dir .. "string_utils.h",
	common_dir .. "system_utils.cpp",
	common_dir .. "system_utils.h",
	{ common_dir .. "system_utils_linux.cpp"; Config = "linux-*" },
	{ common_dir .. "system_utils_mac.cpp"; Config = "mac*-*" },
	{ common_dir .. "system_utils_posix.cpp"; Config = { "linux-*", "mac*-*" } },
	{ common_dir .. "system_utils_win32.cpp"; Config = "win32-*" },
	{ common_dir .. "system_utils_win.cpp"; Config = "win32-*" },
	{ common_dir .. "system_utils_winuwp.cpp"; Config = "win32-*" },
	common_dir .. "tls.cpp",
	common_dir .. "tls.h",
	common_dir .. "uniform_type_info_autogen.cpp",
	common_dir .. "utilities.cpp",
	common_dir .. "utilities.h",
	common_dir .. "vector_utils.h",
	common_dir .. "version.h",
	get_src(common_dir .. "third_party", true),
}

StaticLibrary {
	Name = "angle",

	Defines = {
        "_CRT_SECURE_NO_DEPRECATE",
        "_SCL_SECURE_NO_WARNINGS",
        "_HAS_EXCEPTIONS=0",
        "NOMINMAX",
        "ANGLE_STANDALONE_BUILD",
        "ANGLE_ENABLE_DEBUG_ANNOTATIONS",
        -- TODO: Change this for actually replay target, on PC we assume 64-bit
        "ANGLE_IS_64_BIT_CPU",
    },

    Includes = {
    	angle_dir .. "src/common/third_party/base",
        angle_dir .. "include",
        angle_dir .. "src",
    },

    Sources = {
    	common,
    }
}

Program {
    Name = "dummy",
    Sources = { "src/dummy/dummy.cpp" },
    Depends = { "angle" }
}

Always "dummy"
