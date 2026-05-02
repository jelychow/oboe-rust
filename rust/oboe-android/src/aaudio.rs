use crate::backend::AudioBackend;
use oboe_core::builder::StreamBuilder;
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
            platform: platform::AAudioPlatformStream::open(builder)?,
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
        self.core.set_route_device_id(device_id)
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

#[cfg(target_os = "android")]
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
    use oboe_core::format::{float_to_i16, i16_to_float};
    use oboe_core::types::{Direction, PerformanceMode, SharingMode};

    const AAUDIO_OK: i32 = 0;
    const AAUDIO_DIRECTION_OUTPUT: i32 = 0;
    const AAUDIO_DIRECTION_INPUT: i32 = 1;
    const AAUDIO_FORMAT_PCM_I16: i32 = 1;
    const AAUDIO_FORMAT_PCM_FLOAT: i32 = 2;

    #[repr(C)]
    struct AAudioStreamBuilder {
        _private: [u8; 0],
    }

    #[repr(C)]
    struct AAudioStream {
        _private: [u8; 0],
    }

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
        fn AAudioStream_close(stream: *mut AAudioStream) -> i32;
    }

    pub(super) struct AAudioPlatformStream {
        stream: *mut AAudioStream,
    }

    impl core::fmt::Debug for AAudioPlatformStream {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("AAudioPlatformStream")
                .field("stream", &self.stream)
                .finish()
        }
    }

    unsafe impl Send for AAudioPlatformStream {}

    impl AAudioPlatformStream {
        pub(super) fn open(builder: &StreamBuilder) -> Result<Self> {
            let mut stream_builder = ptr::null_mut();
            let create_result = unsafe { AAudio_createStreamBuilder(&mut stream_builder) };
            if create_result != AAUDIO_OK || stream_builder.is_null() {
                return Err(Error::BackendUnavailable);
            }

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

            Ok(Self { stream })
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
}

#[cfg(not(target_os = "android"))]
mod platform {
    use super::*;

    #[derive(Debug)]
    pub(super) struct AAudioPlatformStream;

    impl AAudioPlatformStream {
        pub(super) fn open(_builder: &StreamBuilder) -> Result<Self> {
            Ok(Self)
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
            _channel_count: i32,
            format: Format,
        ) -> Result<i32> {
            validate_first_phase_format(format)?;
            Ok(audio.len() as i32)
        }

        pub(super) fn read_f32(
            &mut self,
            audio: &mut [f32],
            _timeout_nanos: i64,
            _channel_count: i32,
            format: Format,
        ) -> Result<i32> {
            validate_first_phase_format(format)?;
            for sample in audio.iter_mut() {
                *sample = 0.0;
            }
            Ok(audio.len() as i32)
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
