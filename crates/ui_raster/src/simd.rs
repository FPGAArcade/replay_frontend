#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

//#[cfg(target_arch = "aarch64")]
//use std::arch::asm;

#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

use core::ops::{Add, AddAssign, Div, Mul, Sub};

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
pub struct f32x4 {
    #[cfg(target_arch = "aarch64")]
    pub v: float32x4_t,
    #[cfg(target_arch = "x86_64")]
    pub v: __m128,
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
pub struct i32x4 {
    #[cfg(target_arch = "aarch64")]
    pub v: int32x4_t,
    #[cfg(target_arch = "x86_64")]
    pub v: __m128i,
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
pub struct i16x8 {
    #[cfg(target_arch = "aarch64")]
    pub v: int16x8_t,
    #[cfg(target_arch = "x86_64")]
    pub v: __m128i,
}

/*
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug)]
struct f16x8 {
    #[cfg(target_arch = "aarch64")]
    v: int16x8_t,
    #[cfg(target_arch = "x86_64")]
    v0: f32x4,
    #[cfg(target_arch = "x86_64")]
    v1: f32x4,
}
*/

impl f32x4 {
    #[cfg(target_arch = "aarch64")]
    pub fn load_unaligned(data: &[f32]) -> Self {
        Self {
            v: unsafe { vld1q_f32(data.as_ptr()) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn load_unaligned(data: &[f32]) -> Self {
        Self {
            v: unsafe { _mm_loadu_ps(data.as_ptr()) },
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn store_unaligned(self, data: &mut [f32]) {
        unsafe {
            vst1q_f32(data.as_mut_ptr(), self.v);
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn store_unaligned(self, data: &mut [f32]) {
        unsafe {
            _mm_storeu_ps(data.as_mut_ptr(), self.v);
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn new_splat(a: f32) -> Self {
        Self {
            v: unsafe { vdupq_n_f32(a) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn new_splat(a: f32) -> Self {
        Self {
            v: unsafe { _mm_set1_ps(a) },
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn drop_fraction(self) -> Self {
        Self {
            v: unsafe { vcvtq_f32_s32(vcvtq_s32_f32(self.v)) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn drop_fraction(self) -> Self {
        Self {
            v: unsafe { _mm_cvtepi32_ps(_mm_cvttps_epi32(self.v)) },
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn as_i32x4(self) -> i32x4 {
        i32x4 {
            v: unsafe { vcvtq_s32_f32(self.v) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn as_i32x4(self) -> i32x4 {
        i32x4 {
            v: unsafe { _mm_cvttps_epi32(self.v) },
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn extract<const LANE: i32>(self) -> f32 {
        unsafe { vgetq_lane_f32(self.v, LANE) }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn extract<const LANE: i32>(self) -> f32 {
        unsafe { f32::from_bits(_mm_extract_ps(self.v, LANE) as _) }
    }

    pub fn new_xy(a: f32, b: f32) -> Self {
        Self::new(a, b, a, b)
    }

    #[cfg(target_arch = "aarch64")]
    pub fn new(a: f32, b: f32, c: f32, d: f32) -> Self {
        let t = [a, b, c, d];
        Self {
            v: unsafe { vld1q_f32(t.as_ptr()) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn new(a: f32, b: f32, c: f32, d: f32) -> Self {
        Self {
            v: unsafe { _mm_set_ps(d, c, b, a) },
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn add(self, rhs: Self) -> Self {
        Self {
            v: unsafe { vaddq_f32(self.v, rhs.v) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn add(self, rhs: Self) -> Self {
        Self {
            v: unsafe { _mm_add_ps(self.v, rhs.v) },
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn sub(self, rhs: Self) -> Self {
        Self {
            v: unsafe { vsubq_f32(self.v, rhs.v) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn sub(self, rhs: Self) -> Self {
        Self {
            v: unsafe { _mm_sub_ps(self.v, rhs.v) },
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn mul(self, rhs: Self) -> Self {
        Self {
            v: unsafe { vmulq_f32(self.v, rhs.v) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn mul(self, rhs: Self) -> Self {
        Self {
            v: unsafe { _mm_mul_ps(self.v, rhs.v) },
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn floor(self) -> Self {
        Self {
            v: unsafe { vrndmq_f32(self.v) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn floor(self) -> Self {
        Self {
            v: unsafe { _mm_floor_ps(self.v) },
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn sqrt(self) -> Self {
        Self {
            v: unsafe { vsqrtq_f32(self.v) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn sqrt(self) -> Self {
        Self {
            v: unsafe { _mm_sqrt_ps(self.v) },
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn abs(self) -> Self {
        Self {
            v: unsafe { vabsq_f32(self.v) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn abs(self) -> Self {
        Self {
            v: unsafe { _mm_and_ps(self.v, _mm_castsi128_ps(_mm_set1_epi32(0x7fffffff))) },
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self {
            v: unsafe { vmaxq_f32(min.v, vminq_f32(self.v, max.v)) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn clamp(self, min: Self, max: Self) -> Self {
        Self {
            v: unsafe { _mm_min_ps(_mm_max_ps(self.v, min.v), max.v) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    // Taken from https://gist.github.com/mtsamis/441c16f3d6fc86566eaa2a302ed247c9
    pub fn test_intersect(a: f32x4, b: f32x4) -> bool {
        unsafe {
            // make max.x, max.y negative
            let flip_sign_0 = _mm_set_ps(0.0, 0.0, -0.0, -0.0);
            let a = _mm_xor_ps(a.v, flip_sign_0);
            let b = _mm_xor_ps(b.v, flip_sign_0);

            // we have min_x,  min_y, max_x, max_y and want
            //        max_x, max_y,  min_x,  min_y
            let b = _mm_shuffle_ps(b, b, 78); // [ max_x, max_y, min_x, min_y ]
            let flip_sign = _mm_set1_ps(-0.0);
            let b = _mm_xor_ps(b, flip_sign); // [ min_x, min_y, max_x, max_y ]

            // Check overlap: compare shuffled `a` <= negated `b`
            let cmp = _mm_cmplt_ps(a, b);

            dbg!(cmp);

            // Test if all comparison results are true
            _mm_movemask_ps(cmp) == 0
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn test_intersect(a: f32x4, b: f32x4) -> bool {
        false
    }

    #[cfg(target_arch = "aarch64")]
    pub fn shuffle_0101(self) -> Self {
        Self {
            v: unsafe { vcombine_f32(vget_low_f32(self.v), vget_low_f32(self.v)) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn shuffle_0101(self) -> Self {
        Self {
            v: unsafe { _mm_shuffle_ps(self.v, self.v, 0b01_00_01_00) },
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn shuffle_2323(self) -> Self {
        Self {
            v: unsafe { vcombine_f32(vget_high_f32(self.v), vget_high_f32(self.v)) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn shuffle_2323(self) -> Self {
        Self {
            v: unsafe { _mm_shuffle_ps(self.v, self.v, 0b11_10_11_10) },
        }
    }

    #[cfg(any(test, debug_assertions))]
    pub fn to_array(self) -> [f32; 4] {
        #[cfg(target_arch = "aarch64")]
        unsafe {
            [
                vgetq_lane_f32(self.v, 0),
                vgetq_lane_f32(self.v, 1),
                vgetq_lane_f32(self.v, 2),
                vgetq_lane_f32(self.v, 3),
            ]
        }

        #[cfg(target_arch = "x86_64")]
        unsafe {
            // Use `_mm_storeu_ps` to store the SIMD register into an array.
            let mut arr = [0.0f32; 4];
            _mm_storeu_ps(arr.as_mut_ptr(), self.v);
            arr
        }
    }
}

impl i16x8 {
    #[cfg(target_arch = "aarch64")]
    #[allow(clippy::too_many_arguments)]
    pub fn new(a: i16, b: i16, c: i16, d: i16, e: i16, f: i16, g: i16, h: i16) -> Self {
        let temp = [a, b, c, d, e, f, g, h];
        Self {
            v: unsafe { vld1q_s16(temp.as_ptr()) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[allow(clippy::too_many_arguments)]
    pub fn new(a: i16, b: i16, c: i16, d: i16, e: i16, f: i16, g: i16, h: i16) -> Self {
        Self {
            v: unsafe { _mm_set_epi16(h, g, f, e, d, c, b, a) },
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn new_splat(a: i16) -> Self {
        Self {
            v: unsafe { vdupq_n_s16(a) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn new_splat(a: i16) -> Self {
        Self {
            v: unsafe { _mm_set1_epi16(a) },
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn load_unaligned(data: &[i16]) -> Self {
        Self {
            v: unsafe { vld1q_s16(data.as_ptr()) },
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn store_unaligned_ptr(self, data: *mut i16) {
        unsafe {
            vst1q_s16(data, self.v);
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn store_unaligned_ptr(self, data: *mut i16) {
        unsafe {
            _mm_storeu_si128(data as *mut __m128i, self.v);
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn store_unaligned_ptr_lower(self, data: *mut i16) {
        unsafe {
            vst1_s16(data, vget_low_s16(self.v));
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn store_unaligned_ptr_lower(self, data: *mut i16) {
        unsafe {
            _mm_storeu_si64(data as *mut u8, self.v);
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn load_unaligned(data: &[i16]) -> Self {
        Self {
            v: unsafe { _mm_loadu_si128(data.as_ptr() as *const __m128i) },
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn store_unaligned(self, data: &mut [i16]) {
        unsafe {
            vst1q_s16(data.as_mut_ptr(), self.v);
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn store_unaligned(self, data: &mut [i16]) {
        unsafe {
            _mm_storeu_si128(data.as_mut_ptr() as *mut __m128i, self.v);
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn load_unaligned_ptr(data: *const i16) -> Self {
        Self {
            v: unsafe { vld1q_s16(data) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn load_unaligned_ptr(data: *const i16) -> Self {
        Self {
            v: unsafe { _mm_loadu_si128(data as *const __m128i) },
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn lerp_diff(start: Self, delta: Self, t: i16x8) -> Self {
        Self {
            v: unsafe { vqrdmlahq_s16(start.v, delta.v, t.v) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn lerp_diff(start: Self, delta: Self, t: i16x8) -> Self {
        Self {
            v: unsafe { _mm_add_epi16(_mm_mulhrs_epi16(delta.v, t.v), start.v) },
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn lerp(start: Self, end: Self, t: i16x8) -> Self {
        let delta = unsafe { vsubq_s16(end.v, start.v) };
        Self {
            v: unsafe { vqrdmlahq_s16(start.v, delta, t.v) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn lerp(start: Self, end: Self, t: i16x8) -> Self {
        Self {
            v: unsafe {
                _mm_add_epi16(
                    _mm_mulhrs_epi16(_mm_sub_epi16(end.v, start.v), t.v),
                    start.v,
                )
            },
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn mul_high(a: Self, b: Self) -> Self {
        Self {
            v: unsafe { vqrdmulhq_s16(a.v, b.v) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn mul_high(a: Self, b: Self) -> Self {
        Self {
            v: unsafe { _mm_mulhrs_epi16(a.v, b.v) },
        }
    }

    #[cfg(target_arch = "aarch64")]
    fn splat<const LANE: i32>(self) -> Self {
        Self {
            v: unsafe { vdupq_laneq_s16(self.v, LANE) },
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn splat_0000_0000(self) -> Self {
        self.splat::<0>()
    }

    #[cfg(target_arch = "aarch64")]
    pub fn splat_1111_1111(self) -> Self {
        self.splat::<1>()
    }

    #[cfg(target_arch = "aarch64")]
    pub fn splat_2222_2222(self) -> Self {
        self.splat::<2>()
    }

    #[cfg(target_arch = "x86_64")]
    pub fn splat_0000_0000(self) -> Self {
        unsafe {
            let lower = _mm_shufflelo_epi16(self.v, 0b00_00_00_00);
            Self {
                v: _mm_shuffle_epi32(lower, 0b00_00_00_00),
            }
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn splat_1111_1111(self) -> Self {
        unsafe {
            let lower = _mm_shufflelo_epi16(self.v, 0b01_01_01_01);
            Self {
                v: _mm_shuffle_epi32(lower, 0b00_00_00_00),
            }
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn splat_2222_2222(self) -> Self {
        unsafe {
            let lower = _mm_shufflelo_epi16(self.v, 0b10_10_10_10);
            Self {
                v: _mm_shuffle_epi32(lower, 0b00_00_00_00),
            }
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn rotate_4(self) -> Self {
        Self {
            v: unsafe { vextq_s16(self.v, self.v, 4) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn rotate_4(self) -> Self {
        Self {
            v: unsafe { _mm_shuffle_epi32(self.v, 0b01_00_11_10) },
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn shuffle_0123_0123(self) -> Self {
        Self {
            v: unsafe { vcombine_s16(vget_low_s16(self.v), vget_low_s16(self.v)) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn shuffle_0123_0123(self) -> Self {
        Self {
            v: unsafe { _mm_shuffle_epi32(self.v, 0b01_00_01_00) },
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn shuffle_4567_4567(self) -> Self {
        Self {
            v: unsafe { vcombine_s16(vget_high_s16(self.v), vget_high_s16(self.v)) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn shuffle_4567_4567(self) -> Self {
        Self {
            v: unsafe { _mm_shuffle_epi32(self.v, 0b11_10_11_10) },
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn shuffle_333_0x7fff_777_0x7fff(self) -> Self {
        let splat_7fff = i16x8::new_splat(0x7fff);
        let data = [
            6, 7, 6, 7, 6, 7, 16, 17, 
            14, 15, 14, 15, 14, 15, 16, 17,
        ];

        Self::tablebased_shuffle(self, splat_7fff, data)
    }

    #[cfg(target_arch = "x86_64")]
    pub fn shuffle_333_0x7fff_777_0x7fff(self) -> Self {
        unsafe {
            let splat_7fff = i16x8::new_splat(0x7fff);

            // Blend `xmm0` (self) with `xmm1` (splat_7fff) using a mask of 3 (low two words).
            let blended = _mm_blend_epi16(self.v, splat_7fff.v, 0b00000011);

            // Define the shuffle mask equivalent to .LCPI0_0.
            let shuffle_mask = _mm_set_epi8(
                1, 0, 15, 14, 15, 14, 15, 14, // Replicate A1 and insert splat
                1, 0, 7, 6, 7, 6, 7, 6, // Replicate A0 and insert splat
            );

            // Shuffle the blended vector with the shuffle mask.
            let result = _mm_shuffle_epi8(blended, shuffle_mask);

            Self { v: result }
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn shuffle_1111_3333(self) -> Self {
        let data = [2, 3, 2, 3, 2, 3, 2, 3, 6, 7, 6, 7, 6, 7, 6, 7];

        Self::tablebased_shuffle(self, self, data)
    }

    #[cfg(target_arch = "x86_64")]
    pub fn shuffle_1111_3333(self) -> Self {
        unsafe {
            // pshuflw xmm0, xmm0, 245
            let temp = _mm_shufflelo_epi16(self.v, 245);
            let v = _mm_shuffle_epi32(temp, 80);
            Self { v }
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn shuffle_3333_7777(self) -> Self {
        let data = [6, 7, 6, 7, 6, 7, 6, 7, 14, 15, 14, 15, 14, 15, 14, 15];

        Self::tablebased_shuffle(self, self, data)
    }

    #[cfg(target_arch = "x86_64")]
    pub fn shuffle_3333_7777(self) -> Self {
        unsafe {
            Self {
                v: _mm_shufflehi_epi16(_mm_shufflelo_epi16(self.v, 0xFF), 0xFF),
            }
        }
    }

    #[cfg(target_arch = "aarch64")]
    #[inline(always)]
    fn tablebased_shuffle(self, v1: Self, data: [u8; 16]) -> Self {
        unsafe {
            // Define the shuffle mask as a NEON vector.
            let mask = vld1q_u8(data.as_ptr());

            // Combine the vectors into a table for the lookup.
            let table = uint8x16x2_t {
                0: vreinterpretq_u8_s16(self.v),
                1: vreinterpretq_u8_s16(v1.v),
            };

            // Perform the table lookup.
            let result = vqtbl2q_u8(table, mask);

            // Reinterpret the result as an `int16x8_t` and return it.
            Self {
                v: vreinterpretq_s16_u8(result),
            }
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn shuffle_0000_2222(self) -> Self {
        let data = [
            0, 1, 0, 1, 0, 1, 0, 1, // Duplicate 0th element
            4, 5, 4, 5, 4, 5, 4, 5, // Duplicate 2nd element
        ];

        Self::tablebased_shuffle(self, self, data)
    }

    #[cfg(target_arch = "x86_64")]
    pub fn shuffle_0000_2222(self) -> Self {
        unsafe {
            let temp = _mm_shufflelo_epi16(self.v, 160);
            let v = _mm_shuffle_epi32(temp, 80);
            Self { v }
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn shuffle_4444_6666(self) -> Self {
        let data = [
            8, 9, 8, 9, 8, 9, 8, 9, // Duplicate 4th element
            12, 13, 12, 13, 12, 13, 12, 13, // Duplicate 6th element
        ];

        Self::tablebased_shuffle(self, self, data)
    }

    #[cfg(target_arch = "x86_64")]
    pub fn shuffle_4444_6666(self) -> Self {
        unsafe {
            let temp = _mm_shufflehi_epi16(self.v, 160);
            let v = _mm_shuffle_epi32(temp, 250);
            Self { v }
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn shuffle_5555_7777(self) -> Self {
        let data = [
            10, 11, 10, 11, 10, 11, 10, 11, // Duplicate 5th element
            14, 15, 14, 15, 14, 15, 14, 15, // Duplicate 7th element
        ];

        Self::tablebased_shuffle(self, self, data)
    }

    #[cfg(target_arch = "x86_64")]
    pub fn shuffle_5555_7777(self) -> Self {
        unsafe {
            let temp = _mm_shufflehi_epi16(self.v, 245);
            let v = _mm_shuffle_epi32(temp, 250);
            Self { v }
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn merge(v0: Self, v1: Self) -> Self {
        Self {
            v: unsafe { vcombine_s16(vget_low_s16(v0.v), vget_low_s16(v1.v)) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn merge(v0: Self, v1: Self) -> Self {
        Self {
            v: unsafe {
                _mm_castps_si128(_mm_movelh_ps(
                    _mm_castsi128_ps(v0.v),
                    _mm_castsi128_ps(v1.v),
                ))
            },
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn pack_bytes(self) -> Self {
        let zero = i16x8::new_splat(0);
        unsafe {
            let narrow_v = vqmovn_s16(self.v);
            let narrow_zero = vqmovn_s16(zero.v);
            Self { v: vreinterpretq_s16_s8(vcombine_s8(narrow_v, narrow_zero)) }
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn pack_bytes(self) -> Self {
        let zero = i16x8::new_splat(0);
        i16x8 {
            v: unsafe { _mm_packs_epi16(self.v, zero.v) },
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn shift_right<const LANE: i32>(self) -> Self {
        Self {
            v: unsafe { vshrq_n_s16(self.v, LANE) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn shift_right<const LANE: i32>(self) -> Self {
        Self {
            v: unsafe { _mm_srai_epi16(self.v, LANE) },
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn extract<const LANE: i32>(self) -> i16 {
        unsafe { vgetq_lane_s16(self.v, LANE) }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn extract<const LANE: i32>(self) -> i32 {
        unsafe { _mm_extract_epi16(self.v, LANE) }
    }

    #[cfg(any(test, debug_assertions))]
    pub fn to_array(self) -> [i16; 8] {
        #[cfg(target_arch = "aarch64")]
        unsafe {
            [
                vgetq_lane_s16(self.v, 0),
                vgetq_lane_s16(self.v, 1),
                vgetq_lane_s16(self.v, 2),
                vgetq_lane_s16(self.v, 3),
                vgetq_lane_s16(self.v, 4),
                vgetq_lane_s16(self.v, 5),
                vgetq_lane_s16(self.v, 6),
                vgetq_lane_s16(self.v, 7),
            ]
        }

        #[cfg(target_arch = "x86_64")]
        unsafe {
            let mut arr = [0; 8];
            _mm_storeu_si128(arr.as_mut_ptr() as *mut __m128i, self.v);
            arr
        }
    }

    #[cfg(any(test, debug_assertions))]
    pub fn to_array_u8(self) -> [u8; 16] {
        #[cfg(target_arch = "aarch64")]
        unsafe {
            let mut arr = [0; 16];
            let v = vreinterpretq_u8_s16(self.v);
            vst1q_u8(arr.as_mut_ptr(), v);
            arr
        }

        #[cfg(target_arch = "x86_64")]
        unsafe {
            let mut arr = [0; 16];
            _mm_storeu_si128(arr.as_mut_ptr() as *mut __m128i, self.v);
            arr
        }
    }
}

impl i32x4 {
    #[cfg(target_arch = "aarch64")]
    pub fn new(a: i32, b: i32, c: i32, d: i32) -> Self {
        let t = [a, b, c, d];
        Self {
            v: unsafe { vld1q_s32(t.as_ptr()) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn new(a: i32, b: i32, c: i32, d: i32) -> Self {
        Self {
            v: unsafe { _mm_set_epi32(d, c, b, a) },
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn load_unaligned(data: &[i32]) -> Self {
        Self {
            v: unsafe { vld1q_s32(data.as_ptr()) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn load_unaligned(data: &[i32]) -> Self {
        Self {
            v: unsafe { _mm_loadu_si128(data.as_ptr() as *const __m128i) },
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn new_splat(a: i32) -> Self {
        Self {
            v: unsafe { vdupq_n_s32(a) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn new_splat(a: i32) -> Self {
        Self {
            v: unsafe { _mm_set1_epi32(a) },
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn shuffle_xyxy(self) -> Self {
        unsafe {
            // Extract the lower two elements (x1, y1) from the vector
            let low = vget_low_s32(self.v);
            // Zip the lower part with itself to create (x1, y1, x1, y1)
            let shuffled = vcombine_s32(low, low);
            Self { v: shuffled }
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn shuffle_xyxy(self) -> Self {
        Self {
            v: unsafe { _mm_shuffle_epi32(self.v, 0b01_00_01_00) },
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn shuffle_zwzw(self) -> Self {
        unsafe {
            let high = vget_high_s32(self.v);
            let shuffled = vcombine_s32(high, high);
            Self { v: shuffled }
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn shuffle_zwzw(self) -> Self {
        Self {
            v: unsafe { _mm_shuffle_epi32(self.v, 0b11_10_11_10) },
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn min(self, rhs: Self) -> Self {
        Self {
            v: unsafe { vminq_s32(self.v, rhs.v) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn min(self, rhs: Self) -> Self {
        Self {
            v: unsafe { _mm_min_epi32(self.v, rhs.v) },
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn max(self, rhs: Self) -> Self {
        Self {
            v: unsafe { vmaxq_s32(self.v, rhs.v) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn max(self, rhs: Self) -> Self {
        Self {
            v: unsafe { _mm_max_epi32(self.v, rhs.v) },
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn abs(self) -> Self {
        Self {
            v: unsafe { vabsq_s32(self.v) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn abs(self) -> Self {
        Self {
            v: unsafe { _mm_abs_epi32(self.v) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    // Taken from https://gist.github.com/mtsamis/441c16f3d6fc86566eaa2a302ed247c9
    pub fn test_intersect(a: i32x4, b: i32x4) -> bool {
        unsafe {
            // make max.x, max.y negative
            let flip_sign_0 = _mm_set_epi32(0, 0, 0x7fff_ffffu32 as i32, 0x7fff_ffffu32 as i32);
            let a = _mm_xor_si128(a.v, flip_sign_0);
            let b = _mm_xor_si128(b.v, flip_sign_0);

            // we have min_x,  min_y, -max_x, -max_y and want
            //        -max_x, -max_y,  min_x,  min_y
            let b = _mm_shuffle_epi32(b, 78); // [ -max_x, -max_y, min_x, min_y ]
            let flip_sign = _mm_set1_epi32(0x7fff_ffffu32 as i32);
            let b = _mm_xor_si128(b, flip_sign); // [ -min_x, -min_y, max_x, max_y ]

            // Check overlap: compare shuffled `a` <= negated `b`
            let cmp = _mm_cmplt_epi32(a, b);

            // Test if all comparison results are true
            _mm_movemask_epi8(cmp) == 0
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn test_intersect(a: i32x4, b: i32x4) -> bool {
        unsafe {
            let mask = vld1q_u32(&[0x00000000u32, 0x00000000, 0x8000_0000u32, 0x8000_000fu32] as *const u32);

            let m = i32x4 { v: vreinterpretq_s32_u32(mask) };
            let a = i32x4 { v: vreinterpretq_s32_u32(veorq_u32(vreinterpretq_u32_s32(a.v), mask)) };
            let b = i32x4 { v: vreinterpretq_s32_u32(veorq_u32(vreinterpretq_u32_s32(b.v), mask)) };

            let b = vrev64q_s32(b.v);
            let flip_sign = vreinterpretq_s32_f32(vdupq_n_f32(-0.0));
            let b = veorq_s32(b, flip_sign);

            let cmp = vcltq_s32(a.v, b);

            vmaxvq_u32(cmp) == 0
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn extract<const LANE: i32>(self) -> i32 {
        unsafe { vgetq_lane_s32(self.v, LANE) }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn extract<const LANE: i32>(self) -> i32 {
        unsafe { _mm_extract_epi32(self.v, LANE) }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn as_i16x8(self) -> i16x8 {
        i16x8 {
            v: unsafe { vreinterpretq_s16_s32(self.v) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn as_i16x8(self) -> i16x8 {
        i16x8 { v: self.v }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn as_f32x4(self) -> f32x4 {
        f32x4 {
            v: unsafe { vcvtq_f32_s32(self.v) },
        }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn as_f32x4(self) -> f32x4 {
        f32x4 {
            v: unsafe { _mm_cvtepi32_ps(self.v) },
        }
    }

    #[cfg(any(test, debug_assertions))]
    pub fn to_array(self) -> [i32; 4] {
        #[cfg(target_arch = "aarch64")]
        unsafe {
            [
                vgetq_lane_s32(self.v, 0),
                vgetq_lane_s32(self.v, 1),
                vgetq_lane_s32(self.v, 2),
                vgetq_lane_s32(self.v, 3),
            ]
        }

        #[cfg(target_arch = "x86_64")]
        unsafe {
            let mut arr = [0; 4];
            _mm_storeu_si128(arr.as_mut_ptr() as *mut __m128i, self.v);
            arr
        }
    }
}

/*
impl f16x8 {
    #[cfg(target_arch = "aarch64")]
    #[allow(clippy::too_many_arguments)]
    pub fn new(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32, g: f32, h: f32) -> Self {
        let result: int16x8_t;
        unsafe {
            let t0 = [a, b, c, d];
            let t1 = [e, f, g, h];
            let v0 = vld1q_f32(t0.as_ptr());
            let v1 = vld1q_f32(t1.as_ptr());
            asm!(
                "fcvtn {tmp0}.4h, {v0:v}.4s",       // Convert v0 to f16
                "fcvtn {tmp1}.4h, {v1:v}.4s",       // Convert v1 to f16
                "mov {result:v}.d[0], {tmp0}.d[0]",       // Lower 4 x f16 from v3 into v2
                "mov {result:v}.d[1], {tmp1}.d[0]",       // Upper 4 x f16 from v4 into v2
                v0 = in(vreg) v0,
                v1 = in(vreg) v1,
                tmp0 = out(vreg) _,                 // Temporary register
                tmp1 = out(vreg) _,                 // Temporary register
                result = out(vreg) result,
            );
        }

        Self { v: result }
    }

    #[cfg(target_arch = "x86_64")]
    #[allow(clippy::too_many_arguments)]
    pub fn new(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32, g: f32, h: f32) -> Self {
        Self {
            v0: f32x4::new(a, b, c, d),
            v1: f32x4::new(e, f, g, h),
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn add(self, rhs: Self) -> Self {
        let result: int16x8_t;
        unsafe {
            asm!(
                "fadd {result:v}.8h, {v0:v}.8h, {v1:v}.8h",
                v0 = in(vreg) self.v,
                v1 = in(vreg) rhs.v,
                result = out(vreg) result,
            );
        }

        Self { v: result }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn add(self, rhs: Self) -> Self {
        Self {
            v0: self.v0.add(rhs.v0),
            v1: self.v1.add(rhs.v1),
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn sub(self, rhs: Self) -> Self {
        let result: int16x8_t;
        unsafe {
            asm!(
                "fsub {result:v}.8h, {v0:v}.8h, {v1:v}.8h",
                v0 = in(vreg) self.v,
                v1 = in(vreg) rhs.v,
                result = out(vreg) result,
            );
        }

        Self { v: result }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn sub(self, rhs: Self) -> Self {
        Self {
            v0: self.v0.sub(rhs.v0),
            v1: self.v1.sub(rhs.v1),
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn mul(self, rhs: Self) -> Self {
        let result: int16x8_t;
        unsafe {
            asm!(
                "fmul {result:v}.8h, {v0:v}.8h, {v1:v}.8h",
                v0 = in(vreg) self.v,
                v1 = in(vreg) rhs.v,
                result = out(vreg) result,
            );
        }

        Self { v: result }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn mul(self, rhs: Self) -> Self {
        Self {
            v0: self.v0.mul(rhs.v0),
            v1: self.v1.mul(rhs.v1),
        }
    }

    #[cfg(target_arch = "aarch64")]
    pub fn as_i16x8(self) -> i16x8 {
        let result: int16x8_t;
        unsafe {
            asm!(
                "fcvtzs {result:v}.8h, {value:v}.8h",
                value = in(vreg) self.v,
                result = out(vreg) result,
            );
        }

        i16x8 { v: result }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn as_i16x8(self) -> i16x8 {
        unsafe {
            let int_v1 = self.v0.as_i32x4();
            let int_v2 = self.v1.as_i32x4();

            i16x8 {
                v: _mm_packs_epi16(int_v1.v, int_v2.v),
            }
        }
    }

    #[cfg(target_arch = "aarch64")]
    #[inline(never)]
    pub fn sqrt(self) -> Self {
        let result: int16x8_t;
        unsafe {
            asm!(
                "fsqrt {result:v}.8h, {value:v}.8h",
                value = in(vreg) self.v,
                result = out(vreg) result,
            );
        }

        Self { v: result }
    }

    #[cfg(target_arch = "x86_64")]
    pub fn sqrt(self) -> Self {
        Self {
            v0: self.v0.sqrt(),
            v1: self.v1.sqrt(),
        }
    }

    #[cfg(any(test, debug_assertions))]
    pub fn to_array(self) -> [f32; 8] {
        let mut result: [f32; 8] = [0.0; 8];
        #[cfg(target_arch = "aarch64")]
        unsafe {
            let f32_t0: float32x4_t;
            let f32_t1: float32x4_t;

            asm!(
                "fcvtl {f32_t0:v}.4s, {input:v}.4h",
                "fcvtl2 {f32_t1:v}.4s, {input:v}.8h",
                input = in(vreg) self.v,
                f32_t0 = out(vreg) f32_t0,
                f32_t1 = out(vreg) f32_t1,
            );

            vst1q_f32(result[0..4].as_mut_ptr(), f32_t0);
            vst1q_f32(result[4..8].as_mut_ptr(), f32_t1);
        }

        #[cfg(target_arch = "x86_64")]
        {
            result[0..4].copy_from_slice(&self.v0.to_array());
            result[4..8].copy_from_slice(&self.v1.to_array());
        }

        result
    }
}
*/

impl Sub for f32x4 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        self.sub(rhs)
    }
}

impl Add for f32x4 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        self.add(rhs)
    }
}

impl Mul for f32x4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        self.mul(rhs)
    }
}

impl Div for f32x4 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        Self {
            #[cfg(target_arch = "aarch64")]
            v: unsafe { vdivq_f32(self.v, rhs.v) },
            #[cfg(target_arch = "x86_64")]
            v: unsafe { _mm_div_ps(self.v, rhs.v) },
        }
    }
}

impl AddAssign for f32x4 {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.add(rhs);
    }
}

impl Add for i16x8 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            #[cfg(target_arch = "aarch64")]
            v: unsafe { vaddq_s16(self.v, rhs.v) },
            #[cfg(target_arch = "x86_64")]
            v: unsafe { _mm_add_epi16(self.v, rhs.v) },
        }
    }
}

impl Sub for i16x8 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            #[cfg(target_arch = "aarch64")]
            v: unsafe { vsubq_s16(self.v, rhs.v) },
            #[cfg(target_arch = "x86_64")]
            v: unsafe { _mm_sub_epi16(self.v, rhs.v) },
        }
    }
}

impl Mul for i16x8 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Self {
            #[cfg(target_arch = "aarch64")]
            v: unsafe { vmulq_s16(self.v, rhs.v) },
            #[cfg(target_arch = "x86_64")]
            v: unsafe { _mm_mullo_epi16(self.v, rhs.v) },
        }
    }
}

impl AddAssign for i16x8 {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.add(rhs);
    }
}

impl Sub for i32x4 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self {
            #[cfg(target_arch = "aarch64")]
            v: unsafe { vsubq_s32(self.v, rhs.v) },
            #[cfg(target_arch = "x86_64")]
            v: unsafe { _mm_sub_epi32(self.v, rhs.v) },
        }
    }
}

/*
#[cfg(test)]
mod f32x4_tests {
    use super::*;

    #[test]
    fn test_load_unaligned() {
        let data = [1.0, 2.0, 3.0, 4.0];
        let vec = f32x4::load_unaligned(&data);
        assert_eq!(vec.to_array(), [1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_new_splat() {
        let vec = f32x4::new_splat(5.0);
        assert_eq!(vec.to_array(), [5.0, 5.0, 5.0, 5.0]);
    }

    #[test]
    fn test_new_xy() {
        let vec = f32x4::new_xy(1.0, 2.0);
        assert_eq!(vec.to_array(), [1.0, 2.0, 1.0, 2.0]);
    }

    #[test]
    fn test_new() {
        let vec = f32x4::new(1.0, 2.0, 3.0, 4.0);
        assert_eq!(vec.to_array(), [1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_add() {
        let a = f32x4::new(1.0, 2.0, 3.0, 4.0);
        let b = f32x4::new(4.0, 3.0, 2.0, 1.0);
        let result = a.add(b);
        assert_eq!(result.to_array(), [5.0, 5.0, 5.0, 5.0]);
    }

    #[test]
    fn test_sub() {
        let a = f32x4::new(5.0, 7.0, 9.0, 11.0);
        let b = f32x4::new(1.0, 2.0, 3.0, 4.0);
        let result = a.sub(b);
        assert_eq!(result.to_array(), [4.0, 5.0, 6.0, 7.0]);
    }

    #[test]
    fn test_mul() {
        let a = f32x4::new(1.0, 2.0, 3.0, 4.0);
        let b = f32x4::new(2.0, 3.0, 4.0, 5.0);
        let result = a.mul(b);
        assert_eq!(result.to_array(), [2.0, 6.0, 12.0, 20.0]);
    }

    #[test]
    fn test_drop_fraction() {
        let a = f32x4::new(1.9, 2.1, 3.7, 4.3);
        let result = a.drop_fraction();
        assert_eq!(result.to_array(), [1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_as_i32x4() {
        let a = f32x4::new(1.9, 2.1, 3.7, 4.3);
        let result = a.as_i32x4();
        assert_eq!(result.to_array(), [1, 2, 3, 4]);
    }

    #[test]
    fn test_floor() {
        let a = f32x4::new(1.9, 2.1, 3.7, 4.3);
        let result = a.floor();
        assert_eq!(result.to_array(), [1.0, 2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_sqrt() {
        let a = f32x4::new(1.0, 4.0, 9.0, 16.0);
        let result = a.sqrt();
        assert_eq!(result.to_array(), [1.0, 2.0, 3.0, 4.0]);
    }
}

#[cfg(test)]
mod i32x4_tests {
    use super::*;

    #[test]
    fn test_new_splat() {
        let vec = i32x4::new_splat(42);
        assert_eq!(vec.to_array(), [42, 42, 42, 42]);
    }

    #[test]
    fn test_shuffle_xyxy() {
        let vec = i32x4::new(1, 2, 3, 4);
        let shuffled = vec.shuffle_xyxy();
        assert_eq!(shuffled.to_array(), [1, 2, 1, 2]);
    }

    #[test]
    fn test_shuffle_zwzw() {
        let vec = i32x4::new(1, 2, 3, 4);
        let shuffled = vec.shuffle_zwzw();
        assert_eq!(shuffled.to_array(), [3, 4, 3, 4]);
    }

    #[test]
    fn test_min() {
        let a = i32x4::new(1, 4, 3, 8);
        let b = i32x4::new(2, 3, 4, 7);
        let result = a.min(b);
        assert_eq!(result.to_array(), [1, 3, 3, 7]);
    }

    #[test]
    fn test_max() {
        let a = i32x4::new(1, 4, 3, 8);
        let b = i32x4::new(2, 3, 4, 7);
        let result = a.max(b);
        assert_eq!(result.to_array(), [2, 4, 4, 8]);
    }

    #[test]
    fn test_extract() {
        let vec = i32x4::new(10, 20, 30, 40);
        assert_eq!(vec.extract::<0>(), 10);
        assert_eq!(vec.extract::<1>(), 20);
        assert_eq!(vec.extract::<2>(), 30);
        assert_eq!(vec.extract::<3>(), 40);
    }

    #[test]
    fn test_to_array() {
        let vec = i32x4::new(5, 10, 15, 20);
        assert_eq!(vec.to_array(), [5, 10, 15, 20]);
    }
}

#[cfg(test)]
mod i16x8_tests {
    use super::*;

    #[test]
    fn test_new() {
        let vec = i16x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        assert_eq!(vec.to_array(), [1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn test_load_unaligned() {
        let data = [1, 2, 3, 4, 5, 6, 7, 8];
        let vec = i16x8::load_unaligned(&data);
        assert_eq!(vec.to_array(), data);
    }

    #[test]
    fn test_store_unaligned() {
        let vec = i16x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let mut data = [0; 8];
        vec.store_unaligned(&mut data);
        assert_eq!(data, [1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn test_lerp() {
        // Start and end vectors
        let start = i16x8::new(10, 20, 30, 40, 50, 60, 70, 80);
        let end = i16x8::new(20, 40, 60, 80, 100, 120, 140, 160);

        // `t` values (fixed-point representation in range [0, 0x7FFF])
        // Equivalent to 0.5 in fixed-point
        let t = i16x8::new(
            0x4000, 0x4000, 0x4000, 0x4000, 0x4000, 0x4000, 0x4000, 0x4000,
        );

        // Expected result for t = 0.5
        // Lerp formula: result = start + (end - start) * t
        let expected = i16x8::new(15, 30, 45, 60, 75, 90, 105, 120);

        // Perform lerp and compare
        let result = i16x8::lerp(start, end, t);
        assert_eq!(result.to_array(), expected.to_array());
    }

    #[test]
    fn test_lerp_step_fixed_point() {
        // Start vector
        let start = i16x8::new(10, 20, 30, 40, 50, 60, 70, 80);

        // Delta (end - start) precomputed
        //let end = i16x8::new(20, 40, 60, 80, 100, 120, 140, 160);
        let delta = i16x8::new(10, 20, 30, 40, 50, 60, 70, 80); // delta = end - start

        // `t` values (fixed-point representation in range [0, 0x7FFF])
        // Equivalent to 0.5 in fixed-point
        let t = i16x8::new(
            0x4000, 0x4000, 0x4000, 0x4000, 0x4000, 0x4000, 0x4000, 0x4000,
        );

        // Expected result for t = 0.5
        // Lerp step formula: result = start + delta * t
        let expected = i16x8::new(15, 30, 45, 60, 75, 90, 105, 120);

        // Perform lerp step and compare
        let result = i16x8::lerp_diff(start, delta, t);
        assert_eq!(result.to_array(), expected.to_array());
    }
}

#[cfg(test)]
mod simd_tests {
    use super::*;

    /*
    #[test]
    fn test_new() {
        // Test creating an f16x8 register with specific values
        let vec = f16x8::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
        let result = vec.to_array();
        assert_eq!(result, [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
    }

    #[test]
    fn test_add() {
        // Test adding two f16x8 registers
        let vec1 = f16x8::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
        let vec2 = f16x8::new(0.5, 1.5, 2.5, 3.5, 4.5, 5.5, 6.5, 7.5);
        let result = vec1.add(vec2).to_array();
        assert_eq!(result, [1.5, 3.5, 5.5, 7.5, 9.5, 11.5, 13.5, 15.5]);
    }

    #[test]
    fn test_sub() {
        // Test subtracting two f16x8 registers
        let vec1 = f16x8::new(10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0);
        let vec2 = f16x8::new(5.0, 15.0, 25.0, 35.0, 45.0, 55.0, 65.0, 75.0);
        let result = vec1.sub(vec2).to_array();
        assert_eq!(result, [5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0]);
    }

    #[test]
    fn test_mul() {
        // Test multiplying two f16x8 registers
        let vec1 = f16x8::new(2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
        let vec2 = f16x8::new(1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 5.0);
        let result = vec1.mul(vec2).to_array();
        assert_eq!(result, [3.0, 6.0, 10.0, 15.0, 21.0, 28.0, 36.0, 45.0]);
    }

    #[test]
    fn test_as_i16x8() {
        // Test conversion to i16x8 representation
        let vec = f16x8::new(1.2, 2.2, 3.2, 4.2, 5.2, 6.2, 7.2, 8.2);
        let result = vec.as_i16x8().to_array();
        assert_eq!(result, [1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn test_sqrt() {
        // Test square root operation on f16x8
        let vec = f16x8::new(1.0, 4.0, 9.0, 16.0, 25.0, 36.0, 49.0, 64.0);
        let result = vec.sqrt().to_array();
        assert_eq!(result, [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
    }

    #[test]
    fn test_to_array() {
        // Test conversion of f16x8 to [f32; 8]
        let vec = f16x8::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
        let result = vec.to_array();
        assert_eq!(result, [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
    }
    */

    #[test]
    fn test_i16x_splat_0() {
        // Test splatting a specific lane of an i16x8 register
        let vec = i16x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let result = vec.splat_0000_0000().to_array();
        assert_eq!(result, [1, 1, 1, 1, 1, 1, 1, 1]);
    }

    #[test]
    fn test_i16x_splat_1() {
        // Test splatting a specific lane of an i16x8 register
        let vec = i16x8::load_unaligned(&[1, 2, 3, 4, 5, 6, 7, 8]);
        let result = vec.splat_1111_1111().to_array();
        assert_eq!(result, [2, 2, 2, 2, 2, 2, 2, 2]);
    }

    #[test]
    fn test_i16x_splat_2() {
        // Test splatting a specific lane of an i16x8 register
        let vec = i16x8::load_unaligned(&[1, 2, 3, 4, 5, 6, 7, 8]);
        let result = vec.splat_2222_2222().to_array();
        assert_eq!(result, [3, 3, 3, 3, 3, 3, 3, 3]);
    }

    #[test]
    fn test_i16x8_merge() {
        // Test merging two i16x8 registers
        let v0 = i16x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let v1 = i16x8::new(9, 10, 11, 12, 13, 14, 15, 16);
        let result = i16x8::merge(v0, v1).to_array();
        assert_eq!(result, [1, 2, 3, 4, 9, 10, 11, 12]);
    }

    #[test]
    fn test_i16x8_rotate_4() {
        // Test rotating an i16x8 register by 4 lanes
        let vec = i16x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let result = vec.rotate_4().to_array();
        assert_eq!(result, [5, 6, 7, 8, 1, 2, 3, 4]);
    }

    #[test]
    fn test_i16x8_shuffle_0123_0123() {
        // Test shuffling an i16x8 register
        let vec = i16x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let result = vec.shuffle_0123_0123().to_array();
        assert_eq!(result, [1, 2, 3, 4, 1, 2, 3, 4]);
    }

    #[test]
    fn test_i16x8_shuffle_4567_4567() {
        // Test shuffling an i16x8 register
        let vec = i16x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let result = vec.shuffle_4567_4567().to_array();
        assert_eq!(result, [5, 6, 7, 8, 5, 6, 7, 8]);
    }

    #[test]
    fn test_f32x4_shuffle_0101() {
        // Test shuffling an f32x4 register
        let vec = f32x4::new(1.0, 2.0, 3.0, 4.0);
        let result = vec.shuffle_0101().to_array();
        assert_eq!(result, [1.0, 2.0, 1.0, 2.0]);
    }

    #[test]
    fn test_f32x4_shuffle_2323() {
        // Test shuffling an f32x4 register
        let vec = f32x4::new(1.0, 2.0, 3.0, 4.0);
        let result = vec.shuffle_2323().to_array();
        assert_eq!(result, [3.0, 4.0, 3.0, 4.0]);
    }

    #[test]
    fn test_i16x8_shuffle_333_0x7fff_777_0x7fff() {
        // Test shuffling an i16x8 register
        let vec = i16x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let result = vec.shuffle_333_0x7fff_777_0x7fff().to_array();
        assert_eq!(result, [4, 4, 4, 0x7fff, 8, 8, 8, 0x7fff]);
    }

    #[test]
    fn test_i16x8_shuffle_1111_3333() {
        // Test shuffling an i16x8 register
        let vec = i16x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let result = vec.shuffle_1111_3333().to_array();
        assert_eq!(result, [2, 2, 2, 2, 4, 4, 4, 4]);
    }

    #[test]
    fn test_i16x8_shuffle_5555_7777() {
        // Test shuffling an i16x8 register
        let vec = i16x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let result = vec.shuffle_5555_7777().to_array();
        assert_eq!(result, [6, 6, 6, 6, 8, 8, 8, 8]);
    }

    #[test]
    fn test_i16x8_store_lower() {
        // Test storing the lower half of an i16x8 register
        let vec = i16x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let mut data = [0; 8];
        vec.store_unaligned_ptr_lower(data.as_mut_ptr());
        assert_eq!(data, [1, 2, 3, 4, 0, 0, 0, 0]);
    }

    #[test]
    fn test_i32x4_full_overlap() {
        let a = i32x4::new(1, 1, 3, 3);
        let b = i32x4::new(2, 2, 4, 4);
        let bf = a.as_f32x4();
        let bc = b.as_f32x4();

        let t = f32x4::test_intersect(bf, bc);

        dbg!(t);
        dbg!(i32x4::test_intersect(a, b));

        assert!(i32x4::test_intersect(a, b), "AABBs should overlap");
    }

    #[test]
    fn test_i32x4_partial_overlap() {
        let a = i32x4::new(1, 1, 3, 3);
        let b = i32x4::new(2, 2, 4, 4);
        assert!(i32x4::test_intersect(a, b), "AABBs should overlap");
    }

    #[test]
    fn test_i32x4_full_containment() {
        let a = i32x4::new(2, 2, 3, 3);
        let b = i32x4::new(1, 1, 4, 4);
        assert!(i32x4::test_intersect(a, b), "AABBs should overlap");
    }

    #[test]
    fn test_i32x4_edge_touching() {
        let a = i32x4::new(1, 1, 3, 3);
        let b = i32x4::new(3, 3, 5, 5);
        assert!(
            i32x4::test_intersect(a, b),
            "AABBs should overlap (edge touch)"
        );
    }

    #[test]
    fn test_i32x4_corner_touching() {
        let a = i32x4::new(1, 1, 3, 3);
        let b = i32x4::new(3, 3, 4, 4);
        assert!(
            i32x4::test_intersect(a, b),
            "AABBs should overlap (corner touch)"
        );
    }

    #[test]
    fn test_i32x4_no_overlap_separated() {
        let a = i32x4::new(1, 1, 3, 3);
        let b = i32x4::new(4, 4, 5, 5);
        assert!(!i32x4::test_intersect(a, b), "AABBs should not overlap");
    }

    #[test]
    fn test_i32x4_no_overlap_left() {
        let a = i32x4::new(1, 1, 2, 2);
        let b = i32x4::new(3, 1, 4, 2);
        assert!(
            !i32x4::test_intersect(a, b),
            "AABBs should not overlap (AABB to the left)"
        );
    }

    #[test]
    fn test_i32x4_no_overlap_right() {
        let a = i32x4::new(3, 1, 4, 2);
        let b = i32x4::new(1, 1, 2, 2);
        assert!(
            !i32x4::test_intersect(a, b),
            "AABBs should not overlap (AABB to the right)"
        );
    }

    #[test]
    fn test_i32x4_no_overlap_below() {
        let a = i32x4::new(1, 1, 2, 2);
        let b = i32x4::new(1, 3, 2, 4);
        assert!(
            !i32x4::test_intersect(a, b),
            "AABBs should not overlap (AABB below)"
        );
    }

    #[test]
    fn test_i32x4_no_overlap_above() {
        let a = i32x4::new(1, 3, 2, 4);
        let b = i32x4::new(1, 1, 2, 2);
        assert!(
            !i32x4::test_intersect(a, b),
            "AABBs should not overlap (AABB above)"
        );
    }

    #[test]
    fn test_i32x4_no_overlap_top_right() {
        let a = i32x4::new(1, 1, 2, 2);
        let b = i32x4::new(3, 3, 4, 4);
        assert!(
            !i32x4::test_intersect(a, b),
            "AABBs should not overlap (AABB to the top-right)"
        );
    }

    #[test]
    fn test_i32x4_no_overlap_bottom_left() {
        let a = i32x4::new(3, 3, 4, 4);
        let b = i32x4::new(1, 1, 2, 2);
        assert!(
            !i32x4::test_intersect(a, b),
            "AABBs should not overlap (AABB to the bottom-left)"
        );
    }

    #[test]
    fn test_i16x8_shuffle_3333_7777() {
        let vec = i16x8::new(1, 2, 3, 4, 5, 6, 7, 8);
        let result = vec.shuffle_3333_7777().to_array();
        assert_eq!(result, [4, 4, 4, 4, 8, 8, 8, 8]);
    }

    #[test]
    fn test_i16x8_pack_bytes() {
        let a = i16x8::new(0x02, 0x01, 0x04, 0x03, 0x06, 0x05, 0x08, 0x07);

        let result = a.pack_bytes().to_array_u8(); 
        assert_eq!(result, [0x02,0x01,0x04,0x03,0x6,0x5,0x08,0x07, 0,0,0,0,0,0,0,0]);
    }
}
*/
