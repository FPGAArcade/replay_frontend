typedef struct FlImageApi {
    struct FlInternalData* priv;
    FlImage (*load)(struct FlInternalData* priv, FlString url);
    FlImage (*load_with_options)(struct FlInternalData* priv, FlString url, FlImageOptions options);
    FlImageLoadStatus (*get_status)(struct FlInternalData* priv, FlImage image);
    FlImageInfo* (*get_info)(struct FlInternalData* priv, FlImage image);
    FlData (*get_data)(struct FlInternalData* priv, FlImage image);
} FlImageApi;

extern FlImageApi* g_flowi_image_api;

#ifdef FLOWI_STATIC
FlImage fl_image_load_impl(struct FlInternalData* priv, FlString url);
FlImage fl_image_load_with_options_impl(struct FlInternalData* priv, FlString url, FlImageOptions options);
FlImageLoadStatus fl_image_get_status_impl(struct FlInternalData* priv, FlImage image);
FlImageInfo* fl_image_get_info_impl(struct FlInternalData* priv, FlImage image);
FlData fl_image_get_data_impl(struct FlInternalData* priv, FlImage image);
#endif

// Async Load image from url/file. Supported formats are: JPG, PNG, SVG and GIF
// Notice that this will return a async handle so the data may not be acceassable directly.
FL_INLINE FlImage fl_image_load(const char* url) {
    FlString url_ = fl_cstr_to_flstring(url);
#ifdef FLOWI_STATIC
    return fl_image_load_impl(g_flowi_image_api->priv, url_);
#else
    return (g_flowi_image_api->load)(g_flowi_image_api->priv, url_);
#endif
}

// Async Load image from url/file. Supported formats are: JPG, PNG, SVG and GIF
// Notice that this will return a async handle so the data may not be acceassable directly.
FL_INLINE FlImage fl_image_load_with_options(const char* url, FlImageOptions options) {
    FlString url_ = fl_cstr_to_flstring(url);
#ifdef FLOWI_STATIC
    return fl_image_load_with_options_impl(g_flowi_image_api->priv, url_, options);
#else
    return (g_flowi_image_api->load_with_options)(g_flowi_image_api->priv, url_, options);
#endif
}

// Get the status of the image. See the [ImageLoadStatus] enum
FL_INLINE FlImageLoadStatus fl_image_get_status(FlImage image) {
#ifdef FLOWI_STATIC
    return fl_image_get_status_impl(g_flowi_image_api->priv, image);
#else
    return (g_flowi_image_api->get_status)(g_flowi_image_api->priv, image);
#endif
}

// Get info about the image. Will be null if the image hasn't loaded yet or failed to load.
FL_INLINE FlImageInfo* fl_image_get_info(FlImage image) {
#ifdef FLOWI_STATIC
    return fl_image_get_info_impl(g_flowi_image_api->priv, image);
#else
    return (g_flowi_image_api->get_info)(g_flowi_image_api->priv, image);
#endif
}

// Get data from the image. Will be null if the image hasn't loaded yet or failed to load.
FL_INLINE FlData fl_image_get_data(FlImage image) {
#ifdef FLOWI_STATIC
    return fl_image_get_data_impl(g_flowi_image_api->priv, image);
#else
    return (g_flowi_image_api->get_data)(g_flowi_image_api->priv, image);
#endif
}
