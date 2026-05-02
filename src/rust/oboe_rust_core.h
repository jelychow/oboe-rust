/*
 * Copyright 2026 The Android Open Source Project
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#ifndef OBOE_RUST_CORE_H
#define OBOE_RUST_CORE_H

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef int32_t (*OboeRustAAudioDataCallback)(void *stream, void *user_data,
                                              void *audio_data, int32_t num_frames);
typedef void (*OboeRustAAudioErrorCallback)(void *stream, void *user_data,
                                            int32_t error);
typedef int32_t (*OboeRustAAudioPartialDataCallback)(void *stream, void *user_data,
                                                     void *audio_data, int32_t num_frames);
typedef void (*OboeRustAAudioPresentationCallback)(void *stream, void *user_data);
typedef void (*OboeRustAAudioRoutingChangedCallback)(void *stream, void *user_data,
                                                     const int32_t *device_ids,
                                                     int32_t num_devices);

typedef struct OboeRustAAudioOutputStream OboeRustAAudioOutputStream;
typedef struct OboeRustAAudioInputStream OboeRustAAudioInputStream;

typedef struct OboeRustAAudioPlaybackParameters {
    int32_t fallback_mode;
    int32_t stretch_mode;
    float pitch;
    float speed;
} OboeRustAAudioPlaybackParameters;

typedef struct OboeRustAAudioPlatform {
    int32_t (*create_stream_builder)(void **builder);
    int32_t (*builder_open_stream)(void *builder, void **stream);
    int32_t (*builder_delete)(void *builder);
    void (*builder_set_buffer_capacity_in_frames)(void *builder, int32_t value);
    void (*builder_set_channel_count)(void *builder, int32_t value);
    void (*builder_set_device_id)(void *builder, int32_t value);
    void (*builder_set_direction)(void *builder, int32_t value);
    void (*builder_set_format)(void *builder, int32_t value);
    void (*builder_set_frames_per_data_callback)(void *builder, int32_t value);
    void (*builder_set_performance_mode)(void *builder, int32_t value);
    void (*builder_set_sample_rate)(void *builder, int32_t value);
    void (*builder_set_sharing_mode)(void *builder, int32_t value);
    void (*builder_set_channel_mask)(void *builder, uint32_t value);
    void (*builder_set_usage)(void *builder, int32_t value);
    void (*builder_set_content_type)(void *builder, int32_t value);
    void (*builder_set_input_preset)(void *builder, int32_t value);
    void (*builder_set_session_id)(void *builder, int32_t value);
    void (*builder_set_privacy_sensitive)(void *builder, bool value);
    void (*builder_set_allowed_capture_policy)(void *builder, int32_t value);
    void (*builder_set_package_name)(void *builder, const char *value);
    void (*builder_set_attribution_tag)(void *builder, const char *value);
    void (*builder_set_is_content_spatialized)(void *builder, bool value);
    void (*builder_set_spatialization_behavior)(void *builder, int32_t value);
    void (*builder_set_data_callback)(void *builder, OboeRustAAudioDataCallback callback,
                                      void *user_data);
    void (*builder_set_error_callback)(void *builder, OboeRustAAudioErrorCallback callback,
                                       void *user_data);
    void (*builder_set_partial_data_callback)(void *builder,
                                              OboeRustAAudioPartialDataCallback callback,
                                              void *user_data);
    void (*builder_set_presentation_end_callback)(void *builder,
                                                  OboeRustAAudioPresentationCallback callback,
                                                  void *user_data);
    void (*builder_set_routing_changed_callback)(void *builder,
                                                 OboeRustAAudioRoutingChangedCallback callback,
                                                 void *user_data);
    int32_t (*stream_close)(void *stream);
    int32_t (*stream_release)(void *stream);
    int32_t (*stream_request_start)(void *stream);
    int32_t (*stream_request_pause)(void *stream);
    int32_t (*stream_request_flush)(void *stream);
    int32_t (*stream_request_stop)(void *stream);
    int32_t (*stream_write)(void *stream, const void *buffer, int32_t num_frames,
                            int64_t timeout_nanoseconds);
    int32_t (*stream_read)(void *stream, void *buffer, int32_t num_frames,
                           int64_t timeout_nanoseconds);
    int32_t (*stream_wait_for_state_change)(void *stream, int32_t current_state,
                                            int32_t *next_state,
                                            int64_t timeout_nanoseconds);
    int32_t (*stream_get_timestamp)(void *stream, int32_t clock_id,
                                    int64_t *frame_position, int64_t *time_nanoseconds);
    int32_t (*stream_set_buffer_size)(void *stream, int32_t requested_frames);
    int32_t (*stream_get_channel_count)(void *stream);
    int32_t (*stream_get_device_id)(void *stream);
    int32_t (*stream_get_format)(void *stream);
    int32_t (*stream_get_sample_rate)(void *stream);
    int32_t (*stream_get_sharing_mode)(void *stream);
    int32_t (*stream_get_performance_mode)(void *stream);
    int32_t (*stream_get_buffer_capacity)(void *stream);
    int32_t (*stream_get_buffer_size)(void *stream);
    int32_t (*stream_get_frames_per_burst)(void *stream);
    int32_t (*stream_get_state)(void *stream);
    int32_t (*stream_get_xrun_count)(void *stream);
    int64_t (*stream_get_frames_read)(void *stream);
    int64_t (*stream_get_frames_written)(void *stream);
    int32_t (*stream_get_usage)(void *stream);
    int32_t (*stream_get_content_type)(void *stream);
    int32_t (*stream_get_input_preset)(void *stream);
    int32_t (*stream_get_session_id)(void *stream);
    bool (*stream_is_privacy_sensitive)(void *stream);
    int32_t (*stream_get_allowed_capture_policy)(void *stream);
    uint32_t (*stream_get_channel_mask)(void *stream);
    bool (*stream_is_content_spatialized)(void *stream);
    int32_t (*stream_get_spatialization_behavior)(void *stream);
    int32_t (*stream_get_hardware_channel_count)(void *stream);
    int32_t (*stream_get_hardware_sample_rate)(void *stream);
    int32_t (*stream_get_hardware_format)(void *stream);
    int32_t (*stream_set_offload_delay_padding)(void *stream, int32_t delay_in_frames,
                                                int32_t padding_in_frames);
    int32_t (*stream_get_offload_delay)(void *stream);
    int32_t (*stream_get_offload_padding)(void *stream);
    int32_t (*stream_set_offload_end_of_stream)(void *stream);
    int32_t (*stream_flush_from_frame)(void *stream, int32_t accuracy,
                                       int64_t *position_in_frames);
    int32_t (*stream_get_playback_parameters)(
            void *stream, OboeRustAAudioPlaybackParameters *parameters);
    int32_t (*stream_set_playback_parameters)(
            void *stream, const OboeRustAAudioPlaybackParameters *parameters);
} OboeRustAAudioPlatform;

typedef struct OboeRustAAudioOutputSettings {
    int32_t direction;
    int32_t device_id;
    int32_t sample_rate;
    int32_t channel_count;
    uint32_t channel_mask;
    int32_t format;
    int32_t sharing_mode;
    int32_t performance_mode;
    int32_t buffer_capacity_in_frames;
    int32_t frames_per_data_callback;
    int32_t session_id;
    int32_t usage;
    int32_t content_type;
    int32_t allowed_capture_policy;
    bool is_content_spatialized;
    int32_t spatialization_behavior;
    const char *package_name;
    const char *attribution_tag;
    OboeRustAAudioDataCallback data_callback;
    OboeRustAAudioErrorCallback error_callback;
    OboeRustAAudioPartialDataCallback partial_data_callback;
    OboeRustAAudioPresentationCallback presentation_end_callback;
    OboeRustAAudioRoutingChangedCallback routing_changed_callback;
    void *user_data;
} OboeRustAAudioOutputSettings;

typedef struct OboeRustAAudioOutputProperties {
    int32_t result;
    void *raw_stream;
    int32_t channel_count;
    int32_t device_id;
    int32_t sample_rate;
    int32_t format;
    int32_t sharing_mode;
    int32_t performance_mode;
    int32_t buffer_capacity_in_frames;
    int32_t buffer_size_in_frames;
    int32_t frames_per_burst;
    int32_t usage;
    int32_t content_type;
    int32_t input_preset;
    int32_t session_id;
    int32_t allowed_capture_policy;
    uint32_t channel_mask;
    bool is_content_spatialized;
    int32_t spatialization_behavior;
    int32_t hardware_channel_count;
    int32_t hardware_sample_rate;
    int32_t hardware_format;
} OboeRustAAudioOutputProperties;

typedef struct OboeRustAAudioInputSettings {
    int32_t direction;
    int32_t device_id;
    int32_t sample_rate;
    int32_t channel_count;
    uint32_t channel_mask;
    int32_t format;
    int32_t sharing_mode;
    int32_t performance_mode;
    int32_t buffer_capacity_in_frames;
    int32_t frames_per_data_callback;
    int32_t input_preset;
    int32_t privacy_sensitive_mode;
    int32_t privacy_sensitive_mode_unspecified;
    int32_t privacy_sensitive_mode_enabled;
    int32_t privacy_sensitive_mode_disabled;
    int32_t session_id;
    int32_t usage;
    int32_t content_type;
    int32_t allowed_capture_policy;
    const char *package_name;
    const char *attribution_tag;
    bool is_content_spatialized;
    int32_t spatialization_behavior;
    OboeRustAAudioDataCallback data_callback;
    OboeRustAAudioErrorCallback error_callback;
    OboeRustAAudioPartialDataCallback partial_data_callback;
    OboeRustAAudioPresentationCallback presentation_end_callback;
    OboeRustAAudioRoutingChangedCallback routing_changed_callback;
    void *user_data;
} OboeRustAAudioInputSettings;

typedef struct OboeRustAAudioInputProperties {
    int32_t result;
    void *raw_stream;
    int32_t channel_count;
    int32_t device_id;
    int32_t sample_rate;
    int32_t format;
    int32_t sharing_mode;
    int32_t performance_mode;
    int32_t buffer_capacity_in_frames;
    int32_t buffer_size_in_frames;
    int32_t frames_per_burst;
    int32_t usage;
    int32_t content_type;
    int32_t input_preset;
    int32_t session_id;
    int32_t allowed_capture_policy;
    int32_t privacy_sensitive_mode;
    uint32_t channel_mask;
    bool is_content_spatialized;
    int32_t spatialization_behavior;
    int32_t hardware_channel_count;
    int32_t hardware_sample_rate;
    int32_t hardware_format;
} OboeRustAAudioInputProperties;

OboeRustAAudioOutputStream *oboe_rust_aaudio_output_open(
        const OboeRustAAudioPlatform *platform,
        const OboeRustAAudioOutputSettings *settings,
        OboeRustAAudioOutputProperties *properties);
int32_t oboe_rust_aaudio_output_destroy(OboeRustAAudioOutputStream *stream);
int32_t oboe_rust_aaudio_output_close(OboeRustAAudioOutputStream *stream);
int32_t oboe_rust_aaudio_output_request_start(OboeRustAAudioOutputStream *stream);
int32_t oboe_rust_aaudio_output_request_pause(OboeRustAAudioOutputStream *stream);
int32_t oboe_rust_aaudio_output_request_flush(OboeRustAAudioOutputStream *stream);
int32_t oboe_rust_aaudio_output_request_stop(OboeRustAAudioOutputStream *stream);
int32_t oboe_rust_aaudio_output_write(OboeRustAAudioOutputStream *stream,
                                      const void *buffer, int32_t num_frames,
                                      int64_t timeout_nanoseconds);
int32_t oboe_rust_aaudio_output_read(OboeRustAAudioOutputStream *stream,
                                     void *buffer, int32_t num_frames,
                                     int64_t timeout_nanoseconds);
int32_t oboe_rust_aaudio_output_wait_for_state_change(OboeRustAAudioOutputStream *stream,
                                                      int32_t current_state,
                                                      int32_t *next_state,
                                                      int64_t timeout_nanoseconds);
int32_t oboe_rust_aaudio_output_get_state(OboeRustAAudioOutputStream *stream);
int32_t oboe_rust_aaudio_output_set_buffer_size(OboeRustAAudioOutputStream *stream,
                                                int32_t requested_frames);
int32_t oboe_rust_aaudio_output_get_buffer_size(OboeRustAAudioOutputStream *stream);
int32_t oboe_rust_aaudio_output_get_xrun_count(OboeRustAAudioOutputStream *stream);
int32_t oboe_rust_aaudio_output_get_timestamp(OboeRustAAudioOutputStream *stream,
                                              int32_t clock_id,
                                              int64_t *frame_position,
                                              int64_t *time_nanoseconds);
int64_t oboe_rust_aaudio_output_get_frames_read(OboeRustAAudioOutputStream *stream);
int64_t oboe_rust_aaudio_output_get_frames_written(OboeRustAAudioOutputStream *stream);
void *oboe_rust_aaudio_output_get_raw_stream(OboeRustAAudioOutputStream *stream);
int32_t oboe_rust_aaudio_output_release(OboeRustAAudioOutputStream *stream);
int32_t oboe_rust_aaudio_output_set_offload_delay_padding(
        OboeRustAAudioOutputStream *stream, int32_t delay_in_frames, int32_t padding_in_frames);
int32_t oboe_rust_aaudio_output_get_offload_delay(OboeRustAAudioOutputStream *stream);
int32_t oboe_rust_aaudio_output_get_offload_padding(OboeRustAAudioOutputStream *stream);
int32_t oboe_rust_aaudio_output_set_offload_end_of_stream(OboeRustAAudioOutputStream *stream);
int32_t oboe_rust_aaudio_output_flush_from_frame(OboeRustAAudioOutputStream *stream,
                                                 int32_t accuracy,
                                                 int64_t *position_in_frames);
int32_t oboe_rust_aaudio_output_get_playback_parameters(
        OboeRustAAudioOutputStream *stream, OboeRustAAudioPlaybackParameters *parameters);
int32_t oboe_rust_aaudio_output_set_playback_parameters(
        OboeRustAAudioOutputStream *stream, const OboeRustAAudioPlaybackParameters *parameters);

OboeRustAAudioInputStream *oboe_rust_aaudio_input_open(
        const OboeRustAAudioPlatform *platform,
        const OboeRustAAudioInputSettings *settings,
        OboeRustAAudioInputProperties *properties);
int32_t oboe_rust_aaudio_input_destroy(OboeRustAAudioInputStream *stream);
int32_t oboe_rust_aaudio_input_close(OboeRustAAudioInputStream *stream);
int32_t oboe_rust_aaudio_input_request_start(OboeRustAAudioInputStream *stream);
int32_t oboe_rust_aaudio_input_request_pause(OboeRustAAudioInputStream *stream);
int32_t oboe_rust_aaudio_input_request_flush(OboeRustAAudioInputStream *stream);
int32_t oboe_rust_aaudio_input_request_stop(OboeRustAAudioInputStream *stream);
int32_t oboe_rust_aaudio_input_write(OboeRustAAudioInputStream *stream,
                                     const void *buffer, int32_t num_frames,
                                     int64_t timeout_nanoseconds);
int32_t oboe_rust_aaudio_input_read(OboeRustAAudioInputStream *stream,
                                    void *buffer, int32_t num_frames,
                                    int64_t timeout_nanoseconds);
int32_t oboe_rust_aaudio_input_wait_for_state_change(OboeRustAAudioInputStream *stream,
                                                     int32_t current_state,
                                                     int32_t *next_state,
                                                     int64_t timeout_nanoseconds);
int32_t oboe_rust_aaudio_input_get_state(OboeRustAAudioInputStream *stream);
int32_t oboe_rust_aaudio_input_set_buffer_size(OboeRustAAudioInputStream *stream,
                                               int32_t requested_frames);
int32_t oboe_rust_aaudio_input_get_buffer_size(OboeRustAAudioInputStream *stream);
int32_t oboe_rust_aaudio_input_get_xrun_count(OboeRustAAudioInputStream *stream);
int32_t oboe_rust_aaudio_input_get_timestamp(OboeRustAAudioInputStream *stream,
                                             int32_t clock_id,
                                             int64_t *frame_position,
                                             int64_t *time_nanoseconds);
int64_t oboe_rust_aaudio_input_get_frames_read(OboeRustAAudioInputStream *stream);
int64_t oboe_rust_aaudio_input_get_frames_written(OboeRustAAudioInputStream *stream);
void *oboe_rust_aaudio_input_get_raw_stream(OboeRustAAudioInputStream *stream);
int32_t oboe_rust_aaudio_input_release(OboeRustAAudioInputStream *stream);

bool oboe_rust_aaudio_playback_parameters_valid(
        int32_t fallback_mode, int32_t stretch_mode,
        int32_t fallback_default, int32_t fallback_mute, int32_t fallback_fail,
        int32_t stretch_default, int32_t stretch_voice);

int32_t oboe_rust_convert_format_to_size_in_bytes(int32_t format);
void oboe_rust_convert_float_to_pcm16(const float *source, int16_t *destination,
                                      int32_t num_samples);
void oboe_rust_convert_pcm16_to_float(const int16_t *source, float *destination,
                                      int32_t num_samples);

void oboe_rust_source_i16_to_float(const int16_t *source, float *destination,
                                   int32_t num_samples);
void oboe_rust_source_i24_to_float(const uint8_t *source, float *destination,
                                   int32_t num_samples);
void oboe_rust_sink_float_to_i16(const float *source, int16_t *destination,
                                 int32_t num_samples);
void oboe_rust_sink_float_to_i24(const float *source, uint8_t *destination,
                                 int32_t num_samples);
void oboe_rust_sink_float_to_i32(const float *source, int32_t *destination,
                                 int32_t num_samples);

void oboe_rust_mono_to_multi(const float *source, float *destination,
                             int32_t num_frames, int32_t channel_count);
void oboe_rust_copy_float_buffer(const float *source, float *destination,
                                 int32_t num_samples);
void oboe_rust_multi_to_mono(const float *source, float *destination,
                             int32_t num_frames, int32_t input_channel_count);
void oboe_rust_mono_blend(const float *source, float *destination,
                          int32_t num_frames, int32_t channel_count,
                          float inv_channel_count);
void oboe_rust_many_to_multi_channel(const float *source, float *destination,
                                     int32_t num_frames, int32_t channel_count,
                                     int32_t channel);
void oboe_rust_multi_to_many_channel(const float *source, float *destination,
                                     int32_t num_frames, int32_t channel_count,
                                     int32_t channel);
void oboe_rust_ramp_linear(const float *source, float *destination,
                           int32_t num_frames, int32_t channel_count,
                           float level_to, int32_t *remaining_frames,
                           float scaler);
void oboe_rust_clip_to_range(const float *source, float *destination,
                             int32_t num_samples, float minimum, float maximum);
float oboe_rust_limiter_process_buffer(const float *source, float *destination,
                                       int32_t num_samples, float last_valid_output);

uint32_t oboe_rust_fifo_full_frames_available(uint32_t capacity_in_frames,
                                              uint64_t read_counter,
                                              uint64_t write_counter);
uint32_t oboe_rust_fifo_empty_frames_available(uint32_t capacity_in_frames,
                                               uint64_t read_counter,
                                               uint64_t write_counter);
uint32_t oboe_rust_fifo_read_index(uint32_t capacity_in_frames, uint64_t read_counter);
uint32_t oboe_rust_fifo_write_index(uint32_t capacity_in_frames, uint64_t write_counter);
int32_t oboe_rust_fifo_copy_read(const uint8_t *storage, uint32_t bytes_per_frame,
                                 uint32_t capacity_in_frames, uint64_t read_counter,
                                 uint64_t write_counter, uint8_t *destination,
                                 int32_t frames_to_read);
int32_t oboe_rust_fifo_copy_write(uint8_t *storage, uint32_t bytes_per_frame,
                                  uint32_t capacity_in_frames, uint64_t read_counter,
                                  uint64_t write_counter, const uint8_t *source,
                                  int32_t frames_to_write);
int32_t oboe_rust_fifo_copy_read_now(const uint8_t *storage, uint32_t bytes_per_frame,
                                     uint32_t capacity_in_frames, uint64_t read_counter,
                                     uint64_t write_counter, uint8_t *destination,
                                     int32_t num_frames);

void oboe_rust_integer_ratio_reduce(int32_t *numerator, int32_t *denominator);
void oboe_rust_resampler_write_frame(float *x, const float *frame,
                                     int32_t num_taps, int32_t channel_count,
                                     int32_t *cursor);
void oboe_rust_linear_resampler_read_frame(const float *previous, const float *current,
                                           float *destination, int32_t channel_count,
                                           int32_t integer_phase, int32_t denominator);
void oboe_rust_polyphase_resampler_read_frame(const float *x, const float *coefficients,
                                              float *destination, int32_t num_taps,
                                              int32_t channel_count, int32_t cursor);
void oboe_rust_sinc_resampler_read_frame(const float *x,
                                         const float *coefficients_low,
                                         const float *coefficients_high,
                                         float *destination, int32_t num_taps,
                                         int32_t channel_count, float fraction);

bool oboe_rust_builder_will_use_aaudio(int32_t audio_api,
                                       bool is_aaudio_supported,
                                       bool is_aaudio_recommended);
int32_t oboe_rust_builder_select_backend(int32_t audio_api, int32_t direction,
                                         bool is_aaudio_supported,
                                         bool is_aaudio_recommended);
bool oboe_rust_builder_is_compatible(int32_t builder_sample_rate,
                                     int32_t builder_format,
                                     int32_t builder_frames_per_callback,
                                     int32_t builder_channel_count,
                                     int32_t stream_sample_rate,
                                     int32_t stream_format,
                                     int32_t stream_frames_per_callback,
                                     int32_t stream_channel_count);
int32_t oboe_rust_stream_wait_transition_result(int32_t current_state,
                                                int32_t starting_state,
                                                int32_t ending_state,
                                                int32_t wait_result,
                                                int32_t next_state);
int32_t oboe_rust_stream_available_frames(int64_t frames_read,
                                          int64_t frames_written,
                                          int32_t *frames_available);
int32_t oboe_rust_stream_default_delay_before_close_millis(int32_t frames_per_burst,
                                                           int32_t sample_rate,
                                                           int32_t minimum,
                                                           int32_t maximum);
int32_t oboe_rust_stream_optimal_buffer_size(int32_t direction,
                                             int32_t performance_mode,
                                             int32_t buffer_capacity_in_frames,
                                             int32_t frames_per_burst,
                                             int32_t bursts_for_low_latency);
bool oboe_rust_data_callback_should_continue(int32_t callback_result);
bool oboe_rust_aaudio_callback_should_launch_stop_thread(int32_t callback_result,
                                                         bool workarounds_enabled,
                                                         int32_t sdk_version,
                                                         int32_t android_api_r);
int32_t oboe_rust_aaudio_callback_return_result(int32_t callback_result,
                                                bool workarounds_enabled,
                                                int32_t sdk_version,
                                                int32_t android_api_r);
int32_t oboe_rust_aaudio_adjust_input_capacity(int32_t capacity,
                                               int32_t direction,
                                               int32_t input_direction,
                                               int32_t performance_mode,
                                               int32_t low_latency_mode,
                                               int32_t unspecified,
                                               int32_t required_capacity_for_fast_track,
                                               bool workarounds_enabled);
int32_t oboe_rust_aaudio_session_performance_mode(int32_t performance_mode,
                                                  int32_t session_id,
                                                  int32_t session_id_none,
                                                  int32_t direction,
                                                  int32_t output_direction,
                                                  int32_t low_latency_mode,
                                                  int32_t none_mode,
                                                  bool workarounds_enabled);
int32_t oboe_rust_aaudio_normalize_input_preset(int32_t input_preset,
                                                int32_t sdk_version,
                                                int32_t latest_unsupported_api,
                                                int32_t voice_performance,
                                                int32_t voice_recognition);
int32_t oboe_rust_aaudio_spatialization_behavior(int32_t spatialization_behavior,
                                                 int32_t unspecified,
                                                 int32_t never,
                                                 bool setter_available);
int32_t oboe_rust_aaudio_coerce_open_result(int32_t open_result,
                                            bool workarounds_enabled,
                                            int32_t error_internal);
int32_t oboe_rust_aaudio_force_starting_to_started(bool workarounds_enabled,
                                                   int32_t state,
                                                   int32_t starting_state,
                                                   int32_t started_state);
bool oboe_rust_aaudio_request_already_satisfied(int32_t sdk_version,
                                                int32_t android_api_o_mr1,
                                                int32_t state,
                                                int32_t first_terminal_state,
                                                int32_t second_terminal_state);
double oboe_rust_aaudio_calculate_latency_millis(bool is_output,
                                                 int64_t app_frame_index,
                                                 int64_t hardware_frame_index,
                                                 int64_t app_frame_app_time,
                                                 int64_t hardware_frame_hardware_time,
                                                 int32_t sample_rate,
                                                 int64_t nanos_per_second,
                                                 int64_t nanos_per_millisecond);
bool oboe_rust_mmap_policy_enabled(int32_t policy);
bool oboe_rust_mmap_enabled_from_policy(int32_t policy, bool mmap_supported);
int32_t oboe_rust_mmap_unavailable_result();
int32_t oboe_rust_mmap_load_symbols_result(bool loader_available,
                                           bool lib_handle_available,
                                           bool stream_is_mmap_available,
                                           bool set_mmap_policy_available,
                                           bool get_mmap_policy_available);

typedef void (*OboeRustOpenSLESQueueCallback)(void *user_data);

typedef struct OboeRustOpenSLESOutputBackend OboeRustOpenSLESOutputBackend;
typedef struct OboeRustOpenSLESInputBackend OboeRustOpenSLESInputBackend;

typedef struct OboeRustOpenSLESPlatform {
    int32_t (*engine_open)(void);
    void (*engine_close)(void);
    int32_t (*output_mixer_open)(void);
    void (*output_mixer_close)(void);
    int32_t (*output_create_player)(void **object, void *audio_source);
    int32_t (*input_create_recorder)(void **object, void *audio_source, void *audio_sink);
    int32_t (*object_get_android_configuration)(void *object, void **configuration);
    int32_t (*object_realize)(void *object);
    void (*object_destroy)(void *object);
    int32_t (*object_get_play)(void *object, void **play);
    int32_t (*object_get_record)(void *object, void **record);
    int32_t (*object_get_simple_buffer_queue)(void *object, void **queue);
    int32_t (*configuration_set_performance_mode)(void *configuration, int32_t performance_mode);
    int32_t (*configuration_get_performance_mode)(void *configuration, int32_t *performance_mode);
    int32_t (*configuration_set_stream_type)(void *configuration, int32_t stream_type);
    int32_t (*configuration_set_recording_preset)(void *configuration, int32_t recording_preset);
    int32_t (*queue_register_callback)(void *queue,
                                       OboeRustOpenSLESQueueCallback callback,
                                       void *user_data);
    int32_t (*queue_enqueue)(void *queue, void *buffer, int32_t num_bytes);
    int32_t (*queue_clear)(void *queue);
    int32_t (*queue_get_depth)(void *queue);
    int32_t (*play_set_state)(void *play, int32_t state);
    int32_t (*play_get_position_millis)(void *play, int32_t *position_millis);
    int32_t (*record_set_state)(void *record, int32_t state);
    int32_t (*record_get_position_millis)(void *record, int32_t *position_millis);
} OboeRustOpenSLESPlatform;

typedef struct OboeRustOpenSLESCommonSettings {
    int32_t sdk_version;
    int32_t android_api_n_mr1;
    int32_t android_api_o_mr1;
    int32_t opensl_performance_mode;
    int32_t opensl_performance_none;
    int32_t opensl_performance_latency;
    int32_t opensl_performance_latency_effects;
    int32_t opensl_performance_power_saving;
    int32_t oboe_performance_none;
    int32_t oboe_performance_low_latency;
    int32_t oboe_performance_power_saving;
    OboeRustOpenSLESQueueCallback queue_callback;
    void *queue_callback_user_data;
} OboeRustOpenSLESCommonSettings;

typedef struct OboeRustOpenSLESOutputSettings {
    OboeRustOpenSLESCommonSettings common;
    void *audio_source;
    int32_t opensl_stream_type;
} OboeRustOpenSLESOutputSettings;

typedef struct OboeRustOpenSLESInputSettings {
    OboeRustOpenSLESCommonSettings common;
    void *audio_source;
    void *audio_sink;
    int32_t opensl_recording_preset;
    int32_t opensl_recording_preset_voice_recognition;
    int32_t oboe_input_preset;
    int32_t oboe_input_preset_voice_recognition;
} OboeRustOpenSLESInputSettings;

typedef struct OboeRustOpenSLESOutputProperties {
    int32_t result;
    void *raw_object;
    void *raw_play;
    void *raw_queue;
    int32_t resolved_performance_mode;
} OboeRustOpenSLESOutputProperties;

typedef struct OboeRustOpenSLESInputProperties {
    int32_t result;
    void *raw_object;
    void *raw_record;
    void *raw_queue;
    int32_t resolved_performance_mode;
    int32_t resolved_input_preset;
} OboeRustOpenSLESInputProperties;

OboeRustOpenSLESOutputBackend *oboe_rust_opensles_output_open(
        const OboeRustOpenSLESPlatform *platform,
        const OboeRustOpenSLESOutputSettings *settings,
        OboeRustOpenSLESOutputProperties *properties);
OboeRustOpenSLESInputBackend *oboe_rust_opensles_input_open(
        const OboeRustOpenSLESPlatform *platform,
        const OboeRustOpenSLESInputSettings *settings,
        OboeRustOpenSLESInputProperties *properties);
int32_t oboe_rust_opensles_output_destroy(OboeRustOpenSLESOutputBackend *backend);
int32_t oboe_rust_opensles_input_destroy(OboeRustOpenSLESInputBackend *backend);
int32_t oboe_rust_opensles_output_set_play_state(OboeRustOpenSLESOutputBackend *backend,
                                                 int32_t state);
int32_t oboe_rust_opensles_input_set_record_state(OboeRustOpenSLESInputBackend *backend,
                                                  int32_t state);
int32_t oboe_rust_opensles_output_enqueue(OboeRustOpenSLESOutputBackend *backend,
                                          void *buffer,
                                          int32_t num_bytes);
int32_t oboe_rust_opensles_input_enqueue(OboeRustOpenSLESInputBackend *backend,
                                         void *buffer,
                                         int32_t num_bytes);
int32_t oboe_rust_opensles_output_clear_queue(OboeRustOpenSLESOutputBackend *backend);
int32_t oboe_rust_opensles_output_get_buffer_depth(OboeRustOpenSLESOutputBackend *backend);
int32_t oboe_rust_opensles_input_get_buffer_depth(OboeRustOpenSLESInputBackend *backend);
int32_t oboe_rust_opensles_output_get_position_millis(
        OboeRustOpenSLESOutputBackend *backend, int32_t *position_millis);
int32_t oboe_rust_opensles_input_get_position_millis(
        OboeRustOpenSLESInputBackend *backend, int32_t *position_millis);

int32_t oboe_rust_opensles_round_up_divide(int32_t x, int32_t n);
int32_t oboe_rust_opensles_channel_mask_default(int32_t channel_count,
                                                int32_t sdk_version,
                                                int32_t android_api_n,
                                                int32_t channel_count_max,
                                                int32_t unknown_channel_mask,
                                                int32_t non_positional_mask);
int32_t oboe_rust_opensles_input_channel_mask(int32_t channel_count,
                                              int32_t default_mask,
                                              int32_t front_center,
                                              int32_t front_left,
                                              int32_t front_right);
int32_t oboe_rust_opensles_output_channel_mask(int32_t channel_count,
                                               int32_t default_mask,
                                               int32_t front_center,
                                               int32_t stereo,
                                               int32_t quad,
                                               int32_t five_dot_one,
                                               int32_t seven_dot_one);
int32_t oboe_rust_opensles_optimal_buffer_queue_length(int32_t default_queue_length,
                                                       int32_t max_queue_length,
                                                       int32_t buffer_capacity_in_frames,
                                                       int32_t double_buffer_count,
                                                       int32_t frames_per_callback,
                                                       int32_t likely_frames_per_burst);
int32_t oboe_rust_opensles_estimate_native_frames_per_burst(
        int32_t default_frames_per_burst,
        int32_t default_sample_rate,
        int32_t stream_sample_rate,
        int32_t performance_mode,
        int32_t sdk_version,
        int32_t android_api_n_mr1,
        int32_t performance_mode_low_latency,
        int32_t high_latency_buffer_size_millis,
        int32_t millis_per_second);
int32_t oboe_rust_opensles_configured_callback_frames(int32_t frames_per_callback,
                                                      int32_t frames_per_burst);
int32_t oboe_rust_opensles_select_default_format(int32_t format,
                                                 int32_t sdk_version,
                                                 int32_t minimum_float_api,
                                                 int32_t i16_format,
                                                 int32_t float_format);
int32_t oboe_rust_opensles_convert_oboe_performance_mode(int32_t oboe_mode,
                                                         int32_t session_id,
                                                         int32_t session_id_none,
                                                         int32_t opensl_none,
                                                         int32_t opensl_latency,
                                                         int32_t opensl_latency_effects,
                                                         int32_t opensl_power_saving);
int32_t oboe_rust_opensles_convert_opensl_performance_mode(int32_t opensl_mode,
                                                           int32_t opensl_none,
                                                           int32_t opensl_latency,
                                                           int32_t opensl_latency_effects,
                                                           int32_t opensl_power_saving,
                                                           int32_t oboe_none,
                                                           int32_t oboe_low_latency,
                                                           int32_t oboe_power_saving);
int32_t oboe_rust_opensles_normalize_input_preset(int32_t input_preset,
                                                  int32_t voice_performance,
                                                  int32_t voice_recognition);
int32_t oboe_rust_opensles_convert_input_preset(int32_t input_preset,
                                                int32_t opensl_none,
                                                int32_t opensl_generic,
                                                int32_t opensl_camcorder,
                                                int32_t opensl_voice_recognition,
                                                int32_t opensl_voice_communication,
                                                int32_t opensl_unprocessed);
int32_t oboe_rust_opensles_convert_output_usage(int32_t usage,
                                                int32_t opensl_media,
                                                int32_t opensl_voice,
                                                int32_t opensl_alarm,
                                                int32_t opensl_notification,
                                                int32_t opensl_ring,
                                                int32_t opensl_system);
int64_t oboe_rust_opensles_output_position_millis(int64_t frames_read,
                                                  int32_t sample_rate,
                                                  int32_t millis_per_second);

#ifdef __cplusplus
}
#endif

#endif // OBOE_RUST_CORE_H
