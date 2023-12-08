#pragma once

struct FlApplicationSettings;
struct FlInternalData;

void imgui_create(FlInternalData* state, const FlApplicationSettings* settings);
void imgui_destroy(FlInternalData* state);
void imgui_pre_update(FlInternalData* state);
void imgui_post_update(FlInternalData* state);
