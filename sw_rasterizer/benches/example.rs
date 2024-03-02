use divan::{Bencher, black_box};
use sw_rasterizer::{cat_triangles, Vertex};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

fn main() {
    // Run registered benchmarks.
    divan::main();
}

#[divan::bench(args = [10_000, 20_000, 80_000, 100_000, 200_000, 400_000])]
fn prof_cat_triangles(bencher: Bencher, n: u64) {
    let mut rng = StdRng::seed_from_u64(n as _);

    let mut out = vec![0u32; n as usize];
    let mut vertices = vec![Vertex::default(); (n * 10) as usize];
    let mut indices = vec![0u32; n as usize];

    let mut index = 0usize;
    let mut vertex_index = 0;
    let mut vert = 0;

    while index < (n - 6) as usize {
        let gen_quad: bool = rng.gen_bool(0.5);
        let same_color: bool = rng.gen_bool(0.5);
        let white_uv: bool = rng.gen_bool(0.5);

        for i in 0..6 {
            vertices[vert + i].pos.0 = rng.gen_range(0.0..1.0);
            vertices[vert + i].pos.1 = rng.gen_range(0.0..1.0);

            if same_color {
                vertices[vert + i].color = 0; 
            } else {
                vertices[vert + i].color = rng.gen_range(0..255);
            }

            if white_uv {
                vertices[vert + i].uv.0 = 1;
                vertices[vert + i].uv.1 = 1;
            } else {
                vertices[vert + i].uv.0 = rng.gen_range(0..16);
                vertices[vert + i].uv.1 = rng.gen_range(0..16);
            }
        }

        if gen_quad { 
            indices[index + 0] = vertex_index as u32;
            indices[index + 1] = (vertex_index + 1) as u32;
            indices[index + 2] = (vertex_index + 2) as u32;
            indices[index + 3] = (vertex_index + 2) as u32;
            indices[index + 4] = (vertex_index + 3) as u32;
            indices[index + 5] = vertex_index as u32;

            vertex_index += 4;
            index += 6;

        } else {
            indices[index + 0] = vertex_index as u32;
            indices[index + 1] = (vertex_index + 1) as u32;
            indices[index + 2] = (vertex_index + 2) as u32;

            vertex_index += 4;
            index += 3;
        }

        vert += 6;
    }

    bencher.bench_local(move || {
        unsafe {
            cat_triangles(black_box(&mut out), black_box(&vertices), black_box(&indices));
        }
    });
}
