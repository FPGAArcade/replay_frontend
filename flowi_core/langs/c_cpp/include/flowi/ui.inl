typedef struct FlUiApi {
    struct FlInternalData* priv;
    void (*image)(struct FlInternalData* priv, FlImage image);
} FlUiApi;

extern FlUiApi* g_flowi_ui_api;

#ifdef FLOWI_STATIC
void fl_ui_image_impl(struct FlInternalData* priv, FlImage image);
#endif

// Draw image. Images can be created with [Image::create_from_file] and [Image::create_from_memory]
FL_INLINE void fl_ui_image(FlImage image) {
#ifdef FLOWI_STATIC
    fl_ui_image_impl(g_flowi_ui_api->priv, image);
#else
    (g_flowi_ui_api->image)(g_flowi_ui_api->priv, image);
#endif
}
