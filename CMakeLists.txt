cmake_minimum_required(VERSION 3.22.1)

# Set the name of the project and store it in PROJECT_NAME. Also set the following variables:
# PROJECT_SOURCE_DIR (usually the root directory where Oboe has been cloned e.g.)
# PROJECT_BINARY_DIR (usually the containing project's binary directory,
# e.g. ${OBOE_HOME}/samples/RhythmGame/.externalNativeBuild/cmake/ndkExtractorDebug/x86/oboe-bin)
project(oboe)

option(OBOE_BUILD_LEGACY_CPP "Build the legacy C++ Oboe target for reference and compatibility checks" OFF)

if(NOT OBOE_BUILD_LEGACY_CPP)
    message(STATUS "Oboe: legacy C++ target is disabled. Use android/oboe-wrapper for the Rust-native main path.")
    return()
endif()

set (oboe_sources
    src/aaudio/AAudioLoader.cpp
    src/aaudio/AudioStreamAAudio.cpp
    src/common/AdpfWrapper.cpp
    src/common/AudioStream.cpp
    src/common/AudioStreamBuilder.cpp
    src/common/FixedBlockAdapter.cpp
    src/common/FixedBlockReader.cpp
    src/common/FixedBlockWriter.cpp
    src/common/LatencyTuner.cpp
    src/common/OboeExtensions.cpp
    src/common/Utilities.cpp
    src/common/QuirksManager.cpp
    src/fifo/FifoBuffer.cpp
    src/fifo/FifoController.cpp
    src/fifo/FifoControllerBase.cpp
    src/fifo/FifoControllerIndirect.cpp
    src/opensles/AudioInputStreamOpenSLES.cpp
    src/opensles/AudioOutputStreamOpenSLES.cpp
    src/opensles/AudioStreamBuffered.cpp
    src/opensles/AudioStreamOpenSLES.cpp
    src/opensles/EngineOpenSLES.cpp
    src/opensles/OpenSLESUtilities.cpp
    src/opensles/OutputMixerOpenSLES.cpp
    src/common/StabilizedCallback.cpp
    src/common/Trace.cpp
    src/common/Version.cpp
    )

set (data_conversion_sources
    src/common/AudioSourceCaller.cpp
    src/common/DataConversionFlowGraph.cpp
    src/common/FilterAudioStream.cpp
    src/common/SourceFloatCaller.cpp
    src/common/SourceI16Caller.cpp
    src/common/SourceI24Caller.cpp
    src/common/SourceI32Caller.cpp
    src/flowgraph/FlowGraphNode.cpp
    src/flowgraph/ChannelCountConverter.cpp
    src/flowgraph/ClipToRange.cpp
    src/flowgraph/Limiter.cpp
    src/flowgraph/ManyToMultiConverter.cpp
    src/flowgraph/MonoBlend.cpp
    src/flowgraph/MonoToMultiConverter.cpp
    src/flowgraph/MultiToManyConverter.cpp
    src/flowgraph/MultiToMonoConverter.cpp
    src/flowgraph/RampLinear.cpp
    src/flowgraph/SampleRateConverter.cpp
    src/flowgraph/SinkFloat.cpp
    src/flowgraph/SinkI16.cpp
    src/flowgraph/SinkI24.cpp
    src/flowgraph/SinkI32.cpp
    src/flowgraph/SinkI8_24.cpp
    src/flowgraph/SourceFloat.cpp
    src/flowgraph/SourceI16.cpp
    src/flowgraph/SourceI24.cpp
    src/flowgraph/SourceI32.cpp
    src/flowgraph/SourceI8_24.cpp
    src/flowgraph/resampler/IntegerRatio.cpp
    src/flowgraph/resampler/LinearResampler.cpp
    src/flowgraph/resampler/MultiChannelResampler.cpp
    src/flowgraph/resampler/PolyphaseResampler.cpp
    src/flowgraph/resampler/PolyphaseResamplerMono.cpp
    src/flowgraph/resampler/PolyphaseResamplerStereo.cpp
    src/flowgraph/resampler/SincResampler.cpp
    src/flowgraph/resampler/SincResamplerStereo.cpp
)

# Option to disable data conversion features. Default is OFF (i.e., conversion is ON).
option(OBOE_DISABLE_CONVERSION "Exclude data conversion source files and features" OFF)
option(OBOE_USE_RUST_CORE "Use Rust implementations for Oboe core behavior" ON)

if(OBOE_DISABLE_CONVERSION)
    message(STATUS "Oboe: Data conversion is DISABLED.")
else()
    message(STATUS "Oboe: Data conversion is ENABLED.")
    list(APPEND oboe_sources ${data_conversion_sources})
endif()

add_library(oboe ${oboe_sources})

# Specify directories which the compiler should look for headers
target_include_directories(oboe
        PRIVATE src
        PUBLIC include)

# Compile Flags:
#     Enable -Werror when building debug config
#     Enable -Ofast
target_compile_options(oboe
        PRIVATE
        -std=c++17
        -Wall
        -Wextra-semi
        -Wshadow
        -Wshadow-field
        "$<$<CONFIG:RELEASE>:-Ofast>"
        "$<$<CONFIG:DEBUG>:-O3>"
        "$<$<CONFIG:DEBUG>:-Werror>")

target_compile_definitions(oboe PRIVATE $<$<BOOL:${OBOE_DISABLE_CONVERSION}>:DISABLE_CONVERSION=1>)

# Enable logging of D,V for debug builds
target_compile_definitions(oboe PUBLIC $<$<CONFIG:DEBUG>:OBOE_ENABLE_LOGGING=1>)

option(OBOE_DO_NOT_DEFINE_OPENSL_ES_CONSTANTS "Do not define OpenSLES constants" OFF)
target_compile_definitions(oboe PRIVATE $<$<BOOL:${OBOE_DO_NOT_DEFINE_OPENSL_ES_CONSTANTS}>:DO_NOT_DEFINE_OPENSL_ES_CONSTANTS=1>)

target_link_libraries(oboe PRIVATE log OpenSLES)
target_link_options(oboe PRIVATE "-Wl,-z,max-page-size=16384")

if(OBOE_USE_RUST_CORE)
    find_program(CARGO_EXECUTABLE cargo REQUIRED)

    set(OBOE_RUST_CORE_DIR ${CMAKE_CURRENT_SOURCE_DIR}/rust/oboe_rust_core)
    set(OBOE_RUST_TARGET_DIR ${CMAKE_CURRENT_BINARY_DIR}/rust-target)
    set(OBOE_RUST_CARGO_ARGS
            build
            --manifest-path ${OBOE_RUST_CORE_DIR}/Cargo.toml
            --release)
    set(OBOE_RUST_ENV "CARGO_TARGET_DIR=${OBOE_RUST_TARGET_DIR}")
    set(OBOE_RUST_LIBRARY_PATH ${OBOE_RUST_TARGET_DIR}/release/liboboe_rust_core.a)

    if(ANDROID_ABI)
        if(ANDROID_ABI STREQUAL "arm64-v8a")
            set(OBOE_RUST_TARGET aarch64-linux-android)
            set(OBOE_RUST_LINKER_PREFIX aarch64-linux-android)
        elseif(ANDROID_ABI STREQUAL "armeabi-v7a")
            set(OBOE_RUST_TARGET armv7-linux-androideabi)
            set(OBOE_RUST_LINKER_PREFIX armv7a-linux-androideabi)
        elseif(ANDROID_ABI STREQUAL "x86")
            set(OBOE_RUST_TARGET i686-linux-android)
            set(OBOE_RUST_LINKER_PREFIX i686-linux-android)
        elseif(ANDROID_ABI STREQUAL "x86_64")
            set(OBOE_RUST_TARGET x86_64-linux-android)
            set(OBOE_RUST_LINKER_PREFIX x86_64-linux-android)
        else()
            message(FATAL_ERROR "OBOE_USE_RUST_CORE does not support ANDROID_ABI=${ANDROID_ABI}")
        endif()

        set(OBOE_ANDROID_API_LEVEL ${ANDROID_PLATFORM})
        string(REGEX REPLACE "^android-" "" OBOE_ANDROID_API_LEVEL "${OBOE_ANDROID_API_LEVEL}")
        if(NOT OBOE_ANDROID_API_LEVEL MATCHES "^[0-9]+$")
            message(FATAL_ERROR "OBOE_USE_RUST_CORE requires numeric ANDROID_PLATFORM, got ${ANDROID_PLATFORM}")
        endif()

        if(CMAKE_HOST_SYSTEM_NAME STREQUAL "Darwin")
            if(CMAKE_HOST_SYSTEM_PROCESSOR STREQUAL "arm64")
                set(OBOE_NDK_PREBUILT_TAG darwin-arm64)
            else()
                set(OBOE_NDK_PREBUILT_TAG darwin-x86_64)
            endif()
        elseif(CMAKE_HOST_SYSTEM_NAME STREQUAL "Windows")
            set(OBOE_NDK_PREBUILT_TAG windows-x86_64)
        else()
            set(OBOE_NDK_PREBUILT_TAG linux-x86_64)
        endif()

        set(OBOE_RUST_LINKER
                ${ANDROID_NDK}/toolchains/llvm/prebuilt/${OBOE_NDK_PREBUILT_TAG}/bin/${OBOE_RUST_LINKER_PREFIX}${OBOE_ANDROID_API_LEVEL}-clang)
        if(CMAKE_HOST_SYSTEM_NAME STREQUAL "Windows")
            set(OBOE_RUST_LINKER ${OBOE_RUST_LINKER}.cmd)
        endif()
        if(NOT EXISTS "${OBOE_RUST_LINKER}")
            message(FATAL_ERROR "Rust Android linker was not found: ${OBOE_RUST_LINKER}")
        endif()

        list(APPEND OBOE_RUST_CARGO_ARGS --target ${OBOE_RUST_TARGET})
        set(OBOE_RUST_LIBRARY_PATH
                ${OBOE_RUST_TARGET_DIR}/${OBOE_RUST_TARGET}/release/liboboe_rust_core.a)
        string(TOUPPER ${OBOE_RUST_TARGET} OBOE_RUST_TARGET_ENV_NAME)
        string(REPLACE "-" "_" OBOE_RUST_TARGET_ENV_NAME ${OBOE_RUST_TARGET_ENV_NAME})
        list(APPEND OBOE_RUST_ENV
                "CARGO_TARGET_${OBOE_RUST_TARGET_ENV_NAME}_LINKER=${OBOE_RUST_LINKER}")
    endif()

    add_custom_command(
            OUTPUT ${OBOE_RUST_LIBRARY_PATH}
            COMMAND ${CMAKE_COMMAND} -E env ${OBOE_RUST_ENV}
                    ${CARGO_EXECUTABLE} ${OBOE_RUST_CARGO_ARGS}
            WORKING_DIRECTORY ${OBOE_RUST_CORE_DIR}
            DEPENDS
                    ${OBOE_RUST_CORE_DIR}/Cargo.toml
                    ${OBOE_RUST_CORE_DIR}/Cargo.lock
                    ${OBOE_RUST_CORE_DIR}/src/aaudio.rs
                    ${OBOE_RUST_CORE_DIR}/src/extensions.rs
                    ${OBOE_RUST_CORE_DIR}/src/fifo.rs
                    ${OBOE_RUST_CORE_DIR}/src/lib.rs
                    ${OBOE_RUST_CORE_DIR}/src/opensles.rs
                    ${OBOE_RUST_CORE_DIR}/src/resampler.rs
                    ${OBOE_RUST_CORE_DIR}/src/stream.rs
            VERBATIM)
    add_custom_target(oboe_rust_core_build DEPENDS ${OBOE_RUST_LIBRARY_PATH})
    add_library(oboe_rust_core STATIC IMPORTED GLOBAL)
    set_target_properties(oboe_rust_core PROPERTIES
            IMPORTED_LOCATION ${OBOE_RUST_LIBRARY_PATH})
    add_dependencies(oboe_rust_core oboe_rust_core_build)

    target_compile_definitions(oboe PRIVATE OBOE_USE_RUST_CORE=1)
    target_link_libraries(oboe PRIVATE oboe_rust_core)
endif()

# When installing oboe put the libraries in the lib/<ABI> folder e.g. lib/arm64-v8a
install(TARGETS oboe
        LIBRARY DESTINATION lib/${ANDROID_ABI}
        ARCHIVE DESTINATION lib/${ANDROID_ABI})

# Also install the headers
install(DIRECTORY include/oboe DESTINATION include)
