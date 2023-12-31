typedef struct FlUiApi {
    struct FlInternalData* priv;
    void (*image)(struct FlInternalData* priv, FlImage image);
    void (*image_size)(struct FlInternalData* priv, FlImage image, FlVec2 size);
    void (*image_size_color_shade)(struct FlInternalData* priv, FlImage image, FlVec2 size, FlColor color0,
                                   FlColor color1, FlColor color2, FlColor color3);
    FlIVec2 (*calc_text_size)(struct FlInternalData* priv, FlString text);
} FlUiApi;

extern FlUiApi* g_flowi_ui_api;

#ifdef FLOWI_STATIC
void fl_ui_image_impl(struct FlInternalData* priv, FlImage image);
void fl_ui_image_size_impl(struct FlInternalData* priv, FlImage image, FlVec2 size);
void fl_ui_image_size_color_shade_impl(struct FlInternalData* priv, FlImage image, FlVec2 size, FlColor color0,
                                       FlColor color1, FlColor color2, FlColor color3);
FlIVec2 fl_ui_calc_text_size_impl(struct FlInternalData* priv, FlString text);
#endif

// Draw image. Images can be created with [Image::create_from_file] and [Image::create_from_memory]
FL_INLINE void fl_ui_image(FlImage image) {
#ifdef FLOWI_STATIC
    fl_ui_image_impl(g_flowi_ui_api->priv, image);
#else
    (g_flowi_ui_api->image)(g_flowi_ui_api->priv, image);
#endif
}

FL_INLINE void fl_ui_image_size(FlImage image, FlVec2 size) {
#ifdef FLOWI_STATIC
    fl_ui_image_size_impl(g_flowi_ui_api->priv, image, size);
#else
    (g_flowi_ui_api->image_size)(g_flowi_ui_api->priv, image, size);
#endif
}

FL_INLINE void fl_ui_image_size_color_shade(FlImage image, FlVec2 size, FlColor color0, FlColor color1, FlColor color2,
                                            FlColor color3) {
#ifdef FLOWI_STATIC
    fl_ui_image_size_color_shade_impl(g_flowi_ui_api->priv, image, size, color0, color1, color2, color3);
#else
    (g_flowi_ui_api->image_size_color_shade)(g_flowi_ui_api->priv, image, size, color0, color1, color2, color3);
#endif
}

FL_INLINE FlIVec2 fl_ui_calc_text_size(const char* text) {
    FlString text_ = fl_cstr_to_flstring(text);
#ifdef FLOWI_STATIC
    return fl_ui_calc_text_size_impl(g_flowi_ui_api->priv, text_);
#else
    return (g_flowi_ui_api->calc_text_size)(g_flowi_ui_api->priv, text_);
#endif
}
