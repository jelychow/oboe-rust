/*
 * Copyright 2019 The Android Open Source Project
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

#include "LinearResampler.h"
#if OBOE_USE_RUST_CORE
#include "rust/oboe_rust_core.h"
#endif

using namespace RESAMPLER_OUTER_NAMESPACE::resampler;

LinearResampler::LinearResampler(const MultiChannelResampler::Builder &builder)
        : MultiChannelResampler(builder) {
    mPreviousFrame = std::make_unique<float[]>(getChannelCount());
    mCurrentFrame = std::make_unique<float[]>(getChannelCount());
}

void LinearResampler::writeFrame(const float *frame) {
#if OBOE_USE_RUST_CORE
    oboe_rust_copy_float_buffer(mCurrentFrame.get(), mPreviousFrame.get(), getChannelCount());
    oboe_rust_copy_float_buffer(frame, mCurrentFrame.get(), getChannelCount());
#else
    memcpy(mPreviousFrame.get(), mCurrentFrame.get(), sizeof(float) * getChannelCount());
    memcpy(mCurrentFrame.get(), frame, sizeof(float) * getChannelCount());
#endif
}

void LinearResampler::readFrame(float *frame) {
#if OBOE_USE_RUST_CORE
    oboe_rust_linear_resampler_read_frame(mPreviousFrame.get(), mCurrentFrame.get(),
                                          frame, getChannelCount(), getIntegerPhase(),
                                          mDenominator);
#else
    float *previous = mPreviousFrame.get();
    float *current = mCurrentFrame.get();
    float phase = (float) getIntegerPhase() / mDenominator;
    // iterate across samples in the frame
    for (int channel = 0; channel < getChannelCount(); channel++) {
        float f0 = *previous++;
        float f1 = *current++;
        *frame++ = f0 + (phase * (f1 - f0));
    }
#endif
}
