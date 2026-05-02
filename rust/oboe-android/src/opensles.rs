use crate::backend::AudioBackend;
use oboe_core::builder::StreamBuilder;
use oboe_core::error::{Error, Result};
use oboe_core::extensions::{
    CallbackConfig, OffloadDelayPadding, PlaybackParameters, PresentationTimestamp,
};
use oboe_core::stream::{StreamCore, StreamState};
use oboe_core::types::Format;

#[derive(Debug)]
pub struct OpenSLESBackend {
    core: StreamCore,
    channel_count: i32,
    format: Format,
    platform: platform::OpenSLESPlatformStream,
}

#[cfg(target_os = "android")]
unsafe impl Send for OpenSLESBackend {}

impl AudioBackend for OpenSLESBackend {
    fn open(builder: &StreamBuilder) -> Result<Self> {
        builder.validate()?;
        validate_first_phase_format(builder.format)?;
        Ok(Self {
            core: StreamCore::new_open_with_builder(builder)?,
            channel_count: builder.channel_count,
            format: builder.format,
            platform: platform::OpenSLESPlatformStream::open(builder)?,
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

    fn set_offload_delay_padding(&mut self, _delay_padding: OffloadDelayPadding) -> Result<()> {
        if self.core.state() == StreamState::Closed {
            Err(Error::Closed)
        } else {
            Err(Error::Unimplemented)
        }
    }

    fn set_offload_end_of_stream(&mut self) -> Result<()> {
        if self.core.state() == StreamState::Closed {
            Err(Error::Closed)
        } else {
            Err(Error::Unimplemented)
        }
    }

    fn set_playback_parameters(&mut self, _parameters: PlaybackParameters) -> Result<()> {
        if self.core.state() == StreamState::Closed {
            Err(Error::Closed)
        } else {
            Err(Error::Unimplemented)
        }
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
    if channel_count == 0 || !sample_count.is_multiple_of(channel_count) {
        return Err(Error::InvalidArgument);
    }
    Ok(())
}

#[cfg(target_os = "android")]
mod platform {
    use super::*;
    use core::ffi::c_void;
    use core::ptr;
    use oboe_core::format::float_to_i16;

    type SLboolean = u32;
    type SLuint32 = u32;
    type SLresult = u32;
    type SLInterfaceID = *const SLInterfaceID_;
    type SLObjectItf = *const *const SLObjectItf_;
    type SLEngineItf = *const *const SLEngineItf_;
    type SLPlayItf = *const *const SLPlayItf_;
    type SLAndroidSimpleBufferQueueItf = *const *const SLAndroidSimpleBufferQueueItf_;

    const SL_BOOLEAN_FALSE: SLboolean = 0;
    const SL_BOOLEAN_TRUE: SLboolean = 1;
    const SL_RESULT_SUCCESS: SLresult = 0;
    const SL_DATAFORMAT_PCM: SLuint32 = 0x0000_0002;
    const SL_DATALOCATOR_OUTPUTMIX: SLuint32 = 0x0000_0004;
    const SL_DATALOCATOR_ANDROIDSIMPLEBUFFERQUEUE: SLuint32 = 0x8000_07BD;
    const SL_PCMSAMPLEFORMAT_FIXED_16: SLuint32 = 0x0010;
    const SL_BYTEORDER_LITTLEENDIAN: SLuint32 = 0x0000_0002;
    const SL_SPEAKER_FRONT_LEFT: SLuint32 = 0x0000_0001;
    const SL_SPEAKER_FRONT_RIGHT: SLuint32 = 0x0000_0002;
    const SL_SPEAKER_FRONT_CENTER: SLuint32 = 0x0000_0004;
    const SL_PLAYSTATE_STOPPED: SLuint32 = 0x0000_0001;
    const SL_PLAYSTATE_PLAYING: SLuint32 = 0x0000_0003;

    #[repr(C)]
    struct SLInterfaceID_ {
        time_low: SLuint32,
        time_mid: u16,
        time_hi_and_version: u16,
        clock_seq: u16,
        node: [u8; 6],
    }

    #[repr(C)]
    struct SLDataLocatorAndroidSimpleBufferQueue {
        locator_type: SLuint32,
        num_buffers: SLuint32,
    }

    #[repr(C)]
    struct SLDataLocatorOutputMix {
        locator_type: SLuint32,
        output_mix: SLObjectItf,
    }

    #[repr(C)]
    struct SLDataFormatPcm {
        format_type: SLuint32,
        num_channels: SLuint32,
        samples_per_sec: SLuint32,
        bits_per_sample: SLuint32,
        container_size: SLuint32,
        channel_mask: SLuint32,
        endianness: SLuint32,
    }

    #[repr(C)]
    struct SLDataSource {
        locator: *mut c_void,
        format: *mut c_void,
    }

    #[repr(C)]
    struct SLDataSink {
        locator: *mut c_void,
        format: *mut c_void,
    }

    #[allow(non_snake_case)]
    #[repr(C)]
    struct SLObjectItf_ {
        Realize: Option<unsafe extern "C" fn(SLObjectItf, SLboolean) -> SLresult>,
        Resume: Option<unsafe extern "C" fn(SLObjectItf, SLboolean) -> SLresult>,
        GetState: Option<unsafe extern "C" fn(SLObjectItf, *mut SLuint32) -> SLresult>,
        GetInterface:
            Option<unsafe extern "C" fn(SLObjectItf, SLInterfaceID, *mut c_void) -> SLresult>,
        RegisterCallback: *const c_void,
        AbortAsyncOperation: *const c_void,
        Destroy: Option<unsafe extern "C" fn(SLObjectItf)>,
    }

    #[allow(non_snake_case)]
    #[repr(C)]
    struct SLEngineItf_ {
        CreateLEDDevice: *const c_void,
        CreateVibraDevice: *const c_void,
        CreateAudioPlayer: Option<
            unsafe extern "C" fn(
                SLEngineItf,
                *mut SLObjectItf,
                *mut SLDataSource,
                *mut SLDataSink,
                SLuint32,
                *const SLInterfaceID,
                *const SLboolean,
            ) -> SLresult,
        >,
        CreateAudioRecorder: *const c_void,
        CreateMidiPlayer: *const c_void,
        CreateListener: *const c_void,
        Create3DGroup: *const c_void,
        CreateOutputMix: Option<
            unsafe extern "C" fn(
                SLEngineItf,
                *mut SLObjectItf,
                SLuint32,
                *const SLInterfaceID,
                *const SLboolean,
            ) -> SLresult,
        >,
    }

    #[allow(non_snake_case)]
    #[repr(C)]
    struct SLPlayItf_ {
        SetPlayState: Option<unsafe extern "C" fn(SLPlayItf, SLuint32) -> SLresult>,
    }

    #[allow(non_snake_case)]
    #[repr(C)]
    struct SLAndroidSimpleBufferQueueItf_ {
        Enqueue: Option<
            unsafe extern "C" fn(
                SLAndroidSimpleBufferQueueItf,
                *const c_void,
                SLuint32,
            ) -> SLresult,
        >,
        Clear: Option<unsafe extern "C" fn(SLAndroidSimpleBufferQueueItf) -> SLresult>,
    }

    #[link(name = "OpenSLES")]
    extern "C" {
        static SL_IID_ENGINE: SLInterfaceID;
        static SL_IID_PLAY: SLInterfaceID;
        static SL_IID_ANDROIDSIMPLEBUFFERQUEUE: SLInterfaceID;

        fn slCreateEngine(
            engine: *mut SLObjectItf,
            num_options: SLuint32,
            options: *const c_void,
            num_interfaces: SLuint32,
            interface_ids: *const SLInterfaceID,
            interface_required: *const SLboolean,
        ) -> SLresult;
    }

    pub(super) struct OpenSLESPlatformStream {
        engine_object: SLObjectItf,
        output_mix_object: SLObjectItf,
        player_object: SLObjectItf,
        play: SLPlayItf,
        queue: SLAndroidSimpleBufferQueueItf,
        queued_i16: Vec<i16>,
    }

    impl core::fmt::Debug for OpenSLESPlatformStream {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.debug_struct("OpenSLESPlatformStream")
                .field("engine_object", &self.engine_object)
                .field("output_mix_object", &self.output_mix_object)
                .field("player_object", &self.player_object)
                .finish()
        }
    }

    unsafe impl Send for OpenSLESPlatformStream {}

    impl OpenSLESPlatformStream {
        pub(super) fn open(builder: &StreamBuilder) -> Result<Self> {
            let mut engine_object = ptr::null();
            sl_result(unsafe {
                slCreateEngine(
                    &mut engine_object,
                    0,
                    ptr::null(),
                    0,
                    ptr::null(),
                    ptr::null(),
                )
            })?;
            realize(engine_object)?;

            let mut engine = ptr::null();
            get_interface(engine_object, unsafe { SL_IID_ENGINE }, &mut engine)?;

            let mut output_mix_object = ptr::null();
            create_output_mix(engine, &mut output_mix_object)?;
            realize(output_mix_object)?;

            let mut queue_locator = SLDataLocatorAndroidSimpleBufferQueue {
                locator_type: SL_DATALOCATOR_ANDROIDSIMPLEBUFFERQUEUE,
                num_buffers: 2,
            };
            let mut pcm = SLDataFormatPcm {
                format_type: SL_DATAFORMAT_PCM,
                num_channels: builder.channel_count as SLuint32,
                samples_per_sec: sample_rate_millihertz(builder.sample_rate)?,
                bits_per_sample: SL_PCMSAMPLEFORMAT_FIXED_16,
                container_size: SL_PCMSAMPLEFORMAT_FIXED_16,
                channel_mask: channel_mask(builder.channel_count)?,
                endianness: SL_BYTEORDER_LITTLEENDIAN,
            };
            let mut source = SLDataSource {
                locator: (&mut queue_locator as *mut SLDataLocatorAndroidSimpleBufferQueue).cast(),
                format: (&mut pcm as *mut SLDataFormatPcm).cast(),
            };

            let mut output_locator = SLDataLocatorOutputMix {
                locator_type: SL_DATALOCATOR_OUTPUTMIX,
                output_mix: output_mix_object,
            };
            let mut sink = SLDataSink {
                locator: (&mut output_locator as *mut SLDataLocatorOutputMix).cast(),
                format: ptr::null_mut(),
            };

            let interface_ids = unsafe { [SL_IID_ANDROIDSIMPLEBUFFERQUEUE, SL_IID_PLAY] };
            let interface_required = [SL_BOOLEAN_TRUE, SL_BOOLEAN_TRUE];
            let mut player_object = ptr::null();
            create_audio_player(
                engine,
                &mut player_object,
                &mut source,
                &mut sink,
                &interface_ids,
                &interface_required,
            )?;
            realize(player_object)?;

            let mut play = ptr::null();
            get_interface(player_object, unsafe { SL_IID_PLAY }, &mut play)?;
            let mut queue = ptr::null();
            get_interface(
                player_object,
                unsafe { SL_IID_ANDROIDSIMPLEBUFFERQUEUE },
                &mut queue,
            )?;

            Ok(Self {
                engine_object,
                output_mix_object,
                player_object,
                play,
                queue,
                queued_i16: Vec::new(),
            })
        }

        pub(super) fn request_start(&mut self) -> Result<()> {
            set_play_state(self.play, SL_PLAYSTATE_PLAYING)
        }

        pub(super) fn request_stop(&mut self) -> Result<()> {
            set_play_state(self.play, SL_PLAYSTATE_STOPPED)?;
            clear_queue(self.queue)
        }

        pub(super) fn close(&mut self) -> Result<()> {
            self.destroy_player();
            self.destroy_output_mix();
            self.destroy_engine();
            Ok(())
        }

        pub(super) fn write_f32(
            &mut self,
            audio: &[f32],
            _timeout_nanos: i64,
            _channel_count: i32,
            format: Format,
        ) -> Result<i32> {
            match format {
                Format::Float | Format::I16 => {}
                Format::Unspecified | Format::I24 | Format::I32 => {
                    return Err(Error::InvalidArgument)
                }
            }
            self.queued_i16.clear();
            self.queued_i16
                .extend(audio.iter().copied().map(float_to_i16));
            let byte_count = self
                .queued_i16
                .len()
                .checked_mul(core::mem::size_of::<i16>())
                .and_then(|bytes| SLuint32::try_from(bytes).ok())
                .ok_or(Error::InvalidArgument)?;
            enqueue(self.queue, self.queued_i16.as_ptr().cast(), byte_count)?;
            Ok(audio.len() as i32)
        }

        pub(super) fn read_f32(
            &mut self,
            audio: &mut [f32],
            _timeout_nanos: i64,
            _channel_count: i32,
            format: Format,
        ) -> Result<i32> {
            match format {
                Format::Float | Format::I16 => {}
                Format::Unspecified | Format::I24 | Format::I32 => {
                    return Err(Error::InvalidArgument)
                }
            }
            for sample in audio.iter_mut() {
                *sample = 0.0;
            }
            Ok(audio.len() as i32)
        }

        fn destroy_player(&mut self) {
            destroy_object(&mut self.player_object);
            self.play = ptr::null();
            self.queue = ptr::null();
            self.queued_i16.clear();
        }

        fn destroy_output_mix(&mut self) {
            destroy_object(&mut self.output_mix_object);
        }

        fn destroy_engine(&mut self) {
            destroy_object(&mut self.engine_object);
        }
    }

    impl Drop for OpenSLESPlatformStream {
        fn drop(&mut self) {
            let _ = self.close();
        }
    }

    fn sl_result(result: SLresult) -> Result<()> {
        if result == SL_RESULT_SUCCESS {
            Ok(())
        } else {
            Err(Error::InvalidState)
        }
    }

    fn object_vtable(object: SLObjectItf) -> Result<&'static SLObjectItf_> {
        if object.is_null() {
            return Err(Error::Closed);
        }
        let table = unsafe { *object };
        if table.is_null() {
            return Err(Error::InvalidState);
        }
        Ok(unsafe { &*table })
    }

    fn engine_vtable(engine: SLEngineItf) -> Result<&'static SLEngineItf_> {
        if engine.is_null() {
            return Err(Error::InvalidState);
        }
        let table = unsafe { *engine };
        if table.is_null() {
            return Err(Error::InvalidState);
        }
        Ok(unsafe { &*table })
    }

    fn play_vtable(play: SLPlayItf) -> Result<&'static SLPlayItf_> {
        if play.is_null() {
            return Err(Error::Closed);
        }
        let table = unsafe { *play };
        if table.is_null() {
            return Err(Error::InvalidState);
        }
        Ok(unsafe { &*table })
    }

    fn queue_vtable(
        queue: SLAndroidSimpleBufferQueueItf,
    ) -> Result<&'static SLAndroidSimpleBufferQueueItf_> {
        if queue.is_null() {
            return Err(Error::Closed);
        }
        let table = unsafe { *queue };
        if table.is_null() {
            return Err(Error::InvalidState);
        }
        Ok(unsafe { &*table })
    }

    fn realize(object: SLObjectItf) -> Result<()> {
        let realize = object_vtable(object)?.Realize.ok_or(Error::InvalidState)?;
        sl_result(unsafe { realize(object, SL_BOOLEAN_FALSE) })
    }

    fn get_interface<T>(object: SLObjectItf, iid: SLInterfaceID, out: &mut T) -> Result<()> {
        let get_interface = object_vtable(object)?
            .GetInterface
            .ok_or(Error::InvalidState)?;
        sl_result(unsafe { get_interface(object, iid, (out as *mut T).cast()) })
    }

    fn create_output_mix(engine: SLEngineItf, out: &mut SLObjectItf) -> Result<()> {
        let create = engine_vtable(engine)?
            .CreateOutputMix
            .ok_or(Error::InvalidState)?;
        sl_result(unsafe { create(engine, out, 0, ptr::null(), ptr::null()) })
    }

    fn create_audio_player(
        engine: SLEngineItf,
        out: &mut SLObjectItf,
        source: &mut SLDataSource,
        sink: &mut SLDataSink,
        interface_ids: &[SLInterfaceID],
        interface_required: &[SLboolean],
    ) -> Result<()> {
        let create = engine_vtable(engine)?
            .CreateAudioPlayer
            .ok_or(Error::InvalidState)?;
        sl_result(unsafe {
            create(
                engine,
                out,
                source,
                sink,
                interface_ids.len() as SLuint32,
                interface_ids.as_ptr(),
                interface_required.as_ptr(),
            )
        })
    }

    fn set_play_state(play: SLPlayItf, state: SLuint32) -> Result<()> {
        let set = play_vtable(play)?.SetPlayState.ok_or(Error::InvalidState)?;
        sl_result(unsafe { set(play, state) })
    }

    fn enqueue(
        queue: SLAndroidSimpleBufferQueueItf,
        buffer: *const c_void,
        bytes: SLuint32,
    ) -> Result<()> {
        let enqueue = queue_vtable(queue)?.Enqueue.ok_or(Error::InvalidState)?;
        sl_result(unsafe { enqueue(queue, buffer, bytes) })
    }

    fn clear_queue(queue: SLAndroidSimpleBufferQueueItf) -> Result<()> {
        let clear = queue_vtable(queue)?.Clear.ok_or(Error::InvalidState)?;
        sl_result(unsafe { clear(queue) })
    }

    fn destroy_object(object: &mut SLObjectItf) {
        if object.is_null() {
            return;
        }
        if let Ok(table) = object_vtable(*object) {
            if let Some(destroy) = table.Destroy {
                unsafe {
                    destroy(*object);
                }
            }
        }
        *object = ptr::null();
    }

    fn sample_rate_millihertz(sample_rate: i32) -> Result<SLuint32> {
        let sample_rate = if sample_rate > 0 { sample_rate } else { 48_000 };
        let milli_hertz = sample_rate
            .checked_mul(1_000)
            .ok_or(Error::InvalidArgument)?;
        SLuint32::try_from(milli_hertz).map_err(|_| Error::InvalidArgument)
    }

    fn channel_mask(channel_count: i32) -> Result<SLuint32> {
        match channel_count {
            1 => Ok(SL_SPEAKER_FRONT_CENTER),
            2 => Ok(SL_SPEAKER_FRONT_LEFT | SL_SPEAKER_FRONT_RIGHT),
            _ => Err(Error::InvalidArgument),
        }
    }
}

#[cfg(not(target_os = "android"))]
mod platform {
    use super::*;

    #[derive(Debug)]
    pub(super) struct OpenSLESPlatformStream;

    impl OpenSLESPlatformStream {
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
    fn opensles_backend_supports_core_lifecycle_before_real_ffi() {
        let mut backend = OpenSLESBackend::open(&StreamBuilder::default()).unwrap();
        assert_eq!(backend.state(), StreamState::Open);
        assert_eq!(backend.request_start(), Ok(()));
        assert_eq!(backend.state(), StreamState::Started);
        assert_eq!(backend.request_stop(), Ok(()));
        assert_eq!(backend.state(), StreamState::Stopped);
        assert_eq!(backend.close(), Ok(()));
        assert_eq!(backend.state(), StreamState::Closed);
        assert_eq!(backend.request_stop(), Err(Error::Closed));
    }

    #[test]
    fn opensles_backend_rejects_invalid_builder() {
        let builder = StreamBuilder {
            channel_count: 0,
            ..StreamBuilder::default()
        };

        assert_eq!(
            OpenSLESBackend::open(&builder).unwrap_err(),
            Error::InvalidArgument
        );
    }

    #[test]
    fn opensles_backend_reads_and_writes_float_buffers_on_host() {
        let mut backend = OpenSLESBackend::open(&StreamBuilder::default()).unwrap();
        assert_eq!(backend.write_f32(&[0.0, 0.25, -0.25, 0.5], 0), Ok(4));
        let mut audio = [1.0, 1.0];
        assert_eq!(backend.read_f32(&mut audio, 0), Ok(2));
        assert_eq!(audio, [0.0, 0.0]);
    }

    #[test]
    fn opensles_backend_rejects_unsupported_first_phase_format() {
        let builder = StreamBuilder {
            format: Format::I24,
            ..StreamBuilder::default()
        };
        assert_eq!(
            OpenSLESBackend::open(&builder).unwrap_err(),
            Error::InvalidArgument
        );
    }
}
