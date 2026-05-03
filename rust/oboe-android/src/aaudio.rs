use crate::backend::AudioBackend;
use oboe_core::builder::StreamBuilder;
use oboe_core::callback::{AudioStreamCallback, RouteChange};
use oboe_core::error::{Error, Result};
use oboe_core::extensions::{
    CallbackConfig, OffloadDelayPadding, PlaybackParameters, PresentationTimestamp,
};
use oboe_core::stream::{StreamCore, StreamState};
use oboe_core::types::Format;

#[derive(Debug)]
pub struct AAudioBackend {
    core: StreamCore,
    channel_count: i32,
    format: Format,
    platform: platform::AAudioPlatformStream,
}

// SAFETY: AAudioBackend owns a single AAudio stream handle. Native callback
// state is constrained to `Send + Sync` callbacks and the stream handle is
// closed before callback state is dropped.
#[cfg(target_os = "android")]
unsafe impl Send for AAudioBackend {}

impl AudioBackend for AAudioBackend {
    fn open(builder: &StreamBuilder) -> Result<Self> {
        builder.validate()?;
        validate_first_phase_format(builder.format)?;
        Ok(Self {
            core: StreamCore::new_open_with_builder(builder)?,
            channel_count: builder.channel_count,
            format: builder.format,
            platform: platform::AAudioPlatformStream::open(builder, None)?,
        })
    }

    fn request_start(&mut self) -> Result<()> {
        if self.core.state() == StreamState::Closed {
            return Err(Error::Closed);
        }
        self.platform.request_start()?;
        self.core.request_start()
    }

    fn request_stop(&mut self) -> Result<()> {
        if self.core.state() == StreamState::Closed {
            return Err(Error::Closed);
        }
        self.platform.request_stop()?;
        self.core.request_stop()
    }

    fn close(&mut self) -> Result<()> {
        if self.core.state() == StreamState::Closed {
            return Err(Error::Closed);
        }
        self.platform.close()?;
        self.core.close()
    }

    fn state(&self) -> StreamState {
        self.core.state()
    }

    fn write_f32(&mut self, audio: &[f32], timeout_nanos: i64) -> Result<i32> {
        validate_buffer_len(audio.len(), self.channel_count)?;
        self.platform
            .write_f32(audio, timeout_nanos, self.channel_count, self.format)
    }

    fn read_f32(&mut self, audio: &mut [f32], timeout_nanos: i64) -> Result<i32> {
        validate_buffer_len(audio.len(), self.channel_count)?;
        self.platform
            .read_f32(audio, timeout_nanos, self.channel_count, self.format)
    }

    fn get_timestamp(&self) -> Result<PresentationTimestamp> {
        self.platform.get_timestamp()
    }

    fn get_frames_read(&self) -> Result<i64> {
        self.platform.get_frames_read()
    }

    fn get_frames_written(&self) -> Result<i64> {
        self.platform.get_frames_written()
    }

    fn get_xrun_count(&self) -> Result<i32> {
        self.platform.get_xrun_count()
    }

    fn get_frames_per_burst(&self) -> Result<i32> {
        self.platform.get_frames_per_burst()
    }

    fn get_buffer_size_in_frames(&self) -> Result<i32> {
        self.platform.get_buffer_size_in_frames()
    }

    fn set_buffer_size_in_frames(&mut self, frames: i32) -> Result<i32> {
        self.platform.set_buffer_size_in_frames(frames)
    }

    fn get_buffer_capacity_in_frames(&self) -> Result<i32> {
        self.platform.get_buffer_capacity_in_frames()
    }

    fn get_and_clear_last_error(&mut self) -> Result<i32> {
        self.platform.get_and_clear_last_error()
    }

    fn set_callback_config(&mut self, config: CallbackConfig) -> Result<()> {
        self.core.set_callback_config(config)
    }

    fn set_offload_delay_padding(&mut self, delay_padding: OffloadDelayPadding) -> Result<()> {
        self.core.set_offload_delay_padding(delay_padding)
    }

    fn set_offload_end_of_stream(&mut self) -> Result<()> {
        self.core.set_offload_end_of_stream()
    }

    fn set_playback_parameters(&mut self, parameters: PlaybackParameters) -> Result<()> {
        self.core.set_playback_parameters(parameters)
    }

    fn set_presentation_timestamp(&mut self, timestamp: PresentationTimestamp) -> Result<()> {
        self.core.set_presentation_timestamp(timestamp)
    }

    fn set_route_device_id(&mut self, device_id: i32) -> Result<()> {
        self.core.set_route_device_id(device_id)?;
        self.platform.notify_route_changed(device_id);
        Ok(())
    }
}

impl AAudioBackend {
    pub fn open_with_callback(
        builder: &StreamBuilder,
        callback: Box<dyn AudioStreamCallback>,
    ) -> Result<Self> {
        builder.validate()?;
        validate_first_phase_format(builder.format)?;
        if builder.format != Format::Float {
            return Err(Error::InvalidArgument);
        }

        let mut callback_builder = builder.clone();
        callback_builder.callback_config.data_callback = true;
        Ok(Self {
            core: StreamCore::new_open_with_builder(&callback_builder)?,
            channel_count: callback_builder.channel_count,
            format: callback_builder.format,
            platform: platform::AAudioPlatformStream::open(&callback_builder, Some(callback))?,
        })
    }

    #[cfg(test)]
    fn inject_async_error_for_test(&mut self, error: i32) {
        self.platform.inject_async_error_for_test(error);
    }
}

fn validate_first_phase_format(format: Format) -> Result<()> {
    match format {
        Format::Float | Format::I16 => Ok(()),
        Format::Unspecified | Format::I24 | Format::I32 => Err(Error::InvalidArgument),
    }
}

fn validate_buffer_len(sample_count: usize, channel_count: i32) -> Result<()> {
    let channel_count = usize::try_from(channel_count).map_err(|_| Error::InvalidArgument)?;
    if channel_count == 0 || sample_count % channel_count != 0 {
        return Err(Error::InvalidArgument);
    }
    Ok(())
}

fn frame_count(sample_count: usize, channel_count: i32) -> Result<i32> {
    validate_buffer_len(sample_count, channel_count)?;
    let channels = usize::try_from(channel_count).map_err(|_| Error::InvalidArgument)?;
    i32::try_from(sample_count / channels).map_err(|_| Error::InvalidArgument)
}

#[cfg(target_os = "android")]
fn samples_from_frame_result(frames: i32, channel_count: i32) -> Result<i32> {
    if frames < 0 {
        return Err(Error::InvalidState);
    }
    frames.checked_mul(channel_count).ok_or(Error::InvalidState)
}

#[cfg(target_os = "android")]
mod platform {
    use super::*;
    use core::ffi::c_void;
    use core::ptr;
    use core::sync::atomic::{AtomicI32, Ordering};
    use oboe_core::callback::AudioCallbackInfo;
    use oboe_core::format::{float_to_i16, i16_to_float};
    use oboe_core::types::{Direction, PerformanceMode, SharingMode};

    const AAUDIO_OK: i32 = 0;
    const AAUDIO_CALLBACK_RESULT_CONTINUE: i32 = 0;
    const AAUDIO_CALLBACK_RESULT_STOP: i32 = 1;
    const AAUDIO_DIRECTION_OUTPUT: i32 = 0;
    const AAUDIO_DIRECTION_INPUT: i32 = 1;
    const AAUDIO_FORMAT_PCM_I16: i32 = 1;
    const AAUDIO_FORMAT_PCM_FLOAT: i32 = 2;
    const CLOCK_MONOTONIC: i32 = 1;

    #[repr(C)]
    struct AAudioStreamBuilder {
        _private: [u8; 0],
    }

    #[repr(C)]
    struct AAudioStream {
        _private: [u8; 0],
    }

    type AAudioStreamDataCallback = unsafe extern "C" fn(
        stream: *mut AAudioStream,
        user_data: *mut c_void,
        audio_data: *mut c_void,
        num_frames: i32,
    ) -> i32;

    type AAudioStreamErrorCallback =
        unsafe extern "C" fn(stream: *mut AAudioStream, user_data: *mut c_void, error: i32);

    #[link(name = "aaudio")]
    extern "C" {
        fn AAudio_createStreamBuilder(builder: *mut *mut AAudioStreamBuilder) -> i32;
        fn AAudioStreamBuilder_setDirection(builder: *mut AAudioStreamBuilder, direction: i32);
        fn AAudioStreamBuilder_setFormat(builder: *mut AAudioStreamBuilder, format: i32);
        fn AAudioStreamBuilder_setSampleRate(builder: *mut AAudioStreamBuilder, sample_rate: i32);
        fn AAudioStreamBuilder_setChannelCount(
            builder: *mut AAudioStreamBuilder,
            channel_count: i32,
        );
        fn AAudioStreamBuilder_setSharingMode(builder: *mut AAudioStreamBuilder, sharing_mode: i32);
        fn AAudioStreamBuilder_setPerformanceMode(
            builder: *mut AAudioStreamBuilder,
            performance_mode: i32,
        );
        fn AAudioStreamBuilder_setBufferCapacityInFrames(
            builder: *mut AAudioStreamBuilder,
            num_frames: i32,
        );
        fn AAudioStreamBuilder_setFramesPerDataCallback(
            builder: *mut AAudioStreamBuilder,
            num_frames: i32,
        );
        fn AAudioStreamBuilder_setDataCallback(
            builder: *mut AAudioStreamBuilder,
            callback: Option<AAudioStreamDataCallback>,
            user_data: *mut c_void,
        );
        fn AAudioStreamBuilder_setErrorCallback(
            builder: *mut AAudioStreamBuilder,
            callback: Option<AAudioStreamErrorCallback>,
            user_data: *mut c_void,
        );
        fn AAudioStreamBuilder_openStream(
            builder: *mut AAudioStreamBuilder,
            stream: *mut *mut AAudioStream,
        ) -> i32;
        fn AAudioStreamBuilder_delete(builder: *mut AAudioStreamBuilder) -> i32;
        fn AAudioStream_requestStart(stream: *mut AAudioStream) -> i32;
        fn AAudioStream_requestStop(stream: *mut AAudioStream) -> i32;
        fn AAudioStream_write(
            stream: *mut AAudioStream,
            buffer: *const c_void,
            num_frames: i32,
            timeout_nanos: i64,
        ) -> i32;
        fn AAudioStream_read(
            stream: *mut AAudioStream,
            buffer: *mut c_void,
            num_frames: i32,
            timeout_nanos: i64,
        ) -> i32;
        fn AAudioStream_getTimestamp(
            stream: *mut AAudioStream,
            clockid: i32,
            frame_position: *mut i64,
            time_nanoseconds: *mut i64,
        ) -> i32;
        fn AAudioStream_getFramesRead(stream: *mut AAudioStream) -> i64;
        fn AAudioStream_getFramesWritten(stream: *mut AAudioStream) -> i64;
        fn AAudioStream_getXRunCount(stream: *mut AAudioStream) -> i32;
        fn AAudioStream_getFramesPerBurst(stream: *mut AAudioStream) -> i32;
        fn AAudioStream_getBufferSizeInFrames(stream: *mut AAudioStream) -> i32;
        fn AAudioStream_setBufferSizeInFrames(stream: *mut AAudioStream, num_frames: i32) -> i32;
        fn AAudioStream_getBufferCapacityInFrames(stream: *mut AAudioStream) -> i32;
        fn AAudioStream_close(stream: *mut AAudioStream) -> i32;
    }

    struct AAudioStreamEvents {
        callback: Option<Box<dyn AudioStreamCallback>>,
        channel_count: i32,
        sample_rate: i32,
        input: bool,
        last_error: AtomicI32,
    }

    pub(super) struct AAudioPlatformStream {
        stream: *mut AAudioStream,
        event_state: Box<AAudioStreamEvents>,
    }

    impl core::fmt::Debug for AAudioPlatformStream {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("AAudioPlatformStream")
                .field("stream", &self.stream)
                .finish()
        }
    }

    // SAFETY: The raw AAudioStream pointer is owned by this value, closed in
    // Drop, and event state is `Send + Sync`. Data/error callbacks enter
    // through AAudio using the stable boxed event-state pointer.
    unsafe impl Send for AAudioPlatformStream {}

    impl AAudioPlatformStream {
        pub(super) fn open(
            builder: &StreamBuilder,
            callback: Option<Box<dyn AudioStreamCallback>>,
        ) -> Result<Self> {
            let mut event_state = Box::new(AAudioStreamEvents {
                callback,
                channel_count: builder.channel_count,
                sample_rate: if builder.sample_rate > 0 {
                    builder.sample_rate
                } else {
                    48_000
                },
                input: builder.direction == Direction::Input,
                last_error: AtomicI32::new(0),
            });
            let mut stream_builder = ptr::null_mut();
            let create_result = unsafe { AAudio_createStreamBuilder(&mut stream_builder) };
            if create_result != AAUDIO_OK || stream_builder.is_null() {
                return Err(Error::BackendUnavailable);
            }

            let user_data = (&mut *event_state as *mut AAudioStreamEvents).cast::<c_void>();

            unsafe {
                AAudioStreamBuilder_setDirection(
                    stream_builder,
                    direction_to_aaudio(builder.direction),
                );
                AAudioStreamBuilder_setFormat(stream_builder, format_to_aaudio(builder.format)?);
                AAudioStreamBuilder_setChannelCount(stream_builder, builder.channel_count);
                AAudioStreamBuilder_setSharingMode(
                    stream_builder,
                    sharing_mode_to_aaudio(builder.sharing_mode),
                );
                AAudioStreamBuilder_setPerformanceMode(
                    stream_builder,
                    performance_mode_to_aaudio(builder.performance_mode),
                );
                if builder.buffer_capacity_in_frames > 0 {
                    AAudioStreamBuilder_setBufferCapacityInFrames(
                        stream_builder,
                        builder.buffer_capacity_in_frames,
                    );
                }
                if builder.frames_per_callback > 0 {
                    AAudioStreamBuilder_setFramesPerDataCallback(
                        stream_builder,
                        builder.frames_per_callback,
                    );
                }
                AAudioStreamBuilder_setErrorCallback(
                    stream_builder,
                    Some(error_callback),
                    user_data,
                );
                if event_state.callback.is_some() {
                    AAudioStreamBuilder_setDataCallback(
                        stream_builder,
                        Some(data_callback),
                        user_data,
                    );
                }
                if builder.sample_rate > 0 {
                    AAudioStreamBuilder_setSampleRate(stream_builder, builder.sample_rate);
                }
            }

            let mut stream = ptr::null_mut();
            let open_result =
                unsafe { AAudioStreamBuilder_openStream(stream_builder, &mut stream) };
            let delete_result = unsafe { AAudioStreamBuilder_delete(stream_builder) };
            if open_result != AAUDIO_OK || delete_result != AAUDIO_OK || stream.is_null() {
                if !stream.is_null() {
                    unsafe {
                        AAudioStream_close(stream);
                    }
                }
                return Err(Error::BackendUnavailable);
            }

            Ok(Self {
                stream,
                event_state,
            })
        }

        pub(super) fn request_start(&mut self) -> Result<()> {
            self.with_stream(|stream| unsafe { AAudioStream_requestStart(stream) })
        }

        pub(super) fn request_stop(&mut self) -> Result<()> {
            self.with_stream(|stream| unsafe { AAudioStream_requestStop(stream) })
        }

        pub(super) fn close(&mut self) -> Result<()> {
            if self.stream.is_null() {
                return Ok(());
            }
            let stream = self.stream;
            self.stream = ptr::null_mut();
            result_to_unit(unsafe { AAudioStream_close(stream) })
        }

        pub(super) fn write_f32(
            &mut self,
            audio: &[f32],
            timeout_nanos: i64,
            channel_count: i32,
            format: Format,
        ) -> Result<i32> {
            let frames = frame_count(audio.len(), channel_count)?;
            let written_frames = match format {
                Format::Float => self.write_raw(audio.as_ptr().cast(), frames, timeout_nanos)?,
                Format::I16 => {
                    let converted = audio.iter().copied().map(float_to_i16).collect::<Vec<_>>();
                    self.write_raw(converted.as_ptr().cast(), frames, timeout_nanos)?
                }
                Format::Unspecified | Format::I24 | Format::I32 => {
                    return Err(Error::InvalidArgument)
                }
            };
            samples_from_frame_result(written_frames, channel_count)
        }

        pub(super) fn read_f32(
            &mut self,
            audio: &mut [f32],
            timeout_nanos: i64,
            channel_count: i32,
            format: Format,
        ) -> Result<i32> {
            let frames = frame_count(audio.len(), channel_count)?;
            let read_frames = match format {
                Format::Float => self.read_raw(audio.as_mut_ptr().cast(), frames, timeout_nanos)?,
                Format::I16 => {
                    let mut converted = vec![0_i16; audio.len()];
                    let read =
                        self.read_raw(converted.as_mut_ptr().cast(), frames, timeout_nanos)?;
                    let samples = samples_from_frame_result(read, channel_count)? as usize;
                    for (out, input) in audio.iter_mut().zip(converted.into_iter()).take(samples) {
                        *out = i16_to_float(input);
                    }
                    return Ok(samples as i32);
                }
                Format::Unspecified | Format::I24 | Format::I32 => {
                    return Err(Error::InvalidArgument)
                }
            };
            samples_from_frame_result(read_frames, channel_count)
        }

        fn write_raw(
            &mut self,
            buffer: *const c_void,
            frames: i32,
            timeout_nanos: i64,
        ) -> Result<i32> {
            if self.stream.is_null() {
                return Err(Error::Closed);
            }
            let result = unsafe { AAudioStream_write(self.stream, buffer, frames, timeout_nanos) };
            if result < 0 {
                Err(Error::from_platform_result(result))
            } else {
                Ok(result)
            }
        }

        fn read_raw(
            &mut self,
            buffer: *mut c_void,
            frames: i32,
            timeout_nanos: i64,
        ) -> Result<i32> {
            if self.stream.is_null() {
                return Err(Error::Closed);
            }
            let result = unsafe { AAudioStream_read(self.stream, buffer, frames, timeout_nanos) };
            if result < 0 {
                Err(Error::from_platform_result(result))
            } else {
                Ok(result)
            }
        }

        fn with_stream(&mut self, f: impl FnOnce(*mut AAudioStream) -> i32) -> Result<()> {
            if self.stream.is_null() {
                return Err(Error::Closed);
            }
            result_to_unit(f(self.stream))
        }

        pub(super) fn get_timestamp(&self) -> Result<PresentationTimestamp> {
            if self.stream.is_null() {
                return Err(Error::Closed);
            }
            let mut frame_position = 0_i64;
            let mut timestamp_nanos = 0_i64;
            let result = unsafe {
                AAudioStream_getTimestamp(
                    self.stream,
                    CLOCK_MONOTONIC,
                    &mut frame_position,
                    &mut timestamp_nanos,
                )
            };
            if result == AAUDIO_OK {
                Ok(PresentationTimestamp {
                    frame_position,
                    timestamp_nanos,
                })
            } else {
                Err(Error::from_platform_result(result))
            }
        }

        pub(super) fn get_frames_read(&self) -> Result<i64> {
            self.frame_counter(|stream| unsafe { AAudioStream_getFramesRead(stream) })
        }

        pub(super) fn get_frames_written(&self) -> Result<i64> {
            self.frame_counter(|stream| unsafe { AAudioStream_getFramesWritten(stream) })
        }

        pub(super) fn get_xrun_count(&self) -> Result<i32> {
            self.i32_query(|stream| unsafe { AAudioStream_getXRunCount(stream) })
        }

        pub(super) fn get_frames_per_burst(&self) -> Result<i32> {
            self.i32_query(|stream| unsafe { AAudioStream_getFramesPerBurst(stream) })
        }

        pub(super) fn get_buffer_size_in_frames(&self) -> Result<i32> {
            self.i32_query(|stream| unsafe { AAudioStream_getBufferSizeInFrames(stream) })
        }

        pub(super) fn set_buffer_size_in_frames(&mut self, frames: i32) -> Result<i32> {
            if frames <= 0 {
                return Err(Error::InvalidArgument);
            }
            if self.stream.is_null() {
                return Err(Error::Closed);
            }
            result_to_i32(unsafe { AAudioStream_setBufferSizeInFrames(self.stream, frames) })
        }

        pub(super) fn get_buffer_capacity_in_frames(&self) -> Result<i32> {
            self.i32_query(|stream| unsafe { AAudioStream_getBufferCapacityInFrames(stream) })
        }

        pub(super) fn get_and_clear_last_error(&mut self) -> Result<i32> {
            if self.stream.is_null() {
                return Err(Error::Closed);
            }
            Ok(self.event_state.last_error.swap(0, Ordering::AcqRel))
        }

        pub(super) fn notify_route_changed(&mut self, device_id: i32) {
            if let Some(callback) = self.event_state.callback.as_deref() {
                callback.on_route_changed(RouteChange {
                    device_id: Some(device_id),
                });
            }
        }

        #[cfg(test)]
        pub(super) fn inject_async_error_for_test(&mut self, error: i32) {
            self.event_state.last_error.store(error, Ordering::Release);
        }

        fn i32_query(&self, f: impl FnOnce(*mut AAudioStream) -> i32) -> Result<i32> {
            if self.stream.is_null() {
                return Err(Error::Closed);
            }
            result_to_i32(f(self.stream))
        }

        fn frame_counter(&self, f: impl FnOnce(*mut AAudioStream) -> i64) -> Result<i64> {
            if self.stream.is_null() {
                return Err(Error::Closed);
            }
            let result = f(self.stream);
            if result < 0 {
                Err(Error::Platform(result as i32))
            } else {
                Ok(result)
            }
        }
    }

    impl Drop for AAudioPlatformStream {
        fn drop(&mut self) {
            let _ = self.close();
        }
    }

    fn direction_to_aaudio(direction: Direction) -> i32 {
        match direction {
            Direction::Output => AAUDIO_DIRECTION_OUTPUT,
            Direction::Input => AAUDIO_DIRECTION_INPUT,
        }
    }

    fn format_to_aaudio(format: Format) -> Result<i32> {
        match format {
            Format::I16 => Ok(AAUDIO_FORMAT_PCM_I16),
            Format::Float => Ok(AAUDIO_FORMAT_PCM_FLOAT),
            Format::Unspecified | Format::I24 | Format::I32 => Err(Error::InvalidArgument),
        }
    }

    fn sharing_mode_to_aaudio(sharing_mode: SharingMode) -> i32 {
        match sharing_mode {
            SharingMode::Exclusive => 0,
            SharingMode::Shared => 1,
        }
    }

    fn performance_mode_to_aaudio(performance_mode: PerformanceMode) -> i32 {
        match performance_mode {
            PerformanceMode::None => 10,
            PerformanceMode::PowerSaving => 11,
            PerformanceMode::LowLatency => 12,
        }
    }

    fn result_to_unit(result: i32) -> Result<()> {
        if result == AAUDIO_OK {
            Ok(())
        } else {
            Err(Error::InvalidState)
        }
    }

    fn result_to_i32(result: i32) -> Result<i32> {
        if result < 0 {
            Err(Error::from_platform_result(result))
        } else {
            Ok(result)
        }
    }

    unsafe extern "C" fn data_callback(
        _stream: *mut AAudioStream,
        user_data: *mut c_void,
        audio_data: *mut c_void,
        num_frames: i32,
    ) -> i32 {
        if user_data.is_null() || audio_data.is_null() || num_frames < 0 {
            return AAUDIO_CALLBACK_RESULT_STOP;
        }
        let state = unsafe { &*user_data.cast::<AAudioStreamEvents>() };
        let Some(callback) = state.callback.as_deref() else {
            return AAUDIO_CALLBACK_RESULT_STOP;
        };
        let channel_count = match usize::try_from(state.channel_count) {
            Ok(channel_count) if channel_count > 0 => channel_count,
            _ => return AAUDIO_CALLBACK_RESULT_STOP,
        };
        let frame_count = match usize::try_from(num_frames) {
            Ok(frame_count) => frame_count,
            Err(_) => return AAUDIO_CALLBACK_RESULT_STOP,
        };
        let sample_count = match frame_count.checked_mul(channel_count) {
            Some(sample_count) => sample_count,
            None => return AAUDIO_CALLBACK_RESULT_STOP,
        };
        let audio_data =
            unsafe { core::slice::from_raw_parts_mut(audio_data.cast::<f32>(), sample_count) };
        let result = callback.on_audio_ready(
            AudioCallbackInfo {
                num_frames,
                channel_count: state.channel_count,
                sample_rate: state.sample_rate,
                input: state.input,
            },
            audio_data,
        );
        match result {
            oboe_core::extensions::DataCallbackResult::Continue => AAUDIO_CALLBACK_RESULT_CONTINUE,
            oboe_core::extensions::DataCallbackResult::Stop => AAUDIO_CALLBACK_RESULT_STOP,
        }
    }

    unsafe extern "C" fn error_callback(
        _stream: *mut AAudioStream,
        user_data: *mut c_void,
        error: i32,
    ) {
        if user_data.is_null() {
            return;
        }
        let state = unsafe { &*user_data.cast::<AAudioStreamEvents>() };
        state.last_error.store(error, Ordering::Release);
        if let Some(callback) = state.callback.as_deref() {
            callback.on_error(Error::from_platform_result(error));
        }
    }
}

#[cfg(not(target_os = "android"))]
mod platform {
    use super::*;

    pub(super) struct AAudioPlatformStream {
        sample_rate: i32,
        frames_read: i64,
        frames_written: i64,
        frames_per_burst: i32,
        buffer_capacity_in_frames: i32,
        buffer_size_in_frames: i32,
        xrun_count: i32,
        last_error: i32,
        callback: Option<Box<dyn AudioStreamCallback>>,
    }

    impl core::fmt::Debug for AAudioPlatformStream {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("AAudioPlatformStream")
                .field("sample_rate", &self.sample_rate)
                .field("frames_read", &self.frames_read)
                .field("frames_written", &self.frames_written)
                .field("frames_per_burst", &self.frames_per_burst)
                .field("buffer_capacity_in_frames", &self.buffer_capacity_in_frames)
                .field("buffer_size_in_frames", &self.buffer_size_in_frames)
                .field("xrun_count", &self.xrun_count)
                .field("last_error", &self.last_error)
                .finish()
        }
    }

    impl AAudioPlatformStream {
        pub(super) fn open(
            builder: &StreamBuilder,
            callback: Option<Box<dyn AudioStreamCallback>>,
        ) -> Result<Self> {
            let frames_per_burst = if builder.frames_per_callback > 0 {
                builder.frames_per_callback
            } else {
                192
            };
            let buffer_capacity_in_frames = if builder.buffer_capacity_in_frames > 0 {
                builder.buffer_capacity_in_frames
            } else {
                frames_per_burst * 4
            };
            Ok(Self {
                sample_rate: if builder.sample_rate > 0 {
                    builder.sample_rate
                } else {
                    48_000
                },
                frames_read: 0,
                frames_written: 0,
                frames_per_burst,
                buffer_capacity_in_frames,
                buffer_size_in_frames: buffer_capacity_in_frames,
                xrun_count: 0,
                last_error: 0,
                callback,
            })
        }

        pub(super) fn request_start(&mut self) -> Result<()> {
            Ok(())
        }

        pub(super) fn request_stop(&mut self) -> Result<()> {
            Ok(())
        }

        pub(super) fn close(&mut self) -> Result<()> {
            Ok(())
        }

        pub(super) fn write_f32(
            &mut self,
            audio: &[f32],
            _timeout_nanos: i64,
            channel_count: i32,
            format: Format,
        ) -> Result<i32> {
            validate_first_phase_format(format)?;
            self.frames_written += i64::from(frame_count(audio.len(), channel_count)?);
            Ok(audio.len() as i32)
        }

        pub(super) fn read_f32(
            &mut self,
            audio: &mut [f32],
            _timeout_nanos: i64,
            channel_count: i32,
            format: Format,
        ) -> Result<i32> {
            validate_first_phase_format(format)?;
            for sample in audio.iter_mut() {
                *sample = 0.0;
            }
            self.frames_read += i64::from(frame_count(audio.len(), channel_count)?);
            Ok(audio.len() as i32)
        }

        pub(super) fn get_timestamp(&self) -> Result<PresentationTimestamp> {
            let frame_position = self.frames_written.max(self.frames_read);
            Ok(PresentationTimestamp {
                frame_position,
                timestamp_nanos: frame_position * 1_000_000_000_i64 / i64::from(self.sample_rate),
            })
        }

        pub(super) fn get_frames_read(&self) -> Result<i64> {
            Ok(self.frames_read)
        }

        pub(super) fn get_frames_written(&self) -> Result<i64> {
            Ok(self.frames_written)
        }

        pub(super) fn get_xrun_count(&self) -> Result<i32> {
            Ok(self.xrun_count)
        }

        pub(super) fn get_frames_per_burst(&self) -> Result<i32> {
            Ok(self.frames_per_burst)
        }

        pub(super) fn get_buffer_size_in_frames(&self) -> Result<i32> {
            Ok(self.buffer_size_in_frames)
        }

        pub(super) fn set_buffer_size_in_frames(&mut self, frames: i32) -> Result<i32> {
            if frames <= 0 {
                return Err(Error::InvalidArgument);
            }
            self.buffer_size_in_frames = frames.min(self.buffer_capacity_in_frames);
            Ok(self.buffer_size_in_frames)
        }

        pub(super) fn get_buffer_capacity_in_frames(&self) -> Result<i32> {
            Ok(self.buffer_capacity_in_frames)
        }

        pub(super) fn get_and_clear_last_error(&mut self) -> Result<i32> {
            let error = self.last_error;
            self.last_error = 0;
            Ok(error)
        }

        pub(super) fn notify_route_changed(&mut self, device_id: i32) {
            if let Some(callback) = self.callback.as_deref() {
                callback.on_route_changed(RouteChange {
                    device_id: Some(device_id),
                });
            }
        }

        #[cfg(test)]
        pub(super) fn inject_async_error_for_test(&mut self, error: i32) {
            self.last_error = error;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aaudio_backend_supports_core_lifecycle_before_real_ffi() {
        let mut backend = AAudioBackend::open(&StreamBuilder::default()).unwrap();
        assert_eq!(backend.state(), StreamState::Open);
        assert_eq!(backend.request_start(), Ok(()));
        assert_eq!(backend.state(), StreamState::Started);
        assert_eq!(backend.request_stop(), Ok(()));
        assert_eq!(backend.state(), StreamState::Stopped);
        assert_eq!(backend.close(), Ok(()));
        assert_eq!(backend.state(), StreamState::Closed);
        assert_eq!(backend.request_start(), Err(Error::Closed));
    }

    #[test]
    fn aaudio_backend_rejects_invalid_builder() {
        let builder = StreamBuilder {
            channel_count: 0,
            ..StreamBuilder::default()
        };

        assert_eq!(
            AAudioBackend::open(&builder).unwrap_err(),
            Error::InvalidArgument
        );
    }

    #[test]
    fn aaudio_backend_reads_and_writes_float_buffers_on_host() {
        let mut backend = AAudioBackend::open(&StreamBuilder::default()).unwrap();
        assert_eq!(backend.write_f32(&[0.0, 0.25, -0.25, 0.5], 0), Ok(4));
        let mut audio = [1.0, 1.0];
        assert_eq!(backend.read_f32(&mut audio, 0), Ok(2));
        assert_eq!(audio, [0.0, 0.0]);
    }

    #[test]
    fn aaudio_backend_records_and_clears_async_errors_on_host() {
        let mut backend = AAudioBackend::open(&StreamBuilder::default()).unwrap();
        assert_eq!(backend.get_and_clear_last_error(), Ok(0));

        backend.inject_async_error_for_test(-899);

        assert_eq!(backend.get_and_clear_last_error(), Ok(-899));
        assert_eq!(backend.get_and_clear_last_error(), Ok(0));
    }

    #[test]
    fn aaudio_backend_rejects_unsupported_first_phase_format() {
        let builder = StreamBuilder {
            format: Format::I24,
            ..StreamBuilder::default()
        };
        assert_eq!(
            AAudioBackend::open(&builder).unwrap_err(),
            Error::InvalidArgument
        );
    }

    #[test]
    fn negative_platform_results_preserve_native_code() {
        assert_eq!(Error::from_platform_result(-899), Error::Platform(-899));
    }
}
