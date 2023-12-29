typedef struct FlPainterApi {
    struct FlInternalData* priv;
    void (*draw_rect_filled)(struct FlInternalData* priv, FlVec2 p1, FlVec2 p2, FlColor color, float rounding);
} FlPainterApi;

extern FlPainterApi* g_flowi_painter_api;

#ifdef FLOWI_STATIC
void fl_painter_draw_rect_filled_impl(struct FlInternalData* priv, FlVec2 p1, FlVec2 p2, FlColor color, float rounding);
#endif

FL_INLINE void fl_painter_draw_rect_filled(FlVec2 p1, FlVec2 p2, FlColor color, float rounding) {
#ifdef FLOWI_STATIC
    fl_painter_draw_rect_filled_impl(g_flowi_painter_api->priv, p1, p2, color, rounding);
#else
    (g_flowi_painter_api->draw_rect_filled)(g_flowi_painter_api->priv, p1, p2, color, rounding);
#endif
}
