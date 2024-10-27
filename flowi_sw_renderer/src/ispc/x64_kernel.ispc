export void add_values(uniform float out[], uniform float input[], uniform int count) {
    for (uniform int i = 0; i < count; i += programCount)  {
        out[i + programIndex] = input[i + programIndex] + input[i + programIndex];
    }
}

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
        uniform float x_step_min)
        /*
        uniform float rgba_tl[],
        uniform float rgba_tr[],
        uniform float rgba_bl[],
        uniform float rgba_br[])
        */
{
    /*
    uniform float rgba_top_left[4];
    uniform float rgba_top_right[4];
    uniform float rgba_bottom_left[4];
    uniform float rgba_bottom_right[4];
    uniform float rgba[4];
    */

    /*
    assume(y1 > y0);
    assume(x1 > x0);
    */

    /*
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
    */

    float rgba[4] = { 1.0 * 32200.0f, 0.0, 0.0, 1.0 * 32200.0f };

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
    //uniform float corner_colors[],
    uniform int count) 
{
    for (uniform int i = 0; i < count; i += programCount) {
        // calcuate the area for the quad and adjust to 0.5 as it's our pixel center
        float min_x = floor((rect_min_x[i + programIndex] - tile_offset_x) + 0.5);
        float min_y = floor((rect_min_y[i + programIndex] - tile_offset_y) + 0.5);
        float max_x = floor((rect_max_x[i + programIndex] - tile_offset_x) + 0.5);
        float max_y = floor((rect_max_y[i + programIndex] - tile_offset_x) + 0.5);

        //float clipped_min_x = max(0.0, min_x);
        //float clipped_min_y = max(0.0, min_y);
        //float clipped_max_x = max(max_x, (uniform float)tile_width);
        //float clipped_max_y = max(max_y, (uniform float)tile_height);

        // calculate the delta stepping for the (potentially) clipped rects
        // Calculate interpolation factors based on the clipped coordinates
        float x_min_step = 1.0;//(clipped_min_x - min_x) / (max_x - min_x);
        float y_min_step = 1.0;//(clipped_min_y - min_y) / (max_y - min_y);

        int min_xi = (int)min_x;
        int min_yi = (int)min_y;
        int max_xi = (int)max_x;
        int max_yi = (int)max_y;
            
        //for (uniform int pi = 0; pi < programCount; ++pi)
        {
            uniform int pi = 0;

            uniform int y0 = extract(min_yi, pi);
            uniform int y1 = extract(max_yi, pi);
            uniform int x0 = extract(min_xi, pi);
            uniform int x1 = extract(max_xi, pi);

            uniform float y_step_min = extract(y_min_step, pi);
            uniform float x_step_min = extract(x_min_step, pi);

            rasterize_quad(
                output, tile_width, 
                y0, x0, y1, x1, 
                y_step_min, x_step_min);
                /*
                &corner_colors[i + (programCount * 0)], 
                &corner_colors[i + (programCount * 1)], 
                &corner_colors[i + (programCount * 2)], 
                &corner_colors[i + (programCount * 3)]);
                */
        }
    }
}

