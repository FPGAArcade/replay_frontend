typedef struct FlImageApi {
    struct FlInternalData* priv;
    FlImage (*create_from_file)(struct FlInternalData* priv, FlString filename);
    FlImageLoadStatus (*get_status)(struct FlInternalData* priv, FlImage image);
    FlImageInfo* (*get_info)(struct FlInternalData* priv, FlImage image);
    FlData (*get_data)(struct FlInternalData* priv, FlImage image);
} FlImageApi;

extern FlImageApi* g_flowi_image_api;

#ifdef FLOWI_STATIC
FlImage fl_image_create_from_file_impl(struct FlInternalData* priv, FlString filename);
FlImageLoadStatus fl_image_get_status_impl(struct FlInternalData* priv, FlImage image);
FlImageInfo* fl_image_get_info_impl(struct FlInternalData* priv, FlImage image);
FlData fl_image_get_data_impl(struct FlInternalData* priv, FlImage image);
#endif

// Async Load image from url/file. Supported formats are:
// JPEG baseline & progressive (12 bpc/arithmetic not supported, same as stock IJG lib)
// PNG 1/2/4/8/16-bit-per-channel
// Notice that this will return a async handle so the data may not be acceassable directly.
FL_INLINE FlImage fl_image_create_from_file(const char* filename) {
    FlString filename_ = fl_cstr_to_flstring(filename);
#ifdef FLOWI_STATIC
    return fl_image_create_from_file_impl(g_flowi_image_api->priv, filename_);
#else
    return (g_flowi_image_api->create_from_file)(g_flowi_image_api->priv, filename_);
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
