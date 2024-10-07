use flowi_core::primitives::Primitive;

// Number of bits to repserent a color channel in sRGB color space. We use 16-bit colors to allow
// for high range of colors. Most input images are in 8-bit sRGB color space, but as we convert
// thesee to linear color space, we need to use higher bit depth to avoid banding artifacts.

const SRGB_BIT_COUNT: u32 = 12;
const LINEAR_BIT_COUNT: u32 = 15;

const COLORS: [u32; 16] = [
    0x0FF5733, // Red-Orange
    0x0DAF7A6, // Green-Mint
    0x0FFC300, // Bright Yellow
    0x0900C3F, // Deep Blue
    0x0C70039, // Dark Red
    0x02ECC71, // Emerald Green
    0x09B59B6, // Purple
    0x0F39C12, // Bright Green
    0x0A569BD, // Sky Blue
    0x0F1C40F, // Forest Green
    0x08E44AD, // Red
    0x02C3E50, // Teal
    0x0BDC3C7, // Silver
    0x09B870C, // Dark Cyan
    0x0E74C3C, // Soft Blue
    0x0D35400, // Burnt Orange
];

pub struct SwRenderer {
    _dummy: u32,
    linear_to_srgb: [u8; 1 << SRGB_BIT_COUNT],
    srgb_to_linear: [u16; 1 << 8],
}

fn linear_to_srgb(x: f32) -> f32 {
    if x <= 0.0031308 {
        x * 12.92
    } else {
        1.055 * x.powf(1.0 / 2.4) - 0.055
    }
}

fn srgb_to_linear(x: f32) -> f32 {
    if x <= 0.04045 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}

fn build_srgb_to_linear_table() -> [u16; 1 << 8] {
    let mut table = [0; 1 << 8];

    for i in 0..(1 << 8) {
        let srgb = i as f32 / (1 << 8) as f32;
        let linear = srgb_to_linear(srgb);
        table[i] = (linear * (1 << LINEAR_BIT_COUNT) as f32) as u16;
    }

    table
}

fn build_linear_to_srgb_table() -> [u8; 1 << SRGB_BIT_COUNT] {
    let mut table = [0; 1 << SRGB_BIT_COUNT];

    for i in 0..(1 << SRGB_BIT_COUNT) {
        let linear = i as f32 / (1 << SRGB_BIT_COUNT) as f32;
        let srgb = linear_to_srgb(linear);
        table[i] = (srgb * (1 << 8) as f32) as u8;
    }

    table
}

impl SwRenderer {
    pub fn new() -> Self {
        let linear_to_srgb = build_linear_to_srgb_table();
        let srgb_to_linear = build_srgb_to_linear_table();

        Self {
            _dummy: 0,
            linear_to_srgb,
            srgb_to_linear,
        }
    }

    pub fn render(
        &mut self,
        dest: &mut [u32],
        width: usize,
        height: usize,
        primitives: &[Primitive],
    ) {
        let mut color_index = 0;

        for prim in primitives {
            let min_x = prim.rect.min[0] as usize;
            let min_y = prim.rect.min[1] as usize;
            let max_x = prim.rect.max[0] as usize;
            let max_y = prim.rect.max[1] as usize;

            let max_x = max_x.min(width);
            let max_y = max_y.min(height);
            let min_x = min_x.max(0);
            let min_y = min_y.max(0);
            let color = COLORS[color_index & 0xf];

            for y in min_y..max_y {
                for x in min_x..max_x {
                    dest[y * width + x] = color;
                }
            }

            color_index += 1;
        }
    }

    // Reference implementation for quad rendering. This is used to compare the output of the of
    // the optimized renderer. It is not used in the final implementation as it will be way slower.
    //
    // Supported functionallity of this code:
    //
    // * Rounded corners
    // * Single texture lookup
    // * Linear color space
    // * 16-bit per channel output color buffer
    // * Color interpolation between the corners and blending with the texture.
    // * Clipping to the screen/tile bounds
    //
    fn quad_ref_renderer(dest: &mut [Color16], dest_rect: IRect, primitive: &Primitive) {
        // pixel center at (0.5, 0.5)
        let min_xf = primitive.rect.min[0] + 0.5;
        let min_yf = primitive.rect.min[1] + 0.5;
        let max_xf = primitive.rect.max[0] + 0.5;
        let max_yf = primitive.rect.max[1] + 0.5;

        // Clip the quad's min/max to the screen bounds, using truncation with the 0.5 pixel center
        // Add the offset before applying floor to get proper pixel center alignment
        let clipped_min_x = min_xf.floor().max(dest_rect.min[0] as f32);
        let clipped_min_y = min_yf.floor().max(dest_rect.min[1] as f32);
        let clipped_max_x = max_xf.floor().min(dest_rect.max[0] as f32);
        let clipped_max_y = max_yf.floor().min(dest_rect.max[1] as f32);

        let x_length = clipped_max_x - clipped_min_x;
        let y_length = clipped_max_y - clipped_min_y;

        let y_delta = 1.0 / (max_yf - min_yf);
        let x_delta = 1.0 / (max_xf - min_xf);

        // Calculate interpolation factors based on the clipped coordinates
        let x_min_step = (clipped_min_x - min_xf) * x_delta;
        let x_max_step = (clipped_max_x - min_xf) * x_delta;
        let y_min_step = (clipped_min_y - min_yf) * y_delta;
        let y_max_step = (clipped_max_y - min_yf) * y_delta;

        // Get the corner colors as linear f32
        let c0_color = ColorF32::from_color32_srgb(primitive.colors[0]);
        let c1_color = ColorF32::from_color32_srgb(primitive.colors[1]);
        let c2_color = ColorF32::from_color32_srgb(primitive.colors[2]);
        let c3_color = ColorF32::from_color32_srgb(primitive.colors[3]);

        // Interpolate horizontally first between bottom-left and bottom-right (for bottom side)
        let color_bottom_left = ColorF32_16::interpolate(c0_color, c1_color, x_min_step);
        let color_bottom_right = ColorF32_16::interpolate(c0_color, c1_color, x_max_step);

        // Interpolate horizontally first between top-left and top-right (for top side)
        let color_top_left = ColorF32_16::interpolate(c3_color, c2_color, y_min_step);
        let color_top_right = ColorF32_16::interpolate(c3_color, c2_color, y_max_step);

        // Interpolate vertically between bottom and top sides
        let uv_bottom_left = Uv2::interpolate(primitive.uv[0], primitive.uv[1], x_min_step);
        let uv_bottom_right = Uv2::interpolate(primitive.uv[0], primitive.uv[1], x_max_step);

        // Interpolate vertically between bottom and top sides
        let uv_top_left = Uv2::interpolate(primitive.uv[3], primitive.uv[2], y_min_step);
        let uv_top_right = Uv2::interpolate(primitive.uv[3], primitive.uv[2], y_max_step);

        let mut yc = 0.0;

        for y in y_length {
            let xc = 0.0;

            let c0 = ColorF32::interpolate(color_bottom_left, color_bottom_right, yc);
            let c1 = ColorF32::interpolate(color_top_left, color_top_right, yc);
            let uv0 = Uv2::interpolate(uv_bottom_left, uv_bottom_right, yc);
            let uv1 = Uv2::interpolate(uv_top_left, uv_top_right, yc);

            for x in x_length {
                let color = ColorF32::interpolate(c0, c1, xc);
                let uv = Uv2::interpolate(uv0, uv1, xc);

                xc += x_delta;
            }

            yc += y_delta;
        }
    }
}
