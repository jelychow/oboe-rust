/*
 * Copyright 2016 The Android Open Source Project
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

#include <cassert>
#include <set>
#include <stdint.h>
#include <stdlib.h>

#include "aaudio/AAudioLoader.h"
#include "aaudio/AudioStreamAAudio.h"
#include "common/OboeDebug.h"
#include "oboe/AudioClock.h"
#include "oboe/Utilities.h"
#include "AAudioExtensions.h"
#if OBOE_USE_RUST_CORE
#include "rust/oboe_rust_core.h"
#endif

#ifdef __ANDROID__
#include <sys/system_properties.h>
#include <common/QuirksManager.h>

#endif

#ifndef OBOE_FIX_FORCE_STARTING_TO_STARTED
// Workaround state problems in AAudio
// TODO Which versions does this occur in? Verify fixed in Q.
#define OBOE_FIX_FORCE_STARTING_TO_STARTED 1
#endif // OBOE_FIX_FORCE_STARTING_TO_STARTED

using namespace oboe;
AAudioLoader *AudioStreamAAudio::mLibLoader = nullptr;

/**
 * A singleton class that manages all opened streams. This class is used to track the lifecycle
 * of aaudio streams. When a stream is opened successfully, it will be added to the collection.
 * When a stream is closed, it will be removed from the collection. This class also provides a
 * function to get a shared pointer from a given raw pointer. By using a shared pointer, it avoids
 * using after free. Note that if the stream is opened with raw pointer, there can still be used
 * after free issue happen as there is nothing preventing the raw pointer from being deleted.
 */
class AAudioStreamCollection {
public:
    static AAudioStreamCollection &getInstance() {
        static AAudioStreamCollection instance;
        return instance;
    }

    AAudioStreamCollection(const AAudioStreamCollection &) = delete;
    AAudioStreamCollection &operator=(const AAudioStreamCollection &) = delete;
    AAudioStreamCollection(AAudioStreamCollection &&) = delete;
    AAudioStreamCollection &operator=(AAudioStreamCollection &&) = delete;

    void addStream(AudioStreamAAudio* stream) {
        std::lock_guard<std::mutex> lock(mLock);
        mStreams.insert(stream);
    }

    void removeStream(AudioStreamAAudio* stream) {
        std::lock_guard<std::mutex> lock(mLock);
        mStreams.erase(stream);
    }

    /**
     * Get a shared pointer to the stream and its parent (if wrapped by FilterAudioStream).
     * This is typically called from a callback thread.
     *
     * The shared pointers are valid only if the stream is opened with shared pointer and is not closed.
     *
     * @param stream raw pointer to the stream.
     * @return tuple:
     *         - bool: indicates if the stream is present in the collection.
     *         - shared_ptr to the stream.
     *         - shared_ptr to the parent stream if wrapped by FilterAudioStream, nullptr otherwise.
     */
    std::tuple<bool,
            std::shared_ptr<oboe::AudioStream>,
            std::shared_ptr<oboe::AudioStream>>
    getStream(AudioStreamAAudio* stream) {
        if (stream == nullptr) {
            return {false, nullptr, nullptr};
        }
        std::lock_guard<std::mutex> lock(mLock);
        if (mStreams.find(stream) != mStreams.end()) {
            auto sharedStream = stream->lockWeakThis();

            // If wrapped by FilterAudioStream, the parent must remain alive because
            // callbacks are routed through it.
            std::shared_ptr<AudioStream> sharedParentStream;
            if (sharedStream && sharedStream->hasParentStream()) {
                sharedParentStream = sharedStream->getParentStream()->lockWeakThis();
            }
            return {true, sharedStream, sharedParentStream};
        }
        return {false, nullptr, nullptr};
    }

private:
    // Private constructor to prevent direct instantiation
    AAudioStreamCollection() = default;

    std::mutex mLock;
    std::set<AudioStreamAAudio*> mStreams;
};

// 'C' wrapper for the data callback method
static aaudio_data_callback_result_t oboe_aaudio_data_callback_proc(
        AAudioStream *stream,
        void *userData,
        void *audioData,
        int32_t numFrames) {

    AudioStreamAAudio *oboeStream = reinterpret_cast<AudioStreamAAudio*>(userData);
    auto [isStreamAlive, sharedStream, sharedParentStream] =
            AAudioStreamCollection::getInstance().getStream(oboeStream);
    if (!isStreamAlive) {
        // Note that the stream is removed from the collection when close is called. However,
        // there can be callback fired until the framework fully close the stream. In that case,
        // logging a warning here and quick return to stop the stream.
        LOGW("%s data callback while stream is not longer alive", __func__);
        return static_cast<aaudio_data_callback_result_t>(DataCallbackResult::Stop);
    }
    if (oboeStream != nullptr) {
        return static_cast<aaudio_data_callback_result_t>(
                oboeStream->callOnAudioReady(stream, audioData, numFrames));

    } else {
        return static_cast<aaudio_data_callback_result_t>(DataCallbackResult::Stop);
    }
}

// 'C' wrapper for the partial data callback method
static int32_t oboe_aaudio_partial_data_callback_proc(
        AAudioStream *stream,
        void *userData,
        void *audioData,
        int32_t numFrames) {
    AudioStreamAAudio *oboeStream = reinterpret_cast<AudioStreamAAudio*>(userData);
    auto [isStreamAlive, sharedStream, sharedParentStream] =
            AAudioStreamCollection::getInstance().getStream(oboeStream);
    if (!isStreamAlive) {
        // Note that the stream is removed from the collection when close is called. However,
        // there can be callback fired until the framework fully close the stream. In that case,
        // logging a warning here and return negative number for partial callback to stop the
        // stream.
        LOGW("%s data callback while stream is not longer alive", __func__);
        return -1;
    }
    if (oboeStream != nullptr) {
        return oboeStream->callOnPartialAudioReady(stream, audioData, numFrames);
    } else {
        // Return negative number to stop the stream.
        return -1;
    }
}

// This runs in its own thread.
// Only one of these threads will be launched from internalErrorCallback().
// It calls app error callbacks from a static function in case the stream gets deleted.
static void oboe_aaudio_error_thread_proc_common(AudioStreamAAudio *oboeStream,
                                          Result error) {
#if 0
    LOGE("%s() sleep for 5 seconds", __func__);
    usleep(5*1000*1000);
    LOGD("%s() - woke up -------------------------", __func__);
#endif
    AudioStreamErrorCallback *errorCallback = oboeStream->getErrorCallback();
    if (errorCallback == nullptr) return; // should be impossible
    bool isErrorHandled = errorCallback->onError(oboeStream, error);

    if (!isErrorHandled) {
        oboeStream->requestStop();
        errorCallback->onErrorBeforeClose(oboeStream, error);
        oboeStream->close();
        // Warning, oboeStream may get deleted by this callback.
        errorCallback->onErrorAfterClose(oboeStream, error);
    }
}

// Callback thread for raw pointers.
static void oboe_aaudio_error_thread_proc(AudioStreamAAudio *oboeStream,
                                          Result error) {
    LOGD("%s(,%d) - entering >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>", __func__, error);
    oboe_aaudio_error_thread_proc_common(oboeStream, error);
    LOGD("%s() - exiting <<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<", __func__);
}

// Callback thread for shared pointers.
static void oboe_aaudio_error_thread_proc_shared(std::shared_ptr<AudioStream> sharedStream,
                                          std::shared_ptr<AudioStream> sharedParentStream,
                                          Result error) {
    LOGD("%s(,%d) - entering >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>", __func__, error);
    // Hold the shared pointer(s) while we use the raw pointer.
    AudioStreamAAudio *oboeStream = reinterpret_cast<AudioStreamAAudio*>(sharedStream.get());
    oboe_aaudio_error_thread_proc_common(oboeStream, error);
    LOGD("%s() - exiting <<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<", __func__);
}

static void oboe_aaudio_presentation_thread_proc_common(AudioStreamAAudio *oboeStream) {
    auto presentationCallback = oboeStream->getPresentationCallback();
    if (presentationCallback == nullptr) return; // should be impossible
    presentationCallback->onPresentationEnded(oboeStream);
}

// Callback thread for raw pointers
static void oboe_aaudio_presentation_thread_proc(AudioStreamAAudio *oboeStream) {
    LOGD("%s() - entering >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>", __func__);
    oboe_aaudio_presentation_thread_proc_common(oboeStream);
    LOGD("%s() - exiting <<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<", __func__);
}

// Callback thread for shared pointers
static void oboe_aaudio_presentation_end_thread_proc_shared(
        std::shared_ptr<AudioStream> sharedStream,
        std::shared_ptr<AudioStream> sharedParentStream) {
    LOGD("%s() - entering >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>", __func__);
    AudioStreamAAudio *oboeStream = reinterpret_cast<AudioStreamAAudio*>(sharedStream.get());
    oboe_aaudio_presentation_thread_proc_common(oboeStream);
    LOGD("%s() - exiting <<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<", __func__);
}

static void oboe_aaudio_routing_changed_thread_proc_common(
        AudioStreamAAudio *oboeStream, const int32_t *deviceIds, int32_t numDevices) {
    auto routingCallback = oboeStream->getRoutingCallback();
    if (routingCallback == nullptr) return;
    routingCallback->onRoutingChanged(oboeStream, deviceIds, numDevices);
}

// Callback thread for raw pointers
static void oboe_aaudio_routing_changed_thread_proc(
        AudioStreamAAudio *oboeStream, std::vector<int32_t> deviceIds) {
    LOGD("%s() - entering >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>", __func__);
    oboe_aaudio_routing_changed_thread_proc_common(oboeStream, deviceIds.data(),
                                                   static_cast<int32_t>(deviceIds.size()));
    LOGD("%s() - exiting <<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<", __func__);
}

// Callback thread for shared pointers
static void oboe_aaudio_routing_changed_thread_proc_shared(
        std::shared_ptr<AudioStream> sharedStream,
        std::shared_ptr<AudioStream> sharedParentStream,
        std::vector<int32_t> deviceIds) {
    LOGD("%s() - entering >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>", __func__);
    AudioStreamAAudio *oboeStream = reinterpret_cast<AudioStreamAAudio*>(sharedStream.get());
    oboe_aaudio_routing_changed_thread_proc_common(oboeStream, deviceIds.data(),
                                                   static_cast<int32_t>(deviceIds.size()));
    LOGD("%s() - exiting <<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<", __func__);
}

#if OBOE_USE_RUST_CORE
static int32_t rust_aaudio_create_stream_builder(void **builder) {
    AAudioStreamBuilder *aaudioBuilder = nullptr;
    int32_t result = AAudioLoader::getInstance()->createStreamBuilder(&aaudioBuilder);
    *builder = aaudioBuilder;
    return result;
}

static int32_t rust_aaudio_builder_open_stream(void *builder, void **stream) {
    AAudioStream *aaudioStream = nullptr;
    int32_t result = AAudioLoader::getInstance()->builder_openStream(
            reinterpret_cast<AAudioStreamBuilder *>(builder), &aaudioStream);
    *stream = aaudioStream;
    return result;
}

static int32_t rust_aaudio_builder_delete(void *builder) {
    return AAudioLoader::getInstance()->builder_delete(
            reinterpret_cast<AAudioStreamBuilder *>(builder));
}

static void rust_aaudio_builder_set_buffer_capacity(void *builder, int32_t value) {
    AAudioLoader::getInstance()->builder_setBufferCapacityInFrames(
            reinterpret_cast<AAudioStreamBuilder *>(builder), value);
}

static void rust_aaudio_builder_set_channel_count(void *builder, int32_t value) {
    AAudioLoader::getInstance()->builder_setChannelCount(
            reinterpret_cast<AAudioStreamBuilder *>(builder), value);
}

static void rust_aaudio_builder_set_device_id(void *builder, int32_t value) {
    AAudioLoader::getInstance()->builder_setDeviceId(
            reinterpret_cast<AAudioStreamBuilder *>(builder), value);
}

static void rust_aaudio_builder_set_direction(void *builder, int32_t value) {
    AAudioLoader::getInstance()->builder_setDirection(
            reinterpret_cast<AAudioStreamBuilder *>(builder),
            static_cast<aaudio_direction_t>(value));
}

static void rust_aaudio_builder_set_format(void *builder, int32_t value) {
    AAudioLoader::getInstance()->builder_setFormat(
            reinterpret_cast<AAudioStreamBuilder *>(builder),
            static_cast<aaudio_format_t>(value));
}

static void rust_aaudio_builder_set_frames_per_data_callback(void *builder, int32_t value) {
    AAudioLoader::getInstance()->builder_setFramesPerDataCallback(
            reinterpret_cast<AAudioStreamBuilder *>(builder), value);
}

static void rust_aaudio_builder_set_performance_mode(void *builder, int32_t value) {
    AAudioLoader::getInstance()->builder_setPerformanceMode(
            reinterpret_cast<AAudioStreamBuilder *>(builder),
            static_cast<aaudio_performance_mode_t>(value));
}

static void rust_aaudio_builder_set_sample_rate(void *builder, int32_t value) {
    AAudioLoader::getInstance()->builder_setSampleRate(
            reinterpret_cast<AAudioStreamBuilder *>(builder), value);
}

static void rust_aaudio_builder_set_sharing_mode(void *builder, int32_t value) {
    AAudioLoader::getInstance()->builder_setSharingMode(
            reinterpret_cast<AAudioStreamBuilder *>(builder),
            static_cast<aaudio_sharing_mode_t>(value));
}

static void rust_aaudio_builder_set_channel_mask(void *builder, uint32_t value) {
    AAudioLoader::getInstance()->builder_setChannelMask(
            reinterpret_cast<AAudioStreamBuilder *>(builder),
            static_cast<aaudio_channel_mask_t>(value));
}

static void rust_aaudio_builder_set_usage(void *builder, int32_t value) {
    AAudioLoader::getInstance()->builder_setUsage(
            reinterpret_cast<AAudioStreamBuilder *>(builder),
            static_cast<aaudio_usage_t>(value));
}

static void rust_aaudio_builder_set_content_type(void *builder, int32_t value) {
    AAudioLoader::getInstance()->builder_setContentType(
            reinterpret_cast<AAudioStreamBuilder *>(builder),
            static_cast<aaudio_content_type_t>(value));
}

static void rust_aaudio_builder_set_input_preset(void *builder, int32_t value) {
    AAudioLoader::getInstance()->builder_setInputPreset(
            reinterpret_cast<AAudioStreamBuilder *>(builder),
            static_cast<aaudio_input_preset_t>(value));
}

static void rust_aaudio_builder_set_session_id(void *builder, int32_t value) {
    AAudioLoader::getInstance()->builder_setSessionId(
            reinterpret_cast<AAudioStreamBuilder *>(builder),
            static_cast<aaudio_session_id_t>(value));
}

static void rust_aaudio_builder_set_privacy_sensitive(void *builder, bool value) {
    AAudioLoader::getInstance()->builder_setPrivacySensitive(
            reinterpret_cast<AAudioStreamBuilder *>(builder), value);
}

static void rust_aaudio_builder_set_allowed_capture_policy(void *builder, int32_t value) {
    AAudioLoader::getInstance()->builder_setAllowedCapturePolicy(
            reinterpret_cast<AAudioStreamBuilder *>(builder),
            static_cast<aaudio_allowed_capture_policy_t>(value));
}

static void rust_aaudio_builder_set_package_name(void *builder, const char *value) {
    AAudioLoader::getInstance()->builder_setPackageName(
            reinterpret_cast<AAudioStreamBuilder *>(builder), value);
}

static void rust_aaudio_builder_set_attribution_tag(void *builder, const char *value) {
    AAudioLoader::getInstance()->builder_setAttributionTag(
            reinterpret_cast<AAudioStreamBuilder *>(builder), value);
}

static void rust_aaudio_builder_set_is_content_spatialized(void *builder, bool value) {
    AAudioLoader::getInstance()->builder_setIsContentSpatialized(
            reinterpret_cast<AAudioStreamBuilder *>(builder), value);
}

static void rust_aaudio_builder_set_spatialization_behavior(void *builder, int32_t value) {
    AAudioLoader::getInstance()->builder_setSpatializationBehavior(
            reinterpret_cast<AAudioStreamBuilder *>(builder),
            static_cast<aaudio_spatialization_behavior_t>(value));
}

static void rust_aaudio_builder_set_data_callback(
        void *builder, OboeRustAAudioDataCallback callback, void *userData) {
    AAudioLoader::getInstance()->builder_setDataCallback(
            reinterpret_cast<AAudioStreamBuilder *>(builder),
            reinterpret_cast<AAudioStream_dataCallback>(callback),
            userData);
}

static void rust_aaudio_builder_set_error_callback(
        void *builder, OboeRustAAudioErrorCallback callback, void *userData) {
    AAudioLoader::getInstance()->builder_setErrorCallback(
            reinterpret_cast<AAudioStreamBuilder *>(builder),
            reinterpret_cast<AAudioStream_errorCallback>(callback),
            userData);
}

static void rust_aaudio_builder_set_partial_data_callback(
        void *builder, OboeRustAAudioPartialDataCallback callback, void *userData) {
    AAudioLoader::getInstance()->builder_setPartialDataCallback(
            reinterpret_cast<AAudioStreamBuilder *>(builder),
            reinterpret_cast<AAudioStream_partialDataCallback>(callback),
            userData);
}

static void rust_aaudio_builder_set_presentation_end_callback(
        void *builder, OboeRustAAudioPresentationCallback callback, void *userData) {
    AAudioLoader::getInstance()->builder_setPresentationEndCallback(
            reinterpret_cast<AAudioStreamBuilder *>(builder),
            reinterpret_cast<AAudioStream_presentationEndCallback>(callback),
            userData);
}

static void rust_aaudio_builder_set_routing_changed_callback(
        void *builder, OboeRustAAudioRoutingChangedCallback callback, void *userData) {
    AAudioLoader::getInstance()->builder_setRoutingChangedCallback(
            reinterpret_cast<AAudioStreamBuilder *>(builder),
            reinterpret_cast<AAudioStream_routingChangedCallback>(callback),
            userData);
}

static int32_t rust_aaudio_stream_close(void *stream) {
    return AAudioLoader::getInstance()->stream_close(reinterpret_cast<AAudioStream *>(stream));
}

static int32_t rust_aaudio_stream_release(void *stream) {
    return AAudioLoader::getInstance()->stream_release(reinterpret_cast<AAudioStream *>(stream));
}

static int32_t rust_aaudio_stream_request_start(void *stream) {
    return AAudioLoader::getInstance()->stream_requestStart(reinterpret_cast<AAudioStream *>(stream));
}

static int32_t rust_aaudio_stream_request_pause(void *stream) {
    return AAudioLoader::getInstance()->stream_requestPause(reinterpret_cast<AAudioStream *>(stream));
}

static int32_t rust_aaudio_stream_request_flush(void *stream) {
    return AAudioLoader::getInstance()->stream_requestFlush(reinterpret_cast<AAudioStream *>(stream));
}

static int32_t rust_aaudio_stream_request_stop(void *stream) {
    return AAudioLoader::getInstance()->stream_requestStop(reinterpret_cast<AAudioStream *>(stream));
}

static int32_t rust_aaudio_stream_write(
        void *stream, const void *buffer, int32_t numFrames, int64_t timeoutNanoseconds) {
    return AAudioLoader::getInstance()->stream_write(
            reinterpret_cast<AAudioStream *>(stream), buffer, numFrames, timeoutNanoseconds);
}

static int32_t rust_aaudio_stream_read(
        void *stream, void *buffer, int32_t numFrames, int64_t timeoutNanoseconds) {
    return AAudioLoader::getInstance()->stream_read(
            reinterpret_cast<AAudioStream *>(stream), buffer, numFrames, timeoutNanoseconds);
}

static int32_t rust_aaudio_stream_wait_for_state_change(
        void *stream, int32_t currentState, int32_t *nextState, int64_t timeoutNanoseconds) {
    return AAudioLoader::getInstance()->stream_waitForStateChange(
            reinterpret_cast<AAudioStream *>(stream),
            static_cast<aaudio_stream_state_t>(currentState),
            reinterpret_cast<aaudio_stream_state_t *>(nextState),
            timeoutNanoseconds);
}

static int32_t rust_aaudio_stream_get_timestamp(
        void *stream, int32_t clockId, int64_t *framePosition, int64_t *timeNanoseconds) {
    return AAudioLoader::getInstance()->stream_getTimestamp(
            reinterpret_cast<AAudioStream *>(stream),
            static_cast<clockid_t>(clockId),
            framePosition,
            timeNanoseconds);
}

static int32_t rust_aaudio_stream_set_buffer_size(void *stream, int32_t requestedFrames) {
    return AAudioLoader::getInstance()->stream_setBufferSize(
            reinterpret_cast<AAudioStream *>(stream), requestedFrames);
}

static int32_t rust_aaudio_stream_get_channel_count(void *stream) {
    return AAudioLoader::getInstance()->stream_getChannelCount(reinterpret_cast<AAudioStream *>(stream));
}

static int32_t rust_aaudio_stream_get_device_id(void *stream) {
    return AAudioLoader::getInstance()->stream_getDeviceId(reinterpret_cast<AAudioStream *>(stream));
}

static int32_t rust_aaudio_stream_get_format(void *stream) {
    return static_cast<int32_t>(
            AAudioLoader::getInstance()->stream_getFormat(reinterpret_cast<AAudioStream *>(stream)));
}

static int32_t rust_aaudio_stream_get_sample_rate(void *stream) {
    return AAudioLoader::getInstance()->stream_getSampleRate(reinterpret_cast<AAudioStream *>(stream));
}

static int32_t rust_aaudio_stream_get_sharing_mode(void *stream) {
    return AAudioLoader::getInstance()->stream_getSharingMode(reinterpret_cast<AAudioStream *>(stream));
}

static int32_t rust_aaudio_stream_get_performance_mode(void *stream) {
    return AAudioLoader::getInstance()->stream_getPerformanceMode(reinterpret_cast<AAudioStream *>(stream));
}

static int32_t rust_aaudio_stream_get_buffer_capacity(void *stream) {
    return AAudioLoader::getInstance()->stream_getBufferCapacity(reinterpret_cast<AAudioStream *>(stream));
}

static int32_t rust_aaudio_stream_get_buffer_size(void *stream) {
    return AAudioLoader::getInstance()->stream_getBufferSize(reinterpret_cast<AAudioStream *>(stream));
}

static int32_t rust_aaudio_stream_get_frames_per_burst(void *stream) {
    return AAudioLoader::getInstance()->stream_getFramesPerBurst(reinterpret_cast<AAudioStream *>(stream));
}

static int32_t rust_aaudio_stream_get_state(void *stream) {
    return AAudioLoader::getInstance()->stream_getState(reinterpret_cast<AAudioStream *>(stream));
}

static int32_t rust_aaudio_stream_get_xrun_count(void *stream) {
    return AAudioLoader::getInstance()->stream_getXRunCount(reinterpret_cast<AAudioStream *>(stream));
}

static int64_t rust_aaudio_stream_get_frames_read(void *stream) {
    return AAudioLoader::getInstance()->stream_getFramesRead(reinterpret_cast<AAudioStream *>(stream));
}

static int64_t rust_aaudio_stream_get_frames_written(void *stream) {
    return AAudioLoader::getInstance()->stream_getFramesWritten(reinterpret_cast<AAudioStream *>(stream));
}

static int32_t rust_aaudio_stream_get_usage(void *stream) {
    return AAudioLoader::getInstance()->stream_getUsage(reinterpret_cast<AAudioStream *>(stream));
}

static int32_t rust_aaudio_stream_get_content_type(void *stream) {
    return AAudioLoader::getInstance()->stream_getContentType(reinterpret_cast<AAudioStream *>(stream));
}

static int32_t rust_aaudio_stream_get_input_preset(void *stream) {
    return AAudioLoader::getInstance()->stream_getInputPreset(reinterpret_cast<AAudioStream *>(stream));
}

static int32_t rust_aaudio_stream_get_session_id(void *stream) {
    return AAudioLoader::getInstance()->stream_getSessionId(reinterpret_cast<AAudioStream *>(stream));
}

static bool rust_aaudio_stream_is_privacy_sensitive(void *stream) {
    return AAudioLoader::getInstance()->stream_isPrivacySensitive(reinterpret_cast<AAudioStream *>(stream));
}

static int32_t rust_aaudio_stream_get_allowed_capture_policy(void *stream) {
    return AAudioLoader::getInstance()->stream_getAllowedCapturePolicy(reinterpret_cast<AAudioStream *>(stream));
}

static uint32_t rust_aaudio_stream_get_channel_mask(void *stream) {
    return AAudioLoader::getInstance()->stream_getChannelMask(reinterpret_cast<AAudioStream *>(stream));
}

static bool rust_aaudio_stream_is_content_spatialized(void *stream) {
    return AAudioLoader::getInstance()->stream_isContentSpatialized(reinterpret_cast<AAudioStream *>(stream));
}

static int32_t rust_aaudio_stream_get_spatialization_behavior(void *stream) {
    return AAudioLoader::getInstance()->stream_getSpatializationBehavior(reinterpret_cast<AAudioStream *>(stream));
}

static int32_t rust_aaudio_stream_get_hardware_channel_count(void *stream) {
    return AAudioLoader::getInstance()->stream_getHardwareChannelCount(reinterpret_cast<AAudioStream *>(stream));
}

static int32_t rust_aaudio_stream_get_hardware_sample_rate(void *stream) {
    return AAudioLoader::getInstance()->stream_getHardwareSampleRate(reinterpret_cast<AAudioStream *>(stream));
}

static int32_t rust_aaudio_stream_get_hardware_format(void *stream) {
    return static_cast<int32_t>(
            AAudioLoader::getInstance()->stream_getHardwareFormat(reinterpret_cast<AAudioStream *>(stream)));
}

static int32_t rust_aaudio_stream_set_offload_delay_padding(
        void *stream, int32_t delayInFrames, int32_t paddingInFrames) {
    return AAudioLoader::getInstance()->stream_setOffloadDelayPadding(
            reinterpret_cast<AAudioStream *>(stream), delayInFrames, paddingInFrames);
}

static int32_t rust_aaudio_stream_get_offload_delay(void *stream) {
    return AAudioLoader::getInstance()->stream_getOffloadDelay(
            reinterpret_cast<AAudioStream *>(stream));
}

static int32_t rust_aaudio_stream_get_offload_padding(void *stream) {
    return AAudioLoader::getInstance()->stream_getOffloadPadding(
            reinterpret_cast<AAudioStream *>(stream));
}

static int32_t rust_aaudio_stream_set_offload_end_of_stream(void *stream) {
    return AAudioLoader::getInstance()->stream_setOffloadEndOfStream(
            reinterpret_cast<AAudioStream *>(stream));
}

static int32_t rust_aaudio_stream_flush_from_frame(
        void *stream, int32_t accuracy, int64_t *positionInFrames) {
    return AAudioLoader::getInstance()->stream_flushFromFrame(
            reinterpret_cast<AAudioStream *>(stream), accuracy, positionInFrames);
}

static int32_t rust_aaudio_stream_get_playback_parameters(
        void *stream, OboeRustAAudioPlaybackParameters *parameters) {
    AAudioPlaybackParameters aaudioParameters;
    int32_t result = AAudioLoader::getInstance()->stream_getPlaybackParameters(
            reinterpret_cast<AAudioStream *>(stream), &aaudioParameters);
    if (result == AAUDIO_OK && parameters != nullptr) {
        parameters->fallback_mode = static_cast<int32_t>(aaudioParameters.fallbackMode);
        parameters->stretch_mode = static_cast<int32_t>(aaudioParameters.stretchMode);
        parameters->pitch = aaudioParameters.pitch;
        parameters->speed = aaudioParameters.speed;
    }
    return result;
}

static int32_t rust_aaudio_stream_set_playback_parameters(
        void *stream, const OboeRustAAudioPlaybackParameters *parameters) {
    AAudioPlaybackParameters aaudioParameters = {};
    if (parameters != nullptr) {
        aaudioParameters.fallbackMode = static_cast<AAudio_FallbackMode>(
                parameters->fallback_mode);
        aaudioParameters.stretchMode = static_cast<AAudio_StretchMode>(
                parameters->stretch_mode);
        aaudioParameters.pitch = parameters->pitch;
        aaudioParameters.speed = parameters->speed;
    }
    return AAudioLoader::getInstance()->stream_setPlaybackParameters(
            reinterpret_cast<AAudioStream *>(stream), &aaudioParameters);
}

static OboeRustAAudioPlatform makeRustAAudioPlatform(AAudioLoader *loader) {
    OboeRustAAudioPlatform platform = {};
    platform.create_stream_builder = rust_aaudio_create_stream_builder;
    platform.builder_open_stream = rust_aaudio_builder_open_stream;
    platform.builder_delete = rust_aaudio_builder_delete;
    platform.builder_set_buffer_capacity_in_frames = rust_aaudio_builder_set_buffer_capacity;
    platform.builder_set_channel_count = rust_aaudio_builder_set_channel_count;
    platform.builder_set_device_id = rust_aaudio_builder_set_device_id;
    platform.builder_set_direction = rust_aaudio_builder_set_direction;
    platform.builder_set_format = rust_aaudio_builder_set_format;
    platform.builder_set_frames_per_data_callback = rust_aaudio_builder_set_frames_per_data_callback;
    platform.builder_set_performance_mode = rust_aaudio_builder_set_performance_mode;
    platform.builder_set_sample_rate = rust_aaudio_builder_set_sample_rate;
    platform.builder_set_sharing_mode = rust_aaudio_builder_set_sharing_mode;
    if (loader->builder_setChannelMask != nullptr) {
        platform.builder_set_channel_mask = rust_aaudio_builder_set_channel_mask;
    }
    if (loader->builder_setUsage != nullptr) {
        platform.builder_set_usage = rust_aaudio_builder_set_usage;
    }
    if (loader->builder_setContentType != nullptr) {
        platform.builder_set_content_type = rust_aaudio_builder_set_content_type;
    }
    if (loader->builder_setInputPreset != nullptr) {
        platform.builder_set_input_preset = rust_aaudio_builder_set_input_preset;
    }
    if (loader->builder_setSessionId != nullptr) {
        platform.builder_set_session_id = rust_aaudio_builder_set_session_id;
    }
    if (loader->builder_setPrivacySensitive != nullptr) {
        platform.builder_set_privacy_sensitive = rust_aaudio_builder_set_privacy_sensitive;
    }
    if (loader->builder_setAllowedCapturePolicy != nullptr) {
        platform.builder_set_allowed_capture_policy = rust_aaudio_builder_set_allowed_capture_policy;
    }
    if (loader->builder_setPackageName != nullptr) {
        platform.builder_set_package_name = rust_aaudio_builder_set_package_name;
    }
    if (loader->builder_setAttributionTag != nullptr) {
        platform.builder_set_attribution_tag = rust_aaudio_builder_set_attribution_tag;
    }
    if (loader->builder_setIsContentSpatialized != nullptr) {
        platform.builder_set_is_content_spatialized = rust_aaudio_builder_set_is_content_spatialized;
    }
    if (loader->builder_setSpatializationBehavior != nullptr) {
        platform.builder_set_spatialization_behavior = rust_aaudio_builder_set_spatialization_behavior;
    }
    if (loader->builder_setDataCallback != nullptr) {
        platform.builder_set_data_callback = rust_aaudio_builder_set_data_callback;
    }
    if (loader->builder_setErrorCallback != nullptr) {
        platform.builder_set_error_callback = rust_aaudio_builder_set_error_callback;
    }
    if (loader->builder_setPartialDataCallback != nullptr) {
        platform.builder_set_partial_data_callback = rust_aaudio_builder_set_partial_data_callback;
    }
    if (loader->builder_setPresentationEndCallback != nullptr) {
        platform.builder_set_presentation_end_callback =
                rust_aaudio_builder_set_presentation_end_callback;
    }
    if (loader->builder_setRoutingChangedCallback != nullptr) {
        platform.builder_set_routing_changed_callback =
                rust_aaudio_builder_set_routing_changed_callback;
    }
    platform.stream_close = rust_aaudio_stream_close;
    if (loader->stream_release != nullptr) {
        platform.stream_release = rust_aaudio_stream_release;
    }
    platform.stream_request_start = rust_aaudio_stream_request_start;
    platform.stream_request_pause = rust_aaudio_stream_request_pause;
    platform.stream_request_flush = rust_aaudio_stream_request_flush;
    platform.stream_request_stop = rust_aaudio_stream_request_stop;
    platform.stream_write = rust_aaudio_stream_write;
    platform.stream_read = rust_aaudio_stream_read;
    platform.stream_wait_for_state_change = rust_aaudio_stream_wait_for_state_change;
    platform.stream_get_timestamp = rust_aaudio_stream_get_timestamp;
    platform.stream_set_buffer_size = rust_aaudio_stream_set_buffer_size;
    platform.stream_get_channel_count = rust_aaudio_stream_get_channel_count;
    platform.stream_get_device_id = rust_aaudio_stream_get_device_id;
    platform.stream_get_format = rust_aaudio_stream_get_format;
    platform.stream_get_sample_rate = rust_aaudio_stream_get_sample_rate;
    platform.stream_get_sharing_mode = rust_aaudio_stream_get_sharing_mode;
    platform.stream_get_performance_mode = rust_aaudio_stream_get_performance_mode;
    platform.stream_get_buffer_capacity = rust_aaudio_stream_get_buffer_capacity;
    platform.stream_get_buffer_size = rust_aaudio_stream_get_buffer_size;
    platform.stream_get_frames_per_burst = rust_aaudio_stream_get_frames_per_burst;
    platform.stream_get_state = rust_aaudio_stream_get_state;
    platform.stream_get_xrun_count = rust_aaudio_stream_get_xrun_count;
    platform.stream_get_frames_read = rust_aaudio_stream_get_frames_read;
    platform.stream_get_frames_written = rust_aaudio_stream_get_frames_written;
    if (loader->stream_getUsage != nullptr) {
        platform.stream_get_usage = rust_aaudio_stream_get_usage;
    }
    if (loader->stream_getContentType != nullptr) {
        platform.stream_get_content_type = rust_aaudio_stream_get_content_type;
    }
    if (loader->stream_getInputPreset != nullptr) {
        platform.stream_get_input_preset = rust_aaudio_stream_get_input_preset;
    }
    if (loader->stream_getSessionId != nullptr) {
        platform.stream_get_session_id = rust_aaudio_stream_get_session_id;
    }
    if (loader->stream_isPrivacySensitive != nullptr) {
        platform.stream_is_privacy_sensitive = rust_aaudio_stream_is_privacy_sensitive;
    }
    if (loader->stream_getAllowedCapturePolicy != nullptr) {
        platform.stream_get_allowed_capture_policy = rust_aaudio_stream_get_allowed_capture_policy;
    }
    if (loader->stream_getChannelMask != nullptr) {
        platform.stream_get_channel_mask = rust_aaudio_stream_get_channel_mask;
    }
    if (loader->stream_isContentSpatialized != nullptr) {
        platform.stream_is_content_spatialized = rust_aaudio_stream_is_content_spatialized;
    }
    if (loader->stream_getSpatializationBehavior != nullptr) {
        platform.stream_get_spatialization_behavior = rust_aaudio_stream_get_spatialization_behavior;
    }
    if (loader->stream_getHardwareChannelCount != nullptr) {
        platform.stream_get_hardware_channel_count = rust_aaudio_stream_get_hardware_channel_count;
    }
    if (loader->stream_getHardwareSampleRate != nullptr) {
        platform.stream_get_hardware_sample_rate = rust_aaudio_stream_get_hardware_sample_rate;
    }
    if (loader->stream_getHardwareFormat != nullptr) {
        platform.stream_get_hardware_format = rust_aaudio_stream_get_hardware_format;
    }
    if (loader->stream_setOffloadDelayPadding != nullptr) {
        platform.stream_set_offload_delay_padding =
                rust_aaudio_stream_set_offload_delay_padding;
    }
    if (loader->stream_getOffloadDelay != nullptr) {
        platform.stream_get_offload_delay = rust_aaudio_stream_get_offload_delay;
    }
    if (loader->stream_getOffloadPadding != nullptr) {
        platform.stream_get_offload_padding = rust_aaudio_stream_get_offload_padding;
    }
    if (loader->stream_setOffloadEndOfStream != nullptr) {
        platform.stream_set_offload_end_of_stream =
                rust_aaudio_stream_set_offload_end_of_stream;
    }
    if (loader->stream_flushFromFrame != nullptr) {
        platform.stream_flush_from_frame = rust_aaudio_stream_flush_from_frame;
    }
    if (loader->stream_getPlaybackParameters != nullptr) {
        platform.stream_get_playback_parameters = rust_aaudio_stream_get_playback_parameters;
    }
    if (loader->stream_setPlaybackParameters != nullptr) {
        platform.stream_set_playback_parameters = rust_aaudio_stream_set_playback_parameters;
    }
    return platform;
}
#endif

namespace oboe {

/*
 * Create a stream that uses Oboe Audio API.
 */
AudioStreamAAudio::AudioStreamAAudio(const AudioStreamBuilder &builder)
    : AudioStream(builder)
    , mAAudioStream(nullptr) {
    mCallbackThreadEnabled.store(false);
    mLibLoader = AAudioLoader::getInstance();
}

AudioStreamAAudio::~AudioStreamAAudio() {
#if OBOE_USE_RUST_CORE
    if (mRustAAudioOutputStream != nullptr) {
        oboe_rust_aaudio_output_destroy(mRustAAudioOutputStream);
        mRustAAudioOutputStream = nullptr;
        mAAudioStream.store(nullptr);
    }
    if (mRustAAudioInputStream != nullptr) {
        oboe_rust_aaudio_input_destroy(mRustAAudioInputStream);
        mRustAAudioInputStream = nullptr;
        mAAudioStream.store(nullptr);
    }
#endif
}

bool AudioStreamAAudio::isSupported() {
    mLibLoader = AAudioLoader::getInstance();
    int openResult = mLibLoader->open();
    return openResult == 0;
}

// Static method for the error callback.
// We use a method so we can access protected methods on the stream.
// Launch a thread to handle the error.
// That other thread can safely stop, close and delete the stream.
void AudioStreamAAudio::internalErrorCallback(
        AAudioStream *stream,
        void *userData,
        aaudio_result_t error) {
    oboe::Result oboeResult = static_cast<Result>(error);
    AudioStreamAAudio *oboeStream = reinterpret_cast<AudioStreamAAudio*>(userData);

    // Coerce the error code if needed to workaround a regression in RQ1A that caused
    // the wrong code to be passed when headsets plugged in. See b/173928197.
    if (OboeGlobals::areWorkaroundsEnabled()
            && getSdkVersion() == __ANDROID_API_R__
            && oboeResult == oboe::Result::ErrorTimeout) {
        oboeResult = oboe::Result::ErrorDisconnected;
        LOGD("%s() ErrorTimeout changed to ErrorDisconnected to fix b/173928197", __func__);
    }

    // Prevents deletion of the stream if the app is using AudioStreamBuilder::openStream(shared_ptr)
    auto [isStreamAlive, sharedStream, sharedParentStream] =
            AAudioStreamCollection::getInstance().getStream(oboeStream);
    if (!isStreamAlive) {
        // The stream is already closed. No need to call error callback.
        return;
    }

    oboeStream->mErrorCallbackResult = oboeResult;

    // These checks should be enough because we assume that the stream close()
    // will join() any active callback threads and will not allow new callbacks.
    if (oboeStream->wasErrorCallbackCalled()) { // block extra error callbacks
        LOGE("%s() multiple error callbacks called!", __func__);
    } else if (stream != oboeStream->getUnderlyingStream()) {
        LOGW("%s() stream already closed or closing", __func__); // might happen if there are bugs
    } else if (sharedStream) {
        // Handle error on a separate thread using shared pointer.
        std::thread t(oboe_aaudio_error_thread_proc_shared, sharedStream, sharedParentStream,
                      oboeResult);
        t.detach();
    } else {
        // Handle error on a separate thread.
        std::thread t(oboe_aaudio_error_thread_proc, oboeStream, oboeResult);
        t.detach();
    }
}

void AudioStreamAAudio::beginPerformanceHintInCallback() {
    if (isPerformanceHintEnabled()) {
        if (!mAdpfOpenAttempted) {
            int64_t targetDurationNanos = (mFramesPerBurst * 1e9) / getSampleRate();
            // This has to be called from the callback thread so we get the right TID.
            int adpfResult = mAdpfWrapper.open(gettid(), targetDurationNanos);
            if (adpfResult < 0) {
                LOGW("WARNING ADPF not supported, %d\n", adpfResult);
            } else {
                LOGD("ADPF is now active\n");
            }
            mAdpfOpenAttempted = true;
        }
        mAdpfWrapper.onBeginCallback();
    } else if (!isPerformanceHintEnabled() && mAdpfOpenAttempted) {
        LOGD("ADPF closed\n");
        mAdpfWrapper.close();
        mAdpfOpenAttempted = false;
    }
}

void AudioStreamAAudio::endPerformanceHintInCallback(int32_t numFrames) {
    if (mAdpfWrapper.isOpen()) {
        // Scale the measured duration based on numFrames so it is normalized to a full burst.
        double durationScaler = static_cast<double>(mFramesPerBurst) / numFrames;
        // Skip this callback if numFrames is very small.
        // This can happen when buffers wrap around, particularly when doing sample rate conversion.
        if (durationScaler < 2.0) {
            mAdpfWrapper.onEndCallback(durationScaler);
        }
    }
}

void AudioStreamAAudio::logUnsupportedAttributes() {
    int sdkVersion = getSdkVersion();

    // These attributes are not supported pre Android "P"
    if (sdkVersion < __ANDROID_API_P__) {
        if (mUsage != Usage::Media) {
            LOGW("Usage [AudioStreamBuilder::setUsage()] "
                 "is not supported on AAudio streams running on pre-Android P versions.");
        }

        if (mContentType != ContentType::Music) {
            LOGW("ContentType [AudioStreamBuilder::setContentType()] "
                 "is not supported on AAudio streams running on pre-Android P versions.");
        }

        if (mSessionId != SessionId::None) {
            LOGW("SessionId [AudioStreamBuilder::setSessionId()] "
                 "is not supported on AAudio streams running on pre-Android P versions.");
        }
    }
}

#if OBOE_USE_RUST_CORE
Result AudioStreamAAudio::openRustOutput() {
    if (mAAudioStream != nullptr || mRustAAudioOutputStream != nullptr) {
        return Result::ErrorInvalidState;
    }

    Result result = AudioStream::open();
    if (result != Result::OK) {
        return result;
    }

    logUnsupportedAttributes();

    if (mLibLoader->builder_setSessionId != nullptr) {
        mPerformanceMode = static_cast<PerformanceMode>(
                oboe_rust_aaudio_session_performance_mode(
                        static_cast<int32_t>(mPerformanceMode),
                        static_cast<int32_t>(mSessionId),
                        static_cast<int32_t>(SessionId::None),
                        static_cast<int32_t>(mDirection),
                        static_cast<int32_t>(Direction::Output),
                        static_cast<int32_t>(PerformanceMode::LowLatency),
                        static_cast<int32_t>(PerformanceMode::None),
                        OboeGlobals::areWorkaroundsEnabled()));
    }

    mSpatializationBehavior = static_cast<SpatializationBehavior>(
            oboe_rust_aaudio_spatialization_behavior(
                    static_cast<int32_t>(mSpatializationBehavior),
                    static_cast<int32_t>(SpatializationBehavior::Unspecified),
                    static_cast<int32_t>(SpatializationBehavior::Never),
                    mLibLoader->builder_setSpatializationBehavior != nullptr));

    if (anyDataCallbackSpecified() && !isErrorCallbackSpecified()) {
        mErrorCallback = &mDefaultErrorCallback;
    }
    if (isPartialDataCallbackSpecified() && mLibLoader->builder_setPartialDataCallback == nullptr) {
        LOGE("Using partial data callback while it is not available");
        return Result::ErrorIllegalArgument;
    }

    OboeRustAAudioPlatform platform = makeRustAAudioPlatform(mLibLoader);
    OboeRustAAudioOutputSettings settings = {};
    settings.direction = static_cast<int32_t>(mDirection);
    settings.device_id = AudioStreamBase::getDeviceId();
    settings.sample_rate = mSampleRate;
    settings.channel_count = mChannelCount;
    settings.channel_mask = static_cast<uint32_t>(mChannelMask);
    settings.format = static_cast<int32_t>(mFormat);
    settings.sharing_mode = static_cast<int32_t>(mSharingMode);
    settings.performance_mode = static_cast<int32_t>(mPerformanceMode);
    settings.buffer_capacity_in_frames = mBufferCapacityInFrames;
    settings.frames_per_data_callback = getFramesPerDataCallback();
    settings.session_id = static_cast<int32_t>(mSessionId);
    settings.usage = static_cast<int32_t>(mUsage);
    settings.content_type = static_cast<int32_t>(mContentType);
    settings.allowed_capture_policy = static_cast<int32_t>(mAllowedCapturePolicy);
    settings.is_content_spatialized = mIsContentSpatialized;
    settings.spatialization_behavior = static_cast<int32_t>(mSpatializationBehavior);
    settings.package_name = mPackageName.empty() ? nullptr : mPackageName.c_str();
    settings.attribution_tag = mAttributionTag.empty() ? nullptr : mAttributionTag.c_str();
    settings.data_callback = isDataCallbackSpecified()
            ? reinterpret_cast<OboeRustAAudioDataCallback>(oboe_aaudio_data_callback_proc)
            : nullptr;
    settings.error_callback = anyDataCallbackSpecified()
            ? reinterpret_cast<OboeRustAAudioErrorCallback>(internalErrorCallback)
            : nullptr;
    settings.partial_data_callback = isPartialDataCallbackSpecified()
            ? reinterpret_cast<OboeRustAAudioPartialDataCallback>(
                    oboe_aaudio_partial_data_callback_proc)
            : nullptr;
    settings.presentation_end_callback = isPresentationCallbackSpecified()
            ? reinterpret_cast<OboeRustAAudioPresentationCallback>(
                    internalPresentationEndCallback)
            : nullptr;
    settings.routing_changed_callback = mLibLoader->builder_setRoutingChangedCallback != nullptr
            ? reinterpret_cast<OboeRustAAudioRoutingChangedCallback>(
                    internalRoutingChangedCallback)
            : nullptr;
    settings.user_data = this;

    OboeRustAAudioOutputProperties properties = {};
    mRustAAudioOutputStream = oboe_rust_aaudio_output_open(&platform, &settings, &properties);
    result = static_cast<Result>(properties.result);
    if (mRustAAudioOutputStream == nullptr && result == Result::OK) {
        result = Result::ErrorInternal;
    }
    if (mRustAAudioOutputStream == nullptr || result != Result::OK) {
        if (static_cast<int32_t>(result) > 0) {
            result = static_cast<Result>(oboe_rust_aaudio_coerce_open_result(
                    static_cast<int32_t>(result),
                    OboeGlobals::areWorkaroundsEnabled(),
                    static_cast<int32_t>(Result::ErrorInternal)));
        }
        return result;
    }

    mAAudioStream.store(reinterpret_cast<AAudioStream *>(properties.raw_stream));
    mChannelCount = properties.channel_count;
    mSampleRate = properties.sample_rate;
    mFormat = static_cast<AudioFormat>(properties.format);
    mSharingMode = static_cast<SharingMode>(properties.sharing_mode);
    mPerformanceMode = static_cast<PerformanceMode>(properties.performance_mode);
    mBufferCapacityInFrames = properties.buffer_capacity_in_frames;
    mBufferSizeInFrames = properties.buffer_size_in_frames;
    mFramesPerBurst = properties.frames_per_burst;
    mUsage = static_cast<Usage>(properties.usage);
    mContentType = static_cast<ContentType>(properties.content_type);
    mInputPreset = static_cast<InputPreset>(properties.input_preset);
    mSessionId = static_cast<SessionId>(properties.session_id);
    mAllowedCapturePolicy = static_cast<AllowedCapturePolicy>(properties.allowed_capture_policy);
    mPrivacySensitiveMode = PrivacySensitiveMode::Unspecified;
    mChannelMask = static_cast<ChannelMask>(properties.channel_mask);
    mIsContentSpatialized = properties.is_content_spatialized;
    mSpatializationBehavior = static_cast<SpatializationBehavior>(properties.spatialization_behavior);
    mHardwareChannelCount = properties.hardware_channel_count;
    mHardwareSampleRate = properties.hardware_sample_rate;
    mHardwareFormat = static_cast<AudioFormat>(properties.hardware_format);

    updateDeviceIds();
    calculateDefaultDelayBeforeCloseMillis();
    AAudioStreamCollection::getInstance().addStream(this);

    LOGD("AudioStreamAAudio.openRustOutput() format=%d, sampleRate=%d, capacity=%d",
         static_cast<int>(mFormat), static_cast<int>(mSampleRate),
         static_cast<int>(mBufferCapacityInFrames));
    return Result::OK;
}

Result AudioStreamAAudio::openRustInput() {
    if (mAAudioStream != nullptr || mRustAAudioInputStream != nullptr) {
        return Result::ErrorInvalidState;
    }

    Result result = AudioStream::open();
    if (result != Result::OK) {
        return result;
    }

    logUnsupportedAttributes();

    int32_t capacity = mBufferCapacityInFrames;
    constexpr int kCapacityRequiredForFastLegacyTrack = 4096;
    int32_t adjustedCapacity = oboe_rust_aaudio_adjust_input_capacity(
            capacity,
            static_cast<int32_t>(mDirection),
            static_cast<int32_t>(Direction::Input),
            static_cast<int32_t>(mPerformanceMode),
            static_cast<int32_t>(PerformanceMode::LowLatency),
            Unspecified,
            kCapacityRequiredForFastLegacyTrack,
            OboeGlobals::areWorkaroundsEnabled());
    if (adjustedCapacity != capacity) {
        capacity = adjustedCapacity;
        LOGD("AudioStreamAAudio.openRustInput() capacity changed from %d to %d for lower latency",
             static_cast<int>(mBufferCapacityInFrames), capacity);
    }

    InputPreset inputPreset = mInputPreset;
    if (mLibLoader->builder_setInputPreset != nullptr) {
        InputPreset adjustedInputPreset = static_cast<InputPreset>(
                oboe_rust_aaudio_normalize_input_preset(
                        static_cast<int32_t>(inputPreset),
                        getSdkVersion(),
                        __ANDROID_API_P__,
                        static_cast<int32_t>(InputPreset::VoicePerformance),
                        static_cast<int32_t>(InputPreset::VoiceRecognition)));
        if (adjustedInputPreset != inputPreset) {
            LOGD("InputPreset::VoicePerformance not supported before Q. Using VoiceRecognition.");
            inputPreset = adjustedInputPreset;
        }
    }

    mSpatializationBehavior = static_cast<SpatializationBehavior>(
            oboe_rust_aaudio_spatialization_behavior(
                    static_cast<int32_t>(mSpatializationBehavior),
                    static_cast<int32_t>(SpatializationBehavior::Unspecified),
                    static_cast<int32_t>(SpatializationBehavior::Never),
                    mLibLoader->builder_setSpatializationBehavior != nullptr));

    if (anyDataCallbackSpecified() && !isErrorCallbackSpecified()) {
        mErrorCallback = &mDefaultErrorCallback;
    }
    if (isPartialDataCallbackSpecified() && mLibLoader->builder_setPartialDataCallback == nullptr) {
        LOGE("Using partial data callback while it is not available");
        return Result::ErrorIllegalArgument;
    }

    OboeRustAAudioPlatform platform = makeRustAAudioPlatform(mLibLoader);
    OboeRustAAudioInputSettings settings = {};
    settings.direction = static_cast<int32_t>(mDirection);
    settings.device_id = AudioStreamBase::getDeviceId();
    settings.sample_rate = mSampleRate;
    settings.channel_count = mChannelCount;
    settings.channel_mask = static_cast<uint32_t>(mChannelMask);
    settings.format = static_cast<int32_t>(mFormat);
    settings.sharing_mode = static_cast<int32_t>(mSharingMode);
    settings.performance_mode = static_cast<int32_t>(mPerformanceMode);
    settings.buffer_capacity_in_frames = capacity;
    settings.frames_per_data_callback = getFramesPerDataCallback();
    settings.input_preset = static_cast<int32_t>(inputPreset);
    settings.privacy_sensitive_mode = static_cast<int32_t>(mPrivacySensitiveMode);
    settings.privacy_sensitive_mode_unspecified =
            static_cast<int32_t>(PrivacySensitiveMode::Unspecified);
    settings.privacy_sensitive_mode_enabled =
            static_cast<int32_t>(PrivacySensitiveMode::Enabled);
    settings.privacy_sensitive_mode_disabled =
            static_cast<int32_t>(PrivacySensitiveMode::Disabled);
    settings.session_id = static_cast<int32_t>(mSessionId);
    settings.usage = static_cast<int32_t>(mUsage);
    settings.content_type = static_cast<int32_t>(mContentType);
    settings.allowed_capture_policy = static_cast<int32_t>(AllowedCapturePolicy::Unspecified);
    settings.package_name = mPackageName.empty() ? nullptr : mPackageName.c_str();
    settings.attribution_tag = mAttributionTag.empty() ? nullptr : mAttributionTag.c_str();
    settings.is_content_spatialized = mIsContentSpatialized;
    settings.spatialization_behavior = static_cast<int32_t>(mSpatializationBehavior);
    settings.data_callback = isDataCallbackSpecified()
            ? reinterpret_cast<OboeRustAAudioDataCallback>(oboe_aaudio_data_callback_proc)
            : nullptr;
    settings.error_callback = anyDataCallbackSpecified()
            ? reinterpret_cast<OboeRustAAudioErrorCallback>(internalErrorCallback)
            : nullptr;
    settings.partial_data_callback = isPartialDataCallbackSpecified()
            ? reinterpret_cast<OboeRustAAudioPartialDataCallback>(
                    oboe_aaudio_partial_data_callback_proc)
            : nullptr;
    settings.presentation_end_callback = isPresentationCallbackSpecified()
            ? reinterpret_cast<OboeRustAAudioPresentationCallback>(
                    internalPresentationEndCallback)
            : nullptr;
    settings.routing_changed_callback = mLibLoader->builder_setRoutingChangedCallback != nullptr
            ? reinterpret_cast<OboeRustAAudioRoutingChangedCallback>(
                    internalRoutingChangedCallback)
            : nullptr;
    settings.user_data = this;

    OboeRustAAudioInputProperties properties = {};
    mRustAAudioInputStream = oboe_rust_aaudio_input_open(&platform, &settings, &properties);
    result = static_cast<Result>(properties.result);
    if (mRustAAudioInputStream == nullptr && result == Result::OK) {
        result = Result::ErrorInternal;
    }
    if (mRustAAudioInputStream == nullptr || result != Result::OK) {
        if (mRustAAudioInputStream != nullptr) {
            oboe_rust_aaudio_input_destroy(mRustAAudioInputStream);
            mRustAAudioInputStream = nullptr;
        }
        if (result == Result::ErrorInternal) {
            LOGW("AudioStreamAAudio.openRustInput() may have failed due to lack of "
                 "audio recording permission.");
        }
        if (static_cast<int32_t>(result) > 0) {
            result = static_cast<Result>(oboe_rust_aaudio_coerce_open_result(
                    static_cast<int32_t>(result),
                    OboeGlobals::areWorkaroundsEnabled(),
                    static_cast<int32_t>(Result::ErrorInternal)));
        }
        return result;
    }

    mAAudioStream.store(reinterpret_cast<AAudioStream *>(properties.raw_stream));
    mChannelCount = properties.channel_count;
    mSampleRate = properties.sample_rate;
    mFormat = static_cast<AudioFormat>(properties.format);
    mSharingMode = static_cast<SharingMode>(properties.sharing_mode);
    mPerformanceMode = static_cast<PerformanceMode>(properties.performance_mode);
    mBufferCapacityInFrames = properties.buffer_capacity_in_frames;
    mBufferSizeInFrames = properties.buffer_size_in_frames;
    mFramesPerBurst = properties.frames_per_burst;
    mUsage = static_cast<Usage>(properties.usage);
    mContentType = static_cast<ContentType>(properties.content_type);
    mInputPreset = static_cast<InputPreset>(properties.input_preset);
    mSessionId = static_cast<SessionId>(properties.session_id);
    mAllowedCapturePolicy = static_cast<AllowedCapturePolicy>(properties.allowed_capture_policy);
    mPrivacySensitiveMode = static_cast<PrivacySensitiveMode>(properties.privacy_sensitive_mode);
    mChannelMask = static_cast<ChannelMask>(properties.channel_mask);
    mIsContentSpatialized = properties.is_content_spatialized;
    mSpatializationBehavior = static_cast<SpatializationBehavior>(properties.spatialization_behavior);
    mHardwareChannelCount = properties.hardware_channel_count;
    mHardwareSampleRate = properties.hardware_sample_rate;
    mHardwareFormat = static_cast<AudioFormat>(properties.hardware_format);

    updateDeviceIds();
    calculateDefaultDelayBeforeCloseMillis();
    AAudioStreamCollection::getInstance().addStream(this);

    LOGD("AudioStreamAAudio.openRustInput() format=%d, sampleRate=%d, capacity=%d",
         static_cast<int>(mFormat), static_cast<int>(mSampleRate),
         static_cast<int>(mBufferCapacityInFrames));
    return Result::OK;
}
#endif

Result AudioStreamAAudio::open() {
#if OBOE_USE_RUST_CORE
    if (mDirection == Direction::Output) {
        return openRustOutput();
    }
    if (mDirection == Direction::Input) {
        return openRustInput();
    }
#endif

    Result result = Result::OK;

    if (mAAudioStream != nullptr) {
        return Result::ErrorInvalidState;
    }

    result = AudioStream::open();
    if (result != Result::OK) {
        return result;
    }

    AAudioStreamBuilder *aaudioBuilder;
    result = static_cast<Result>(mLibLoader->createStreamBuilder(&aaudioBuilder));
    if (result != Result::OK) {
        return result;
    }

    // Do not set INPUT capacity below 4096 because that prevents us from getting a FAST track
    // when using the Legacy data path.
    // If the app requests > 4096 then we allow it but we are less likely to get LowLatency.
    // See internal bug b/80308183 for more details.
    // Fixed in Q but let's still clip the capacity because high input capacity
    // does not increase latency.
    int32_t capacity = mBufferCapacityInFrames;
    constexpr int kCapacityRequiredForFastLegacyTrack = 4096; // matches value in AudioFinger
#if OBOE_USE_RUST_CORE
    int32_t adjustedCapacity = oboe_rust_aaudio_adjust_input_capacity(
            capacity,
            static_cast<int32_t>(mDirection),
            static_cast<int32_t>(oboe::Direction::Input),
            static_cast<int32_t>(mPerformanceMode),
            static_cast<int32_t>(oboe::PerformanceMode::LowLatency),
            oboe::Unspecified,
            kCapacityRequiredForFastLegacyTrack,
            OboeGlobals::areWorkaroundsEnabled());
    if (adjustedCapacity != capacity) {
        capacity = adjustedCapacity;
        LOGD("AudioStreamAAudio.open() capacity changed from %d to %d for lower latency",
             static_cast<int>(mBufferCapacityInFrames), capacity);
    }
#else
    if (OboeGlobals::areWorkaroundsEnabled()
            && mDirection == oboe::Direction::Input
            && capacity != oboe::Unspecified
            && capacity < kCapacityRequiredForFastLegacyTrack
            && mPerformanceMode == oboe::PerformanceMode::LowLatency) {
        capacity = kCapacityRequiredForFastLegacyTrack;
        LOGD("AudioStreamAAudio.open() capacity changed from %d to %d for lower latency",
             static_cast<int>(mBufferCapacityInFrames), capacity);
    }
#endif
    mLibLoader->builder_setBufferCapacityInFrames(aaudioBuilder, capacity);

    if (mLibLoader->builder_setSessionId != nullptr) {
        mLibLoader->builder_setSessionId(aaudioBuilder,
                                         static_cast<aaudio_session_id_t>(mSessionId));
        // Output effects do not support PerformanceMode::LowLatency.
#if OBOE_USE_RUST_CORE
        PerformanceMode adjustedPerformanceMode = static_cast<PerformanceMode>(
                oboe_rust_aaudio_session_performance_mode(
                        static_cast<int32_t>(mPerformanceMode),
                        static_cast<int32_t>(mSessionId),
                        static_cast<int32_t>(SessionId::None),
                        static_cast<int32_t>(mDirection),
                        static_cast<int32_t>(oboe::Direction::Output),
                        static_cast<int32_t>(PerformanceMode::LowLatency),
                        static_cast<int32_t>(PerformanceMode::None),
                        OboeGlobals::areWorkaroundsEnabled()));
        if (adjustedPerformanceMode != mPerformanceMode) {
            mPerformanceMode = adjustedPerformanceMode;
            LOGD("AudioStreamAAudio.open() performance mode changed to None when session "
                 "id is requested");
        }
#else
        if (OboeGlobals::areWorkaroundsEnabled()
                && mSessionId != SessionId::None
                && mDirection == oboe::Direction::Output
                && mPerformanceMode == PerformanceMode::LowLatency) {
                    mPerformanceMode = PerformanceMode::None;
                    LOGD("AudioStreamAAudio.open() performance mode changed to None when session "
                         "id is requested");
        }
#endif
    }

    // Channel mask was added in SC_V2. Given the corresponding channel count of selected channel
    // mask may be different from selected channel count, the last set value will be respected.
    // If channel count is set after channel mask, the previously set channel mask will be cleared.
    // If channel mask is set after channel count, the channel count will be automatically
    // calculated from selected channel mask. In that case, only set channel mask when the API
    // is available and the channel mask is specified.
    if (mLibLoader->builder_setChannelMask != nullptr && mChannelMask != ChannelMask::Unspecified) {
        mLibLoader->builder_setChannelMask(aaudioBuilder,
                                           static_cast<aaudio_channel_mask_t>(mChannelMask));
    } else {
        mLibLoader->builder_setChannelCount(aaudioBuilder, mChannelCount);
    }
    mLibLoader->builder_setDeviceId(aaudioBuilder, AudioStreamBase::getDeviceId());
    mLibLoader->builder_setDirection(aaudioBuilder, static_cast<aaudio_direction_t>(mDirection));
    mLibLoader->builder_setFormat(aaudioBuilder, static_cast<aaudio_format_t>(mFormat));
    mLibLoader->builder_setSampleRate(aaudioBuilder, mSampleRate);
    mLibLoader->builder_setSharingMode(aaudioBuilder,
                                       static_cast<aaudio_sharing_mode_t>(mSharingMode));
    mLibLoader->builder_setPerformanceMode(aaudioBuilder,
                                           static_cast<aaudio_performance_mode_t>(mPerformanceMode));

    // These were added in P so we have to check for the function pointer.
    if (mLibLoader->builder_setUsage != nullptr) {
        mLibLoader->builder_setUsage(aaudioBuilder,
                                     static_cast<aaudio_usage_t>(mUsage));
    }

    if (mLibLoader->builder_setContentType != nullptr) {
        mLibLoader->builder_setContentType(aaudioBuilder,
                                           static_cast<aaudio_content_type_t>(mContentType));
    }

    if (mLibLoader->builder_setInputPreset != nullptr) {
        aaudio_input_preset_t inputPreset = mInputPreset;
#if OBOE_USE_RUST_CORE
        aaudio_input_preset_t adjustedInputPreset = static_cast<aaudio_input_preset_t>(
                oboe_rust_aaudio_normalize_input_preset(
                        static_cast<int32_t>(inputPreset),
                        getSdkVersion(),
                        __ANDROID_API_P__,
                        static_cast<int32_t>(InputPreset::VoicePerformance),
                        static_cast<int32_t>(InputPreset::VoiceRecognition)));
        if (adjustedInputPreset != inputPreset) {
            LOGD("InputPreset::VoicePerformance not supported before Q. Using VoiceRecognition.");
            inputPreset = adjustedInputPreset;
        }
#else
        if (getSdkVersion() <= __ANDROID_API_P__ && inputPreset == InputPreset::VoicePerformance) {
            LOGD("InputPreset::VoicePerformance not supported before Q. Using VoiceRecognition.");
            inputPreset = InputPreset::VoiceRecognition; // most similar preset
        }
#endif
        mLibLoader->builder_setInputPreset(aaudioBuilder,
                                           static_cast<aaudio_input_preset_t>(inputPreset));
    }

    // These were added in S so we have to check for the function pointer.
    if (mLibLoader->builder_setPackageName != nullptr && !mPackageName.empty()) {
        mLibLoader->builder_setPackageName(aaudioBuilder,
                                           mPackageName.c_str());
    }

    if (mLibLoader->builder_setAttributionTag != nullptr && !mAttributionTag.empty()) {
        mLibLoader->builder_setAttributionTag(aaudioBuilder,
                                           mAttributionTag.c_str());
    }

    // This was added in Q so we have to check for the function pointer.
    if (mLibLoader->builder_setAllowedCapturePolicy != nullptr && mDirection == oboe::Direction::Output) {
        mLibLoader->builder_setAllowedCapturePolicy(aaudioBuilder,
                                           static_cast<aaudio_allowed_capture_policy_t>(mAllowedCapturePolicy));
    }

    if (mLibLoader->builder_setPrivacySensitive != nullptr && mDirection == oboe::Direction::Input
            && mPrivacySensitiveMode != PrivacySensitiveMode::Unspecified) {
        mLibLoader->builder_setPrivacySensitive(aaudioBuilder,
                mPrivacySensitiveMode == PrivacySensitiveMode::Enabled);
    }

    if (mLibLoader->builder_setIsContentSpatialized != nullptr) {
        mLibLoader->builder_setIsContentSpatialized(aaudioBuilder, mIsContentSpatialized);
    }

    if (mLibLoader->builder_setSpatializationBehavior != nullptr) {
        // Override Unspecified as Never to reduce latency.
        mSpatializationBehavior = static_cast<SpatializationBehavior>(
#if OBOE_USE_RUST_CORE
                oboe_rust_aaudio_spatialization_behavior(
                        static_cast<int32_t>(mSpatializationBehavior),
                        static_cast<int32_t>(SpatializationBehavior::Unspecified),
                        static_cast<int32_t>(SpatializationBehavior::Never),
                        true)
#else
                (mSpatializationBehavior == SpatializationBehavior::Unspecified)
                        ? static_cast<int32_t>(SpatializationBehavior::Never)
                        : static_cast<int32_t>(mSpatializationBehavior)
#endif
        );
        mLibLoader->builder_setSpatializationBehavior(aaudioBuilder,
                static_cast<aaudio_spatialization_behavior_t>(mSpatializationBehavior));
    } else {
        mSpatializationBehavior = static_cast<SpatializationBehavior>(
#if OBOE_USE_RUST_CORE
                oboe_rust_aaudio_spatialization_behavior(
                        static_cast<int32_t>(mSpatializationBehavior),
                        static_cast<int32_t>(SpatializationBehavior::Unspecified),
                        static_cast<int32_t>(SpatializationBehavior::Never),
                        false)
#else
                SpatializationBehavior::Never
#endif
        );
    }

    if (anyDataCallbackSpecified()) {
        if (isDataCallbackSpecified()) {
            mLibLoader->builder_setDataCallback(
                    aaudioBuilder, oboe_aaudio_data_callback_proc, this);
        } else if (isPartialDataCallbackSpecified()) {
            if (mLibLoader->builder_setPartialDataCallback == nullptr) {
                // This must not happen. The stream should fail open from the builder.
                // But having a check here to avoid crashing.
                LOGE("Using partial data callback while it is not available");
                return Result::ErrorIllegalArgument;
            }
            mLibLoader->builder_setPartialDataCallback(
                    aaudioBuilder, oboe_aaudio_partial_data_callback_proc, this);
        }
        mLibLoader->builder_setFramesPerDataCallback(aaudioBuilder, getFramesPerDataCallback());

        if (!isErrorCallbackSpecified()) {
            // The app did not specify a callback so we should specify
            // our own so the stream gets closed and stopped.
            mErrorCallback = &mDefaultErrorCallback;
        }
        mLibLoader->builder_setErrorCallback(aaudioBuilder, internalErrorCallback, this);
    }
    // Else if the data callback is not being used then the write method will return an error
    // and the app can stop and close the stream.

    if (isPresentationCallbackSpecified() &&
        mLibLoader->builder_setPresentationEndCallback != nullptr) {
        mLibLoader->builder_setPresentationEndCallback(aaudioBuilder,
                                                       internalPresentationEndCallback,
                                                       this);
    }

    if (mLibLoader->builder_setRoutingChangedCallback != nullptr) {
        mLibLoader->builder_setRoutingChangedCallback(aaudioBuilder,
                                                      internalRoutingChangedCallback,
                                                      this);
    }

    // ============= OPEN THE STREAM ================
    {
        AAudioStream *stream = nullptr;
        result = static_cast<Result>(mLibLoader->builder_openStream(aaudioBuilder, &stream));
        mAAudioStream.store(stream);
    }
    if (result != Result::OK) {
        // Warn developer because ErrorInternal is not very informative.
        if (result == Result::ErrorInternal && mDirection == Direction::Input) {
            LOGW("AudioStreamAAudio.open() may have failed due to lack of "
                 "audio recording permission.");
        }
        goto error2;
    }

    // Query and cache the stream properties
    mChannelCount = mLibLoader->stream_getChannelCount(mAAudioStream);
    mSampleRate = mLibLoader->stream_getSampleRate(mAAudioStream);
    mFormat = static_cast<AudioFormat>(mLibLoader->stream_getFormat(mAAudioStream));
    mSharingMode = static_cast<SharingMode>(mLibLoader->stream_getSharingMode(mAAudioStream));
    mPerformanceMode = static_cast<PerformanceMode>(
            mLibLoader->stream_getPerformanceMode(mAAudioStream));
    mBufferCapacityInFrames = mLibLoader->stream_getBufferCapacity(mAAudioStream);
    mBufferSizeInFrames = mLibLoader->stream_getBufferSize(mAAudioStream);
    mFramesPerBurst = mLibLoader->stream_getFramesPerBurst(mAAudioStream);

    // These were added in P so we have to check for the function pointer.
    if (mLibLoader->stream_getUsage != nullptr) {
        mUsage = static_cast<Usage>(mLibLoader->stream_getUsage(mAAudioStream));
    }
    if (mLibLoader->stream_getContentType != nullptr) {
        mContentType = static_cast<ContentType>(mLibLoader->stream_getContentType(mAAudioStream));
    }
    if (mLibLoader->stream_getInputPreset != nullptr) {
        mInputPreset = static_cast<InputPreset>(mLibLoader->stream_getInputPreset(mAAudioStream));
    }
    if (mLibLoader->stream_getSessionId != nullptr) {
        mSessionId = static_cast<SessionId>(mLibLoader->stream_getSessionId(mAAudioStream));
    } else {
        mSessionId = SessionId::None;
    }

    // This was added in Q so we have to check for the function pointer.
    if (mLibLoader->stream_getAllowedCapturePolicy != nullptr && mDirection == oboe::Direction::Output) {
        mAllowedCapturePolicy = static_cast<AllowedCapturePolicy>(mLibLoader->stream_getAllowedCapturePolicy(mAAudioStream));
    } else {
        mAllowedCapturePolicy = AllowedCapturePolicy::Unspecified;
    }

    if (mLibLoader->stream_isPrivacySensitive != nullptr && mDirection == oboe::Direction::Input) {
        bool isPrivacySensitive = mLibLoader->stream_isPrivacySensitive(mAAudioStream);
        mPrivacySensitiveMode = isPrivacySensitive ? PrivacySensitiveMode::Enabled :
                PrivacySensitiveMode::Disabled;
    } else {
        mPrivacySensitiveMode = PrivacySensitiveMode::Unspecified;
    }

    if (mLibLoader->stream_getChannelMask != nullptr) {
        mChannelMask = static_cast<ChannelMask>(mLibLoader->stream_getChannelMask(mAAudioStream));
    }

    if (mLibLoader->stream_isContentSpatialized != nullptr) {
        mIsContentSpatialized = mLibLoader->stream_isContentSpatialized(mAAudioStream);
    }

    if (mLibLoader->stream_getSpatializationBehavior != nullptr) {
        mSpatializationBehavior = static_cast<SpatializationBehavior>(
                mLibLoader->stream_getSpatializationBehavior(mAAudioStream));
    }

    if (mLibLoader->stream_getHardwareChannelCount != nullptr) {
        mHardwareChannelCount = mLibLoader->stream_getHardwareChannelCount(mAAudioStream);
    }
    if (mLibLoader->stream_getHardwareSampleRate != nullptr) {
        mHardwareSampleRate = mLibLoader->stream_getHardwareSampleRate(mAAudioStream);
    }
    if (mLibLoader->stream_getHardwareFormat != nullptr) {
        mHardwareFormat = static_cast<AudioFormat>(mLibLoader->stream_getHardwareFormat(mAAudioStream));
    }

    updateDeviceIds();

    LOGD("AudioStreamAAudio.open() format=%d, sampleRate=%d, capacity = %d",
            static_cast<int>(mFormat), static_cast<int>(mSampleRate),
            static_cast<int>(mBufferCapacityInFrames));

    calculateDefaultDelayBeforeCloseMillis();

error2:
    mLibLoader->builder_delete(aaudioBuilder);
    if (static_cast<int>(result) > 0) {
        // Possibly due to b/267531411
        LOGW("AudioStreamAAudio.open: AAudioStream_Open() returned positive error = %d",
             static_cast<int>(result));
#if OBOE_USE_RUST_CORE
        result = static_cast<Result>(oboe_rust_aaudio_coerce_open_result(
                static_cast<int32_t>(result),
                OboeGlobals::areWorkaroundsEnabled(),
                static_cast<int32_t>(Result::ErrorInternal)));
#else
        if (OboeGlobals::areWorkaroundsEnabled()) {
            result = Result::ErrorInternal; // Coerce to negative error.
        }
#endif
    } else {
        LOGD("AudioStreamAAudio.open: AAudioStream_Open() returned %s = %d",
             mLibLoader->convertResultToText(static_cast<aaudio_result_t>(result)),
             static_cast<int>(result));
        if (result == Result::OK) {
            // Only add the stream to collection when successfully open.
            AAudioStreamCollection::getInstance().addStream(this);
        }
    }
    return result;
}

Result AudioStreamAAudio::release() {
    if (getSdkVersion() < __ANDROID_API_R__) {
        return Result::ErrorUnimplemented;
    }

    // AAudioStream_release() is buggy on Android R.
    if (OboeGlobals::areWorkaroundsEnabled() && getSdkVersion() == __ANDROID_API_R__) {
        LOGW("Skipping release() on Android R");
        return Result::ErrorUnimplemented;
    }

    std::lock_guard<std::mutex> lock(mLock);
#if OBOE_USE_RUST_CORE
    if (mRustAAudioOutputStream != nullptr) {
        if (OboeGlobals::areWorkaroundsEnabled()) {
            requestStop_l(mAAudioStream.load());
        }
        return static_cast<Result>(oboe_rust_aaudio_output_release(mRustAAudioOutputStream));
    }
    if (mRustAAudioInputStream != nullptr) {
        if (OboeGlobals::areWorkaroundsEnabled()) {
            requestStop_l(mAAudioStream.load());
        }
        return static_cast<Result>(oboe_rust_aaudio_input_release(mRustAAudioInputStream));
    }
#endif
    AAudioStream *stream = mAAudioStream.load();
    if (stream != nullptr) {
        if (OboeGlobals::areWorkaroundsEnabled()) {
            // Make sure we are really stopped. Do it under mLock
            // so another thread cannot call requestStart() right before the close.
            requestStop_l(stream);
        }
        return static_cast<Result>(mLibLoader->stream_release(stream));
    } else {
        return Result::ErrorClosed;
    }
}

Result AudioStreamAAudio::close() {
#if OBOE_USE_RUST_CORE
    if (mRustAAudioOutputStream != nullptr) {
        LOGD("%s(Rust output)", __func__);
        AAudioStreamCollection::getInstance().removeStream(this);
        std::lock_guard<std::mutex> lock(mLock);
        AudioStream::close();
        {
            std::unique_lock<std::shared_mutex> lock2(mAAudioStreamLock);
            mAAudioStream.exchange(nullptr);
        }
        if (OboeGlobals::areWorkaroundsEnabled()) {
            oboe_rust_aaudio_output_request_stop(mRustAAudioOutputStream);
            sleepBeforeClose();
        }
        Result closeResult = static_cast<Result>(
                oboe_rust_aaudio_output_close(mRustAAudioOutputStream));
        Result destroyResult = static_cast<Result>(
                oboe_rust_aaudio_output_destroy(mRustAAudioOutputStream));
        mRustAAudioOutputStream = nullptr;
        return closeResult == Result::OK ? destroyResult : closeResult;
    }
    if (mRustAAudioInputStream != nullptr) {
        LOGD("%s(Rust input)", __func__);
        AAudioStreamCollection::getInstance().removeStream(this);
        std::lock_guard<std::mutex> lock(mLock);
        AudioStream::close();
        {
            std::unique_lock<std::shared_mutex> lock2(mAAudioStreamLock);
            mAAudioStream.exchange(nullptr);
        }
        if (OboeGlobals::areWorkaroundsEnabled()) {
            oboe_rust_aaudio_input_request_stop(mRustAAudioInputStream);
            sleepBeforeClose();
        }
        Result closeResult = static_cast<Result>(
                oboe_rust_aaudio_input_close(mRustAAudioInputStream));
        Result destroyResult = static_cast<Result>(
                oboe_rust_aaudio_input_destroy(mRustAAudioInputStream));
        mRustAAudioInputStream = nullptr;
        return closeResult == Result::OK ? destroyResult : closeResult;
    }
#endif

    LOGD("%s", __func__);
    // Always remove the stream from the collection before closing it as after closing, the client
    // will free the resource of the stream.
    AAudioStreamCollection::getInstance().removeStream(this);

    // Prevent two threads from closing the stream at the same time and crashing.
    // This could occur, for example, if an application called close() at the same
    // time that an onError callback was being executed because of a disconnect.
    std::lock_guard<std::mutex> lock(mLock);

    AudioStream::close();

    AAudioStream *stream = nullptr;
    {
        // Wait for any methods using mAAudioStream to finish.
        std::unique_lock<std::shared_mutex> lock2(mAAudioStreamLock);
        // Closing will delete *mAAudioStream so we need to null out the pointer atomically.
        stream = mAAudioStream.exchange(nullptr);
    }
    if (stream != nullptr) {
        if (OboeGlobals::areWorkaroundsEnabled()) {
            // Make sure we are really stopped. Do it under mLock
            // so another thread cannot call requestStart() right before the close.
            requestStop_l(stream);
            sleepBeforeClose();
        }
        return static_cast<Result>(mLibLoader->stream_close(stream));
    } else {
        return Result::ErrorClosed;
    }
}

static void oboe_stop_thread_proc(AudioStream *oboeStream) {
    if (oboeStream != nullptr) {
        oboeStream->requestStop();
    }
}

void AudioStreamAAudio::launchStopThread() {
    // Prevent multiple stop threads from being launched.
    if (mStopThreadAllowed.exchange(false)) {
        // Stop this stream on a separate thread
        std::thread t(oboe_stop_thread_proc, this);
        t.detach();
    }
}

DataCallbackResult AudioStreamAAudio::callOnAudioReady(AAudioStream * /*stream*/,
                                                       void *audioData,
                                                       int32_t numFrames) {
    DataCallbackResult result = fireDataCallback(audioData, numFrames);
    if (result == DataCallbackResult::Continue) {
        return result;
    } else {
        if (result == DataCallbackResult::Stop) {
            LOGD("Oboe callback returned DataCallbackResult::Stop");
        } else {
            LOGE("Oboe callback returned unexpected value. Error: %d", static_cast<int>(result));
        }

        const bool shouldLaunchStopThread =
#if OBOE_USE_RUST_CORE
                oboe_rust_aaudio_callback_should_launch_stop_thread(
                        static_cast<int32_t>(result),
                        OboeGlobals::areWorkaroundsEnabled(),
                        getSdkVersion(),
                        __ANDROID_API_R__);
#else
                OboeGlobals::areWorkaroundsEnabled() && getSdkVersion() <= __ANDROID_API_R__;
#endif
        // Returning Stop caused various problems before S. See #1230
        if (shouldLaunchStopThread) {
            launchStopThread();
        }
        return static_cast<DataCallbackResult>(
#if OBOE_USE_RUST_CORE
                oboe_rust_aaudio_callback_return_result(
                        static_cast<int32_t>(result),
                        OboeGlobals::areWorkaroundsEnabled(),
                        getSdkVersion(),
                        __ANDROID_API_R__)
#else
                shouldLaunchStopThread
                        ? static_cast<int32_t>(DataCallbackResult::Continue)
                        : static_cast<int32_t>(DataCallbackResult::Stop)
#endif
        );
    }
}

int32_t AudioStreamAAudio::callOnPartialAudioReady(AAudioStream * /*stream*/,
                                                   void *audioData,
                                                   int32_t numFrames) {
    return firePartialDataCallback(audioData, numFrames);
}

Result AudioStreamAAudio::requestStart() {
#if OBOE_USE_RUST_CORE
    if (mRustAAudioOutputStream != nullptr) {
        std::lock_guard<std::mutex> lock(mLock);
        if (anyDataCallbackSpecified()) {
            setDataCallbackEnabled(true);
        }
        mStopThreadAllowed = true;
        closePerformanceHint();
        return static_cast<Result>(
                oboe_rust_aaudio_output_request_start(mRustAAudioOutputStream));
    }
    if (mRustAAudioInputStream != nullptr) {
        std::lock_guard<std::mutex> lock(mLock);
        if (anyDataCallbackSpecified()) {
            setDataCallbackEnabled(true);
        }
        mStopThreadAllowed = true;
        closePerformanceHint();
        return static_cast<Result>(
                oboe_rust_aaudio_input_request_start(mRustAAudioInputStream));
    }
#endif

    std::lock_guard<std::mutex> lock(mLock);
    AAudioStream *stream = mAAudioStream.load();
    if (stream != nullptr) {
        // Avoid state machine errors in O_MR1.
        if (getSdkVersion() <= __ANDROID_API_O_MR1__) {
            StreamState state = static_cast<StreamState>(mLibLoader->stream_getState(stream));
            if (
#if OBOE_USE_RUST_CORE
                    oboe_rust_aaudio_request_already_satisfied(
                            getSdkVersion(),
                            __ANDROID_API_O_MR1__,
                            static_cast<int32_t>(state),
                            static_cast<int32_t>(StreamState::Starting),
                            static_cast<int32_t>(StreamState::Started))
#else
                    state == StreamState::Starting || state == StreamState::Started
#endif
            ) {
                // WARNING: On P, AAudio is returning ErrorInvalidState for Output and OK for Input.
                return Result::OK;
            }
        }
        if (anyDataCallbackSpecified()) {
            setDataCallbackEnabled(true);
        }
        mStopThreadAllowed = true;
        closePerformanceHint();
        return static_cast<Result>(mLibLoader->stream_requestStart(stream));
    } else {
        return Result::ErrorClosed;
    }
}

Result AudioStreamAAudio::requestPause() {
#if OBOE_USE_RUST_CORE
    if (mRustAAudioOutputStream != nullptr) {
        std::lock_guard<std::mutex> lock(mLock);
        return static_cast<Result>(
                oboe_rust_aaudio_output_request_pause(mRustAAudioOutputStream));
    }
    if (mRustAAudioInputStream != nullptr) {
        std::lock_guard<std::mutex> lock(mLock);
        return static_cast<Result>(
                oboe_rust_aaudio_input_request_pause(mRustAAudioInputStream));
    }
#endif

    std::lock_guard<std::mutex> lock(mLock);
    AAudioStream *stream = mAAudioStream.load();
    if (stream != nullptr) {
        // Avoid state machine errors in O_MR1.
        if (getSdkVersion() <= __ANDROID_API_O_MR1__) {
            StreamState state = static_cast<StreamState>(mLibLoader->stream_getState(stream));
            if (
#if OBOE_USE_RUST_CORE
                    oboe_rust_aaudio_request_already_satisfied(
                            getSdkVersion(),
                            __ANDROID_API_O_MR1__,
                            static_cast<int32_t>(state),
                            static_cast<int32_t>(StreamState::Pausing),
                            static_cast<int32_t>(StreamState::Paused))
#else
                    state == StreamState::Pausing || state == StreamState::Paused
#endif
            ) {
                return Result::OK;
            }
        }
        return static_cast<Result>(mLibLoader->stream_requestPause(stream));
    } else {
        return Result::ErrorClosed;
    }
}

Result AudioStreamAAudio::requestFlush() {
#if OBOE_USE_RUST_CORE
    if (mRustAAudioOutputStream != nullptr) {
        std::lock_guard<std::mutex> lock(mLock);
        return static_cast<Result>(
                oboe_rust_aaudio_output_request_flush(mRustAAudioOutputStream));
    }
    if (mRustAAudioInputStream != nullptr) {
        std::lock_guard<std::mutex> lock(mLock);
        return static_cast<Result>(
                oboe_rust_aaudio_input_request_flush(mRustAAudioInputStream));
    }
#endif

    std::lock_guard<std::mutex> lock(mLock);
    AAudioStream *stream = mAAudioStream.load();
    if (stream != nullptr) {
        // Avoid state machine errors in O_MR1.
        if (getSdkVersion() <= __ANDROID_API_O_MR1__) {
            StreamState state = static_cast<StreamState>(mLibLoader->stream_getState(stream));
            if (
#if OBOE_USE_RUST_CORE
                    oboe_rust_aaudio_request_already_satisfied(
                            getSdkVersion(),
                            __ANDROID_API_O_MR1__,
                            static_cast<int32_t>(state),
                            static_cast<int32_t>(StreamState::Flushing),
                            static_cast<int32_t>(StreamState::Flushed))
#else
                    state == StreamState::Flushing || state == StreamState::Flushed
#endif
            ) {
                return Result::OK;
            }
        }
        return static_cast<Result>(mLibLoader->stream_requestFlush(stream));
    } else {
        return Result::ErrorClosed;
    }
}

Result AudioStreamAAudio::requestStop() {
#if OBOE_USE_RUST_CORE
    if (mRustAAudioOutputStream != nullptr) {
        std::lock_guard<std::mutex> lock(mLock);
        return static_cast<Result>(
                oboe_rust_aaudio_output_request_stop(mRustAAudioOutputStream));
    }
    if (mRustAAudioInputStream != nullptr) {
        std::lock_guard<std::mutex> lock(mLock);
        return static_cast<Result>(
                oboe_rust_aaudio_input_request_stop(mRustAAudioInputStream));
    }
#endif

    std::lock_guard<std::mutex> lock(mLock);
    AAudioStream *stream = mAAudioStream.load();
    if (stream != nullptr) {
        return requestStop_l(stream);
    } else {
        return Result::ErrorClosed;
    }
}

// Call under mLock
Result AudioStreamAAudio::requestStop_l(AAudioStream *stream) {
#if OBOE_USE_RUST_CORE
    if (mRustAAudioOutputStream != nullptr) {
        (void) stream;
        return static_cast<Result>(
                oboe_rust_aaudio_output_request_stop(mRustAAudioOutputStream));
    }
    if (mRustAAudioInputStream != nullptr) {
        (void) stream;
        return static_cast<Result>(
                oboe_rust_aaudio_input_request_stop(mRustAAudioInputStream));
    }
#endif

    // Avoid state machine errors in O_MR1.
    if (getSdkVersion() <= __ANDROID_API_O_MR1__) {
        StreamState state = static_cast<StreamState>(mLibLoader->stream_getState(stream));
        if (
#if OBOE_USE_RUST_CORE
                oboe_rust_aaudio_request_already_satisfied(
                        getSdkVersion(),
                        __ANDROID_API_O_MR1__,
                        static_cast<int32_t>(state),
                        static_cast<int32_t>(StreamState::Stopping),
                        static_cast<int32_t>(StreamState::Stopped))
#else
                state == StreamState::Stopping || state == StreamState::Stopped
#endif
        ) {
            return Result::OK;
        }
    }
    return static_cast<Result>(mLibLoader->stream_requestStop(stream));
}

ResultWithValue<int32_t>   AudioStreamAAudio::write(const void *buffer,
                                     int32_t numFrames,
                                     int64_t timeoutNanoseconds) {
#if OBOE_USE_RUST_CORE
    if (mRustAAudioOutputStream != nullptr) {
        std::shared_lock<std::shared_mutex> lock(mAAudioStreamLock);
        int32_t result = oboe_rust_aaudio_output_write(
                mRustAAudioOutputStream, buffer, numFrames, timeoutNanoseconds);
        return ResultWithValue<int32_t>::createBasedOnSign(result);
    }
    if (mRustAAudioInputStream != nullptr) {
        std::shared_lock<std::shared_mutex> lock(mAAudioStreamLock);
        int32_t result = oboe_rust_aaudio_input_write(
                mRustAAudioInputStream, buffer, numFrames, timeoutNanoseconds);
        return ResultWithValue<int32_t>::createBasedOnSign(result);
    }
#endif

    std::shared_lock<std::shared_mutex> lock(mAAudioStreamLock);
    AAudioStream *stream = mAAudioStream.load();
    if (stream != nullptr) {
        int32_t result = mLibLoader->stream_write(mAAudioStream, buffer,
                                                  numFrames, timeoutNanoseconds);
        return ResultWithValue<int32_t>::createBasedOnSign(result);
    } else {
        return ResultWithValue<int32_t>(Result::ErrorClosed);
    }
}

ResultWithValue<int32_t>   AudioStreamAAudio::read(void *buffer,
                                 int32_t numFrames,
                                 int64_t timeoutNanoseconds) {
#if OBOE_USE_RUST_CORE
    if (mRustAAudioOutputStream != nullptr) {
        std::shared_lock<std::shared_mutex> lock(mAAudioStreamLock);
        int32_t result = oboe_rust_aaudio_output_read(
                mRustAAudioOutputStream, buffer, numFrames, timeoutNanoseconds);
        return ResultWithValue<int32_t>::createBasedOnSign(result);
    }
    if (mRustAAudioInputStream != nullptr) {
        std::shared_lock<std::shared_mutex> lock(mAAudioStreamLock);
        int32_t result = oboe_rust_aaudio_input_read(
                mRustAAudioInputStream, buffer, numFrames, timeoutNanoseconds);
        return ResultWithValue<int32_t>::createBasedOnSign(result);
    }
#endif

    std::shared_lock<std::shared_mutex> lock(mAAudioStreamLock);
    AAudioStream *stream = mAAudioStream.load();
    if (stream != nullptr) {
        int32_t result = mLibLoader->stream_read(mAAudioStream, buffer,
                                                 numFrames, timeoutNanoseconds);
        return ResultWithValue<int32_t>::createBasedOnSign(result);
    } else {
        return ResultWithValue<int32_t>(Result::ErrorClosed);
    }
}


// AAudioStream_waitForStateChange() can crash if it is waiting on a stream and that stream
// is closed from another thread.  We do not want to lock the stream for the duration of the call.
// So we call AAudioStream_waitForStateChange() with a timeout of zero so that it will not block.
// Then we can do our own sleep with the lock unlocked.
Result AudioStreamAAudio::waitForStateChange(StreamState currentState,
                                        StreamState *nextState,
                                        int64_t timeoutNanoseconds) {
#if OBOE_USE_RUST_CORE
    if (mRustAAudioOutputStream != nullptr) {
        int32_t rustNextState = nextState == nullptr
                ? static_cast<int32_t>(StreamState::Unknown)
                : static_cast<int32_t>(*nextState);
        Result result = static_cast<Result>(
                oboe_rust_aaudio_output_wait_for_state_change(
                        mRustAAudioOutputStream,
                        static_cast<int32_t>(currentState),
                        nextState == nullptr ? nullptr : &rustNextState,
                        timeoutNanoseconds));
        if (nextState != nullptr) {
            *nextState = static_cast<StreamState>(rustNextState);
        }
        return result;
    }
    if (mRustAAudioInputStream != nullptr) {
        int32_t rustNextState = nextState == nullptr
                ? static_cast<int32_t>(StreamState::Unknown)
                : static_cast<int32_t>(*nextState);
        Result result = static_cast<Result>(
                oboe_rust_aaudio_input_wait_for_state_change(
                        mRustAAudioInputStream,
                        static_cast<int32_t>(currentState),
                        nextState == nullptr ? nullptr : &rustNextState,
                        timeoutNanoseconds));
        if (nextState != nullptr) {
            *nextState = static_cast<StreamState>(rustNextState);
        }
        return result;
    }
#endif

    Result oboeResult = Result::ErrorTimeout;
    int64_t sleepTimeNanos = 20 * kNanosPerMillisecond; // arbitrary
    aaudio_stream_state_t currentAAudioState = static_cast<aaudio_stream_state_t>(currentState);

    aaudio_result_t result = AAUDIO_OK;
    int64_t timeLeftNanos = timeoutNanoseconds;

    mLock.lock();
    while (true) {
        // Do we still have an AAudio stream? If not then stream must have been closed.
        AAudioStream *stream = mAAudioStream.load();
        if (stream == nullptr) {
            if (nextState != nullptr) {
                *nextState = StreamState::Closed;
            }
            oboeResult = Result::ErrorClosed;
            break;
        }

        // Update and query state change with no blocking.
        aaudio_stream_state_t aaudioNextState;
        result = mLibLoader->stream_waitForStateChange(
                mAAudioStream,
                currentAAudioState,
                &aaudioNextState,
                0); // timeout=0 for non-blocking
        // AAudio will return AAUDIO_ERROR_TIMEOUT if timeout=0 and the state does not change.
        if (result != AAUDIO_OK && result != AAUDIO_ERROR_TIMEOUT) {
            oboeResult = static_cast<Result>(result);
            break;
        }
#if OBOE_FIX_FORCE_STARTING_TO_STARTED
#if OBOE_USE_RUST_CORE
        aaudioNextState = static_cast<aaudio_stream_state_t>(
                oboe_rust_aaudio_force_starting_to_started(
                        OboeGlobals::areWorkaroundsEnabled(),
                        static_cast<int32_t>(aaudioNextState),
                        static_cast<int32_t>(StreamState::Starting),
                        static_cast<int32_t>(StreamState::Started)));
#else
        if (OboeGlobals::areWorkaroundsEnabled()
            && aaudioNextState == static_cast<aaudio_stream_state_t >(StreamState::Starting)) {
            aaudioNextState = static_cast<aaudio_stream_state_t >(StreamState::Started);
        }
#endif
#endif // OBOE_FIX_FORCE_STARTING_TO_STARTED
        if (nextState != nullptr) {
            *nextState = static_cast<StreamState>(aaudioNextState);
        }
        if (currentAAudioState != aaudioNextState) { // state changed?
            oboeResult = Result::OK;
            break;
        }

        // Did we timeout or did user ask for non-blocking?
        if (timeLeftNanos <= 0) {
            break;
        }

        // No change yet so sleep.
        mLock.unlock(); // Don't sleep while locked.
        if (sleepTimeNanos > timeLeftNanos) {
            sleepTimeNanos = timeLeftNanos; // last little bit
        }
        AudioClock::sleepForNanos(sleepTimeNanos);
        timeLeftNanos -= sleepTimeNanos;
        mLock.lock();
    }

    mLock.unlock();
    return oboeResult;
}

ResultWithValue<int32_t> AudioStreamAAudio::setBufferSizeInFrames(int32_t requestedFrames) {
    int32_t adjustedFrames = requestedFrames;
    if (adjustedFrames > mBufferCapacityInFrames) {
        adjustedFrames = mBufferCapacityInFrames;
    }
    // This calls getBufferSize() so avoid recursive lock.
    adjustedFrames = QuirksManager::getInstance().clipBufferSize(*this, adjustedFrames);

#if OBOE_USE_RUST_CORE
    if (mRustAAudioOutputStream != nullptr) {
        std::shared_lock<std::shared_mutex> lock(mAAudioStreamLock);
        int32_t newBufferSize = oboe_rust_aaudio_output_set_buffer_size(
                mRustAAudioOutputStream, adjustedFrames);
        if (newBufferSize > 0) mBufferSizeInFrames = newBufferSize;
        return ResultWithValue<int32_t>::createBasedOnSign(newBufferSize);
    }
    if (mRustAAudioInputStream != nullptr) {
        std::shared_lock<std::shared_mutex> lock(mAAudioStreamLock);
        int32_t newBufferSize = oboe_rust_aaudio_input_set_buffer_size(
                mRustAAudioInputStream, adjustedFrames);
        if (newBufferSize > 0) mBufferSizeInFrames = newBufferSize;
        return ResultWithValue<int32_t>::createBasedOnSign(newBufferSize);
    }
#endif

    std::shared_lock<std::shared_mutex> lock(mAAudioStreamLock);
    AAudioStream *stream = mAAudioStream.load();
    if (stream != nullptr) {
        int32_t newBufferSize = mLibLoader->stream_setBufferSize(mAAudioStream, adjustedFrames);
        // Cache the result if it's valid
        if (newBufferSize > 0) mBufferSizeInFrames = newBufferSize;
        return ResultWithValue<int32_t>::createBasedOnSign(newBufferSize);
    } else {
        return ResultWithValue<int32_t>(Result::ErrorClosed);
    }
}

StreamState AudioStreamAAudio::getState() {
#if OBOE_USE_RUST_CORE
    if (mRustAAudioOutputStream != nullptr) {
        std::shared_lock<std::shared_mutex> lock(mAAudioStreamLock);
        return static_cast<StreamState>(
                oboe_rust_aaudio_output_get_state(mRustAAudioOutputStream));
    }
    if (mRustAAudioInputStream != nullptr) {
        std::shared_lock<std::shared_mutex> lock(mAAudioStreamLock);
        return static_cast<StreamState>(
                oboe_rust_aaudio_input_get_state(mRustAAudioInputStream));
    }
#endif

    std::shared_lock<std::shared_mutex> lock(mAAudioStreamLock);
    AAudioStream *stream = mAAudioStream.load();
    if (stream != nullptr) {
        aaudio_stream_state_t aaudioState = mLibLoader->stream_getState(stream);
#if OBOE_FIX_FORCE_STARTING_TO_STARTED
#if OBOE_USE_RUST_CORE
        aaudioState = static_cast<aaudio_stream_state_t>(
                oboe_rust_aaudio_force_starting_to_started(
                        OboeGlobals::areWorkaroundsEnabled(),
                        static_cast<int32_t>(aaudioState),
                        static_cast<int32_t>(AAUDIO_STREAM_STATE_STARTING),
                        static_cast<int32_t>(AAUDIO_STREAM_STATE_STARTED)));
#else
        if (OboeGlobals::areWorkaroundsEnabled()
            && aaudioState == AAUDIO_STREAM_STATE_STARTING) {
            aaudioState = AAUDIO_STREAM_STATE_STARTED;
        }
#endif
#endif // OBOE_FIX_FORCE_STARTING_TO_STARTED
        return static_cast<StreamState>(aaudioState);
    } else {
        return StreamState::Closed;
    }
}

void AudioStreamAAudio::onRoutingChanged(std::vector<int32_t> deviceIds) {
    int nextIdx = mUpdatedDeviceIds.idx.load() ^ 1;
    mUpdatedDeviceIds.deviceIds[nextIdx] = deviceIds;
    mUpdatedDeviceIds.idx.store(nextIdx);
}

int32_t AudioStreamAAudio::getDeviceId() const {
    auto deviceIds = mUpdatedDeviceIds.deviceIds[mUpdatedDeviceIds.idx.load()];
    return deviceIds.empty() ? kUnspecified : deviceIds[0];
}

std::vector<int32_t> AudioStreamAAudio::getDeviceIds() const {
    auto deviceIds = mUpdatedDeviceIds.deviceIds[mUpdatedDeviceIds.idx.load()];
    return deviceIds;
}

int32_t AudioStreamAAudio::getBufferSizeInFrames() {
#if OBOE_USE_RUST_CORE
    if (mRustAAudioOutputStream != nullptr) {
        std::shared_lock<std::shared_mutex> lock(mAAudioStreamLock);
        mBufferSizeInFrames = oboe_rust_aaudio_output_get_buffer_size(mRustAAudioOutputStream);
        return mBufferSizeInFrames;
    }
    if (mRustAAudioInputStream != nullptr) {
        std::shared_lock<std::shared_mutex> lock(mAAudioStreamLock);
        mBufferSizeInFrames = oboe_rust_aaudio_input_get_buffer_size(mRustAAudioInputStream);
        return mBufferSizeInFrames;
    }
#endif

    std::shared_lock<std::shared_mutex> lock(mAAudioStreamLock);
    AAudioStream *stream = mAAudioStream.load();
    if (stream != nullptr) {
        mBufferSizeInFrames = mLibLoader->stream_getBufferSize(stream);
    }
    return mBufferSizeInFrames;
}

void AudioStreamAAudio::updateFramesRead() {
#if OBOE_USE_RUST_CORE
    if (mRustAAudioOutputStream != nullptr) {
        std::shared_lock<std::shared_mutex> lock(mAAudioStreamLock);
        mFramesRead = oboe_rust_aaudio_output_get_frames_read(mRustAAudioOutputStream);
        return;
    }
    if (mRustAAudioInputStream != nullptr) {
        std::shared_lock<std::shared_mutex> lock(mAAudioStreamLock);
        mFramesRead = oboe_rust_aaudio_input_get_frames_read(mRustAAudioInputStream);
        return;
    }
#endif

    std::shared_lock<std::shared_mutex> lock(mAAudioStreamLock);
    AAudioStream *stream = mAAudioStream.load();
// Set to 1 for debugging race condition #1180 with mAAudioStream.
// See also DEBUG_CLOSE_RACE in OboeTester.
// This was left in the code so that we could test the fix again easily in the future.
// We could not trigger the race condition without adding these get calls and the sleeps.
#define DEBUG_CLOSE_RACE 0
#if DEBUG_CLOSE_RACE
    // This is used when testing race conditions with close().
    // See DEBUG_CLOSE_RACE in OboeTester
    AudioClock::sleepForNanos(400 * kNanosPerMillisecond);
#endif // DEBUG_CLOSE_RACE
    if (stream != nullptr) {
        mFramesRead = mLibLoader->stream_getFramesRead(stream);
    }
}

void AudioStreamAAudio::updateFramesWritten() {
#if OBOE_USE_RUST_CORE
    if (mRustAAudioOutputStream != nullptr) {
        std::shared_lock<std::shared_mutex> lock(mAAudioStreamLock);
        mFramesWritten = oboe_rust_aaudio_output_get_frames_written(mRustAAudioOutputStream);
        return;
    }
    if (mRustAAudioInputStream != nullptr) {
        std::shared_lock<std::shared_mutex> lock(mAAudioStreamLock);
        mFramesWritten = oboe_rust_aaudio_input_get_frames_written(mRustAAudioInputStream);
        return;
    }
#endif

    std::shared_lock<std::shared_mutex> lock(mAAudioStreamLock);
    AAudioStream *stream = mAAudioStream.load();
    if (stream != nullptr) {
        mFramesWritten = mLibLoader->stream_getFramesWritten(stream);
    }
}

ResultWithValue<int32_t> AudioStreamAAudio::getXRunCount() {
#if OBOE_USE_RUST_CORE
    if (mRustAAudioOutputStream != nullptr) {
        std::shared_lock<std::shared_mutex> lock(mAAudioStreamLock);
        int32_t result = oboe_rust_aaudio_output_get_xrun_count(mRustAAudioOutputStream);
        return ResultWithValue<int32_t>::createBasedOnSign(result);
    }
    if (mRustAAudioInputStream != nullptr) {
        std::shared_lock<std::shared_mutex> lock(mAAudioStreamLock);
        int32_t result = oboe_rust_aaudio_input_get_xrun_count(mRustAAudioInputStream);
        return ResultWithValue<int32_t>::createBasedOnSign(result);
    }
#endif

    std::shared_lock<std::shared_mutex> lock(mAAudioStreamLock);
    AAudioStream *stream = mAAudioStream.load();
    if (stream != nullptr) {
        return ResultWithValue<int32_t>::createBasedOnSign(mLibLoader->stream_getXRunCount(stream));
    } else {
        return ResultWithValue<int32_t>(Result::ErrorNull);
    }
}

Result AudioStreamAAudio::getTimestamp(clockid_t clockId,
                                   int64_t *framePosition,
                                   int64_t *timeNanoseconds) {
    if (getState() != StreamState::Started) {
        return Result::ErrorInvalidState;
    }
#if OBOE_USE_RUST_CORE
    if (mRustAAudioOutputStream != nullptr) {
        std::shared_lock<std::shared_mutex> lock(mAAudioStreamLock);
        return static_cast<Result>(
                oboe_rust_aaudio_output_get_timestamp(
                        mRustAAudioOutputStream,
                        static_cast<int32_t>(clockId),
                        framePosition,
                        timeNanoseconds));
    }
    if (mRustAAudioInputStream != nullptr) {
        std::shared_lock<std::shared_mutex> lock(mAAudioStreamLock);
        return static_cast<Result>(
                oboe_rust_aaudio_input_get_timestamp(
                        mRustAAudioInputStream,
                        static_cast<int32_t>(clockId),
                        framePosition,
                        timeNanoseconds));
    }
#endif

    std::shared_lock<std::shared_mutex> lock(mAAudioStreamLock);
    AAudioStream *stream = mAAudioStream.load();
    if (stream != nullptr) {
        return static_cast<Result>(mLibLoader->stream_getTimestamp(stream, clockId,
                                               framePosition, timeNanoseconds));
    } else {
        return Result::ErrorNull;
    }
}

ResultWithValue<double> AudioStreamAAudio::calculateLatencyMillis() {
    // Get the time that a known audio frame was presented.
    int64_t hardwareFrameIndex;
    int64_t hardwareFrameHardwareTime;
    auto result = getTimestamp(CLOCK_MONOTONIC,
                               &hardwareFrameIndex,
                               &hardwareFrameHardwareTime);
    if (result != oboe::Result::OK) {
        return ResultWithValue<double>(static_cast<Result>(result));
    }

    // Get counter closest to the app.
    bool isOutput = (getDirection() == oboe::Direction::Output);
    int64_t appFrameIndex = isOutput ? getFramesWritten() : getFramesRead();

    // Assume that the next frame will be processed at the current time
    using namespace std::chrono;
    int64_t appFrameAppTime =
            duration_cast<nanoseconds>(steady_clock::now().time_since_epoch()).count();

#if OBOE_USE_RUST_CORE
    double latencyMillis = oboe_rust_aaudio_calculate_latency_millis(
            isOutput,
            appFrameIndex,
            hardwareFrameIndex,
            appFrameAppTime,
            hardwareFrameHardwareTime,
            getSampleRate(),
            oboe::kNanosPerSecond,
            kNanosPerMillisecond);
#else
    // Calculate the number of frames between app and hardware
    int64_t frameIndexDelta = appFrameIndex - hardwareFrameIndex;

    // Calculate the time which the next frame will be or was presented
    int64_t frameTimeDelta = (frameIndexDelta * oboe::kNanosPerSecond) / getSampleRate();
    int64_t appFrameHardwareTime = hardwareFrameHardwareTime + frameTimeDelta;

    // The current latency is the difference in time between when the current frame is at
    // the app and when it is at the hardware.
    double latencyNanos = static_cast<double>(isOutput
                          ? (appFrameHardwareTime - appFrameAppTime) // hardware is later
                          : (appFrameAppTime - appFrameHardwareTime)); // hardware is earlier
    double latencyMillis = latencyNanos / kNanosPerMillisecond;
#endif

    return ResultWithValue<double>(latencyMillis);
}

bool AudioStreamAAudio::isMMapUsed() {
    std::shared_lock<std::shared_mutex> lock(mAAudioStreamLock);
    AAudioStream *stream = mAAudioStream.load();
    if (stream != nullptr) {
        return AAudioExtensions::getInstance().isMMapUsed(stream);
    } else {
        return false;
    }
}

// static
// Static method for the presentation end callback.
// We use a method so we can access protected methods on the stream.
// Launch a thread to handle the error.
// That other thread can safely stop, close and delete the stream.
void AudioStreamAAudio::internalPresentationEndCallback(AAudioStream *stream, void *userData) {
    AudioStreamAAudio *oboeStream = reinterpret_cast<AudioStreamAAudio*>(userData);

    // Prevents deletion of the stream if the app is using AudioStreamBuilder::openStream(shared_ptr)
    auto [isStreamAlive, sharedStream, sharedParentStream] =
            AAudioStreamCollection::getInstance().getStream(oboeStream);
    if (!isStreamAlive) {
        // Client has closed the stream, no need to call the presentation end callback here.
        return;
    }

    if (stream != oboeStream->getUnderlyingStream()) {
        LOGW("%s() stream already closed or closing", __func__); // might happen if there are bugs
    } else if (sharedStream) {
        // Handle error on a separate thread using shared pointer.
        std::thread t(oboe_aaudio_presentation_end_thread_proc_shared, sharedStream,
                      sharedParentStream);
        t.detach();
    } else {
        // Handle error on a separate thread.
        std::thread t(oboe_aaudio_presentation_thread_proc, oboeStream);
        t.detach();
    }
}

void AudioStreamAAudio::internalRoutingChangedCallback(
        AAudioStream *stream, void *userData, const int32_t *deviceIds, int32_t numDevices) {
    AudioStreamAAudio *oboeStream = reinterpret_cast<AudioStreamAAudio*>(userData);

    // Prevents deletion of the stream if the app is using AudioStreamBuilder::openStream(shared_ptr)
    auto [isStreamAlive, sharedStream, sharedParentStream] =
            AAudioStreamCollection::getInstance().getStream(oboeStream);
    if (!isStreamAlive) {
        // Client has closed the stream, no need to call the routing changed callback here.
        return;
    }

    std::vector<int32_t> deviceIdsCopy(deviceIds, deviceIds + numDevices);

    if (stream != oboeStream->getUnderlyingStream()) {
        LOGW("%s() stream already closed or closing", __func__); // might happen if there are bugs
    } else if (sharedStream) {
        oboeStream->onRoutingChanged(deviceIdsCopy);
        if (oboeStream->getRoutingCallback() != nullptr) {
            // Handle routing change on a separate thread using shared pointer.
            std::thread t(oboe_aaudio_routing_changed_thread_proc_shared, sharedStream,
                          sharedParentStream, deviceIdsCopy);
            t.detach();
        }
    } else {
        oboeStream->onRoutingChanged(deviceIdsCopy);
        if (oboeStream->getRoutingCallback() != nullptr) {
            // Handle routing change on a separate thread.
            std::thread t(oboe_aaudio_routing_changed_thread_proc, oboeStream, deviceIdsCopy);
            t.detach();
        }
    }
}

Result AudioStreamAAudio::setOffloadDelayPadding(
        int32_t delayInFrames, int32_t paddingInFrames) {
    if (mLibLoader->stream_setOffloadDelayPadding == nullptr) {
        return Result::ErrorUnimplemented;
    }
    std::shared_lock<std::shared_mutex> lock(mAAudioStreamLock);
#if OBOE_USE_RUST_CORE
    if (mRustAAudioOutputStream != nullptr) {
        return static_cast<Result>(oboe_rust_aaudio_output_set_offload_delay_padding(
                mRustAAudioOutputStream, delayInFrames, paddingInFrames));
    }
#endif
    AAudioStream *stream = mAAudioStream.load();
    if (stream == nullptr) {
        return Result::ErrorClosed;
    }
    return static_cast<Result>(
            mLibLoader->stream_setOffloadDelayPadding(stream, delayInFrames, paddingInFrames));
}

ResultWithValue<int32_t> AudioStreamAAudio::getOffloadDelay() {
    if (mLibLoader->stream_getOffloadDelay == nullptr) {
        return ResultWithValue<int32_t>(Result::ErrorUnimplemented);
    }
    std::shared_lock<std::shared_mutex> lock(mAAudioStreamLock);
#if OBOE_USE_RUST_CORE
    if (mRustAAudioOutputStream != nullptr) {
        return ResultWithValue<int32_t>::createBasedOnSign(
                oboe_rust_aaudio_output_get_offload_delay(mRustAAudioOutputStream));
    }
#endif
    AAudioStream *stream = mAAudioStream.load();
    if (stream == nullptr) {
        return Result::ErrorClosed;
    }
    return ResultWithValue<int32_t>::createBasedOnSign(mLibLoader->stream_getOffloadDelay(stream));
}

ResultWithValue<int32_t> AudioStreamAAudio::getOffloadPadding() {
    if (mLibLoader->stream_getOffloadPadding == nullptr) {
        return ResultWithValue<int32_t>(Result::ErrorUnimplemented);
    }
    std::shared_lock<std::shared_mutex> lock(mAAudioStreamLock);
#if OBOE_USE_RUST_CORE
    if (mRustAAudioOutputStream != nullptr) {
        return ResultWithValue<int32_t>::createBasedOnSign(
                oboe_rust_aaudio_output_get_offload_padding(mRustAAudioOutputStream));
    }
#endif
    AAudioStream *stream = mAAudioStream.load();
    if (stream == nullptr) {
        return ResultWithValue<int32_t>(Result::ErrorClosed);
    }
    return ResultWithValue<int32_t>::createBasedOnSign(
            mLibLoader->stream_getOffloadPadding(stream));
}

Result AudioStreamAAudio::setOffloadEndOfStream() {
    if (mLibLoader->stream_setOffloadEndOfStream == nullptr) {
        return Result::ErrorUnimplemented;
    }
    std::shared_lock<std::shared_mutex> lock(mAAudioStreamLock);
#if OBOE_USE_RUST_CORE
    if (mRustAAudioOutputStream != nullptr) {
        return static_cast<Result>(
                oboe_rust_aaudio_output_set_offload_end_of_stream(mRustAAudioOutputStream));
    }
#endif
    AAudioStream *stream = mAAudioStream.load();
    if (stream == nullptr) {
        return ResultWithValue<int32_t>(Result::ErrorClosed);
    }
    return static_cast<Result>(mLibLoader->stream_setOffloadEndOfStream(stream));
}

void AudioStreamAAudio::updateDeviceIds() {
    // If stream_getDeviceIds is not supported, use stream_getDeviceId.
    if (mLibLoader->stream_getDeviceIds == nullptr) {
        mDeviceIds.clear();
        int32_t deviceId = mLibLoader->stream_getDeviceId(mAAudioStream);
        if (deviceId != kUnspecified) {
            mDeviceIds.push_back(deviceId);
        }
    } else {
        // Allocate a temp vector with 16 elements. This should be enough to cover all cases.
        // Please file a bug on Oboe if you discover that this returns AAUDIO_ERROR_OUT_OF_RANGE.
        // When AAUDIO_ERROR_OUT_OF_RANGE is returned, the actual size will be still returned as the
        // value of deviceIdSize but deviceIds will be empty.

        static constexpr int kDefaultDeviceIdSize = 16;
        int deviceIdSize = kDefaultDeviceIdSize;
        std::vector<int32_t> deviceIds(deviceIdSize);
        aaudio_result_t getDeviceIdResult =
                mLibLoader->stream_getDeviceIds(mAAudioStream, deviceIds.data(), &deviceIdSize);
        if (getDeviceIdResult != AAUDIO_OK) {
            LOGE("stream_getDeviceIds did not return AAUDIO_OK. Error: %d",
                    static_cast<int>(getDeviceIdResult));
            return;
        }

        mDeviceIds.clear();
        for (int i = 0; i < deviceIdSize; i++) {
            mDeviceIds.push_back(deviceIds[i]);
        }
    }
    mUpdatedDeviceIds.deviceIds[mUpdatedDeviceIds.idx.load()] = mDeviceIds;

    // This should not happen in most cases. Please file a bug on Oboe if you see this happening.
    if (getDeviceIds().empty()) {
        LOGW("updateDeviceIds() returns an empty array.");
    }
}

ResultWithValue<int64_t> AudioStreamAAudio::flushFromFrame(FlushFromAccuracy accuracy,
                                                           int64_t positionInFrames) {
    if (mLibLoader->stream_flushFromFrame == nullptr) {
        return ResultWithValue<int64_t>(positionInFrames, Result::ErrorUnimplemented);
    }
    std::shared_lock<std::shared_mutex> lock(mAAudioStreamLock);
#if OBOE_USE_RUST_CORE
    if (mRustAAudioOutputStream != nullptr) {
        auto result = static_cast<Result>(oboe_rust_aaudio_output_flush_from_frame(
                mRustAAudioOutputStream, static_cast<int32_t>(accuracy), &positionInFrames));
        return ResultWithValue<int64_t>(positionInFrames, result);
    }
#endif
    AAudioStream *stream = mAAudioStream.load();
    if (stream == nullptr) {
        return ResultWithValue<int64_t>(positionInFrames, Result::ErrorClosed);
    }
    // TODO: use aaudio_flush_from_frame_accuracy_t when it is defined.
    auto result = static_cast<Result>(mLibLoader->stream_flushFromFrame(
                    stream, static_cast<int32_t>(accuracy), &positionInFrames));
    return ResultWithValue<int64_t>(positionInFrames, result);
}

namespace {

ResultWithValue<AAudioPlaybackParameters> oboe2AAudio_PlaybackParameters_AAudioPlaybackParameters(
        const PlaybackParameters& playbackParameters) {
    AAudioPlaybackParameters aaudioPlaybackParameters;
    switch (playbackParameters.fallbackMode) {
        case FallbackMode::Default:
            aaudioPlaybackParameters.fallbackMode = AAUDIO_FALLBACK_MODE_DEFAULT;
            break;
        case FallbackMode::Mute:
            aaudioPlaybackParameters.fallbackMode = AAUDIO_FALLBACK_MODE_MUTE;
            break;
        case FallbackMode::Fail:
            aaudioPlaybackParameters.fallbackMode = AAUDIO_FALLBACK_MODE_FAIL;
            break;
        default:
            return ResultWithValue<AAudioPlaybackParameters>(Result::ErrorIllegalArgument);
    }

    switch (playbackParameters.stretchMode) {
        case StretchMode::Default:
            aaudioPlaybackParameters.stretchMode = AAUDIO_STRETCH_MODE_DEFAULT;
            break;
        case StretchMode::Voice:
            aaudioPlaybackParameters.stretchMode = AAUDIO_STRETCH_MODE_VOICE;
            break;
        default:
            return ResultWithValue<AAudioPlaybackParameters>(Result::ErrorIllegalArgument);
    }

    aaudioPlaybackParameters.pitch = playbackParameters.pitch;
    aaudioPlaybackParameters.speed = playbackParameters.speed;
    return ResultWithValue<AAudioPlaybackParameters>(aaudioPlaybackParameters);
}

ResultWithValue<PlaybackParameters> aaudio2oboe_AAudioPlaybackParameters_PlaybackParameters(
        const AAudioPlaybackParameters& aaudioPlaybackParameters) {
    PlaybackParameters playbackParameters;
    switch (aaudioPlaybackParameters.fallbackMode) {
        case AAUDIO_FALLBACK_MODE_DEFAULT:
            playbackParameters.fallbackMode = FallbackMode::Default;
            break;
        case AAUDIO_FALLBACK_MODE_MUTE:
            playbackParameters.fallbackMode = FallbackMode::Mute;
            break;
        case AAUDIO_FALLBACK_MODE_FAIL:
            playbackParameters.fallbackMode = FallbackMode::Fail;
            break;
        default:
            LOGE("%s unknown fallback mode %d", __func__, aaudioPlaybackParameters.fallbackMode);
            return ResultWithValue<PlaybackParameters>(Result::ErrorIllegalArgument);
    }

    switch (aaudioPlaybackParameters.stretchMode) {
        case AAUDIO_STRETCH_MODE_DEFAULT:
            playbackParameters.stretchMode = StretchMode::Default;
            break;
        case AAUDIO_STRETCH_MODE_VOICE:
            playbackParameters.stretchMode = StretchMode::Voice;
            break;
        default:
            LOGE("%s unknown stretch mode %d", __func__, aaudioPlaybackParameters.stretchMode);
            return ResultWithValue<PlaybackParameters>(Result::ErrorIllegalArgument);
    }
    playbackParameters.pitch = aaudioPlaybackParameters.pitch;
    playbackParameters.speed = aaudioPlaybackParameters.speed;
    return ResultWithValue<PlaybackParameters>(playbackParameters);
}

} // namespace

Result AudioStreamAAudio::setPlaybackParameters(const PlaybackParameters &parameters) {
    if (mLibLoader->stream_setPlaybackParameters == nullptr) {
        LOGD("%s, the NDK function is not available", __func__);
        return Result::ErrorUnimplemented;
    }
    std::shared_lock _l(mAAudioStreamLock);
#if OBOE_USE_RUST_CORE
    if (mRustAAudioOutputStream != nullptr) {
        OboeRustAAudioPlaybackParameters rustParameters = {
                static_cast<int32_t>(parameters.fallbackMode),
                static_cast<int32_t>(parameters.stretchMode),
                parameters.pitch,
                parameters.speed,
        };
        if (!oboe_rust_aaudio_playback_parameters_valid(
                rustParameters.fallback_mode,
                rustParameters.stretch_mode,
                AAUDIO_FALLBACK_MODE_DEFAULT,
                AAUDIO_FALLBACK_MODE_MUTE,
                AAUDIO_FALLBACK_MODE_FAIL,
                AAUDIO_STRETCH_MODE_DEFAULT,
                AAUDIO_STRETCH_MODE_VOICE)) {
            LOGE("%s, invalid parameters, %s", __func__, toString(parameters).c_str());
            return Result::ErrorIllegalArgument;
        }
        return static_cast<Result>(oboe_rust_aaudio_output_set_playback_parameters(
                mRustAAudioOutputStream, &rustParameters));
    }
#endif
    AAudioStream *stream = mAAudioStream.load();
    if (stream == nullptr) {
        LOGE("%s the stream is already closed", __func__);
        return Result::ErrorClosed;
    }
    auto convertResult =
            oboe2AAudio_PlaybackParameters_AAudioPlaybackParameters(parameters);
    if (!convertResult) {
        LOGE("%s, invalid parameters, %s", __func__, toString(parameters).c_str());
        return Result::ErrorIllegalArgument;
    }
    auto aaudioPlaybackParameters = convertResult.value();
    return static_cast<Result>(mLibLoader->stream_setPlaybackParameters(
            stream, &aaudioPlaybackParameters));
}

ResultWithValue<PlaybackParameters> AudioStreamAAudio::getPlaybackParameters() {
    if (mLibLoader->stream_getPlaybackParameters == nullptr) {
        LOGD("%s, the NDK function is not available", __func__);
        return Result::ErrorUnimplemented;
    }
    std::shared_lock _l(mAAudioStreamLock);
#if OBOE_USE_RUST_CORE
    if (mRustAAudioOutputStream != nullptr) {
        OboeRustAAudioPlaybackParameters rustParameters = {};
        auto result = static_cast<Result>(oboe_rust_aaudio_output_get_playback_parameters(
                mRustAAudioOutputStream, &rustParameters));
        if (result != Result::OK) {
            return ResultWithValue<PlaybackParameters>(result);
        }
        if (!oboe_rust_aaudio_playback_parameters_valid(
                rustParameters.fallback_mode,
                rustParameters.stretch_mode,
                AAUDIO_FALLBACK_MODE_DEFAULT,
                AAUDIO_FALLBACK_MODE_MUTE,
                AAUDIO_FALLBACK_MODE_FAIL,
                AAUDIO_STRETCH_MODE_DEFAULT,
                AAUDIO_STRETCH_MODE_VOICE)) {
            LOGE("%s unknown playback parameters %d/%d", __func__,
                 rustParameters.fallback_mode, rustParameters.stretch_mode);
            return ResultWithValue<PlaybackParameters>(Result::ErrorIllegalArgument);
        }
        PlaybackParameters playbackParameters;
        playbackParameters.fallbackMode = static_cast<FallbackMode>(rustParameters.fallback_mode);
        playbackParameters.stretchMode = static_cast<StretchMode>(rustParameters.stretch_mode);
        playbackParameters.pitch = rustParameters.pitch;
        playbackParameters.speed = rustParameters.speed;
        return ResultWithValue<PlaybackParameters>(playbackParameters);
    }
#endif
    AAudioStream *stream = mAAudioStream.load();
    if (stream == nullptr) {
        LOGE("%s the stream is already closed", __func__);
        return Result::ErrorClosed;
    }

    AAudioPlaybackParameters aaudioPlaybackParameters;
    auto result = static_cast<Result>(
            mLibLoader->stream_getPlaybackParameters(stream, &aaudioPlaybackParameters));
    if (result != Result::OK) {
        return ResultWithValue<PlaybackParameters>(result);
    }

    return aaudio2oboe_AAudioPlaybackParameters_PlaybackParameters(aaudioPlaybackParameters);
}

} // namespace oboe
