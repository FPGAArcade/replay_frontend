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

    pub fn render(&mut self, dest: &mut [u32], width: usize, height: usize, primitives: &[Primitive]) {
        let mut color_index = 0;

        println!("Rendering {} primitives", primitives.len());

        for prim in primitives {
            let min_x = prim.rect.min[0] as usize;
            let min_y = prim.rect.min[1] as usize;
            let max_x = prim.rect.max[0] as usize;
            let max_y = prim.rect.max[1] as usize;

            dbg!(min_x, min_y, max_x, max_y);

            let max_x = max_x.min(width);
            let max_y = max_y.min(height);
            let min_x = min_x.max(0);
            let min_y = min_y.max(0);
            let color = COLORS[color_index & 0xf]; 

            dbg!(color);

            for y in min_y..max_y {
                for x in min_x..max_x {
                    dest[y * width + x] = color;
                }
            }

            color_index += 1;
        }
    }
}
