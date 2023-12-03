typedef struct FlIoApi {
    struct FlInternalData* priv;
    void (*dummy)(struct FlInternalData* priv);
} FlIoApi;

extern FlIoApi* g_flowi_io_api;

#ifdef FLOWI_STATIC
void fl_io_dummy_impl(struct FlInternalData* priv);
#endif

// Load image from file/url. Supported formats are: JPG, PNG, WEBP
FL_INLINE void fl_io_dummy() {
#ifdef FLOWI_STATIC
    fl_io_dummy_impl(g_flowi_io_api->priv);
#else
    (g_flowi_io_api->dummy)(g_flowi_io_api->priv);
#endif
}
