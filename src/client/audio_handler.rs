use cef_sys::cef_string_t;
use crate::string::CefString;
use crate::{
    browser::Browser,
    refcounted::{RefCountedPtr, Wrapper},
};
use cef_sys::{
    cef_browser_t,
    cef_audio_handler_t,
    cef_channel_layout_t,
    cef_audio_parameters_t,
};
use std::{sync::atomic::{AtomicUsize, Ordering}, os::raw::c_int, slice};

/// Enumerates the various representations of the ordering of audio channels.
#[repr(C)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum ChannelLayout {
    LayoutNone = cef_channel_layout_t::CEF_CHANNEL_LAYOUT_NONE as isize,
    LayoutUnsupported = cef_channel_layout_t::CEF_CHANNEL_LAYOUT_UNSUPPORTED as isize,
    /// Front C
    LayoutMono = cef_channel_layout_t::CEF_CHANNEL_LAYOUT_MONO as isize,
    /// Front L, Front R
    LayoutStereo = cef_channel_layout_t::CEF_CHANNEL_LAYOUT_STEREO as isize,
    /// Front L, Front R, Back C
    Layout2_1 = cef_channel_layout_t::CEF_CHANNEL_LAYOUT_2_1 as isize,
    /// Front L, Front R, Front C
    LayoutSurround = cef_channel_layout_t::CEF_CHANNEL_LAYOUT_SURROUND as isize,
    /// Front L, Front R, Front C, Back C
    Layout4_0 = cef_channel_layout_t::CEF_CHANNEL_LAYOUT_4_0 as isize,
    /// Front L, Front R, Side L, Side R
    Layout2_2 = cef_channel_layout_t::CEF_CHANNEL_LAYOUT_2_2 as isize,
    /// Front L, Front R, Back L, Back R
    LayoutQuad = cef_channel_layout_t::CEF_CHANNEL_LAYOUT_QUAD as isize,
    /// Front L, Front R, Front C, Side L, Side R
    Layout5_0 = cef_channel_layout_t::CEF_CHANNEL_LAYOUT_5_0 as isize,
    /// Front L, Front R, Front C, LFE, Side L, Side R
    Layout5_1 = cef_channel_layout_t::CEF_CHANNEL_LAYOUT_5_1 as isize,
    /// Front L, Front R, Front C, Back L, Back R
    Layout5_0Back = cef_channel_layout_t::CEF_CHANNEL_LAYOUT_5_0_BACK as isize,
    /// Front L, Front R, Front C, LFE, Back L, Back R
    Layout5_1Back = cef_channel_layout_t::CEF_CHANNEL_LAYOUT_5_1_BACK as isize,
    /// Front L, Front R, Front C, Side L, Side R, Back L, Back R
    Layout7_0 = cef_channel_layout_t::CEF_CHANNEL_LAYOUT_7_0 as isize,
    /// Front L, Front R, Front C, LFE, Side L, Side R, Back L, Back R
    Layout7_1 = cef_channel_layout_t::CEF_CHANNEL_LAYOUT_7_1 as isize,
    /// Front L, Front R, Front C, LFE, Side L, Side R, Front LofC, Front RofC
    Layout7_1Wide = cef_channel_layout_t::CEF_CHANNEL_LAYOUT_7_1_WIDE as isize,
    /// Stereo L, Stereo R
    LayoutStereoDownmix = cef_channel_layout_t::CEF_CHANNEL_LAYOUT_STEREO_DOWNMIX as isize,
    /// Stereo L, Stereo R, LFE
    Layout2point1 = cef_channel_layout_t::CEF_CHANNEL_LAYOUT_2POINT1 as isize,
    /// Stereo L, Stereo R, Front C, LFE
    Layout3_1 = cef_channel_layout_t::CEF_CHANNEL_LAYOUT_3_1 as isize,
    /// Stereo L, Stereo R, Front C, Rear C, LFE
    Layout4_1 = cef_channel_layout_t::CEF_CHANNEL_LAYOUT_4_1 as isize,
    /// Stereo L, Stereo R, Front C, Side L, Side R, Back C
    Layout6_0 = cef_channel_layout_t::CEF_CHANNEL_LAYOUT_6_0 as isize,
    /// Stereo L, Stereo R, Side L, Side R, Front LofC, Front RofC
    Layout6_0Front = cef_channel_layout_t::CEF_CHANNEL_LAYOUT_6_0_FRONT as isize,
    /// Stereo L, Stereo R, Front C, Rear L, Rear R, Rear C
    LayoutHexagonal = cef_channel_layout_t::CEF_CHANNEL_LAYOUT_HEXAGONAL as isize,
    /// Stereo L, Stereo R, Front C, LFE, Side L, Side R, Rear Center
    Layout6_1 = cef_channel_layout_t::CEF_CHANNEL_LAYOUT_6_1 as isize,
    /// Stereo L, Stereo R, Front C, LFE, Back L, Back R, Rear Center
    Layout6_1Back = cef_channel_layout_t::CEF_CHANNEL_LAYOUT_6_1_BACK as isize,
    /// Stereo L, Stereo R, Side L, Side R, Front LofC, Front RofC, LFE
    Layout6_1Front = cef_channel_layout_t::CEF_CHANNEL_LAYOUT_6_1_FRONT as isize,
    /// Front L, Front R, Front C, Side L, Side R, Front LofC, Front RofC
    Layout7_0Front = cef_channel_layout_t::CEF_CHANNEL_LAYOUT_7_0_FRONT as isize,
    /// Front L, Front R, Front C, LFE, Back L, Back R, Front LofC, Front RofC
    Layout7_1WideBack = cef_channel_layout_t::CEF_CHANNEL_LAYOUT_7_1_WIDE_BACK as isize,
    /// Front L, Front R, Front C, Side L, Side R, Rear L, Back R, Back C.
    LayoutOctagonal = cef_channel_layout_t::CEF_CHANNEL_LAYOUT_OCTAGONAL as isize,
    /// Channels are not explicitly mapped to speakers.
    LayoutDiscrete = cef_channel_layout_t::CEF_CHANNEL_LAYOUT_DISCRETE as isize,
    /// Front L, Front R, Front C. Front C contains the keyboard mic audio. This
    /// layout is only intended for input for WebRTC. The Front C channel
    /// is stripped away in the WebRTC audio input pipeline and never seen outside
    /// of that.
    LayoutStereoAndKeyboardMic = cef_channel_layout_t::CEF_CHANNEL_LAYOUT_STEREO_AND_KEYBOARD_MIC as isize,
    /// Front L, Front R, Side L, Side R, LFE
    Layout4_1QuadSide = cef_channel_layout_t::CEF_CHANNEL_LAYOUT_4_1_QUAD_SIDE as isize,
    /// Actual channel layout is specified in the bitstream and the actual channel
    /// count is unknown at Chromium media pipeline level (useful for audio
    /// pass-through mode).
    LayoutBitstream = cef_channel_layout_t::CEF_CHANNEL_LAYOUT_BITSTREAM as isize,
}

impl ChannelLayout {
    pub fn num_channels(&self) -> usize {
        match *self {
            Self::LayoutNone => 0,
            Self::LayoutUnsupported => 0,
            Self::LayoutMono => 1,
            Self::LayoutStereo => 2,
            Self::Layout2_1 => 3,
            Self::LayoutSurround => 3,
            Self::Layout4_0 => 4,
            Self::Layout2_2 => 4,
            Self::LayoutQuad => 4,
            Self::Layout5_0 => 5,
            Self::Layout5_1 => 6,
            Self::Layout5_0Back => 5,
            Self::Layout5_1Back => 6,
            Self::Layout7_0 => 7,
            Self::Layout7_1 => 8,
            Self::Layout7_1Wide => 8,
            Self::LayoutStereoDownmix => 2,
            Self::Layout2point1 => 3,
            Self::Layout3_1 => 4,
            Self::Layout4_1 => 5,
            Self::Layout6_0 => 6,
            Self::Layout6_0Front => 6,
            Self::LayoutHexagonal => 6,
            Self::Layout6_1 => 7,
            Self::Layout6_1Back => 7,
            Self::Layout6_1Front => 7,
            Self::Layout7_0Front => 7,
            Self::Layout7_1WideBack => 8,
            Self::LayoutOctagonal => 8,
            Self::LayoutDiscrete => 0,
            Self::LayoutStereoAndKeyboardMic => 3,
            Self::Layout4_1QuadSide => 5,
            Self::LayoutBitstream => 0,
        }
    }
}

impl ChannelLayout {
    pub unsafe fn from_unchecked(state: cef_channel_layout_t::Type) -> ChannelLayout {
        std::mem::transmute(state)
    }
}

pub struct AudioParameters {
    pub channel_layout: ChannelLayout,
    pub sample_rate: i32,
    pub frames_per_buffer: i32,
}

impl AudioParameters {
    pub unsafe fn from_raw(raw: &cef_audio_parameters_t) -> AudioParameters {
        AudioParameters {
            channel_layout: ChannelLayout::from_unchecked(raw.channel_layout),
            sample_rate: raw.sample_rate,
            frames_per_buffer: raw.frames_per_buffer,
        }
    }

    pub fn into_raw(&self) -> cef_audio_parameters_t {
        cef_audio_parameters_t {
            channel_layout: self.channel_layout as _,
            sample_rate: self.sample_rate,
            frames_per_buffer: self.frames_per_buffer,
        }
    }
}

ref_counted_ptr!{
    /// Instantiate this structure to handle events related to browser display state.
    pub struct AudioHandler(*mut cef_audio_handler_t);
}

impl AudioHandler {
    pub fn new<C: AudioHandlerCallbacks>(callbacks: C) -> AudioHandler {
        unsafe{ AudioHandler::from_ptr_unchecked(AudioHandlerWrapper {
            c: Box::new(callbacks),
            floats_per_frame: AtomicUsize::new(0),
        }.wrap().into_raw()) }
    }
}

/// Implement this trait to handle events related to browser display state.
pub trait AudioHandlerCallbacks: 'static + Send + Sync {
    /// Called on the UI thread to allow configuration of audio stream parameters.
    /// Return `true` to proceed with audio stream capture, or `false` to
    /// cancel it. All members of `params` can optionally be configured here, but
    /// they are also pre-filled with some sensible defaults.
    fn get_audio_parameters(
        &self,
        browser: Browser,
        params: &mut AudioParameters,
    ) -> bool;
    /// Called on a browser audio capture thread when the browser starts streaming
    /// audio. `on_audio_stream_stopped` will always be called after
    /// `on_audio_stream_started`; both functions may be called multiple times for the
    /// same browser. `params` contains the audio parameters like sample rate and
    /// channel layout. `channels` is the number of channels.
    fn on_audio_stream_started(
        &self,
        browser: Browser,
        params: &AudioParameters,
        channels: usize,
    );
    /// Called on the audio stream thread when a PCM packet is received for the
    /// stream. `data` is an array representing the raw PCM data as a floating
    /// point type, i.e. 4-byte value(s). `frames` is the number of frames in the
    /// PCM packet. `pts` is the presentation timestamp (in milliseconds since the
    /// Unix Epoch) and represents the time at which the decompressed packet should
    /// be presented to the user.
    fn on_audio_stream_packet(
        &self,
        browser: Browser,
        data: &[&f32],
        frames: usize,
        pts: i64, // TODO MAKE INSTANT?
    );
    /// Called on the UI thread when the stream has stopped. `on_audio_stream_stopped`
    /// will always be called after `on_audio_stream_started`; both functions may be
    /// called multiple times for the same stream.
    fn on_audio_stream_stopped(
        &self,
        browser: Browser
    );
    /// Called on the UI or audio stream thread when an error occurred. During the
    /// stream creation phase this callback will be called on the UI thread while
    /// in the capturing phase it will be called on the audio stream thread. The
    /// stream will be stopped immediately.
    fn on_audio_stream_error(
        &self,
        browser: Browser,
        message: &str,
    );
}

struct AudioHandlerWrapper {
    c: Box<dyn AudioHandlerCallbacks>,
    floats_per_frame: AtomicUsize,
}

impl Wrapper for AudioHandlerWrapper {
    type Cef = cef_audio_handler_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_audio_handler_t {
                base: unsafe { std::mem::zeroed() },
                get_audio_parameters: Some(Self::get_audio_parameters),
                on_audio_stream_started: Some(Self::on_audio_stream_started),
                on_audio_stream_packet: Some(Self::on_audio_stream_packet),
                on_audio_stream_stopped: Some(Self::on_audio_stream_stopped),
                on_audio_stream_error: Some(Self::on_audio_stream_error),
            },
            self,
        )
    }
}

cef_callback_impl!{
    impl for AudioHandlerWrapper: cef_audio_handler_t {
        fn get_audio_parameters(
            &self,
            browser: Browser: *mut cef_browser_t,
            params: &mut cef_audio_parameters_t: *mut cef_audio_parameters_t,
        ) -> c_int {
            let mut rust_params = unsafe{ AudioParameters::from_raw(params) };
            let result = self.c.get_audio_parameters(
                browser,
                &mut rust_params,
            ) as c_int;
            *params = rust_params.into_raw();
            result
        }
        fn on_audio_stream_started(
            &self,
            browser: Browser: *mut cef_browser_t,
            params: &cef_audio_parameters_t: *const cef_audio_parameters_t,
            channels: i32: i32,
        ) {
            let params = unsafe{ AudioParameters::from_raw(params) };
            self.floats_per_frame.store(params.channel_layout.num_channels(), Ordering::SeqCst);
            self.c.on_audio_stream_started(
                browser,
                &params,
                channels as usize,
            );
        }
        fn on_audio_stream_packet(
            &self,
            browser: Browser: *mut cef_browser_t,
            data: *mut *const f32: *mut *const f32,
            frames: i32: c_int,
            pts: i64: i64,
        ) {
            assert!(frames > 0);
            let data_len = self.floats_per_frame.load(Ordering::SeqCst);
            let data = unsafe{ slice::from_raw_parts(data as *const &f32, data_len) };
            self.c.on_audio_stream_packet(
                browser,
                data,
                frames as usize,
                pts,
            );
        }
        fn on_audio_stream_stopped(
            &self,
            browser: Browser: *mut cef_browser_t,
        ) {
            self.c.on_audio_stream_stopped(browser);
        }
        fn on_audio_stream_error(
            &self,
            browser: Browser: *mut cef_browser_t,
            message: &CefString: *const cef_string_t,
        ) {
            self.c.on_audio_stream_error(browser, &String::from(message));
        }
    }
}
