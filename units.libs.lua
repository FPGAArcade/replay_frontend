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

-- Common sources/headers
local common = {
	"external/angle/src/common/aligned_memory.cpp",
	"external/angle/src/common/aligned_memory.h",
	"external/angle/src/common/android_util.cpp",
	"external/angle/src/common/android_util.h",
	"external/angle/src/common/angleutils.cpp",
	"external/angle/src/common/angleutils.h",
	"external/angle/src/common/apple_platform_utils.h",
	"external/angle/src/common/bitset_utils.h",
	"external/angle/src/common/Color.h",
	"external/angle/src/common/debug.cpp",
	"external/angle/src/common/debug.h",
	"external/angle/src/common/event_tracer.cpp",
	"external/angle/src/common/event_tracer.h",
	"external/angle/src/common/FastVector.h",
	"external/angle/src/common/FixedVector.h",
	"external/angle/src/common/Float16ToFloat32.cpp",
	"external/angle/src/common/hash_utils.h",
	"external/angle/src/common/mathutil.cpp",
	"external/angle/src/common/mathutil.h",
	"external/angle/src/common/matrix_utils.cpp",
	"external/angle/src/common/matrix_utils.h",
	"external/angle/src/common/MemoryBuffer.cpp",
	"external/angle/src/common/MemoryBuffer.h",
	"external/angle/src/common/Optional.h",
	"external/angle/src/common/PackedEGLEnums_autogen.cpp",
	"external/angle/src/common/PackedEGLEnums_autogen.h",
	"external/angle/src/common/PackedEnums.cpp",
	"external/angle/src/common/PackedEnums.h",
	"external/angle/src/common/PackedGLEnums_autogen.cpp",
	"external/angle/src/common/PackedGLEnums_autogen.h",
	"external/angle/src/common/platform.h",
	"external/angle/src/common/PoolAlloc.cpp",
	"external/angle/src/common/PoolAlloc.h",
	"external/angle/src/common/string_utils.cpp",
	"external/angle/src/common/string_utils.h",
	"external/angle/src/common/system_utils.cpp",
	"external/angle/src/common/system_utils.h",
	{ "external/angle/src/common/system_utils_linux.cpp"; Config = "linux-*" },
	{ "external/angle/src/common/system_utils_mac.cpp"; Config = "mac*-*" },
	{ "external/angle/src/common/system_utils_posix.cpp"; Config = { "linux-*", "mac*-*" } },
	{ "external/angle/src/common/system_utils_win32.cpp"; Config = "win32-*" },
	{ "external/angle/src/common/system_utils_win.cpp"; Config = "win32-*" },
	{ "external/angle/src/common/system_utils_winuwp.cpp"; Config = "win32-*" },
	"external/angle/src/common/tls.cpp",
	"external/angle/src/common/tls.h",
	"external/angle/src/common/uniform_type_info_autogen.cpp",
	"external/angle/src/common/utilities.cpp",
	"external/angle/src/common/utilities.h",
	"external/angle/src/common/vector_utils.h",
	"external/angle/src/common/version.h",

	get_src("external/angle/src/common/third_party", true),
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
    	"external/angle/src/common/third_party/base",
        "external/angle/include",
        "external/angle/src",
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
