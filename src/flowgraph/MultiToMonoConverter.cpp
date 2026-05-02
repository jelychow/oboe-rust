/*
 * Copyright 2015 The Android Open Source Project
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

#include <unistd.h>
#include "FlowGraphNode.h"
#include "MultiToMonoConverter.h"
#if OBOE_USE_RUST_CORE
#include "rust/oboe_rust_core.h"
#endif

using namespace FLOWGRAPH_OUTER_NAMESPACE::flowgraph;

MultiToMonoConverter::MultiToMonoConverter(int32_t inputChannelCount)
        : input(*this, inputChannelCount)
        , output(*this, 1) {
}

MultiToMonoConverter::~MultiToMonoConverter() = default;

int32_t MultiToMonoConverter::onProcess(int32_t numFrames) {
    const float *inputBuffer = input.getBuffer();
    float *outputBuffer = output.getBuffer();
    int32_t channelCount = input.getSamplesPerFrame();
#if OBOE_USE_RUST_CORE
    oboe_rust_multi_to_mono(inputBuffer, outputBuffer, numFrames, channelCount);
#else
    for (int i = 0; i < numFrames; i++) {
        // read first channel of multi stream, write many
        *outputBuffer++ = *inputBuffer;
        inputBuffer += channelCount;
    }
#endif
    return numFrames;
}

