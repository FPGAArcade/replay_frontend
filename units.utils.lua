-- vim: noexpandtab ts=4 sw=4
require "tundra.syntax.rust-cargo"
require "tundra.syntax.dotnet"
local native = require "tundra.native"
local path = require "tundra.path"

local function get_rs_src(dir)
	return Glob {
		Dir = dir,
		Extensions = { ".rs" },
		Recursive = true,
	}
end

local function get_src(dir, recursive)
	return FGlob {
		Dir = dir,
		Extensions = { ".cpp", ".c", ".h", ".s", ".m" },
		Filters = {
			{ Pattern = "[/\\]test[/\\]"; Config = "test-*" }, -- Directories named "test" and their subdirectories will be excluded from builds

			{ Pattern = "%.s$"; Config = "amiga-*" },
			{ Pattern = "[/\\]amiga[/\\]"; Config = "amiga-*" },
			{ Pattern = "[/\\]win32[/\\]"; Config = "win32-*" },
			{ Pattern = "[/\\]macosx[/\\]"; Config = "mac*-*" },
			{ Pattern = "[/\\]linux[/\\]"; Config = "linux-*" },
		},
		Recursive = recursive and true or false,
	}
end

local host_configs = {
	"mac*-*-*-default",	"win32-*-*-default", "linux-*-*-default",
}

local host_relase_configs = {
	"mac*-*-release-default", "win32-*-release-default", "linux-*-release-default",
}

Program {
	Name = "customvasm",

	Target = "$(VASM)",

	Pass = "BuildTools",

	Includes = {
		"external/vasm",
		"external/vasm/cpus/m68k",
		"external/vasm/syntax/mot",
	},
	SourceDir = "external/vasm",

	Defines = {
		"OUTAOUT",
		"OUTBIN",
		"OUTELF",
		"OUTHUNK",
		"OUTSREC",
		"OUTTOS",
		"OUTVOBJ",
	},

	ReplaceEnv = {
		-- We need to dial warnings way down for this stuff or it won't build.
		CCOPTS = {
			{
				{ "-w", "-fpermissive"; Config = "*-clang-*" },
				{ "-O0", "-g"; Config = { "macosx-*-debug", "linux-*-debug" }},
				{ "-O2"; Config = { "macosx-*-release", "linux-*-release" }},
				{ "/FS /Od"; Config = "win32-*-debug" },
				{ "/FS /O1"; Config = "win32-*-production" },
				{ "/FS /O2"; Config = "win32-*-release" },
			}
			-- XXX need similar detuning for MSVC
		},
	},

	Sources = {
		"atom.c",
		"atom.h",
		"cond.c",
		"cond.h",
		"elf_reloc_68k.h",
		"error.c",
		"error.h",
		"expr.c",
		"expr.h",
		"general_errors.h",
		"hugeint.c",
		"hugeint.h",
		"output_aout.c",
		"output_aout.h",
		"output_bin.c",
		"output_elf.c",
		"output_elf.h",
		"output_errors.h",
		"output_hunk.c",
		"output_hunk.h",
		"output_srec.c",
		"output_test.c",
		"output_tos.c",
		"output_tos.h",
		"output_vobj.c",
		"parse.c",
		"parse.h",
		"reloc.c",
		"reloc.h",
		"stabs.h",
		"supp.c",
		"supp.h",
		"symbol.c",
		"symbol.h",
		"symtab.c",
		"symtab.h",
		"tfloat.h",
		"vasm.c",
		"vasm.h",
		"cpus/m68k/cpu.c",
		"cpus/m68k/cpu.h",
		"cpus/m68k/cpu_errors.h",
		"cpus/m68k/cpu_models.h",
		"cpus/m68k/opcodes.h",
		"cpus/m68k/operands.h",
		"cpus/m68k/specregs.h",
		"syntax/mot/syntax.c",
		"syntax/mot/syntax.h",
		"syntax/mot/syntax_errors.h",
	},
}

StaticLibrary {
	Name = "musashi",

	Pass = "BuildTools",

	Includes = {
		"external/musashi",
	},

	SourceDir = "external/musashi",

	Sources = {
		"m68kops.c",
		"m68kops.h",
		"m68kconf.h",
		"m68kcpu.h",
		"m68k.h",
		"m68kcpu.c",
		"m68kdasm.c",
		"m68kfpu.c",
	},
}

Program {
	Name = "runner",
	Includes = {
		"external/musashi",
	},
	Depends = { "musashi" },
	Sources = { "src/runner.c" },
}

Always "customvasm"
Always "runner"
