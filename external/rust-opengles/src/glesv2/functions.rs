use super::*;

pub fn active_texture(texture: GLenum) {
    unsafe { ffi::glActiveTexture(texture) }
}

pub fn attach_shader(program: GLuint, shader: GLuint) {
    unsafe { ffi::glAttachShader(program, shader) }
}

pub fn bind_attrib_location(program: GLuint, index: GLuint, name: &str) {
    unsafe {
        let c_str = CString::new(name).unwrap();

        ffi::glBindAttribLocation(program, index, c_str.as_ptr() as *const c_char)
    }
}

pub fn bind_buffer(target: GLenum, buffer: GLuint) {
    unsafe { ffi::glBindBuffer(target, buffer) }
}

pub fn bind_framebuffer(target: GLenum, framebuffer: GLuint) {
    unsafe { ffi::glBindFramebuffer(target, framebuffer) }
}

pub fn bind_renderbuffer(target: GLenum, renderbuffer: GLuint) {
    unsafe { ffi::glBindRenderbuffer(target, renderbuffer) }
}

pub fn bind_texture(target: GLenum, texture: GLuint) {
    unsafe { ffi::glBindTexture(target, texture) }
}

pub fn blend_color(red: GLclampf, green: GLclampf, blue: GLclampf, alpha: GLclampf) {
    unsafe { ffi::glBlendColor(red, green, blue, alpha) }
}

pub fn blend_equation(mode: GLenum) {
    unsafe { ffi::glBlendEquation(mode) }
}

pub fn blend_equation_separate(mode_rgb: GLenum, mode_alpha: GLenum) {
    unsafe { ffi::glBlendEquationSeparate(mode_rgb, mode_alpha) }
}

pub fn blend_func(src_factor: GLenum, dst_factor: GLenum) {
    unsafe { ffi::glBlendFunc(src_factor, dst_factor) }
}

pub fn blend_func_separate(src_rgb: GLenum, dst_rgb: GLenum, src_alpha: GLenum, dst_alpha: GLenum) {
    unsafe { ffi::glBlendFuncSeparate(src_rgb, dst_rgb, src_alpha, dst_alpha) }
}

pub fn buffer_data<T>(target: GLenum, buffer: &[T], usage: GLenum) {
    unsafe {
        let ptr = if buffer.len() == 0 {
            0 as *const GLvoid
        } else {
            buffer.as_ptr() as *const GLvoid
        };

        ffi::glBufferData(
            target,
            (buffer.len() * size_of::<T>()) as GLsizeiptr,
            ptr,
            usage,
        )
    }
}

pub fn buffer_sub_data<T>(target: GLenum, offset: GLintptr, buffer: &[T]) {
    unsafe {
        let size = size_of::<T>();
        let ptr = if buffer.len() == 0 {
            0 as *const GLvoid
        } else {
            buffer.as_ptr() as *const GLvoid
        };

        ffi::glBufferSubData(
            target,
            offset * (size as GLintptr),
            (buffer.len() * size) as GLsizeiptr,
            ptr,
        )
    }
}

pub fn check_framebuffer_status(target: GLenum) -> GLenum {
    unsafe { ffi::glCheckFramebufferStatus(target) }
}

pub fn clear(mask: GLbitfield) {
    unsafe { ffi::glClear(mask) }
}

pub fn clear_color(red: GLclampf, green: GLclampf, blue: GLclampf, alpha: GLclampf) {
    unsafe { ffi::glClearColor(red, green, blue, alpha) }
}

pub fn clear_depthf(depth: GLclampf) {
    unsafe { ffi::glClearDepthf(depth) }
}

pub fn clear_stencil(stencil: GLint) {
    unsafe { ffi::glClearStencil(stencil) }
}

pub fn color_mask(red: bool, green: bool, blue: bool, alpha: bool) {
    unsafe {
        ffi::glColorMask(
            red as GLboolean,
            green as GLboolean,
            blue as GLboolean,
            alpha as GLboolean,
        )
    }
}

pub fn compile_shader(shader: GLuint) {
    unsafe { ffi::glCompileShader(shader) }
}

pub fn compressed_tex_image_2d<T>(
    target: GLenum,
    level: GLint,
    internal_format: GLenum,
    width: GLsizei,
    height: GLsizei,
    border: GLint,
    image_size: GLsizei,
    buffer: &[T],
) {
    unsafe {
        let ptr = if buffer.len() == 0 {
            0 as *const GLvoid
        } else {
            buffer.as_ptr() as *const GLvoid
        };

        ffi::glCompressedTexImage2D(
            target,
            level,
            internal_format,
            width,
            height,
            border,
            image_size,
            ptr,
        )
    }
}

pub fn compressed_tex_sub_image_2d<T>(
    target: GLenum,
    level: GLint,
    x_offset: GLint,
    y_offset: GLint,
    width: GLsizei,
    height: GLsizei,
    format: GLenum,
    image_size: GLsizei,
    buffer: &[T],
) {
    unsafe {
        let ptr = if buffer.len() == 0 {
            0 as *const GLvoid
        } else {
            buffer.as_ptr() as *const GLvoid
        };

        ffi::glCompressedTexSubImage2D(
            target, level, x_offset, y_offset, width, height, format, image_size, ptr,
        )
    }
}

pub fn copy_tex_image_2d(
    target: GLenum,
    level: GLint,
    internal_format: GLenum,
    x: GLint,
    y: GLint,
    width: GLsizei,
    height: GLsizei,
    border: GLint,
) {
    unsafe { ffi::glCopyTexImage2D(target, level, internal_format, x, y, width, height, border) }
}

pub fn copy_tex_sub_image_2d(
    target: GLenum,
    level: GLint,
    x_offset: GLint,
    y_offset: GLint,
    x: GLint,
    y: GLint,
    width: GLsizei,
    height: GLsizei,
) {
    unsafe { ffi::glCopyTexSubImage2D(target, level, x_offset, y_offset, x, y, width, height) }
}

pub fn create_program() -> GLuint {
    unsafe { ffi::glCreateProgram() }
}

pub fn create_shader(type_: GLenum) -> GLuint {
    unsafe { ffi::glCreateShader(type_) }
}

pub fn cull_face(mode: GLenum) {
    unsafe { ffi::glCullFace(mode) }
}

pub fn delete_buffers(buffers: &[GLuint]) {
    unsafe { ffi::glDeleteBuffers(buffers.len() as GLsizei, buffers.as_ptr()) }
}

pub fn delete_framebuffers(framebuffers: &[GLuint]) {
    unsafe { ffi::glDeleteFramebuffers(framebuffers.len() as GLsizei, framebuffers.as_ptr()) }
}

pub fn delete_program(program: GLuint) {
    unsafe { ffi::glDeleteProgram(program) }
}

pub fn delete_renderbuffers(renderbuffers: &[GLuint]) {
    unsafe { ffi::glDeleteRenderbuffers(renderbuffers.len() as GLsizei, renderbuffers.as_ptr()) }
}

pub fn delete_shader(shader: GLuint) {
    unsafe { ffi::glDeleteShader(shader) }
}

pub fn delete_textures(textures: &[GLuint]) {
    unsafe { ffi::glDeleteTextures(textures.len() as GLsizei, textures.as_ptr()) }
}

pub fn depth_func(func: GLenum) {
    unsafe { ffi::glDepthFunc(func) }
}

pub fn depth_mask(flag: bool) {
    unsafe { ffi::glDepthMask(flag as GLboolean) }
}

pub fn depth_rangef(z_near: GLclampf, z_far: GLclampf) {
    unsafe { ffi::glDepthRangef(z_near, z_far) }
}

pub fn detach_shader(program: GLuint, shader: GLuint) {
    unsafe { ffi::glDetachShader(program, shader) }
}

pub fn disable(feature: GLenum) {
    unsafe { ffi::glDisable(feature) }
}

pub fn disable_vertex_attrib_array(index: GLuint) {
    unsafe { ffi::glDisableVertexAttribArray(index) }
}

pub fn draw_arrays(mode: GLenum, first: GLint, count: GLsizei) {
    unsafe { ffi::glDrawArrays(mode, first, count) }
}

pub fn draw_elements<T>(mode: GLenum, count: GLsizei, type_: GLenum, indices: &[T]) {
    unsafe {
        let ptr = if indices.len() == 0 {
            0 as *const GLvoid
        } else {
            indices.as_ptr() as *const GLvoid
        };

        ffi::glDrawElements(mode, count, type_, ptr)
    }
}

pub fn enable(feature: GLenum) {
    unsafe { ffi::glEnable(feature) }
}

pub fn enable_vertex_attrib_array(index: GLuint) {
    unsafe { ffi::glEnableVertexAttribArray(index) }
}

pub fn finish() {
    unsafe { ffi::glFinish() }
}

pub fn flush() {
    unsafe { ffi::glFlush() }
}

pub fn framebuffer_renderbuffer(
    target: GLenum,
    attachment: GLenum,
    renderbuffer_target: GLenum,
    renderbuffer: GLuint,
) {
    unsafe { ffi::glFramebufferRenderbuffer(target, attachment, renderbuffer_target, renderbuffer) }
}

pub fn framebuffer_texture_2d(
    target: GLenum,
    attachment: GLenum,
    texture_target: GLenum,
    texture: GLuint,
    level: GLint,
) {
    unsafe { ffi::glFramebufferTexture2D(target, attachment, texture_target, texture, level) }
}

pub fn front_face(mode: GLenum) {
    unsafe { ffi::glFrontFace(mode) }
}

pub fn gen_buffers(count: GLsizei) -> Vec<GLuint> {
    unsafe {
        let mut vec: Vec<GLuint> = Vec::with_capacity(count as usize);

        ffi::glGenBuffers(count, vec.as_mut_ptr());

        vec.set_len(count as usize);
        vec
    }
}

pub fn generate_mipmap(target: GLenum) {
    unsafe { ffi::glGenerateMipmap(target) }
}

pub fn gen_framebuffers(count: GLsizei) -> Vec<GLuint> {
    unsafe {
        let mut vec: Vec<GLuint> = Vec::with_capacity(count as usize);

        ffi::glGenFramebuffers(count, vec.as_mut_ptr());

        vec.set_len(count as usize);
        vec
    }
}

pub fn gen_renderbuffers(count: GLsizei) -> Vec<GLuint> {
    unsafe {
        let mut vec: Vec<GLuint> = Vec::with_capacity(count as usize);

        ffi::glGenRenderbuffers(count, vec.as_mut_ptr());

        vec.set_len(count as usize);
        vec
    }
}

pub fn gen_textures(count: GLsizei) -> Vec<GLuint> {
    unsafe {
        let mut vec: Vec<GLuint> = Vec::with_capacity(count as usize);

        ffi::glGenTextures(count, vec.as_mut_ptr());

        vec.set_len(count as usize);
        vec
    }
}

pub fn get_active_attrib(program: GLuint, index: GLuint) -> Option<Active> {
    unsafe {
        let mut length: GLsizei = 0;
        let mut size: GLint = 0;
        let mut type_: GLenum = 0;

        let mut name = String::with_capacity(256);

        ffi::glGetActiveAttrib(
            program,
            index,
            256,
            &mut length,
            &mut size,
            &mut type_,
            name.as_mut_vec().as_mut_ptr() as *mut GLchar,
        );

        if length > 0 {
            name.as_mut_vec().set_len(length as usize);
            name.truncate(length as usize);

            Some(Active {
                name: name,
                size: size,
                type_: type_,
            })
        } else {
            None
        }
    }
}

pub fn get_active_uniform(program: GLuint, index: GLuint) -> Option<Active> {
    unsafe {
        let mut length: GLsizei = 0;
        let mut size: GLint = 0;
        let mut type_: GLenum = 0;

        let mut name = String::with_capacity(256);

        ffi::glGetActiveUniform(
            program,
            index,
            256,
            &mut length,
            &mut size,
            &mut type_,
            name.as_mut_vec().as_mut_ptr() as *mut GLchar,
        );

        if length > 0 {
            name.as_mut_vec().set_len(length as usize);
            name.truncate(length as usize);

            Some(Active {
                name: name,
                size: size,
                type_: type_,
            })
        } else {
            None
        }
    }
}

pub fn get_attached_shaders(program: GLuint, max_count: GLsizei) -> Vec<GLuint> {
    unsafe {
        let mut count: GLsizei = 0;
        let mut vec: Vec<GLuint> = Vec::with_capacity(max_count as usize);

        ffi::glGetAttachedShaders(program, max_count, &mut count, vec.as_mut_ptr());

        vec.set_len(count as usize);
        vec.truncate(count as usize);
        vec
    }
}

pub fn get_attrib_location(program: GLuint, name: &str) -> GLint {
    unsafe {
        let c_str = CString::new(name).unwrap();

        ffi::glGetAttribLocation(program, c_str.as_ptr() as *const c_char)
    }
}

pub fn get_booleanv(name: GLenum) -> bool {
    unsafe {
        let mut value: GLboolean = 0;

        ffi::glGetBooleanv(name, &mut value);

        value == GL_TRUE
    }
}

pub fn get_buffer_parameteriv(target: GLenum, name: GLenum) -> GLint {
    unsafe {
        let mut value: GLint = 0;

        ffi::glGetBufferParameteriv(target, name, &mut value);

        value
    }
}

pub fn get_error() -> GLenum {
    unsafe { ffi::glGetError() }
}

pub fn get_floatv(name: GLenum) -> GLfloat {
    unsafe {
        let mut value: GLfloat = 0.0;

        ffi::glGetFloatv(name, &mut value);

        value
    }
}

pub fn get_framebuffer_attachment_parameteriv(
    target: GLenum,
    attachment: GLenum,
    name: GLenum,
) -> GLint {
    unsafe {
        let mut value: GLint = 0;

        ffi::glGetFramebufferAttachmentParameteriv(target, attachment, name, &mut value);

        value
    }
}

pub fn get_integerv(name: GLenum) -> GLint {
    unsafe {
        let mut value: GLint = 0;

        ffi::glGetIntegerv(name, &mut value);

        value
    }
}

pub fn get_programiv(program: GLuint, name: GLenum) -> GLint {
    unsafe {
        let mut value: GLint = 0;

        ffi::glGetProgramiv(program, name, &mut value);

        value
    }
}

pub fn get_program_info_log(program: GLuint, max_length: GLsizei) -> Option<String> {
    unsafe {
        let mut length: GLsizei = 0;
        let mut log = String::with_capacity(max_length as usize);

        ffi::glGetProgramInfoLog(
            program,
            max_length,
            &mut length,
            log.as_mut_vec().as_mut_ptr() as *mut i8,
        );

        if length > 0 {
            log.as_mut_vec().set_len(length as usize);
            log.truncate(length as usize);

            Some(log)
        } else {
            None
        }
    }
}

pub fn get_renderbuffer_parameteriv(target: GLenum, name: GLenum) -> GLint {
    unsafe {
        let mut value: GLint = 0;

        ffi::glGetRenderbufferParameteriv(target, name, &mut value);

        value
    }
}

pub fn get_shaderiv(shader: GLuint, name: GLenum) -> GLint {
    unsafe {
        let mut value: GLint = 0;

        ffi::glGetShaderiv(shader, name, &mut value);

        value
    }
}

pub fn get_shader_info_log(shader: GLuint, max_length: GLsizei) -> Option<String> {
    unsafe {
        let mut length: GLsizei = 0;
        let mut log = String::with_capacity(max_length as usize);

        ffi::glGetShaderInfoLog(
            shader,
            max_length,
            &mut length,
            log.as_mut_vec().as_mut_ptr() as *mut i8,
        );

        if length > 0 {
            log.as_mut_vec().set_len(length as usize);
            log.truncate(length as usize);

            Some(log)
        } else {
            None
        }
    }
}

pub fn get_shader_precision_format(
    shader_type: GLenum,
    precision_type: GLenum,
) -> ShaderPrecisionFormat {
    unsafe {
        let mut precision: GLint = 0;
        let mut range: [GLint; 2] = [0, 0];

        ffi::glGetShaderPrecisionFormat(
            shader_type,
            precision_type,
            range.as_mut_ptr(),
            &mut precision,
        );

        ShaderPrecisionFormat {
            precision: precision,
            range: range,
        }
    }
}

pub fn get_shader_source(shader: GLuint, max_length: GLsizei) -> Option<String> {
    unsafe {
        let mut length: GLsizei = 0;
        let mut source = String::with_capacity(max_length as usize);

        ffi::glGetShaderSource(
            shader,
            max_length,
            &mut length,
            source.as_mut_vec().as_mut_ptr() as *mut GLchar,
        );

        if length > 0 {
            source.as_mut_vec().set_len(length as usize);
            source.truncate(length as usize);

            Some(source)
        } else {
            None
        }
    }
}

pub fn get_string(name: GLenum) -> Option<String> {
    unsafe {
        let c_str = ffi::glGetString(name);

        if !c_str.is_null() {
            match from_utf8(CStr::from_ptr(c_str as *const _).to_bytes()) {
                Ok(s) => Some(s.to_string()),
                Err(_) => None,
            }
        } else {
            None
        }
    }
}

pub fn get_tex_parameterfv(target: GLenum, name: GLenum) -> GLfloat {
    unsafe {
        let mut value: GLfloat = 0.0;

        ffi::glGetTexParameterfv(target, name, &mut value);

        value
    }
}

pub fn get_tex_parameteriv(target: GLenum, name: GLenum) -> GLint {
    unsafe {
        let mut value: GLint = 0;

        ffi::glGetTexParameteriv(target, name, &mut value);

        value
    }
}

pub fn get_uniformfv(program: GLuint, location: GLint) -> GLfloat {
    unsafe {
        let mut value: GLfloat = 0.0;

        ffi::glGetUniformfv(program, location, &mut value);

        value
    }
}

pub fn get_uniformiv(program: GLuint, location: GLint) -> GLint {
    unsafe {
        let mut value: GLint = 0;

        ffi::glGetUniformiv(program, location, &mut value);

        value
    }
}

pub fn get_uniform_location(program: GLuint, name: &str) -> GLint {
    unsafe {
        let c_str = CString::new(name).unwrap();
        ffi::glGetUniformLocation(program, c_str.as_ptr() as *const c_char)
    }
}

pub fn get_vertex_attribfv(index: GLuint, name: GLenum) -> GLfloat {
    unsafe {
        let mut value: GLfloat = 0.0;

        ffi::glGetVertexAttribfv(index, name, &mut value);

        value
    }
}

pub fn get_vertex_attribiv(index: GLuint, name: GLenum) -> GLint {
    unsafe {
        let mut value: GLint = 0;

        ffi::glGetVertexAttribiv(index, name, &mut value);

        value
    }
}

/*
pub fn get_vertex_attrib_pointerv(index: GLuint, name: GLenum, pointer: &mut &mut GLvoid) {
    unsafe {
    }
}
*/

pub fn hint(target: GLenum, mode: GLenum) {
    unsafe { ffi::glHint(target, mode) }
}

pub fn is_buffer(buffer: GLuint) -> bool {
    unsafe { ffi::glIsBuffer(buffer) == GL_TRUE }
}

pub fn is_enabled(feature: GLenum) -> bool {
    unsafe { ffi::glIsEnabled(feature) == GL_TRUE }
}

pub fn is_framebuffer(framebuffer: GLuint) -> bool {
    unsafe { ffi::glIsFramebuffer(framebuffer) == GL_TRUE }
}

pub fn is_program(program: GLuint) -> bool {
    unsafe { ffi::glIsProgram(program) == GL_TRUE }
}

pub fn is_renderbuffer(renderbuffer: GLuint) -> bool {
    unsafe { ffi::glIsRenderbuffer(renderbuffer) == GL_TRUE }
}

pub fn is_shader(shader: GLuint) -> bool {
    unsafe { ffi::glIsShader(shader) == GL_TRUE }
}

pub fn is_texture(texture: GLuint) -> bool {
    unsafe { ffi::glIsTexture(texture) == GL_TRUE }
}

pub fn line_width(width: GLfloat) {
    unsafe { ffi::glLineWidth(width) }
}

pub fn link_program(program: GLuint) {
    unsafe { ffi::glLinkProgram(program) }
}

pub fn pixel_storei(name: GLenum, param: GLint) {
    unsafe { ffi::glPixelStorei(name, param) }
}

pub fn polygon_offset(factor: GLfloat, units: GLfloat) {
    unsafe { ffi::glPolygonOffset(factor, units) }
}

pub fn read_pixels<T>(
    x: GLint,
    y: GLint,
    width: GLsizei,
    height: GLsizei,
    format: GLenum,
    type_: GLenum,
    buffer: &mut [T],
) {
    unsafe {
        ffi::glReadPixels(
            x,
            y,
            width,
            height,
            format,
            type_,
            buffer.as_mut_ptr() as *mut GLvoid,
        )
    }
}

pub fn read_pixels_rgba(x: GLint, y: GLint, width: GLsizei, height: GLsizei) -> Vec<u8> {
    unsafe {
        let mut buffer: Vec<u8> = Vec::with_capacity((width * height * 4) as usize);

        ffi::glReadPixels(
            x,
            y,
            width,
            height,
            GL_RGBA,
            GL_UNSIGNED_BYTE,
            buffer.as_mut_ptr() as *mut GLvoid,
        );

        buffer.set_len((width * height * 4) as usize);
        buffer
    }
}

pub fn release_shader_compiler() {
    unsafe { ffi::glReleaseShaderCompiler() }
}

pub fn renderbuffer_storage(
    target: GLenum,
    internal_format: GLenum,
    width: GLsizei,
    height: GLsizei,
) {
    unsafe { ffi::glRenderbufferStorage(target, internal_format, width, height) }
}

pub fn sample_coverage(value: GLclampf, invert: bool) {
    unsafe { ffi::glSampleCoverage(value, invert as GLboolean) }
}

pub fn scissor(x: GLint, y: GLint, width: GLsizei, height: GLsizei) {
    unsafe { ffi::glScissor(x, y, width, height) }
}

pub fn shader_binary<T>(shaders: &[GLuint], data_format: GLenum, data: &[T], length: GLsizei) {
    unsafe {
        let ptr = if data.len() == 0 {
            0 as *const GLvoid
        } else {
            data.as_ptr() as *const GLvoid
        };

        ffi::glShaderBinary(
            shaders.len() as GLsizei,
            shaders.as_ptr(),
            data_format,
            ptr,
            length,
        )
    }
}

pub fn shader_source(shader: GLuint, source: &[u8]) {
    unsafe {
        let length: GLsizei = source.len() as GLsizei;

        ffi::glShaderSource(shader, 1, &(source.as_ptr() as *const GLchar), &length)
    }
}

pub fn stencil_func(func: GLenum, ref_: GLint, mask: GLuint) {
    unsafe { ffi::glStencilFunc(func, ref_, mask) }
}

pub fn stencil_func_separate(face: GLenum, func: GLenum, ref_: GLint, mask: GLuint) {
    unsafe { ffi::glStencilFuncSeparate(face, func, ref_, mask) }
}

pub fn stencil_mask(mask: GLuint) {
    unsafe { ffi::glStencilMask(mask) }
}

pub fn stencil_mask_separate(face: GLenum, mask: GLuint) {
    unsafe { ffi::glStencilMaskSeparate(face, mask) }
}

pub fn stencil_op(fail: GLenum, z_fail: GLenum, z_pass: GLenum) {
    unsafe { ffi::glStencilOp(fail, z_fail, z_pass) }
}

pub fn stencil_op_separate(face: GLenum, fail: GLenum, z_fail: GLenum, z_pass: GLenum) {
    unsafe { ffi::glStencilOpSeparate(face, fail, z_fail, z_pass) }
}

/// Upload image data to a texture
///
/// If `buffer` length is 0, the function allocates an uninitialized texture for given size and
/// format.
pub fn tex_image_2d<T>(
    target: GLenum,
    level: GLint,
    internal_format: GLint,
    width: GLsizei,
    height: GLsizei,
    border: GLint,
    format: GLenum,
    type_: GLenum,
    buffer: &[T],
) {
    unsafe {
        let ptr = if buffer.len() == 0 {
            0 as *const GLvoid
        } else {
            buffer.as_ptr() as *const GLvoid
        };

        ffi::glTexImage2D(
            target,
            level,
            internal_format,
            width,
            height,
            border,
            format,
            type_,
            ptr,
        )
    }
}

pub fn tex_parameterf(target: GLenum, name: GLenum, value: GLfloat) {
    unsafe { ffi::glTexParameterf(target, name, value) }
}

pub fn tex_parameterfv(target: GLenum, name: GLenum, value: &GLfloat) {
    unsafe { ffi::glTexParameterfv(target, name, value) }
}

pub fn tex_parameteri(target: GLenum, name: GLenum, value: GLint) {
    unsafe { ffi::glTexParameteri(target, name, value) }
}

pub fn tex_parameteriv(target: GLenum, name: GLenum, value: &GLint) {
    unsafe { ffi::glTexParameteriv(target, name, value) }
}

pub fn tex_sub_image_2d<T>(
    target: GLenum,
    level: GLint,
    x_offset: GLint,
    y_offset: GLint,
    width: GLsizei,
    height: GLsizei,
    format: GLenum,
    type_: GLenum,
    buffer: &[T],
) {
    unsafe {
        let ptr = if buffer.len() == 0 {
            0 as *const GLvoid
        } else {
            buffer.as_ptr() as *const GLvoid
        };

        ffi::glTexSubImage2D(
            target, level, x_offset, y_offset, width, height, format, type_, ptr,
        )
    }
}

pub fn uniform1f(location: GLint, x: GLfloat) {
    unsafe { ffi::glUniform1f(location, x) }
}

pub fn uniform1fv(location: GLint, values: &[GLfloat]) {
    unsafe { ffi::glUniform1fv(location, values.len() as GLsizei, values.as_ptr()) }
}

pub fn uniform1i(location: GLint, x: GLint) {
    unsafe { ffi::glUniform1i(location, x) }
}

pub fn uniform1iv(location: GLint, values: &[GLint]) {
    unsafe { ffi::glUniform1iv(location, values.len() as GLsizei, values.as_ptr()) }
}

pub fn uniform2f(location: GLint, x: GLfloat, y: GLfloat) {
    unsafe { ffi::glUniform2f(location, x, y) }
}

pub fn uniform2fv(location: GLint, values: &[GLfloat]) {
    unsafe { ffi::glUniform2fv(location, (values.len() / 2) as GLsizei, values.as_ptr()) }
}

pub fn uniform2i(location: GLint, x: GLint, y: GLint) {
    unsafe { ffi::glUniform2i(location, x, y) }
}

pub fn uniform2iv(location: GLint, values: &[GLint]) {
    unsafe { ffi::glUniform2iv(location, (values.len() / 2) as GLsizei, values.as_ptr()) }
}

pub fn uniform3f(location: GLint, x: GLfloat, y: GLfloat, z: GLfloat) {
    unsafe { ffi::glUniform3f(location, x, y, z) }
}

pub fn uniform3fv(location: GLint, values: &[GLfloat]) {
    unsafe { ffi::glUniform3fv(location, (values.len() / 3) as GLsizei, values.as_ptr()) }
}

pub fn uniform3i(location: GLint, x: GLint, y: GLint, z: GLint) {
    unsafe { ffi::glUniform3i(location, x, y, z) }
}

pub fn uniform3iv(location: GLint, values: &[GLint]) {
    unsafe { ffi::glUniform3iv(location, (values.len() / 3) as GLsizei, values.as_ptr()) }
}

pub fn uniform4f(location: GLint, x: GLfloat, y: GLfloat, z: GLfloat, w: GLfloat) {
    unsafe { ffi::glUniform4f(location, x, y, z, w) }
}

pub fn uniform4fv(location: GLint, values: &[GLfloat]) {
    unsafe { ffi::glUniform4fv(location, (values.len() / 4) as GLsizei, values.as_ptr()) }
}

pub fn uniform4i(location: GLint, x: GLint, y: GLint, z: GLint, w: GLint) {
    unsafe { ffi::glUniform4i(location, x, y, z, w) }
}

pub fn uniform4iv(location: GLint, values: &[GLint]) {
    unsafe { ffi::glUniform4iv(location, (values.len() / 4) as GLsizei, values.as_ptr()) }
}

pub fn uniform_matrix2fv(location: GLint, transpose: bool, values: &[GLfloat]) {
    unsafe {
        ffi::glUniformMatrix2fv(
            location,
            (values.len() / 4) as GLsizei,
            transpose as GLboolean,
            values.as_ptr() as *const GLfloat,
        )
    }
}

pub fn uniform_matrix3fv(location: GLint, transpose: bool, values: &[GLfloat]) {
    unsafe {
        ffi::glUniformMatrix3fv(
            location,
            (values.len() / 9) as GLsizei,
            transpose as GLboolean,
            values.as_ptr() as *const GLfloat,
        )
    }
}

pub fn uniform_matrix4fv(location: GLint, transpose: bool, values: &[GLfloat]) {
    unsafe {
        ffi::glUniformMatrix4fv(
            location,
            (values.len() / 16) as GLsizei,
            transpose as GLboolean,
            values.as_ptr() as *const GLfloat,
        )
    }
}

pub fn use_program(program: GLuint) {
    unsafe { ffi::glUseProgram(program) }
}

pub fn validate_program(program: GLuint) {
    unsafe { ffi::glValidateProgram(program) }
}

pub fn vertex_attrib1f(index: GLuint, x: GLfloat) {
    unsafe { ffi::glVertexAttrib1f(index, x) }
}

pub fn vertex_attrib1fv(index: GLuint, values: &[GLfloat]) {
    unsafe { ffi::glVertexAttrib1fv(index, values.as_ptr()) }
}

pub fn vertex_attrib2f(index: GLuint, x: GLfloat, y: GLfloat) {
    unsafe { ffi::glVertexAttrib2f(index, x, y) }
}

pub fn vertex_attrib2fv(index: GLuint, values: &[GLfloat]) {
    unsafe { ffi::glVertexAttrib2fv(index, values.as_ptr()) }
}

pub fn vertex_attrib3f(index: GLuint, x: GLfloat, y: GLfloat, z: GLfloat) {
    unsafe { ffi::glVertexAttrib3f(index, x, y, z) }
}

pub fn vertex_attrib3fv(index: GLuint, values: &[GLfloat]) {
    unsafe { ffi::glVertexAttrib3fv(index, values.as_ptr()) }
}

pub fn vertex_attrib4f(index: GLuint, x: GLfloat, y: GLfloat, z: GLfloat, w: GLfloat) {
    unsafe { ffi::glVertexAttrib4f(index, x, y, z, w) }
}

pub fn vertex_attrib4fv(index: GLuint, values: &[GLfloat]) {
    unsafe { ffi::glVertexAttrib4fv(index, values.as_ptr()) }
}

pub fn vertex_attrib_pointer<T>(
    index: GLuint,
    size: GLint,
    type_: GLenum,
    normalized: bool,
    stride: GLsizei,
    buffer: &[T],
) {
    unsafe {
        let ptr = if buffer.len() == 0 {
            0 as *const GLvoid
        } else {
            buffer.as_ptr() as *const GLvoid
        };

        ffi::glVertexAttribPointer(index, size, type_, normalized as GLboolean, stride, ptr)
    }
}

pub fn vertex_attrib_pointer_offset(
    index: GLuint,
    size: GLint,
    type_: GLenum,
    normalized: bool,
    stride: GLsizei,
    offset: GLuint,
) {
    unsafe {
        ffi::glVertexAttribPointer(
            index,
            size,
            type_,
            normalized as GLboolean,
            stride,
            offset as *const GLvoid,
        )
    }
}

pub fn viewport(x: GLint, y: GLint, width: GLsizei, height: GLsizei) {
    unsafe { ffi::glViewport(x, y, width, height) }
}
