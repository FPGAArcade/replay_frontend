require "tundra.syntax.glob"
require "tundra.path"
require "tundra.util"
require "tundra.syntax.rust-cargo"

local native = require('tundra.native')

local BIMG_DIR = "external/bimg/"
local BX_DIR = "external/bx/"
local BGFX_DIR = "external/bgfx/"
local GLSL_OPTIMIZER = BGFX_DIR  .. "3rdparty/glsl-optimizer/"
local FCPP_DIR = BGFX_DIR .. "3rdparty/fcpp/"

-- setup target for shader
local shaderc_platform = "windows"
local shaderc_vs_extra_params = " -p vs_5_0"
local shaderc_ps_extra_params = " -p ps_5_0"
if native.host_platform == "macosx" then
	shaderc_platform = "osx"
	shaderc_vs_extra_params = ""
	shaderc_ps_extra_params = ""
elseif native.host_platform == "linux" then
	shaderc_platform = "linux"
	shaderc_vs_extra_params = ""
	shaderc_ps_extra_params = ""
end

-----------------------------------------------------------------------------------------------------------------------

local function get_c_cpp_src(dir)
    return Glob {
        Dir = dir,
        Extensions = { ".cpp", ".c", ".h" },
        Recursive = true,
}
end

-----------------------------------------------------------------------------------------------------------------------

DefRule {
	Name = "ShadercFS",
	Command = "$(BGFX_SHADERC) -f $(<) -o $(@) --type fragment --platform " .. shaderc_platform .. shaderc_ps_extra_params,

	Blueprint = {
		Source = { Required = true, Type = "string", Help = "Input filename", },
		OutName = { Required = false, Type = "string", Help = "Output filename", },
	},

	Setup = function (env, data)
		return {
			InputFiles    = { data.Source },
			OutputFiles   = { "$(OBJECTDIR)/_generated/" .. path.drop_suffix(data.Source) .. ".fs" },
		}
	end,
}

-----------------------------------------------------------------------------------------------------------------------

DefRule {
	Name = "ShadercVS",
	Command = "$(BGFX_SHADERC) -f $(<) -o $(@) --type vertex --platform " .. shaderc_platform .. shaderc_vs_extra_params,

	Blueprint = {
		Source = { Required = true, Type = "string", Help = "Input filename", },
		OutName = { Required = false, Type = "string", Help = "Output filename", },
	},

	Setup = function (env, data)
		return {
			InputFiles    = { data.Source },
			OutputFiles   = { "$(OBJECTDIR)/_generated/" .. path.drop_suffix(data.Source) .. ".vs" },
		}
	end,
}

-----------------------------------------------------------------------------------------------------------------------

Program {
	Name = "bgfx_shaderc",
	Target = "$(BGFX_SHADERC)",
	Pass = "BuildTools",

    Env = {
        CCOPTS = {
			{ "/wd4291", "/W3", "-D__STDC__", "-D__STDC_VERSION__=199901L", "-Dstrdup=_strdup", "-Dalloca=_alloca", "-Disascii=__isascii"; Config = "win64-*-*" },
        	{ "-Wno-everything"; Config = "macosx-*-*" },
			{ "-fno-strict-aliasing"; Config = { "macosx-*-*", "linux-*-*" } },
        },

        CXXOPTS = {
			{ "/wd4291", "/W3", "-D__STDC__", "-D__STDC_VERSION__=199901L", "-Dstrdup=_strdup", "-Dalloca=_alloca", "-Disascii=__isascii"; Config = "win64-*-*" },
        	{ "-Wno-everything"; Config = "macosx-*-*" },
			{ "-fno-strict-aliasing"; Config = { "macosx-*-*", "linux-*-*" } },
        },

		CPPDEFS = {
			{ "NINCLUDE=64", "NWORK=65536", "NBUFF=65536", "OLD_PREPROCESSOR=0" },
		},

		CPPPATH = {
			{
				BX_DIR .. "include",
				BGFX_DIR .. "include",
				FCPP_DIR,
				GLSL_OPTIMIZER .. "src",
				GLSL_OPTIMIZER .. "include",
				GLSL_OPTIMIZER .. "mesa",
				GLSL_OPTIMIZER .. "mapi",
				GLSL_OPTIMIZER .. "glsl",
				GLSL_OPTIMIZER .. "glsl/glcpp",
			},

			{
				BX_DIR .. "include/compat/osx" ; Config = "macosx-*-*"
			},

			{
				BX_DIR .. "include/compat/msvc"; Config = "win64-*-*"
			},
		},

		PROGCOM = {
			{ "-lstdc++"; Config = { "macosx-clang-*", "linux-gcc-*" } },
			{ "-lm -lpthread -ldl -lX11"; Config = "linux-*-*" },
		},
	},

	Sources = {
		BGFX_DIR .. "tools/shaderc/shaderc.cpp",
		BGFX_DIR .. "tools/shaderc/shaderc_glsl.cpp",
		BGFX_DIR .. "tools/shaderc/shaderc_hlsl.cpp",
		BGFX_DIR .. "vertexdecl.cpp",
		BGFX_DIR .. "vertexdecl.h",

		FCPP_DIR .. "cpp1.c",
		FCPP_DIR .. "cpp2.c",
		FCPP_DIR .. "cpp3.c",
		FCPP_DIR .. "cpp4.c",
		FCPP_DIR .. "cpp5.c",
		FCPP_DIR .. "cpp6.c",

		FGlob {
			Dir = GLSL_OPTIMIZER .. "mesa",
			Extensions = { ".c", ".h" },
			Filters = {
				{ Pattern = "main.cpp", Config = "never" }
			}
		},
		FGlob {
			Dir = GLSL_OPTIMIZER .. "glsl",
			Extensions = { ".cpp", ".c", ".h" },
			Filters = {
				{ Pattern = "main.cpp", Config = "never" }
			}
		},
		FGlob {
			Dir = GLSL_OPTIMIZER .. "util",
			Extensions = { ".c", ".h" },
			Filters = {
				{ Pattern = "main.cpp", Config = "never" }
			}
		},
	},

    Libs = { { "kernel32.lib", "d3dcompiler.lib", "dxguid.lib" ; Config = "win64-*-*" } },

    Frameworks = {
        { "Cocoa" },
        { "Metal" },
        { "QuartzCore" },
        { "OpenGL" }
    },

	IdeGenerationHints = { Msvc = { SolutionFolder = "Tools" } },
}

-----------------------------------------------------------------------------------------

StaticLibrary {
    Name = "glfw",

    Env = {
        CPPPATH = {
            "external/glfw/src",
            "external/glfw/include",
        },

        CPPDEFS = {
            { "_GLFW_WIN32", "_GLFW_WGL", "WIN32"; Config = "win64-*-*" },
            { "_GLFW_X11", "_GLFW_GFX", "LINUX"; Config = "linux-*-*" },
            { "_GLFW_NSGL", "MACOSX"; Config = "macosx-*-*" },
        },
    },

    Sources = {
		"external/glfw/src/window.c",
		"external/glfw/src/context.c",
		"external/glfw/src/init.c",
		"external/glfw/src/input.c",
		"external/glfw/src/monitor.c",
		"external/glfw/src/vulkan.c",

        {
			"external/glfw/src/cocoa_init.m",
			"external/glfw/src/cocoa_joystick.m",
			"external/glfw/src/cocoa_monitor.m",
			"external/glfw/src/cocoa_time.c",
			"external/glfw/src/cocoa_window.m",
			"external/glfw/src/nsgl_context.h",
			"external/glfw/src/nsgl_context.m" ; Config = "macosx-*-*"
		},

		{
			"external/glfw/src/glx_context.c",
			"external/glfw/src/egl_context.c",
			-- "external/glfw/src/wl_init.c",
			-- "external/glfw/src/wl_monitor.c",
			-- "external/glfw/src/wl_window.c",
			"external/glfw/src/x11_init.c",
			"external/glfw/src/x11_monitor.c",
			"external/glfw/src/x11_window.c",
			"external/glfw/src/linux_joystick.c",
			"external/glfw/src/osmesa_context.c",
			"external/glfw/src/posix_thread.c",
			"external/glfw/src/posix_time.c",
			"external/glfw/src/xkb_unicode.c" ; Config = "linux-*-*",
		},

		{
			"external/glfw/src/wgl_context.c",
			"external/glfw/src/win32_init.c",
			"external/glfw/src/win32_joystick.c",
			"external/glfw/src/win32_monitor.c",
			"external/glfw/src/win32_thread.c",
			"external/glfw/src/win32_time.c",
			"external/glfw/src/win32_window.c" ; Config = "win64-*-*",
		},
    },
}


-----------------------------------------------------------------------------------------

StaticLibrary {
    Name = "bgfx",

    Env = {
        CPPPATH = {
        { { "external/bx/include/compat/msvc",
		   "external/bgfx/3rdparty/dxsdk/include" } ; Config = "win64-*-*" },
          { "external/bx/include/compat/osx" ; Config = "macosx-*-*" },
            "external/remotery/lib",
            "external/bgfx/include",
            "external/bx/include",
            "external/bx/3rdparty",
            "external/bimg/include",
            "external/bimg/3rdparty",
            "external/bimg/3rdparty/iqa/include",
            "external/bimg/3rdparty/astc-codec/include",
            "external/bgfx/3rdparty/khronos",
            "external/bgfx/3rdparty",
        },

        CXXOPTS = {
			{ "-Wno-variadic-macros", "-Wno-everything" ; Config = "macosx-*-*" },
			{ "/Iexternal/bx/include/compat/msvc",
			"/EHsc"; Config = "win64-*-*" },
        },
    },

    Defines = {
		"BGFX_CONFIG_RENDERER_WEBGPU=0",
		"BGFX_CONFIG_RENDERER_GNM=0",
		"BGFX_CONFIG_RENDERER_VULKAN=0",
	},

    Sources = {
    	   get_c_cpp_src("external/bimg/src"),
		  "external/bx/src/amalgamated.cpp",
		  "external/bgfx/src/bgfx.cpp",
		  "external/bgfx/src/vertexlayout.cpp",
		  "external/bgfx/src/debug_renderdoc.cpp",
		  "external/bgfx/src/topology.cpp",
		  "external/bgfx/src/shader_dxbc.cpp",
		  "external/bgfx/src/renderer_gnm.cpp",
		  "external/bgfx/src/renderer_webgpu.cpp",
		  "external/bgfx/src/renderer_nvn.cpp",
		  "external/bgfx/src/renderer_gl.cpp",
		  "external/bgfx/src/renderer_vk.cpp",
		  "external/bgfx/src/renderer_noop.cpp",
		  "external/bgfx/src/renderer_d3d9.cpp",
		  "external/bgfx/src/renderer_d3d11.cpp",
		  "external/bgfx/src/renderer_d3d12.cpp",
        { "external/bgfx/src/renderer_mtl.mm" ; Config = "macosx-*-*" },
	    { "external/bgfx/src/glcontext_wgl.cpp" ; Config = "win64-*-*" },
	    { "external/bgfx/src/glcontext_glx.cpp" ; Config = "linux-*-*" },
	    { "external/bgfx/src/glcontext_nsgl.mm" ; Config = "macosx-*-*" },
    },

	IdeGenerationHints = { Msvc = { SolutionFolder = "External" } },
}

