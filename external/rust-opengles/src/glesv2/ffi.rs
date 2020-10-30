use super::*;

extern "C" {
    pub fn glActiveTexture(texture: GLenum);

    pub fn glAttachShader(program: GLuint, shader: GLuint);

    pub fn glBindAttribLocation(program: GLuint, index: GLuint, name: *const GLchar);

    pub fn glBindBuffer(target: GLenum, buffer: GLuint);

    pub fn glBindFramebuffer(target: GLenum, framebuffer: GLuint);

    pub fn glBindRenderbuffer(target: GLenum, renderbuffer: GLuint);

    pub fn glBindTexture(target: GLenum, texture: GLuint);

    pub fn glBlendColor(red: GLclampf, green: GLclampf, blue: GLclampf, alpha: GLclampf);

    pub fn glBlendEquation(mode: GLenum);

    pub fn glBlendEquationSeparate(modeRGB: GLenum, modeAlpha: GLenum);

    pub fn glBlendFunc(sfactor: GLenum, dfactor: GLenum);

    pub fn glBlendFuncSeparate(srcRGB: GLenum, dstRGB: GLenum, srcAlpha: GLenum, dstAlpha: GLenum);

    pub fn glBufferData(target: GLenum, size: GLsizeiptr, data: *const GLvoid, usage: GLenum);

    pub fn glBufferSubData(target: GLenum, offset: GLintptr, size: GLsizeiptr, data: *const GLvoid);

    pub fn glCheckFramebufferStatus(target: GLenum) -> GLenum;

    pub fn glClear(mask: GLbitfield);

    pub fn glClearColor(red: GLclampf, green: GLclampf, blue: GLclampf, alpha: GLclampf);

    pub fn glClearDepthf(depth: GLclampf);

    pub fn glClearStencil(s: GLint);

    pub fn glColorMask(red: GLboolean, green: GLboolean, blue: GLboolean, alpha: GLboolean);

    pub fn glCompileShader(shader: GLuint);

    pub fn glCompressedTexImage2D(
        target: GLenum,
        level: GLint,
        internalformat: GLenum,
        width: GLsizei,
        height: GLsizei,
        border: GLint,
        imageSize: GLsizei,
        data: *const GLvoid,
    );

    pub fn glCompressedTexSubImage2D(
        target: GLenum,
        level: GLint,
        xoffset: GLint,
        yoffset: GLint,
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        imageSize: GLsizei,
        data: *const GLvoid,
    );

    pub fn glCopyTexImage2D(
        target: GLenum,
        level: GLint,
        internalformat: GLenum,
        x: GLint,
        y: GLint,
        width: GLsizei,
        height: GLsizei,
        border: GLint,
    );

    pub fn glCopyTexSubImage2D(
        target: GLenum,
        level: GLint,
        xoffset: GLint,
        yoffset: GLint,
        x: GLint,
        y: GLint,
        width: GLsizei,
        height: GLsizei,
    );

    pub fn glCreateProgram() -> GLuint;

    pub fn glCreateShader(type_: GLenum) -> GLuint;

    pub fn glCullFace(mode: GLenum);

    pub fn glDeleteBuffers(n: GLsizei, buffers: *const GLuint);

    pub fn glDeleteFramebuffers(n: GLsizei, framebuffers: *const GLuint);

    pub fn glDeleteProgram(program: GLuint);

    pub fn glDeleteRenderbuffers(n: GLsizei, renderbuffers: *const GLuint);

    pub fn glDeleteShader(shader: GLuint);

    pub fn glDeleteTextures(n: GLsizei, textures: *const GLuint);

    pub fn glDepthFunc(func: GLenum);

    pub fn glDepthMask(flag: GLboolean);

    pub fn glDepthRangef(zNear: GLclampf, zFar: GLclampf);

    pub fn glDetachShader(program: GLuint, shader: GLuint);

    pub fn glDisable(cap: GLenum);

    pub fn glDisableVertexAttribArray(index: GLuint);

    pub fn glDrawArrays(mode: GLenum, first: GLint, count: GLsizei);

    pub fn glDrawElements(mode: GLenum, count: GLsizei, type_: GLenum, indices: *const GLvoid);

    pub fn glEnable(cap: GLenum);

    pub fn glEnableVertexAttribArray(index: GLuint);

    pub fn glFinish();

    pub fn glFlush();

    pub fn glFramebufferRenderbuffer(
        target: GLenum,
        attachment: GLenum,
        renderbuffertarget: GLenum,
        renderbuffer: GLuint,
    );

    pub fn glFramebufferTexture2D(
        target: GLenum,
        attachment: GLenum,
        textarget: GLenum,
        texture: GLuint,
        level: GLint,
    );

    pub fn glFrontFace(mode: GLenum);

    pub fn glGenBuffers(n: GLsizei, buffers: *mut GLuint);

    pub fn glGenerateMipmap(target: GLenum);

    pub fn glGenFramebuffers(n: GLsizei, framebuffers: *mut GLuint);

    pub fn glGenRenderbuffers(n: GLsizei, renderbuffers: *mut GLuint);

    pub fn glGenTextures(n: GLsizei, textures: *mut GLuint);

    pub fn glGetActiveAttrib(
        program: GLuint,
        index: GLuint,
        bufsize: GLsizei,
        length: *mut GLsizei,
        size: *mut GLint,
        type_: *mut GLenum,
        name: *mut GLchar,
    );

    pub fn glGetActiveUniform(
        program: GLuint,
        index: GLuint,
        bufsize: GLsizei,
        length: *mut GLsizei,
        size: *mut GLint,
        type_: *mut GLenum,
        name: *mut GLchar,
    );

    pub fn glGetAttachedShaders(
        program: GLuint,
        maxcount: GLsizei,
        count: *mut GLsizei,
        shaders: *mut GLuint,
    );

    pub fn glGetAttribLocation(program: GLuint, name: *const GLchar) -> GLint;

    pub fn glGetBooleanv(pname: GLenum, params: *mut GLboolean);

    pub fn glGetBufferParameteriv(target: GLenum, pname: GLenum, params: *mut GLint);

    pub fn glGetError() -> GLenum;

    pub fn glGetFloatv(pname: GLenum, params: *mut GLfloat);

    pub fn glGetFramebufferAttachmentParameteriv(
        target: GLenum,
        attachment: GLenum,
        pname: GLenum,
        params: *mut GLint,
    );

    pub fn glGetIntegerv(pname: GLenum, params: *mut GLint);

    pub fn glGetProgramiv(program: GLuint, pname: GLenum, params: *mut GLint);

    pub fn glGetProgramInfoLog(
        program: GLuint,
        bufsize: GLsizei,
        length: *mut GLsizei,
        infolog: *mut GLchar,
    );

    pub fn glGetRenderbufferParameteriv(target: GLenum, pname: GLenum, params: *mut GLint);

    pub fn glGetShaderiv(shader: GLuint, pname: GLenum, params: *mut GLint);

    pub fn glGetShaderInfoLog(
        shader: GLuint,
        bufsize: GLsizei,
        length: *mut GLsizei,
        infolog: *mut GLchar,
    );

    pub fn glGetShaderPrecisionFormat(
        shadertype: GLenum,
        precisiontype: GLenum,
        range: *mut GLint,
        precision: *mut GLint,
    );

    pub fn glGetShaderSource(
        shader: GLuint,
        bufsize: GLsizei,
        length: *mut GLsizei,
        source: *mut GLchar,
    );

    pub fn glGetString(name: GLenum) -> *const GLubyte;

    pub fn glGetTexParameterfv(target: GLenum, pname: GLenum, params: *mut GLfloat);

    pub fn glGetTexParameteriv(target: GLenum, pname: GLenum, params: *mut GLint);

    pub fn glGetUniformfv(program: GLuint, location: GLint, params: *mut GLfloat);

    pub fn glGetUniformiv(program: GLuint, location: GLint, params: *mut GLint);

    pub fn glGetUniformLocation(program: GLuint, name: *const GLchar) -> GLint;

    pub fn glGetVertexAttribfv(index: GLuint, pname: GLenum, params: *mut GLfloat);

    pub fn glGetVertexAttribiv(index: GLuint, pname: GLenum, params: *mut GLint);

    pub fn glGetVertexAttribPointerv(index: GLuint, pname: GLenum, pointer: *mut *mut GLvoid);

    pub fn glHint(target: GLenum, mode: GLenum);

    pub fn glIsBuffer(buffer: GLuint) -> GLboolean;

    pub fn glIsEnabled(cap: GLenum) -> GLboolean;

    pub fn glIsFramebuffer(framebuffer: GLuint) -> GLboolean;

    pub fn glIsProgram(program: GLuint) -> GLboolean;

    pub fn glIsRenderbuffer(renderbuffer: GLuint) -> GLboolean;

    pub fn glIsShader(shader: GLuint) -> GLboolean;

    pub fn glIsTexture(texture: GLuint) -> GLboolean;

    pub fn glLineWidth(width: GLfloat);

    pub fn glLinkProgram(program: GLuint);

    pub fn glPixelStorei(pname: GLenum, param: GLint);

    pub fn glPolygonOffset(factor: GLfloat, units: GLfloat);

    pub fn glReadPixels(
        x: GLint,
        y: GLint,
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        type_: GLenum,
        pixels: *mut GLvoid,
    );

    pub fn glReleaseShaderCompiler();

    pub fn glRenderbufferStorage(
        target: GLenum,
        internalformat: GLenum,
        width: GLsizei,
        height: GLsizei,
    );

    pub fn glSampleCoverage(value: GLclampf, invert: GLboolean);

    pub fn glScissor(x: GLint, y: GLint, width: GLsizei, height: GLsizei);

    pub fn glShaderBinary(
        n: GLsizei,
        shaders: *const GLuint,
        binaryformat: GLenum,
        binary: *const GLvoid,
        length: GLsizei,
    );

    pub fn glShaderSource(
        shader: GLuint,
        count: GLsizei,
        string: *const *const GLchar,
        length: *const GLint,
    );

    pub fn glStencilFunc(func: GLenum, ref_: GLint, mask: GLuint);

    pub fn glStencilFuncSeparate(face: GLenum, func: GLenum, ref_: GLint, mask: GLuint);

    pub fn glStencilMask(mask: GLuint);

    pub fn glStencilMaskSeparate(face: GLenum, mask: GLuint);

    pub fn glStencilOp(fail: GLenum, zfail: GLenum, zpass: GLenum);

    pub fn glStencilOpSeparate(face: GLenum, fail: GLenum, zfail: GLenum, zpass: GLenum);

    pub fn glTexImage2D(
        target: GLenum,
        level: GLint,
        internalformat: GLint,
        width: GLsizei,
        height: GLsizei,
        border: GLint,
        format: GLenum,
        type_: GLenum,
        pixels: *const GLvoid,
    );

    pub fn glTexParameterf(target: GLenum, pname: GLenum, param: GLfloat);

    pub fn glTexParameterfv(target: GLenum, pname: GLenum, params: *const GLfloat);

    pub fn glTexParameteri(target: GLenum, pname: GLenum, param: GLint);

    pub fn glTexParameteriv(target: GLenum, pname: GLenum, params: *const GLint);

    pub fn glTexSubImage2D(
        target: GLenum,
        level: GLint,
        xoffset: GLint,
        yoffset: GLint,
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        type_: GLenum,
        pixels: *const GLvoid,
    );

    pub fn glUniform1f(location: GLint, x: GLfloat);

    pub fn glUniform1fv(location: GLint, count: GLsizei, v: *const GLfloat);

    pub fn glUniform1i(location: GLint, x: GLint);

    pub fn glUniform1iv(location: GLint, count: GLsizei, v: *const GLint);

    pub fn glUniform2f(location: GLint, x: GLfloat, y: GLfloat);

    pub fn glUniform2fv(location: GLint, count: GLsizei, v: *const GLfloat);

    pub fn glUniform2i(location: GLint, x: GLint, y: GLint);

    pub fn glUniform2iv(location: GLint, count: GLsizei, v: *const GLint);

    pub fn glUniform3f(location: GLint, x: GLfloat, y: GLfloat, z: GLfloat);

    pub fn glUniform3fv(location: GLint, count: GLsizei, v: *const GLfloat);

    pub fn glUniform3i(location: GLint, x: GLint, y: GLint, z: GLint);

    pub fn glUniform3iv(location: GLint, count: GLsizei, v: *const GLint);

    pub fn glUniform4f(location: GLint, x: GLfloat, y: GLfloat, z: GLfloat, w: GLfloat);

    pub fn glUniform4fv(location: GLint, count: GLsizei, v: *const GLfloat);

    pub fn glUniform4i(location: GLint, x: GLint, y: GLint, z: GLint, w: GLint);

    pub fn glUniform4iv(location: GLint, count: GLsizei, v: *const GLint);

    pub fn glUniformMatrix2fv(
        location: GLint,
        count: GLsizei,
        transpose: GLboolean,
        value: *const GLfloat,
    );

    pub fn glUniformMatrix3fv(
        location: GLint,
        count: GLsizei,
        transpose: GLboolean,
        value: *const GLfloat,
    );

    pub fn glUniformMatrix4fv(
        location: GLint,
        count: GLsizei,
        transpose: GLboolean,
        value: *const GLfloat,
    );

    pub fn glUseProgram(program: GLuint);

    pub fn glValidateProgram(program: GLuint);

    pub fn glVertexAttrib1f(indx: GLuint, x: GLfloat);

    pub fn glVertexAttrib1fv(indx: GLuint, values: *const GLfloat);

    pub fn glVertexAttrib2f(indx: GLuint, x: GLfloat, y: GLfloat);

    pub fn glVertexAttrib2fv(indx: GLuint, values: *const GLfloat);

    pub fn glVertexAttrib3f(indx: GLuint, x: GLfloat, y: GLfloat, z: GLfloat);

    pub fn glVertexAttrib3fv(indx: GLuint, values: *const GLfloat);

    pub fn glVertexAttrib4f(indx: GLuint, x: GLfloat, y: GLfloat, z: GLfloat, w: GLfloat);

    pub fn glVertexAttrib4fv(indx: GLuint, values: *const GLfloat);

    pub fn glVertexAttribPointer(
        indx: GLuint,
        size: GLint,
        type_: GLenum,
        normalized: GLboolean,
        stride: GLsizei,
        ptr: *const GLvoid,
    );

    pub fn glViewport(x: GLint, y: GLint, width: GLsizei, height: GLsizei);
}
