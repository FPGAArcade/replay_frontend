export void add_values(uniform float out[], uniform float input[], uniform int count) {
    for (uniform int i = 0; i < count; i += programCount)  {
        out[i + programIndex] = input[i + programIndex] + input[i + programIndex];
    }
}
