use super::*;

// OpenGL ES core versions
pub const GL_ES_VERSION_2_0: GLint = 1;

// ClearBufferMask
pub const GL_DEPTH_BUFFER_BIT: GLuint = 0x00000100;
pub const GL_STENCIL_BUFFER_BIT: GLuint = 0x00000400;
pub const GL_COLOR_BUFFER_BIT: GLuint = 0x00004000;

// boolean
pub const GL_FALSE: GLboolean = 0;
pub const GL_TRUE: GLboolean = 1;

// BeginMode
pub const GL_POINTS: GLuint = 0x0000;
pub const GL_LINES: GLuint = 0x0001;
pub const GL_LINE_LOOP: GLuint = 0x0002;
pub const GL_LINE_STRIP: GLuint = 0x0003;
pub const GL_TRIANGLES: GLuint = 0x0004;
pub const GL_TRIANGLE_STRIP: GLuint = 0x0005;
pub const GL_TRIANGLE_FAN: GLuint = 0x0006;

// AlphaFunction(not supported in ES20)
//      GL_NEVER
//      GL_LESS
//      GL_EQUAL
//      GL_LEQUAL
//      GL_GREATER
//      GL_NOTEQUAL
//      GL_GEQUAL
//      GL_ALWAYS

// BlendingFactorDest
pub const GL_ZERO: GLuint = 0;
pub const GL_ONE: GLuint = 1;
pub const GL_SRC_COLOR: GLuint = 0x0300;
pub const GL_ONE_MINUS_SRC_COLOR: GLuint = 0x0301;
pub const GL_SRC_ALPHA: GLuint = 0x0302;
pub const GL_ONE_MINUS_SRC_ALPHA: GLuint = 0x0303;
pub const GL_DST_ALPHA: GLuint = 0x0304;
pub const GL_ONE_MINUS_DST_ALPHA: GLuint = 0x0305;

// BlendingFactorSrc
//      GL_ZERO
//      GL_ONE
pub const GL_DST_COLOR: GLuint = 0x0306;
pub const GL_ONE_MINUS_DST_COLOR: GLuint = 0x0307;
pub const GL_SRC_ALPHA_SATURATE: GLuint = 0x0308;
//      GL_SRC_ALPHA
//      GL_ONE_MINUS_SRC_ALPHA
//      GL_DST_ALPHA
//      GL_ONE_MINUS_DST_ALPHA

// BlendEquationSeparate
pub const GL_FUNC_ADD: GLuint = 0x8006;
pub const GL_BLEND_EQUATION: GLuint = 0x8009;
pub const GL_BLEND_EQUATION_RGB: GLuint = 0x8009; // same as BLEND_EQUATION
pub const GL_BLEND_EQUATION_ALPHA: GLuint = 0x883D;

// BlendSubtract
pub const GL_FUNC_SUBTRACT: GLuint = 0x800A;
pub const GL_FUNC_REVERSE_SUBTRACT: GLuint = 0x800B;

// Separate Blend Functions
pub const GL_BLEND_DST_RGB: GLuint = 0x80C8;
pub const GL_BLEND_SRC_RGB: GLuint = 0x80C9;
pub const GL_BLEND_DST_ALPHA: GLuint = 0x80CA;
pub const GL_BLEND_SRC_ALPHA: GLuint = 0x80CB;
pub const GL_CONSTANT_COLOR: GLuint = 0x8001;
pub const GL_ONE_MINUS_CONSTANT_COLOR: GLuint = 0x8002;
pub const GL_CONSTANT_ALPHA: GLuint = 0x8003;
pub const GL_ONE_MINUS_CONSTANT_ALPHA: GLuint = 0x8004;
pub const GL_BLEND_COLOR: GLuint = 0x8005;

// Buffer Objects
pub const GL_ARRAY_BUFFER: GLuint = 0x8892;
pub const GL_ELEMENT_ARRAY_BUFFER: GLuint = 0x8893;
pub const GL_ARRAY_BUFFER_BINDING: GLuint = 0x8894;
pub const GL_ELEMENT_ARRAY_BUFFER_BINDING: GLuint = 0x8895;

pub const GL_STREAM_DRAW: GLuint = 0x88E0;
pub const GL_STATIC_DRAW: GLuint = 0x88E4;
pub const GL_DYNAMIC_DRAW: GLuint = 0x88E8;

pub const GL_BUFFER_SIZE: GLuint = 0x8764;
pub const GL_BUFFER_USAGE: GLuint = 0x8765;

pub const GL_CURRENT_VERTEX_ATTRIB: GLuint = 0x8626;

// CullFaceMode
pub const GL_FRONT: GLuint = 0x0404;
pub const GL_BACK: GLuint = 0x0405;
pub const GL_FRONT_AND_BACK: GLuint = 0x0408;

// DepthFunction
//      GL_NEVER
//      GL_LESS
//      GL_EQUAL
//      GL_LEQUAL
//      GL_GREATER
//      GL_NOTEQUAL
//      GL_GEQUAL
//      GL_ALWAYS

// EnableCap
pub const GL_TEXTURE_2D: GLuint = 0x0DE1;
pub const GL_CULL_FACE: GLuint = 0x0B44;
pub const GL_BLEND: GLuint = 0x0BE2;
pub const GL_DITHER: GLuint = 0x0BD0;
pub const GL_STENCIL_TEST: GLuint = 0x0B90;
pub const GL_DEPTH_TEST: GLuint = 0x0B71;
pub const GL_SCISSOR_TEST: GLuint = 0x0C11;
pub const GL_POLYGON_OFFSET_FILL: GLuint = 0x8037;
pub const GL_SAMPLE_ALPHA_TO_COVERAGE: GLuint = 0x809E;
pub const GL_SAMPLE_COVERAGE: GLuint = 0x80A0;

// ErrorCode
pub const GL_NO_ERROR: GLuint = 0;
pub const GL_INVALID_ENUM: GLuint = 0x0500;
pub const GL_INVALID_VALUE: GLuint = 0x0501;
pub const GL_INVALID_OPERATION: GLuint = 0x0502;
pub const GL_OUT_OF_MEMORY: GLuint = 0x0505;

// FrontFaceDirection
pub const GL_CW: GLint = 0x0900;
pub const GL_CCW: GLint = 0x0901;

// GetPName
pub const GL_LINE_WIDTH: GLuint = 0x0B21;
pub const GL_ALIASED_POINT_SIZE_RANGE: GLuint = 0x846D;
pub const GL_ALIASED_LINE_WIDTH_RANGE: GLuint = 0x846E;
pub const GL_CULL_FACE_MODE: GLuint = 0x0B45;
pub const GL_FRONT_FACE: GLuint = 0x0B46;
pub const GL_DEPTH_RANGE: GLuint = 0x0B70;
pub const GL_DEPTH_WRITEMASK: GLuint = 0x0B72;
pub const GL_DEPTH_CLEAR_VALUE: GLuint = 0x0B73;
pub const GL_DEPTH_FUNC: GLuint = 0x0B74;
pub const GL_STENCIL_CLEAR_VALUE: GLuint = 0x0B91;
pub const GL_STENCIL_FUNC: GLuint = 0x0B92;
pub const GL_STENCIL_FAIL: GLuint = 0x0B94;
pub const GL_STENCIL_PASS_DEPTH_FAIL: GLuint = 0x0B95;
pub const GL_STENCIL_PASS_DEPTH_PASS: GLuint = 0x0B96;
pub const GL_STENCIL_REF: GLuint = 0x0B97;
pub const GL_STENCIL_VALUE_MASK: GLuint = 0x0B93;
pub const GL_STENCIL_WRITEMASK: GLuint = 0x0B98;
pub const GL_STENCIL_BACK_FUNC: GLuint = 0x8800;
pub const GL_STENCIL_BACK_FAIL: GLuint = 0x8801;
pub const GL_STENCIL_BACK_PASS_DEPTH_FAIL: GLuint = 0x8802;
pub const GL_STENCIL_BACK_PASS_DEPTH_PASS: GLuint = 0x8803;
pub const GL_STENCIL_BACK_REF: GLuint = 0x8CA3;
pub const GL_STENCIL_BACK_VALUE_MASK: GLuint = 0x8CA4;
pub const GL_STENCIL_BACK_WRITEMASK: GLuint = 0x8CA5;
pub const GL_VIEWPORT: GLuint = 0x0BA2;
pub const GL_SCISSOR_BOX: GLuint = 0x0C10;
//      GL_SCISSOR_TEST
pub const GL_COLOR_CLEAR_VALUE: GLuint = 0x0C22;
pub const GL_COLOR_WRITEMASK: GLuint = 0x0C23;
pub const GL_UNPACK_ALIGNMENT: GLuint = 0x0CF5;
pub const GL_PACK_ALIGNMENT: GLuint = 0x0D05;
pub const GL_MAX_TEXTURE_SIZE: GLuint = 0x0D33;
pub const GL_MAX_VIEWPORT_DIMS: GLuint = 0x0D3A;
pub const GL_SUBPIXEL_BITS: GLuint = 0x0D50;
pub const GL_RED_BITS: GLuint = 0x0D52;
pub const GL_GREEN_BITS: GLuint = 0x0D53;
pub const GL_BLUE_BITS: GLuint = 0x0D54;
pub const GL_ALPHA_BITS: GLuint = 0x0D55;
pub const GL_DEPTH_BITS: GLuint = 0x0D56;
pub const GL_STENCIL_BITS: GLuint = 0x0D57;
pub const GL_POLYGON_OFFSET_UNITS: GLuint = 0x2A00;
//      GL_POLYGON_OFFSET_FILL
pub const GL_POLYGON_OFFSET_FACTOR: GLuint = 0x8038;
pub const GL_TEXTURE_BINDING_2D: GLuint = 0x8069;
pub const GL_SAMPLE_BUFFERS: GLuint = 0x80A8;
pub const GL_SAMPLES: GLuint = 0x80A9;
pub const GL_SAMPLE_COVERAGE_VALUE: GLuint = 0x80AA;
pub const GL_SAMPLE_COVERAGE_INVERT: GLuint = 0x80AB;

// GetTextureParameter
//      GL_TEXTURE_MAG_FILTER
//      GL_TEXTURE_MIN_FILTER
//      GL_TEXTURE_WRAP_S
//      GL_TEXTURE_WRAP_T

pub const GL_NUM_COMPRESSED_TEXTURE_FORMATS: GLuint = 0x86A2;
pub const GL_COMPRESSED_TEXTURE_FORMATS: GLuint = 0x86A3;

// HintMode
pub const GL_DONT_CARE: GLuint = 0x1100;
pub const GL_FASTEST: GLuint = 0x1101;
pub const GL_NICEST: GLuint = 0x1102;

// HintTarget
pub const GL_GENERATE_MIPMAP_HINT: GLint = 0x8192;

// DataType
pub const GL_BYTE: GLuint = 0x1400;
pub const GL_UNSIGNED_BYTE: GLuint = 0x1401;
pub const GL_SHORT: GLuint = 0x1402;
pub const GL_UNSIGNED_SHORT: GLuint = 0x1403;
pub const GL_INT: GLuint = 0x1404;
pub const GL_UNSIGNED_INT: GLuint = 0x1405;
pub const GL_FLOAT: GLuint = 0x1406;
pub const GL_FIXED: GLuint = 0x140C;

// PixelFormat
pub const GL_DEPTH_COMPONENT: GLuint = 0x1902;
pub const GL_ALPHA: GLuint = 0x1906;
pub const GL_RGB: GLuint = 0x1907;
pub const GL_RGBA: GLuint = 0x1908;
pub const GL_LUMINANCE: GLuint = 0x1909;
pub const GL_LUMINANCE_ALPHA: GLuint = 0x190A;

// PixelType
//      GL_UNSIGNED_BYTE
pub const GL_UNSIGNED_SHORT_4_4_4_4: GLuint = 0x8033;
pub const GL_UNSIGNED_SHORT_5_5_5_1: GLuint = 0x8034;
pub const GL_UNSIGNED_SHORT_5_6_5: GLuint = 0x8363;

// Shaders
pub const GL_FRAGMENT_SHADER: GLuint = 0x8B30;
pub const GL_VERTEX_SHADER: GLuint = 0x8B31;
pub const GL_MAX_VERTEX_ATTRIBS: GLuint = 0x8869;
pub const GL_MAX_VERTEX_UNIFORM_VECTORS: GLuint = 0x8DFB;
pub const GL_MAX_VARYING_VECTORS: GLuint = 0x8DFC;
pub const GL_MAX_COMBINED_TEXTURE_IMAGE_UNITS: GLuint = 0x8B4D;
pub const GL_MAX_VERTEX_TEXTURE_IMAGE_UNITS: GLuint = 0x8B4C;
pub const GL_MAX_TEXTURE_IMAGE_UNITS: GLuint = 0x8872;
pub const GL_MAX_FRAGMENT_UNIFORM_VECTORS: GLuint = 0x8DFD;
pub const GL_SHADER_TYPE: GLuint = 0x8B4F;
pub const GL_DELETE_STATUS: GLuint = 0x8B80;
pub const GL_LINK_STATUS: GLuint = 0x8B82;
pub const GL_VALIDATE_STATUS: GLuint = 0x8B83;
pub const GL_ATTACHED_SHADERS: GLuint = 0x8B85;
pub const GL_ACTIVE_UNIFORMS: GLuint = 0x8B86;
pub const GL_ACTIVE_UNIFORM_MAX_LENGTH: GLuint = 0x8B87;
pub const GL_ACTIVE_ATTRIBUTES: GLuint = 0x8B89;
pub const GL_ACTIVE_ATTRIBUTE_MAX_LENGTH: GLuint = 0x8B8A;
pub const GL_SHADING_LANGUAGE_VERSION: GLuint = 0x8B8C;
pub const GL_CURRENT_PROGRAM: GLuint = 0x8B8D;

// StencilFunction
pub const GL_NEVER: GLuint = 0x0200;
pub const GL_LESS: GLuint = 0x0201;
pub const GL_EQUAL: GLuint = 0x0202;
pub const GL_LEQUAL: GLuint = 0x0203;
pub const GL_GREATER: GLuint = 0x0204;
pub const GL_NOTEQUAL: GLuint = 0x0205;
pub const GL_GEQUAL: GLuint = 0x0206;
pub const GL_ALWAYS: GLuint = 0x0207;

// StencilOp
//      GL_ZERO
pub const GL_KEEP: GLuint = 0x1E00;
pub const GL_REPLACE: GLuint = 0x1E01;
pub const GL_INCR: GLuint = 0x1E02;
pub const GL_DECR: GLuint = 0x1E03;
pub const GL_INVERT: GLuint = 0x150A;
pub const GL_INCR_WRAP: GLuint = 0x8507;
pub const GL_DECR_WRAP: GLuint = 0x8508;

// StringName
pub const GL_VENDOR: GLuint = 0x1F00;
pub const GL_RENDERER: GLuint = 0x1F01;
pub const GL_VERSION: GLuint = 0x1F02;
pub const GL_EXTENSIONS: GLuint = 0x1F03;

// TextureMagFilter
pub const GL_NEAREST: GLuint = 0x2600;
pub const GL_LINEAR: GLuint = 0x2601;

// TextureMinFilter
//      GL_NEAREST
//      GL_LINEAR
pub const GL_NEAREST_MIPMAP_NEAREST: GLuint = 0x2700;
pub const GL_LINEAR_MIPMAP_NEAREST: GLuint = 0x2701;
pub const GL_NEAREST_MIPMAP_LINEAR: GLuint = 0x2702;
pub const GL_LINEAR_MIPMAP_LINEAR: GLuint = 0x2703;

// TextureParameterName
pub const GL_TEXTURE_MAG_FILTER: GLuint = 0x2800;
pub const GL_TEXTURE_MIN_FILTER: GLuint = 0x2801;
pub const GL_TEXTURE_WRAP_S: GLuint = 0x2802;
pub const GL_TEXTURE_WRAP_T: GLuint = 0x2803;

// TextureTarget
//      GL_TEXTURE_2D
pub const GL_TEXTURE: GLuint = 0x1702;
pub const GL_TEXTURE_CUBE_MAP: GLuint = 0x8513;
pub const GL_TEXTURE_BINDING_CUBE_MAP: GLuint = 0x8514;
pub const GL_TEXTURE_CUBE_MAP_POSITIVE_X: GLuint = 0x8515;
pub const GL_TEXTURE_CUBE_MAP_NEGATIVE_X: GLuint = 0x8516;
pub const GL_TEXTURE_CUBE_MAP_POSITIVE_Y: GLuint = 0x8517;
pub const GL_TEXTURE_CUBE_MAP_NEGATIVE_Y: GLuint = 0x8518;
pub const GL_TEXTURE_CUBE_MAP_POSITIVE_Z: GLuint = 0x8519;
pub const GL_TEXTURE_CUBE_MAP_NEGATIVE_Z: GLuint = 0x851A;
pub const GL_MAX_CUBE_MAP_TEXTURE_SIZE: GLuint = 0x851C;

// TextureUnit
pub const GL_TEXTURE0: GLuint = 0x84C0;
pub const GL_TEXTURE1: GLuint = 0x84C1;
pub const GL_TEXTURE2: GLuint = 0x84C2;
pub const GL_TEXTURE3: GLuint = 0x84C3;
pub const GL_TEXTURE4: GLuint = 0x84C4;
pub const GL_TEXTURE5: GLuint = 0x84C5;
pub const GL_TEXTURE6: GLuint = 0x84C6;
pub const GL_TEXTURE7: GLuint = 0x84C7;
pub const GL_TEXTURE8: GLuint = 0x84C8;
pub const GL_TEXTURE9: GLuint = 0x84C9;
pub const GL_TEXTURE10: GLuint = 0x84CA;
pub const GL_TEXTURE11: GLuint = 0x84CB;
pub const GL_TEXTURE12: GLuint = 0x84CC;
pub const GL_TEXTURE13: GLuint = 0x84CD;
pub const GL_TEXTURE14: GLuint = 0x84CE;
pub const GL_TEXTURE15: GLuint = 0x84CF;
pub const GL_TEXTURE16: GLuint = 0x84D0;
pub const GL_TEXTURE17: GLuint = 0x84D1;
pub const GL_TEXTURE18: GLuint = 0x84D2;
pub const GL_TEXTURE19: GLuint = 0x84D3;
pub const GL_TEXTURE20: GLuint = 0x84D4;
pub const GL_TEXTURE21: GLuint = 0x84D5;
pub const GL_TEXTURE22: GLuint = 0x84D6;
pub const GL_TEXTURE23: GLuint = 0x84D7;
pub const GL_TEXTURE24: GLuint = 0x84D8;
pub const GL_TEXTURE25: GLuint = 0x84D9;
pub const GL_TEXTURE26: GLuint = 0x84DA;
pub const GL_TEXTURE27: GLuint = 0x84DB;
pub const GL_TEXTURE28: GLuint = 0x84DC;
pub const GL_TEXTURE29: GLuint = 0x84DD;
pub const GL_TEXTURE30: GLuint = 0x84DE;
pub const GL_TEXTURE31: GLuint = 0x84DF;
pub const GL_ACTIVE_TEXTURE: GLuint = 0x84E0;

// TextureWrapMode
pub const GL_REPEAT: GLuint = 0x2901;
pub const GL_CLAMP_TO_EDGE: GLuint = 0x812F;
pub const GL_MIRRORED_REPEAT: GLuint = 0x8370;

// Uniform Types
pub const GL_FLOAT_VEC2: GLuint = 0x8B50;
pub const GL_FLOAT_VEC3: GLuint = 0x8B51;
pub const GL_FLOAT_VEC4: GLuint = 0x8B52;
pub const GL_INT_VEC2: GLuint = 0x8B53;
pub const GL_INT_VEC3: GLuint = 0x8B54;
pub const GL_INT_VEC4: GLuint = 0x8B55;
pub const GL_BOOL: GLuint = 0x8B56;
pub const GL_BOOL_VEC2: GLuint = 0x8B57;
pub const GL_BOOL_VEC3: GLuint = 0x8B58;
pub const GL_BOOL_VEC4: GLuint = 0x8B59;
pub const GL_FLOAT_MAT2: GLuint = 0x8B5A;
pub const GL_FLOAT_MAT3: GLuint = 0x8B5B;
pub const GL_FLOAT_MAT4: GLuint = 0x8B5C;
pub const GL_SAMPLER_2D: GLuint = 0x8B5E;
pub const GL_SAMPLER_CUBE: GLuint = 0x8B60;

// Vertex Arrays
pub const GL_VERTEX_ATTRIB_ARRAY_ENABLED: GLuint = 0x8622;
pub const GL_VERTEX_ATTRIB_ARRAY_SIZE: GLuint = 0x8623;
pub const GL_VERTEX_ATTRIB_ARRAY_STRIDE: GLuint = 0x8624;
pub const GL_VERTEX_ATTRIB_ARRAY_TYPE: GLuint = 0x8625;
pub const GL_VERTEX_ATTRIB_ARRAY_NORMALIZED: GLuint = 0x886A;
pub const GL_VERTEX_ATTRIB_ARRAY_POINTER: GLuint = 0x8645;
pub const GL_VERTEX_ATTRIB_ARRAY_BUFFER_BINDING: GLuint = 0x889F;

// Read Format
pub const GL_IMPLEMENTATION_COLOR_READ_TYPE: GLuint = 0x8B9A;
pub const GL_IMPLEMENTATION_COLOR_READ_FORMAT: GLuint = 0x8B9B;

// Shader Source
pub const GL_COMPILE_STATUS: GLuint = 0x8B81;
pub const GL_INFO_LOG_LENGTH: GLuint = 0x8B84;
pub const GL_SHADER_SOURCE_LENGTH: GLuint = 0x8B88;
pub const GL_SHADER_COMPILER: GLuint = 0x8DFA;

// Shader Binary
pub const GL_SHADER_BINARY_FORMATS: GLuint = 0x8DF8;
pub const GL_NUM_SHADER_BINARY_FORMATS: GLuint = 0x8DF9;

// Shader Precision-Specified Types
pub const GL_LOW_FLOAT: GLuint = 0x8DF0;
pub const GL_MEDIUM_FLOAT: GLuint = 0x8DF1;
pub const GL_HIGH_FLOAT: GLuint = 0x8DF2;
pub const GL_LOW_INT: GLuint = 0x8DF3;
pub const GL_MEDIUM_INT: GLuint = 0x8DF4;
pub const GL_HIGH_INT: GLuint = 0x8DF5;

// Framebuffer Object.
pub const GL_FRAMEBUFFER: GLuint = 0x8D40;
pub const GL_RENDERBUFFER: GLuint = 0x8D41;

pub const GL_RGBA4: GLuint = 0x8056;
pub const GL_RGB5_A1: GLuint = 0x8057;
pub const GL_RGB565: GLuint = 0x8D62;
pub const GL_DEPTH_COMPONENT16: GLuint = 0x81A5;
pub const GL_STENCIL_INDEX: GLuint = 0x1901;
pub const GL_STENCIL_INDEX8: GLuint = 0x8D48;

pub const GL_RENDERBUFFER_WIDTH: GLuint = 0x8D42;
pub const GL_RENDERBUFFER_HEIGHT: GLuint = 0x8D43;
pub const GL_RENDERBUFFER_INTERNAL_FORMAT: GLuint = 0x8D44;
pub const GL_RENDERBUFFER_RED_SIZE: GLuint = 0x8D50;
pub const GL_RENDERBUFFER_GREEN_SIZE: GLuint = 0x8D51;
pub const GL_RENDERBUFFER_BLUE_SIZE: GLuint = 0x8D52;
pub const GL_RENDERBUFFER_ALPHA_SIZE: GLuint = 0x8D53;
pub const GL_RENDERBUFFER_DEPTH_SIZE: GLuint = 0x8D54;
pub const GL_RENDERBUFFER_STENCIL_SIZE: GLuint = 0x8D55;

pub const GL_FRAMEBUFFER_ATTACHMENT_OBJECT_TYPE: GLuint = 0x8CD0;
pub const GL_FRAMEBUFFER_ATTACHMENT_OBJECT_NAME: GLuint = 0x8CD1;
pub const GL_FRAMEBUFFER_ATTACHMENT_TEXTURE_LEVEL: GLuint = 0x8CD2;
pub const GL_FRAMEBUFFER_ATTACHMENT_TEXTURE_CUBE_MAP_FACE: GLuint = 0x8CD3;

pub const GL_COLOR_ATTACHMENT0: GLuint = 0x8CE0;
pub const GL_DEPTH_ATTACHMENT: GLuint = 0x8D00;
pub const GL_STENCIL_ATTACHMENT: GLuint = 0x8D20;

pub const GL_NONE: GLuint = 0;

pub const GL_FRAMEBUFFER_COMPLETE: GLuint = 0x8CD5;
pub const GL_FRAMEBUFFER_INCOMPLETE_ATTACHMENT: GLuint = 0x8CD6;
pub const GL_FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT: GLuint = 0x8CD7;
pub const GL_FRAMEBUFFER_INCOMPLETE_DIMENSIONS: GLuint = 0x8CD9;
pub const GL_FRAMEBUFFER_UNSUPPORTED: GLuint = 0x8CDD;

pub const GL_FRAMEBUFFER_BINDING: GLuint = 0x8CA6;
pub const GL_RENDERBUFFER_BINDING: GLuint = 0x8CA7;
pub const GL_MAX_RENDERBUFFER_SIZE: GLuint = 0x84E8;

pub const GL_INVALID_FRAMEBUFFER_OPERATION: GLuint = 0x0506;
