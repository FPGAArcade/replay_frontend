typedef struct FlInputApi {
    struct FlInternalData* priv;
    void (*add_key_event)(struct FlInternalData* priv, FlKey key, bool down);
    void (*add_key_analog_event)(struct FlInternalData* priv, FlKey key, bool down, float value);
    void (*add_mouse_pos_event)(struct FlInternalData* priv, float x, float y);
    void (*add_mouse_button_event)(struct FlInternalData* priv, int button, bool down);
    void (*add_mouse_wheel_event)(struct FlInternalData* priv, float x, float y);
    void (*add_mouse_source_event)(struct FlInternalData* priv, FlMouseSource source);
    void (*add_focus_event)(struct FlInternalData* priv, bool focused);
    void (*add_char_event)(struct FlInternalData* priv, int c);
    void (*update_screen_size_time)(struct FlInternalData* priv, float display_h, float display_w, float w, float h,
                                    float delta_time);
} FlInputApi;

extern FlInputApi* g_flowi_input_api;

#ifdef FLOWI_STATIC
void fl_input_add_key_event_impl(struct FlInternalData* priv, FlKey key, bool down);
void fl_input_add_key_analog_event_impl(struct FlInternalData* priv, FlKey key, bool down, float value);
void fl_input_add_mouse_pos_event_impl(struct FlInternalData* priv, float x, float y);
void fl_input_add_mouse_button_event_impl(struct FlInternalData* priv, int button, bool down);
void fl_input_add_mouse_wheel_event_impl(struct FlInternalData* priv, float x, float y);
void fl_input_add_mouse_source_event_impl(struct FlInternalData* priv, FlMouseSource source);
void fl_input_add_focus_event_impl(struct FlInternalData* priv, bool focused);
void fl_input_add_char_event_impl(struct FlInternalData* priv, int c);
void fl_input_update_screen_size_time_impl(struct FlInternalData* priv, float display_h, float display_w, float w,
                                           float h, float delta_time);
#endif

// Queue a new key down/up event.
// Key should be "translated" (as in, generally [Key::A] matches the key end-user would use to emit an 'A' character)
FL_INLINE void fl_input_add_key_event(FlKey key, bool down) {
#ifdef FLOWI_STATIC
    fl_input_add_key_event_impl(g_flowi_input_api->priv, key, down);
#else
    (g_flowi_input_api->add_key_event)(g_flowi_input_api->priv, key, down);
#endif
}

// Queue a new key down/up event for analog values (
// e.g. ImGuiKey_Gamepad_ values). Dead-zones should be handled by the backend.
FL_INLINE void fl_input_add_key_analog_event(FlKey key, bool down, float value) {
#ifdef FLOWI_STATIC
    fl_input_add_key_analog_event_impl(g_flowi_input_api->priv, key, down, value);
#else
    (g_flowi_input_api->add_key_analog_event)(g_flowi_input_api->priv, key, down, value);
#endif
}

// Queue a mouse position update. Use -FLT_MAX,-FLT_MAX to signify no mouse (e.g. app not focused and not hovered)
FL_INLINE void fl_input_add_mouse_pos_event(float x, float y) {
#ifdef FLOWI_STATIC
    fl_input_add_mouse_pos_event_impl(g_flowi_input_api->priv, x, y);
#else
    (g_flowi_input_api->add_mouse_pos_event)(g_flowi_input_api->priv, x, y);
#endif
}

// Queue a mouse button change
FL_INLINE void fl_input_add_mouse_button_event(int button, bool down) {
#ifdef FLOWI_STATIC
    fl_input_add_mouse_button_event_impl(g_flowi_input_api->priv, button, down);
#else
    (g_flowi_input_api->add_mouse_button_event)(g_flowi_input_api->priv, button, down);
#endif
}

// Queue a mouse wheel update.
// wheel_y<0: scroll down, wheel_y>0: scroll up, wheel_x<0: scroll right, wheel_x>0: scroll left.
FL_INLINE void fl_input_add_mouse_wheel_event(float x, float y) {
#ifdef FLOWI_STATIC
    fl_input_add_mouse_wheel_event_impl(g_flowi_input_api->priv, x, y);
#else
    (g_flowi_input_api->add_mouse_wheel_event)(g_flowi_input_api->priv, x, y);
#endif
}

// Queue a mouse source change (Mouse/TouchScreen/Pen)
FL_INLINE void fl_input_add_mouse_source_event(FlMouseSource source) {
#ifdef FLOWI_STATIC
    fl_input_add_mouse_source_event_impl(g_flowi_input_api->priv, source);
#else
    (g_flowi_input_api->add_mouse_source_event)(g_flowi_input_api->priv, source);
#endif
}

// Queue a gain/loss of focus for the application (generally based on OS/platform focus of your window)
FL_INLINE void fl_input_add_focus_event(bool focused) {
#ifdef FLOWI_STATIC
    fl_input_add_focus_event_impl(g_flowi_input_api->priv, focused);
#else
    (g_flowi_input_api->add_focus_event)(g_flowi_input_api->priv, focused);
#endif
}

// Queue a new character input
FL_INLINE void fl_input_add_char_event(int c) {
#ifdef FLOWI_STATIC
    fl_input_add_char_event_impl(g_flowi_input_api->priv, c);
#else
    (g_flowi_input_api->add_char_event)(g_flowi_input_api->priv, c);
#endif
}

// This is a bit temporary and should be moved
FL_INLINE void fl_input_update_screen_size_time(float display_h, float display_w, float w, float h, float delta_time) {
#ifdef FLOWI_STATIC
    fl_input_update_screen_size_time_impl(g_flowi_input_api->priv, display_h, display_w, w, h, delta_time);
#else
    (g_flowi_input_api->update_screen_size_time)(g_flowi_input_api->priv, display_h, display_w, w, h, delta_time);
#endif
}
