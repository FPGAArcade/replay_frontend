typedef struct FlUiApi {
    struct FlInternalData* priv;
    void (*image)(struct FlInternalData* priv, FlImage image);
    FlVec2 (*calc_text_size)(struct FlInternalData* priv, FlString text);
} FlUiApi;

extern FlUiApi* g_flowi_ui_api;

#ifdef FLOWI_STATIC
void fl_ui_image_impl(struct FlInternalData* priv, FlImage image);
FlVec2 fl_ui_calc_text_size_impl(struct FlInternalData* priv, FlString text);
#endif

// Draw image. Images can be created with [Image::create_from_file] and [Image::create_from_memory]
FL_INLINE void fl_ui_image(FlImage image) {
#ifdef FLOWI_STATIC
    fl_ui_image_impl(g_flowi_ui_api->priv, image);
#else
    (g_flowi_ui_api->image)(g_flowi_ui_api->priv, image);
#endif
}

FL_INLINE FlVec2 fl_ui_calc_text_size(const char* text) {
    FlString text_ = fl_cstr_to_flstring(text);
#ifdef FLOWI_STATIC
    return fl_ui_calc_text_size_impl(g_flowi_ui_api->priv, text_);
#else
    return (g_flowi_ui_api->calc_text_size)(g_flowi_ui_api->priv, text_);
#endif
}
