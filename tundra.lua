-- vim: noexpandtab ts=4 sw=4

require "tundra.syntax.glob"
local native = require('tundra.native')
local common = { }

local win32 = {
	Inherit = common,
	Env = {
		HOSTPLATFORM = "win32",
		GENERATE_PDB = "1",
		CCOPTS = {
			"/FS",
			"/W4",
			"/WX", "/I.", "/DWIN32", "/D_CRT_SECURE_NO_WARNINGS", "\"/DOBJECT_DIR=$(OBJECTDIR:#)\"",
			{ "/Od"; Config = "*-*-debug" },
			{ "/O2"; Config = "*-*-release" },
		},
		CXXOPTS = {
			"/FS",
			"/EHsc",
			"/W4",
			"/I.", "/DWIN32", "/D_CRT_SECURE_NO_WARNINGS", "\"/DOBJECT_DIR=$(OBJECTDIR:#)\"",
			{ "/Od"; Config = "*-*-debug" },
			{ "/O2"; Config = "*-*-release" },
		},
		PROGOPTS = {
			"/INCREMENTAL:NO"			-- Disable incremental linking. It doesn't work properly in our use case (nearly all code in libs) and causes log spam.
		},

	},

	ReplaceEnv = {
		["OBJCCOM"] = "meh",
		["NIBCC"] = "meh",
	},
}

local macosx = {
	Inherit = common,
	Env = {
		HOSTPLATFORM = "Darwin",
		CCOPTS = {
			"-I.", "-DMACOS", "-Wall",
			{ "-O0", "-g"; Config = "*-*-debug" },
			{ "-O3"; Config = "*-*-release" },
		},
		CXXOPTS = {
			"-I.",
			"-std=c++14",
			{ "-O0", "-g"; Config = "*-*-debug" },
			{ "-O3"; Config = "*-*-release" },
		},
	},

	ReplaceEnv = {
		["LD"] = "$(CXX)",
	},

	Frameworks = { "Cocoa" },
}

local linux = {
	Inherit = common,
	Env = {
		HOSTPLATFORM = "Linux",
		CCOPTS = {
			"-I.", "-DLINUX", "-Wall",
			{ "-O0", "-g"; Config = "*-*-debug" },
			{ "-O3"; Config = "*-*-release" },
		},
		CXXOPTS = {
		"-I.",
			"-std=c++14",
			{ "-O0", "-g"; Config = "*-*-debug" },
			{ "-O3"; Config = "*-*-release" },
		}
	},

	ReplaceEnv = {
		["LD"] = "$(CXX)",
	},
}

Build {
	Passes = {
		BuildTools = { Name = "Build Tools", BuildOrder = 1 },
		CodeGeneration = { Name = "Generate sources", BuildOrder = 2 },
	},

	Configs = {
		Config {
			Name = "win32-msvc",
			Inherit = win32,
			Tools = { "msvc" },
			DefaultOnHost = "windows",
			SupportedHosts = { "windows"},
		},

		Config {
			Name = "macos-clang",
			Inherit = macosx,
			Tools = { "clang-osx" },
			DefaultOnHost = "macosx",
			SupportedHosts = { "macosx" },
		},

		Config {
			Name = "linux-gcc",
			Inherit = linux,
			Tools = { "gcc" },
			DefaultOnHost = "linux",
			SupportedHosts = { "linux" },
		},

		Config {
			Name = "linux-clang",
			Inherit = linux,
			Tools = { "clang" },
			SupportedHosts = { "linux" },
		},
	},

	Units = {
		"units.libs.lua",
	},
}
