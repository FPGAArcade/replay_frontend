# ui_raster

This is a testbed a UI rasterizer, designed to target ARM v8.2 hardware with NEON SIMD instructions. The CPU features 4 cores: two high-performance cores and two low-power cores, running at 1.2 GHz and 800 MHz respectively. Additionally, a reference rasterizer for x86 is provided.

The plan is to first implement an ISPC version, followed by an intrinsic version, to compare their performance and determine which is faster. The ISPC implementation will reside in the ispc directory, while the intrinsic implementation will be located in the intrinsic directory.

## ISPC Implementation status

### 1. Basic Shape Drawing
- [x] Draw rectangles.
- [ ] Draw rectangles with rounded corners.

### 2. Text and Image Rendering
- [ ] Draw text.
- [x] Draw images.

### 3. Styling and Effects
- [ ] Draw borders.
- [x] Support clipping for contained drawing.

### 4. Rendering Features
- [x] Support vertex color interpolation across rectangles.
- [x] Implement pre-multiplied alpha blending.
- [x] Enable full subpixel rendering.
- [x] Render in linear color space using 16-bit components per channel.

### 5. Texture Sampling
- [x] Sample textures with bilinear filtering.

### 6. Code Structure
- [x] Create code permutations to optimize based on rendering primitives.

## Intrinsic Implementation status

### 1. Basic Shape Drawing
- [ ] Draw rectangles.
- [ ] Draw rectangles with rounded corners.

### 2. Text and Image Rendering
- [ ] Draw text.
- [ ] Draw images.

### 3. Styling and Effects
- [ ] Draw borders.
- [ ] Support clipping for contained drawing.

### 4. Rendering Features
- [ ] Support vertex color interpolation across rectangles.
- [ ] Implement pre-multiplied alpha blending.
- [ ] Enable full subpixel rendering.
- [ ] Render in linear color space using 16-bit components per channel.

### 5. Texture Sampling
- [ ] Sample textures with bilinear filtering.

### 6. Code Structure
- [ ] Create code permutations to optimize based on rendering primitives.
