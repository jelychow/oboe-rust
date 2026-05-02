/* Copyright 2015 The Android Open Source Project
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
#include <sys/types.h>
#include <cassert>
#include <android/log.h>

#include "common/OboeDebug.h"
#include "oboe/AudioClock.h"
#include "oboe/AudioStream.h"
#include "oboe/AudioStreamBuilder.h"
#include "EngineOpenSLES.h"
#include "AudioStreamOpenSLES.h"
#include "OpenSLESUtilities.h"
#include "OutputMixerOpenSLES.h"
#if OBOE_USE_RUST_CORE
#include "rust/oboe_rust_core.h"
#endif

using namespace oboe;

AudioStreamOpenSLES::AudioStreamOpenSLES(const AudioStreamBuilder &builder)
    : AudioStreamBuffered(builder) {
    // OpenSL ES does not support device IDs. So overwrite value from builder.
    mDeviceIds.clear();
    // OpenSL ES does not support session IDs. So overwrite value from builder.
    mSessionId = SessionId::None;
}

static constexpr int32_t   kHighLatencyBufferSizeMillis = 20; // typical Android period
static constexpr SLuint32  kAudioChannelCountMax = 30; // TODO Why 30?
static constexpr SLuint32  SL_ANDROID_UNKNOWN_CHANNELMASK  = 0; // Matches name used internally.

#if OBOE_USE_RUST_CORE
static int32_t rust_opensles_engine_open() {
    return static_cast<int32_t>(EngineOpenSLES::getInstance().open());
}

static void rust_opensles_engine_close() {
    EngineOpenSLES::getInstance().close();
}

static int32_t rust_opensles_output_mixer_open() {
    return static_cast<int32_t>(OutputMixerOpenSL::getInstance().open());
}

static void rust_opensles_output_mixer_close() {
    OutputMixerOpenSL::getInstance().close();
}

static int32_t rust_opensles_output_create_player(void **object, void *audioSource) {
    SLObjectItf objectItf = nullptr;
    SLresult result = OutputMixerOpenSL::getInstance().createAudioPlayer(
            &objectItf, reinterpret_cast<SLDataSource *>(audioSource));
    *object = const_cast<void *>(reinterpret_cast<const void *>(objectItf));
    return static_cast<int32_t>(result);
}

static int32_t rust_opensles_input_create_recorder(
        void **object, void *audioSource, void *audioSink) {
    SLObjectItf objectItf = nullptr;
    SLresult result = EngineOpenSLES::getInstance().createAudioRecorder(
            &objectItf,
            reinterpret_cast<SLDataSource *>(audioSource),
            reinterpret_cast<SLDataSink *>(audioSink));
    *object = const_cast<void *>(reinterpret_cast<const void *>(objectItf));
    return static_cast<int32_t>(result);
}

static int32_t rust_opensles_object_get_android_configuration(
        void *object, void **configuration) {
    auto objectItf = reinterpret_cast<SLObjectItf>(object);
    SLAndroidConfigurationItf configurationItf = nullptr;
    SLresult result = (*objectItf)->GetInterface(
            objectItf,
            EngineOpenSLES::getInstance().getIidAndroidConfiguration(),
            reinterpret_cast<void *>(&configurationItf));
    *configuration = const_cast<void *>(reinterpret_cast<const void *>(configurationItf));
    return static_cast<int32_t>(result);
}

static int32_t rust_opensles_object_realize(void *object) {
    auto objectItf = reinterpret_cast<SLObjectItf>(object);
    return static_cast<int32_t>((*objectItf)->Realize(objectItf, SL_BOOLEAN_FALSE));
}

static void rust_opensles_object_destroy(void *object) {
    auto objectItf = reinterpret_cast<SLObjectItf>(object);
    (*objectItf)->Destroy(objectItf);
}

static int32_t rust_opensles_object_get_play(void *object, void **play) {
    auto objectItf = reinterpret_cast<SLObjectItf>(object);
    SLPlayItf playItf = nullptr;
    SLresult result = (*objectItf)->GetInterface(
            objectItf, EngineOpenSLES::getInstance().getIidPlay(), &playItf);
    *play = const_cast<void *>(reinterpret_cast<const void *>(playItf));
    return static_cast<int32_t>(result);
}

static int32_t rust_opensles_object_get_record(void *object, void **record) {
    auto objectItf = reinterpret_cast<SLObjectItf>(object);
    SLRecordItf recordItf = nullptr;
    SLresult result = (*objectItf)->GetInterface(
            objectItf, EngineOpenSLES::getInstance().getIidRecord(), &recordItf);
    *record = const_cast<void *>(reinterpret_cast<const void *>(recordItf));
    return static_cast<int32_t>(result);
}

static int32_t rust_opensles_object_get_simple_buffer_queue(void *object, void **queue) {
    auto objectItf = reinterpret_cast<SLObjectItf>(object);
    SLAndroidSimpleBufferQueueItf queueItf = nullptr;
    SLresult result = (*objectItf)->GetInterface(
            objectItf,
            EngineOpenSLES::getInstance().getIidAndroidSimpleBufferQueue(),
            &queueItf);
    *queue = const_cast<void *>(reinterpret_cast<const void *>(queueItf));
    return static_cast<int32_t>(result);
}

static int32_t rust_opensles_configuration_set_performance_mode(
        void *configuration, int32_t performanceMode) {
    auto configurationItf = reinterpret_cast<SLAndroidConfigurationItf>(configuration);
    SLuint32 value = static_cast<SLuint32>(performanceMode);
    return static_cast<int32_t>((*configurationItf)->SetConfiguration(
            configurationItf, SL_ANDROID_KEY_PERFORMANCE_MODE, &value, sizeof(value)));
}

static int32_t rust_opensles_configuration_get_performance_mode(
        void *configuration, int32_t *performanceMode) {
    auto configurationItf = reinterpret_cast<SLAndroidConfigurationItf>(configuration);
    SLuint32 value = 0;
    SLuint32 valueSize = sizeof(value);
    SLresult result = (*configurationItf)->GetConfiguration(
            configurationItf, SL_ANDROID_KEY_PERFORMANCE_MODE, &valueSize, &value);
    *performanceMode = static_cast<int32_t>(value);
    return static_cast<int32_t>(result);
}

static int32_t rust_opensles_configuration_set_stream_type(
        void *configuration, int32_t streamType) {
    auto configurationItf = reinterpret_cast<SLAndroidConfigurationItf>(configuration);
    SLuint32 value = static_cast<SLuint32>(streamType);
    return static_cast<int32_t>((*configurationItf)->SetConfiguration(
            configurationItf, SL_ANDROID_KEY_STREAM_TYPE, &value, sizeof(value)));
}

static int32_t rust_opensles_configuration_set_recording_preset(
        void *configuration, int32_t recordingPreset) {
    auto configurationItf = reinterpret_cast<SLAndroidConfigurationItf>(configuration);
    SLuint32 value = static_cast<SLuint32>(recordingPreset);
    return static_cast<int32_t>((*configurationItf)->SetConfiguration(
            configurationItf, SL_ANDROID_KEY_RECORDING_PRESET, &value, sizeof(value)));
}

static void rust_opensles_queue_callback_glue(
        SLAndroidSimpleBufferQueueItf queue, void *context) {
    auto stream = reinterpret_cast<AudioStreamOpenSLES *>(context);
    bool shouldStopStream = stream->processBufferCallback(queue);
    if (shouldStopStream) {
        stream->requestStop();
    }
}

static int32_t rust_opensles_queue_register_callback(
        void *queue, OboeRustOpenSLESQueueCallback callback, void *userData) {
    (void) callback;
    auto queueItf = reinterpret_cast<SLAndroidSimpleBufferQueueItf>(queue);
    return static_cast<int32_t>((*queueItf)->RegisterCallback(
            queueItf, rust_opensles_queue_callback_glue, userData));
}

static int32_t rust_opensles_queue_enqueue(void *queue, void *buffer, int32_t numBytes) {
    auto queueItf = reinterpret_cast<SLAndroidSimpleBufferQueueItf>(queue);
    return static_cast<int32_t>((*queueItf)->Enqueue(
            queueItf, buffer, static_cast<SLuint32>(numBytes)));
}

static int32_t rust_opensles_queue_clear(void *queue) {
    auto queueItf = reinterpret_cast<SLAndroidSimpleBufferQueueItf>(queue);
    return static_cast<int32_t>((*queueItf)->Clear(queueItf));
}

static int32_t rust_opensles_queue_get_depth(void *queue) {
    auto queueItf = reinterpret_cast<SLAndroidSimpleBufferQueueItf>(queue);
    SLAndroidSimpleBufferQueueState queueState;
    SLresult result = (*queueItf)->GetState(queueItf, &queueState);
    return (result == SL_RESULT_SUCCESS) ? static_cast<int32_t>(queueState.count) : -1;
}

static int32_t rust_opensles_play_set_state(void *play, int32_t state) {
    auto playItf = reinterpret_cast<SLPlayItf>(play);
    return static_cast<int32_t>((*playItf)->SetPlayState(
            playItf, static_cast<SLuint32>(state)));
}

static int32_t rust_opensles_play_get_position_millis(void *play, int32_t *positionMillis) {
    auto playItf = reinterpret_cast<SLPlayItf>(play);
    SLmillisecond value = 0;
    SLresult result = (*playItf)->GetPosition(playItf, &value);
    *positionMillis = static_cast<int32_t>(value);
    return static_cast<int32_t>(result);
}

static int32_t rust_opensles_record_set_state(void *record, int32_t state) {
    auto recordItf = reinterpret_cast<SLRecordItf>(record);
    return static_cast<int32_t>((*recordItf)->SetRecordState(
            recordItf, static_cast<SLuint32>(state)));
}

static int32_t rust_opensles_record_get_position_millis(void *record, int32_t *positionMillis) {
    auto recordItf = reinterpret_cast<SLRecordItf>(record);
    SLmillisecond value = 0;
    SLresult result = (*recordItf)->GetPosition(recordItf, &value);
    *positionMillis = static_cast<int32_t>(value);
    return static_cast<int32_t>(result);
}

OboeRustOpenSLESPlatform AudioStreamOpenSLES::makeRustOpenSLESPlatform() {
    OboeRustOpenSLESPlatform platform{};
    platform.engine_open = rust_opensles_engine_open;
    platform.engine_close = rust_opensles_engine_close;
    platform.output_mixer_open = rust_opensles_output_mixer_open;
    platform.output_mixer_close = rust_opensles_output_mixer_close;
    platform.output_create_player = rust_opensles_output_create_player;
    platform.input_create_recorder = rust_opensles_input_create_recorder;
    platform.object_get_android_configuration = rust_opensles_object_get_android_configuration;
    platform.object_realize = rust_opensles_object_realize;
    platform.object_destroy = rust_opensles_object_destroy;
    platform.object_get_play = rust_opensles_object_get_play;
    platform.object_get_record = rust_opensles_object_get_record;
    platform.object_get_simple_buffer_queue = rust_opensles_object_get_simple_buffer_queue;
    platform.configuration_set_performance_mode =
            rust_opensles_configuration_set_performance_mode;
    platform.configuration_get_performance_mode =
            rust_opensles_configuration_get_performance_mode;
    platform.configuration_set_stream_type = rust_opensles_configuration_set_stream_type;
    platform.configuration_set_recording_preset = rust_opensles_configuration_set_recording_preset;
    platform.queue_register_callback = rust_opensles_queue_register_callback;
    platform.queue_enqueue = rust_opensles_queue_enqueue;
    platform.queue_clear = rust_opensles_queue_clear;
    platform.queue_get_depth = rust_opensles_queue_get_depth;
    platform.play_set_state = rust_opensles_play_set_state;
    platform.play_get_position_millis = rust_opensles_play_get_position_millis;
    platform.record_set_state = rust_opensles_record_set_state;
    platform.record_get_position_millis = rust_opensles_record_get_position_millis;
    return platform;
}

OboeRustOpenSLESCommonSettings AudioStreamOpenSLES::makeRustOpenSLESCommonSettings() {
    OboeRustOpenSLESCommonSettings settings{};
    settings.sdk_version = getSdkVersion();
    settings.android_api_n_mr1 = __ANDROID_API_N_MR1__;
    settings.android_api_o_mr1 = __ANDROID_API_O_MR1__;
    settings.opensl_performance_mode =
            static_cast<int32_t>(convertPerformanceMode(getPerformanceMode()));
    settings.opensl_performance_none = SL_ANDROID_PERFORMANCE_NONE;
    settings.opensl_performance_latency = SL_ANDROID_PERFORMANCE_LATENCY;
    settings.opensl_performance_latency_effects = SL_ANDROID_PERFORMANCE_LATENCY_EFFECTS;
    settings.opensl_performance_power_saving = SL_ANDROID_PERFORMANCE_POWER_SAVING;
    settings.oboe_performance_none = static_cast<int32_t>(PerformanceMode::None);
    settings.oboe_performance_low_latency = static_cast<int32_t>(PerformanceMode::LowLatency);
    settings.oboe_performance_power_saving = static_cast<int32_t>(PerformanceMode::PowerSaving);
    settings.queue_callback = nullptr;
    settings.queue_callback_user_data = this;
    return settings;
}
#endif

SLuint32 AudioStreamOpenSLES::channelCountToChannelMaskDefault(int channelCount) const {
#if OBOE_USE_RUST_CORE
    return static_cast<SLuint32>(oboe_rust_opensles_channel_mask_default(
            channelCount,
            getSdkVersion(),
            __ANDROID_API_N__,
            kAudioChannelCountMax,
            SL_ANDROID_UNKNOWN_CHANNELMASK,
            SL_ANDROID_SPEAKER_NON_POSITIONAL));
#else
    if (channelCount > kAudioChannelCountMax) {
        return SL_ANDROID_UNKNOWN_CHANNELMASK;
    }

    SLuint32 bitfield = (1 << channelCount) - 1;

    // Check for OS at run-time.
    if(getSdkVersion() >= __ANDROID_API_N__) {
        return SL_ANDROID_MAKE_INDEXED_CHANNEL_MASK(bitfield);
    }

    // Indexed channels masks were added in N.
    // For before N, the best we can do is use a positional channel mask.
    return bitfield;
#endif
}

static bool s_isLittleEndian() {
    static uint32_t value = 1;
    return (*reinterpret_cast<uint8_t *>(&value) == 1);  // Does address point to LSB?
}

SLuint32 AudioStreamOpenSLES::getDefaultByteOrder() {
    return s_isLittleEndian() ? SL_BYTEORDER_LITTLEENDIAN : SL_BYTEORDER_BIGENDIAN;
}

Result AudioStreamOpenSLES::open() {
#ifndef OBOE_SUPPRESS_LOG_SPAM
    LOGI("AudioStreamOpenSLES::open() chans=%d, rate=%d", mChannelCount, mSampleRate);
#endif

    // OpenSL ES only supports I16 and Float
    if (mFormat != AudioFormat::I16 && mFormat != AudioFormat::Float) {
        LOGW("%s() Android's OpenSL ES implementation only supports I16 and Float. Format: %s",
             __func__, oboe::convertToText(mFormat));
        return Result::ErrorInvalidFormat;
    }

#if !OBOE_USE_RUST_CORE
    SLresult result = EngineOpenSLES::getInstance().open();
    if (SL_RESULT_SUCCESS != result) {
        return Result::ErrorInternal;
    }
#endif

    Result oboeResult = AudioStreamBuffered::open();
    if (oboeResult != Result::OK) {
#if !OBOE_USE_RUST_CORE
        EngineOpenSLES::getInstance().close();
#endif
        return oboeResult;
    }
    // Convert to defaults if UNSPECIFIED
    if (mSampleRate == kUnspecified) {
        mSampleRate = DefaultStreamValues::SampleRate;
    }
    if (mChannelCount == kUnspecified) {
        mChannelCount = DefaultStreamValues::ChannelCount;
    }
    if (mContentType == kUnspecified) {
        mContentType = ContentType::Music;
    }
    if (static_cast<const int32_t>(mUsage) == kUnspecified) {
        mUsage = Usage::Media;
    }

    mSharingMode = SharingMode::Shared;

    return Result::OK;
}


SLresult AudioStreamOpenSLES::finishCommonOpen(SLAndroidConfigurationItf configItf) {
    // Setting privacy sensitive mode and allowed capture policy are not supported for OpenSL ES.
    mPrivacySensitiveMode = PrivacySensitiveMode::Unspecified;
    mAllowedCapturePolicy = AllowedCapturePolicy::Unspecified;

    // Spatialization Behavior is not supported for OpenSL ES.
    mSpatializationBehavior = SpatializationBehavior::Never;

#if OBOE_USE_RUST_CORE
    SLresult result = SL_RESULT_SUCCESS;
    if (mRustOutputBackend == nullptr && mRustInputBackend == nullptr) {
        result = registerBufferQueueCallback();
    }
#else
    SLresult result = registerBufferQueueCallback();
#endif
    if (SL_RESULT_SUCCESS != result) {
        return result;
    }

#if OBOE_USE_RUST_CORE
    if (mRustOutputBackend == nullptr && mRustInputBackend == nullptr) {
        result = updateStreamParameters(configItf);
    }
#else
    result = updateStreamParameters(configItf);
#endif
    if (SL_RESULT_SUCCESS != result) {
        return result;
    }

    Result oboeResult = configureBufferSizes(mSampleRate);
    if (Result::OK != oboeResult) {
        return (SLresult) oboeResult;
    }

    allocateFifo();

    calculateDefaultDelayBeforeCloseMillis();

    return SL_RESULT_SUCCESS;
}

#if !OBOE_USE_RUST_CORE
static int32_t roundUpDivideByN(int32_t x, int32_t n) {
    return (x + n - 1) / n;
}
#endif

int32_t AudioStreamOpenSLES::calculateOptimalBufferQueueLength() {
#if OBOE_USE_RUST_CORE
    return oboe_rust_opensles_optimal_buffer_queue_length(
            kBufferQueueLengthDefault,
            kBufferQueueLengthMax,
            mBufferCapacityInFrames,
            kDoubleBufferCount,
            mFramesPerCallback,
            estimateNativeFramesPerBurst());
#else
    int32_t queueLength = kBufferQueueLengthDefault;
    int32_t likelyFramesPerBurst = estimateNativeFramesPerBurst();
    int32_t minCapacity = mBufferCapacityInFrames; // specified by app or zero
    // The buffer capacity needs to be at least twice the size of the requested callbackSize
    // so that we can have double buffering.
    minCapacity = std::max(minCapacity, kDoubleBufferCount * mFramesPerCallback);
    if (minCapacity > 0) {
        int32_t queueLengthFromCapacity = roundUpDivideByN(minCapacity, likelyFramesPerBurst);
        queueLength = std::max(queueLength, queueLengthFromCapacity);
    }
    queueLength = std::min(queueLength, kBufferQueueLengthMax); // clip to max
    // TODO Investigate the effect of queueLength on latency for normal streams. (not low latency)
    return queueLength;
#endif
}

/**
 * The best information we have is if DefaultStreamValues::FramesPerBurst
 * was set by the app based on AudioManager.PROPERTY_OUTPUT_FRAMES_PER_BUFFER.
 * Without that we just have to guess.
 * @return
 */
int32_t AudioStreamOpenSLES::estimateNativeFramesPerBurst() {
#if OBOE_USE_RUST_CORE
    LOGD("AudioStreamOpenSLES:%s() DefaultStreamValues::FramesPerBurst = %d",
            __func__, DefaultStreamValues::FramesPerBurst);
    int32_t framesPerBurst = oboe_rust_opensles_estimate_native_frames_per_burst(
            DefaultStreamValues::FramesPerBurst,
            DefaultStreamValues::SampleRate,
            mSampleRate,
            static_cast<int32_t>(mPerformanceMode),
            getSdkVersion(),
            __ANDROID_API_N_MR1__,
            static_cast<int32_t>(PerformanceMode::LowLatency),
            kHighLatencyBufferSizeMillis,
            kMillisPerSecond);
    LOGD("AudioStreamOpenSLES:%s() mSampleRate = %d, set framesPerBurst = %d",
         __func__, mSampleRate, framesPerBurst);
    return framesPerBurst;
#else
    int32_t framesPerBurst = DefaultStreamValues::FramesPerBurst;
    LOGD("AudioStreamOpenSLES:%s() DefaultStreamValues::FramesPerBurst = %d",
            __func__, DefaultStreamValues::FramesPerBurst);
    framesPerBurst = std::max(framesPerBurst, 16);
    // Calculate the size of a fixed duration high latency buffer based on sample rate.
    // Estimate sample based on default options in order of priority.
    int32_t sampleRate = 48000;
    sampleRate = (DefaultStreamValues::SampleRate > 0)
            ? DefaultStreamValues::SampleRate : sampleRate;
    sampleRate = (mSampleRate > 0) ? mSampleRate : sampleRate;
    int32_t framesPerHighLatencyBuffer =
            (kHighLatencyBufferSizeMillis * sampleRate) / kMillisPerSecond;
    // For high latency streams, use a larger buffer size.
    // Performance Mode support was added in N_MR1 (7.1)
    if (getSdkVersion() >= __ANDROID_API_N_MR1__
            && mPerformanceMode != PerformanceMode::LowLatency
            && framesPerBurst < framesPerHighLatencyBuffer) {
        // Find a multiple of framesPerBurst >= framesPerHighLatencyBuffer.
        int32_t numBursts = roundUpDivideByN(framesPerHighLatencyBuffer, framesPerBurst);
        framesPerBurst *= numBursts;
        LOGD("AudioStreamOpenSLES:%s() NOT low latency, numBursts = %d, mSampleRate = %d, set framesPerBurst = %d",
             __func__, numBursts, mSampleRate, framesPerBurst);
    }
    return framesPerBurst;
#endif
}

Result AudioStreamOpenSLES::configureBufferSizes(int32_t sampleRate) {
    LOGD("AudioStreamOpenSLES:%s(%d) initial mFramesPerBurst = %d, mFramesPerCallback = %d",
            __func__, mSampleRate, mFramesPerBurst, mFramesPerCallback);
    mFramesPerBurst = estimateNativeFramesPerBurst();
    mFramesPerCallback =
#if OBOE_USE_RUST_CORE
            oboe_rust_opensles_configured_callback_frames(mFramesPerCallback, mFramesPerBurst);
#else
            (mFramesPerCallback > 0) ? mFramesPerCallback : mFramesPerBurst;
#endif
    LOGD("AudioStreamOpenSLES:%s(%d) final mFramesPerBurst = %d, mFramesPerCallback = %d",
         __func__, mSampleRate, mFramesPerBurst, mFramesPerCallback);

    mBytesPerCallback = mFramesPerCallback * getBytesPerFrame();
    if (mBytesPerCallback <= 0) {
        LOGE("AudioStreamOpenSLES::open() bytesPerCallback < 0 = %d, bad format?",
             mBytesPerCallback);
        return Result::ErrorInvalidFormat; // causing bytesPerFrame == 0
    }

    for (int i = 0; i < mBufferQueueLength; ++i) {
        mCallbackBuffer[i] = std::make_unique<uint8_t[]>(mBytesPerCallback);
    }

    if (!usingFIFO()) {
        mBufferCapacityInFrames = mFramesPerBurst * mBufferQueueLength;
        // Check for overflow.
        if (mBufferCapacityInFrames <= 0) {
            mBufferCapacityInFrames = 0;
            LOGE("AudioStreamOpenSLES::open() numeric overflow because mFramesPerBurst = %d",
                 mFramesPerBurst);
            return Result::ErrorOutOfRange;
        }
        mBufferSizeInFrames = mBufferCapacityInFrames;
    }

    return Result::OK;
}

SLuint32 AudioStreamOpenSLES::convertPerformanceMode(PerformanceMode oboeMode) const {
#if OBOE_USE_RUST_CORE
    return static_cast<SLuint32>(oboe_rust_opensles_convert_oboe_performance_mode(
            static_cast<int32_t>(oboeMode),
            static_cast<int32_t>(getSessionId()),
            static_cast<int32_t>(SessionId::None),
            SL_ANDROID_PERFORMANCE_NONE,
            SL_ANDROID_PERFORMANCE_LATENCY,
            SL_ANDROID_PERFORMANCE_LATENCY_EFFECTS,
            SL_ANDROID_PERFORMANCE_POWER_SAVING));
#else
    SLuint32 openslMode = SL_ANDROID_PERFORMANCE_NONE;
    switch(oboeMode) {
        case PerformanceMode::None:
            openslMode =  SL_ANDROID_PERFORMANCE_NONE;
            break;
        case PerformanceMode::LowLatency:
            openslMode =  (getSessionId() == SessionId::None) ?  SL_ANDROID_PERFORMANCE_LATENCY : SL_ANDROID_PERFORMANCE_LATENCY_EFFECTS;
            break;
        case PerformanceMode::PowerSaving:
            openslMode =  SL_ANDROID_PERFORMANCE_POWER_SAVING;
            break;
        default:
            break;
    }
    return openslMode;
#endif
}

PerformanceMode AudioStreamOpenSLES::convertPerformanceMode(SLuint32 openslMode) const {
#if OBOE_USE_RUST_CORE
    return static_cast<PerformanceMode>(oboe_rust_opensles_convert_opensl_performance_mode(
            openslMode,
            SL_ANDROID_PERFORMANCE_NONE,
            SL_ANDROID_PERFORMANCE_LATENCY,
            SL_ANDROID_PERFORMANCE_LATENCY_EFFECTS,
            SL_ANDROID_PERFORMANCE_POWER_SAVING,
            static_cast<int32_t>(PerformanceMode::None),
            static_cast<int32_t>(PerformanceMode::LowLatency),
            static_cast<int32_t>(PerformanceMode::PowerSaving)));
#else
    PerformanceMode oboeMode = PerformanceMode::None;
    switch(openslMode) {
        case SL_ANDROID_PERFORMANCE_NONE:
            oboeMode =  PerformanceMode::None;
            break;
        case SL_ANDROID_PERFORMANCE_LATENCY:
        case SL_ANDROID_PERFORMANCE_LATENCY_EFFECTS:
            oboeMode =  PerformanceMode::LowLatency;
            break;
        case SL_ANDROID_PERFORMANCE_POWER_SAVING:
            oboeMode =  PerformanceMode::PowerSaving;
            break;
        default:
            break;
    }
    return oboeMode;
#endif
}

void AudioStreamOpenSLES::logUnsupportedAttributes() {
    // Log unsupported attributes
    // only report if changed from the default

    // Device ID
    if (!mDeviceIds.empty()) {
        LOGW("Device ID [AudioStreamBuilder::setDeviceId()] "
             "is not supported on OpenSLES streams.");
    }
    // Sharing Mode
    if (mSharingMode != SharingMode::Shared) {
        LOGW("SharingMode [AudioStreamBuilder::setSharingMode()] "
             "is not supported on OpenSLES streams.");
    }
    // Performance Mode
    int sdkVersion = getSdkVersion();
    if (mPerformanceMode != PerformanceMode::None && sdkVersion < __ANDROID_API_N_MR1__) {
        LOGW("PerformanceMode [AudioStreamBuilder::setPerformanceMode()] "
             "is not supported on OpenSLES streams running on pre-Android N-MR1 versions.");
    }
    // Content Type
    if (static_cast<const int32_t>(mContentType) != kUnspecified) {
        LOGW("ContentType [AudioStreamBuilder::setContentType()] "
             "is not supported on OpenSLES streams.");
    }

    // Session Id
    if (mSessionId != SessionId::None) {
        LOGW("SessionId [AudioStreamBuilder::setSessionId()] "
             "is not supported on OpenSLES streams.");
    }

    // Privacy Sensitive Mode
    if (mPrivacySensitiveMode != PrivacySensitiveMode::Unspecified) {
        LOGW("PrivacySensitiveMode [AudioStreamBuilder::setPrivacySensitiveMode()] "
             "is not supported on OpenSLES streams.");
    }

    // Spatialization Behavior
    if (mSpatializationBehavior != SpatializationBehavior::Unspecified) {
        LOGW("SpatializationBehavior [AudioStreamBuilder::setSpatializationBehavior()] "
             "is not supported on OpenSLES streams.");
    }

    if (mIsContentSpatialized) {
        LOGW("Boolean [AudioStreamBuilder::setIsContentSpatialized()] "
             "is not supported on OpenSLES streams.");
    }

    // Allowed Capture Policy
    if (mAllowedCapturePolicy != AllowedCapturePolicy::Unspecified) {
        LOGW("AllowedCapturePolicy [AudioStreamBuilder::setAllowedCapturePolicy()] "
             "is not supported on OpenSLES streams.");
    }

    // Package Name
    if (!mPackageName.empty()) {
        LOGW("PackageName [AudioStreamBuilder::setPackageName()] "
             "is not supported on OpenSLES streams.");
    }

    // Attribution Tag
    if (!mAttributionTag.empty()) {
        LOGW("AttributionTag [AudioStreamBuilder::setAttributionTag()] "
             "is not supported on OpenSLES streams.");
    }
}

SLresult AudioStreamOpenSLES::configurePerformanceMode(SLAndroidConfigurationItf configItf) {

    if (configItf == nullptr) {
        LOGW("%s() called with NULL configuration", __func__);
        mPerformanceMode = PerformanceMode::None;
        return SL_RESULT_INTERNAL_ERROR;
    }
    if (getSdkVersion() < __ANDROID_API_N_MR1__) {
        LOGW("%s() not supported until N_MR1", __func__);
        mPerformanceMode = PerformanceMode::None;
        return SL_RESULT_SUCCESS;
    }

    SLresult result = SL_RESULT_SUCCESS;
    SLuint32 performanceMode = convertPerformanceMode(getPerformanceMode());
    result = (*configItf)->SetConfiguration(configItf, SL_ANDROID_KEY_PERFORMANCE_MODE,
                                                     &performanceMode, sizeof(performanceMode));
    if (SL_RESULT_SUCCESS != result) {
        LOGW("SetConfiguration(PERFORMANCE_MODE, SL %u) returned %s",
             performanceMode, getSLErrStr(result));
        mPerformanceMode = PerformanceMode::None;
    }

    return result;
}

SLresult AudioStreamOpenSLES::updateStreamParameters(SLAndroidConfigurationItf configItf) {
    SLresult result = SL_RESULT_SUCCESS;
    if(getSdkVersion() >= __ANDROID_API_N_MR1__ && configItf != nullptr) {
        SLuint32 performanceMode = 0;
        SLuint32 performanceModeSize = sizeof(performanceMode);
        result = (*configItf)->GetConfiguration(configItf, SL_ANDROID_KEY_PERFORMANCE_MODE,
                                                &performanceModeSize, &performanceMode);
        // A bug in GetConfiguration() before P caused a wrong result code to be returned.
        if (getSdkVersion() <= __ANDROID_API_O_MR1__) {
            result = SL_RESULT_SUCCESS; // Ignore actual result before P.
        }

        if (SL_RESULT_SUCCESS != result) {
            LOGW("GetConfiguration(SL_ANDROID_KEY_PERFORMANCE_MODE) returned %d", result);
            mPerformanceMode = PerformanceMode::None; // If we can't query it then assume None.
        } else {
            mPerformanceMode = convertPerformanceMode(performanceMode); // convert SL to Oboe mode
        }
    } else {
        mPerformanceMode = PerformanceMode::None; // If we can't query it then assume None.
    }
    return result;
}

// This is called under mLock.
Result AudioStreamOpenSLES::close_l() {
    LOGD("AudioOutputStreamOpenSLES::%s() called", __func__);
    if (mState == StreamState::Closed) {
        return Result::ErrorClosed;
    }

    AudioStreamBuffered::close();

    onBeforeDestroy();

    // Mark as CLOSED before we unlock for the join.
    // This will prevent other threads from trying to close().
    setState(StreamState::Closed);

    SLObjectItf  tempObjectInterface = mObjectInterface;
    bool destroyedByRustBackend = false;
    mObjectInterface = nullptr;
#if OBOE_USE_RUST_CORE
    if (mRustOutputBackend != nullptr) {
        auto tempRustOutputBackend = mRustOutputBackend;
        mRustOutputBackend = nullptr;
        mLock.unlock();
        (void) oboe_rust_opensles_output_destroy(tempRustOutputBackend);
        mLock.lock();
        destroyedByRustBackend = true;
    } else if (mRustInputBackend != nullptr) {
        auto tempRustInputBackend = mRustInputBackend;
        mRustInputBackend = nullptr;
        mLock.unlock();
        (void) oboe_rust_opensles_input_destroy(tempRustInputBackend);
        mLock.lock();
        destroyedByRustBackend = true;
    } else
#endif
    if (tempObjectInterface != nullptr) {
        // Temporarily unlock so we can join() the callback thread.
        mLock.unlock();
        (*tempObjectInterface)->Destroy(tempObjectInterface); // Will join the callback!
        mLock.lock();
    }

    if (!destroyedByRustBackend) {
        onAfterDestroy();
    }

    mSimpleBufferQueueInterface = nullptr;
#if OBOE_USE_RUST_CORE
    if (tempObjectInterface != nullptr) {
        EngineOpenSLES::getInstance().close();
    }
#else
    EngineOpenSLES::getInstance().close();
#endif

    return Result::OK;
}

SLresult AudioStreamOpenSLES::enqueueCallbackBuffer(SLAndroidSimpleBufferQueueItf bq) {
#if OBOE_USE_RUST_CORE
    if (mRustOutputBackend != nullptr) {
        SLresult rustResult = static_cast<SLresult>(oboe_rust_opensles_output_enqueue(
                mRustOutputBackend,
                mCallbackBuffer[mCallbackBufferIndex].get(),
                mBytesPerCallback));
        mCallbackBufferIndex = (mCallbackBufferIndex + 1) % mBufferQueueLength;
        return rustResult;
    }
    if (mRustInputBackend != nullptr) {
        SLresult rustResult = static_cast<SLresult>(oboe_rust_opensles_input_enqueue(
                mRustInputBackend,
                mCallbackBuffer[mCallbackBufferIndex].get(),
                mBytesPerCallback));
        mCallbackBufferIndex = (mCallbackBufferIndex + 1) % mBufferQueueLength;
        return rustResult;
    }
#endif
    if (bq == nullptr) {
        return SL_RESULT_INTERNAL_ERROR;
    }
    SLresult result = (*bq)->Enqueue(
            bq, mCallbackBuffer[mCallbackBufferIndex].get(), mBytesPerCallback);
    mCallbackBufferIndex = (mCallbackBufferIndex + 1) % mBufferQueueLength;
    return result;
}

int32_t AudioStreamOpenSLES::getBufferDepth(SLAndroidSimpleBufferQueueItf bq) {
#if OBOE_USE_RUST_CORE
    if (mRustOutputBackend != nullptr) {
        return oboe_rust_opensles_output_get_buffer_depth(mRustOutputBackend);
    }
    if (mRustInputBackend != nullptr) {
        return oboe_rust_opensles_input_get_buffer_depth(mRustInputBackend);
    }
#endif
    SLAndroidSimpleBufferQueueState queueState;
    SLresult result = (*bq)->GetState(bq, &queueState);
    return (result == SL_RESULT_SUCCESS) ? queueState.count : -1;
}

bool AudioStreamOpenSLES::processBufferCallback(SLAndroidSimpleBufferQueueItf bq) {
    if (getState() == StreamState::Closed) {
        mCallbackBufferIndex = 0;
        return true;
    }

    bool shouldStopStream = false;
    // Ask the app callback to process the buffer.
    DataCallbackResult result =
            fireDataCallback(mCallbackBuffer[mCallbackBufferIndex].get(), mFramesPerCallback);
    if (result == DataCallbackResult::Continue) {
        // Pass the buffer to OpenSLES.
        SLresult enqueueResult = enqueueCallbackBuffer(bq);
        if (enqueueResult != SL_RESULT_SUCCESS) {
            LOGE("%s() returned %d", __func__, enqueueResult);
            shouldStopStream = true;
        }
        // Update Oboe client position with frames handled by the callback.
        if (getDirection() == Direction::Input) {
            mFramesRead += mFramesPerCallback;
        } else {
            mFramesWritten += mFramesPerCallback;
        }
    } else if (result == DataCallbackResult::Stop) {
        LOGD("Oboe callback returned Stop");
        shouldStopStream = true;
    } else {
        LOGW("Oboe callback returned unexpected value = %d", static_cast<int>(result));
        shouldStopStream = true;
    }
    if (shouldStopStream) {
        mCallbackBufferIndex = 0;
    }
    return shouldStopStream;
}

// This callback handler is called every time a buffer has been processed by OpenSL ES.
static void bqCallbackGlue(SLAndroidSimpleBufferQueueItf bq, void *context) {
    bool shouldStopStream = (reinterpret_cast<AudioStreamOpenSLES *>(context))
            ->processBufferCallback(bq);
    if (shouldStopStream) {
        (reinterpret_cast<AudioStreamOpenSLES *>(context))->requestStop();
    }
}

SLresult AudioStreamOpenSLES::registerBufferQueueCallback() {
    // The BufferQueue
    SLresult result = (*mObjectInterface)->GetInterface(mObjectInterface,
            EngineOpenSLES::getInstance().getIidAndroidSimpleBufferQueue(),
            &mSimpleBufferQueueInterface);
    if (SL_RESULT_SUCCESS != result) {
        LOGE("get buffer queue interface:%p result:%s",
             mSimpleBufferQueueInterface,
             getSLErrStr(result));
    } else {
        // Register the BufferQueue callback
        result = (*mSimpleBufferQueueInterface)->RegisterCallback(mSimpleBufferQueueInterface,
                                                                  bqCallbackGlue, this);
        if (SL_RESULT_SUCCESS != result) {
            LOGE("RegisterCallback result:%s", getSLErrStr(result));
        }
    }
    return result;
}

int64_t AudioStreamOpenSLES::getFramesProcessedByServer() {
    updateServiceFrameCounter();
    int64_t millis64 = mPositionMillis.get();
    int64_t framesProcessed = millis64 * getSampleRate() / kMillisPerSecond;
    return framesProcessed;
}

Result AudioStreamOpenSLES::waitForStateChange(StreamState currentState,
                                                     StreamState *nextState,
                                                     int64_t timeoutNanoseconds) {
    Result oboeResult = Result::ErrorTimeout;
    int64_t sleepTimeNanos = 20 * kNanosPerMillisecond; // arbitrary
    int64_t timeLeftNanos = timeoutNanoseconds;

    while (true) {
        const StreamState state = getState(); // this does not require a lock
        if (nextState != nullptr) {
            *nextState = state;
        }
        if (currentState != state) { // state changed?
            oboeResult = Result::OK;
            break;
        }

        // Did we timeout or did user ask for non-blocking?
        if (timeLeftNanos <= 0) {
            break;
        }

        if (sleepTimeNanos > timeLeftNanos){
            sleepTimeNanos = timeLeftNanos;
        }
        AudioClock::sleepForNanos(sleepTimeNanos);
        timeLeftNanos -= sleepTimeNanos;
    }

    return oboeResult;
}
