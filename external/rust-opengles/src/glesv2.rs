// +-----------------------------------------------------------------------------------------------+
// | Copyright 2015 Sean Kerr, 2019, 2020 Lauri Gustafsson                                         |
// |                                                                                               |
// | Licensed under the Apache License, Version 2.0(the "License");                                |
// | you may not use this file except in compliance with the License.                              |
// | You may obtain a copy of the License Author                                                   |
// |                                                                                               |
// |  http://www.apache.org/licenses/LICENSE-2.0                                                   |
// |                                                                                               |
// | Unless required by applicable law or agreed to in writing, software                           |
// | distributed under the License is distributed on an "AS IS" BASIS,                             |
// | WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.                      |
// | See the License for the specific language governing permissions and                           |
// | limitations under the License.                                                                |
// +-----------------------------------------------------------------------------------------------+
// | Authors: Sean Kerr <sean@metatomic.io>, Lauri Gustafsson <me@gustafla.space>                  |
// +-----------------------------------------------------------------------------------------------+

#![allow(dead_code)]

// -------------------------------------------------------------------------------------------------
// LINKING
// -------------------------------------------------------------------------------------------------

#[link(name = "GLESv2")]
extern "C" {}

// -------------------------------------------------------------------------------------------------
// DEPENDENCIES
// -------------------------------------------------------------------------------------------------

use std::ffi::CStr;
use std::ffi::CString;
use std::mem::size_of;
use std::str::from_utf8;

use khronos::{
    khronos_float_t, khronos_int32_t, khronos_int8_t, khronos_intptr_t, khronos_ssize_t,
    khronos_uint8_t,
};

use std::os::raw::{c_char, c_int, c_short, c_uchar, c_uint, c_ushort, c_void};

// -------------------------------------------------------------------------------------------------
// TYPES
// -------------------------------------------------------------------------------------------------

pub mod types;
pub use self::types::*;

// -------------------------------------------------------------------------------------------------
// STRUCTS
// -------------------------------------------------------------------------------------------------

pub struct Active {
    pub name: String,
    pub size: GLint,
    pub type_: GLenum,
}

pub struct ShaderPrecisionFormat {
    pub precision: GLint,
    pub range: [GLint; 2],
}

// -------------------------------------------------------------------------------------------------
// CONSTANTS
// -------------------------------------------------------------------------------------------------

pub mod constants;
pub use self::constants::*;

// -------------------------------------------------------------------------------------------------
// FUNCTIONS
// -------------------------------------------------------------------------------------------------

mod functions;
pub use self::functions::*;

// -------------------------------------------------------------------------------------------------
// FFI
// -------------------------------------------------------------------------------------------------

mod ffi;
