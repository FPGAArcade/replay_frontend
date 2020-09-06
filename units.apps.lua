-- vim: noexpandtab ts=4 sw=4
require "tundra.syntax.glob"
require "tundra.syntax.rust-cargo"
require "tundra.path"
require "tundra.util"

local native = require "tundra.native"
local path = require "tundra.path"

-----------------------------------------------------------------------------------------------------------------------

local function get_rs_src(dir)
    return Glob {
        Dir = dir,
        Extensions = { ".rs" },
        Recursive = true,
}
end

-----------------------------------------------------------------------------------------------------------------------

Program {
    Name = "dummy",
    Sources = { "src/dummy/dummy.cpp" },
    Depends = { "bgfx", "glfw" }
}

-----------------------------------------------------------------------------------------------------------------------

RustProgram  {
    Name = "frontend",
    CargoConfig = "src/frontend/Cargo.toml",
    Sources = {
        get_rs_src("src/frontend"),
    },

    Depends = { "bgfx", "glfw" }
}

-----------------------------------------------------------------------------------------------------------------------

RustProgram  {
    Name = "testbed",
    CargoConfig = "src/testbed/Cargo.toml",
    Sources = {
        get_rs_src("src/egui"),
        get_rs_src("src/egui_glium"),
        get_rs_src("src/testbed"),
    },
}

-----------------------------------------------------------------------------------------------------------------------

Default "frontend"
Default "testbed"
Always "dummy"

