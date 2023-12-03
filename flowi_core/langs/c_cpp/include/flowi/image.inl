typedef struct FlImageApi {
    struct FlInternalData* priv;
    FlImage (*create_from_file)(struct FlInternalData* priv, FlString filename);
    FlImage (*create_from_file_block)(struct FlInternalData* priv, FlString filename);
    FlImageInfo* (*get_info)(struct FlInternalData* priv, FlImage image);
} FlImageApi;

extern FlImageApi* g_flowi_image_api;

#ifdef FLOWI_STATIC
FlImage fl_image_create_from_file_impl(struct FlInternalData* priv, FlString filename);
FlImage fl_image_create_from_file_block_impl(struct FlInternalData* priv, FlString filename);
FlImageInfo* fl_image_get_info_impl(struct FlInternalData* priv, FlImage image);
#endif

// Load image from file. Supported formats are:
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

// Load image from file. Supported formats are:
// JPEG baseline & progressive (12 bpc/arithmetic not supported, same as stock IJG lib)
// PNG 1/2/4/8/16-bit-per-channel
// This call will block until the loading has finished. It's recommended to use the async version instead.
FL_INLINE FlImage fl_image_create_from_file_block(const char* filename) {
    FlString filename_ = fl_cstr_to_flstring(filename);
#ifdef FLOWI_STATIC
    return fl_image_create_from_file_block_impl(g_flowi_image_api->priv, filename_);
#else
    return (g_flowi_image_api->create_from_file_block)(g_flowi_image_api->priv, filename_);
#endif
}

// Get data amout the image
FL_INLINE FlImageInfo* fl_image_get_info(FlImage image) {
#ifdef FLOWI_STATIC
    return fl_image_get_info_impl(g_flowi_image_api->priv, image);
#else
    return (g_flowi_image_api->get_info)(g_flowi_image_api->priv, image);
#endif
}
