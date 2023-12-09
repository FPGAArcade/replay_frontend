typedef struct FlRendererApi {
    struct FlInternalData* priv;
    FlTexture (*get_texture)(struct FlInternalData* priv, FlImage image);
} FlRendererApi;

extern FlRendererApi* g_flowi_renderer_api;

#ifdef FLOWI_STATIC
FlTexture fl_renderer_get_texture_impl(struct FlInternalData* priv, FlImage image);
#endif

// Get a texture from the active Renderer given a image handle. The renderer can return None if the image handle isnt't
// valid or that that image hasn't been created as a texture yet
FL_INLINE FlTexture fl_renderer_get_texture(FlImage image) {
#ifdef FLOWI_STATIC
    return fl_renderer_get_texture_impl(g_flowi_renderer_api->priv, image);
#else
    return (g_flowi_renderer_api->get_texture)(g_flowi_renderer_api->priv, image);
#endif
}
