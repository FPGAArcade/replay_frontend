static float lerp(float a, float b, uniform float t) {
    return (1.0 - t) * a + (b * t);
}

unmasked void rasterize_quad(
        uniform int16 output[], 
        uniform int tile_width, 
        uniform int y0,
        uniform int x0,
        uniform int y1,
        uniform int x1,
        uniform float y_step_min,
        uniform float x_step_min,
        uniform float corner_colors[])
{
    uniform float* rgba_tl = &corner_colors[0];
    uniform float* rgba_tr = &corner_colors[4];
    uniform float* rgba_bl = &corner_colors[8];
    uniform float* rgba_br = &corner_colors[12];

    uniform float rgba_top_left[4];
    uniform float rgba_top_right[4];
    uniform float rgba_bottom_left[4];
    uniform float rgba_bottom_right[4];
    uniform float rgba[4];

    foreach (tl = 0 ... 4) {
        rgba_top_left[tl] = lerp(rgba_tl[tl], rgba_bl[tl], y_step_min);
        rgba_top_right[tl] = lerp(rgba_tr[tl], rgba_br[tl], y_step_min);

        rgba_bottom_left[tl] = lerp(rgba_top_left[tl], rgba_top_right[tl], x_step_min);
        rgba_bottom_right[tl] = lerp(rgba_bl[tl], rgba_br[tl], x_step_min);

        rgba[tl] =  
            rgba_top_left[tl] +
            rgba_top_right[tl] +
            rgba_bottom_left[tl] +
            rgba_bottom_right[tl];
    }

    for (uniform int y = y0; y < y1; y++) {
        for (uniform int x = x0; x < x1; x += programCount) {
            uniform int32 index = (y * tile_width) + x;
            output[index + programIndex] = rgba[programIndex]; 
        }
    }
}

export void draw_rects(
    uniform int16 output[], 
    uniform int tile_width, 
    uniform int tile_height, 
    uniform float tile_offset_x,
    uniform float tile_offset_y,
    uniform float rect_min_x[],
    uniform float rect_min_y[],
    uniform float rect_max_x[],
    uniform float rect_max_y[],
    uniform float corner_colors[],
    uniform int count) 
{
    for (uniform int i = 0; i < count; i += programCount) {
        // calcuate the area for the quad and adjust to 0.5 as it's our pixel center
        float min_x = floor((rect_min_x[i + programIndex] - tile_offset_x) + 0.5);
        float min_y = floor((rect_min_y[i + programIndex] - tile_offset_y) + 0.5);
        float max_x = floor((rect_max_x[i + programIndex] - tile_offset_x) + 0.5);
        float max_y = floor((rect_max_y[i + programIndex] - tile_offset_x) + 0.5);

        float clipped_min_x = max(0.0, min_x);
        float clipped_min_y = max(0.0, min_y);
        float clipped_max_x = max(max_x, (uniform float)tile_width);
        float clipped_max_y = max(max_y, (uniform float)tile_height);

        // calculate the delta stepping for the (potentially) clipped rects
        // Calculate interpolation factors based on the clipped coordinates
        float x_min_step = (clipped_min_x - min_x) / (max_x - min_x);
        float y_min_step = (clipped_min_y - min_y) / (max_y - min_y);

        // make sure we stay in range
        uniform int channel_count = min(count - i, programCount);
            
        for (uniform int pi = 0; pi < channel_count; ++pi)
        {
            uniform int y0 = extract((int)min_y, pi);
            uniform int y1 = extract((int)max_y, pi);
            uniform int x0 = extract((int)min_x, pi);
            uniform int x1 = extract((int)max_x, pi);

            uniform float y_step_min = extract(y_min_step, pi);
            uniform float x_step_min = extract(x_min_step, pi);

            rasterize_quad(
                output, tile_width, 
                y0, x0, y1, x1, 
                y_step_min, x_step_min,
                corner_colors);
        }
    }
}


