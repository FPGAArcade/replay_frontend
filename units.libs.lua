-- vim: noexpandtab ts=4 sw=4
require "tundra.syntax.rust-cargo"
require "tundra.syntax.dotnet"
local native = require "tundra.native"
local path = require "tundra.path"

Program {
    Name = "dummy",
    Sources = { "src/dummy/dummy.cpp" },
    Depends = { "bgfx", "glfw" }
}

Always "dummy"
