use cef_sys::{
    cef_string_t,
    cef_media_router_t, cef_media_observer_t, cef_media_source_t, cef_media_sink_t,
    cef_media_route_create_callback_t, cef_media_route_t, cef_media_route_create_result_t,
    cef_media_route_connection_state_t, cef_media_sink_icon_type_t, cef_media_sink_device_info_t,
    cef_media_sink_device_info_callback_t,
};

use std::{
    slice,
    os::raw::c_void,
};

use crate::{
    refcounted::{RefCountedPtr, Wrapper},
    string::CefString,
    registration::Registration, send_protector::SendProtectorMut,
};

/// Result codes for `MediaRouter::create_route`.
#[repr(C)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum MediaRouteCreateResult {
    UnknownError = cef_media_route_create_result_t::CEF_MRCR_UNKNOWN_ERROR as isize,
    Ok = cef_media_route_create_result_t::CEF_MRCR_OK as isize,
    TimedOut = cef_media_route_create_result_t::CEF_MRCR_TIMED_OUT as isize,
    RouteNotFound = cef_media_route_create_result_t::CEF_MRCR_ROUTE_NOT_FOUND as isize,
    SinkNotFound = cef_media_route_create_result_t::CEF_MRCR_SINK_NOT_FOUND as isize,
    InvalidOrigin = cef_media_route_create_result_t::CEF_MRCR_INVALID_ORIGIN as isize,
    NoSupportedProvider = cef_media_route_create_result_t::CEF_MRCR_NO_SUPPORTED_PROVIDER as isize,
    Cancelled = cef_media_route_create_result_t::CEF_MRCR_CANCELLED as isize,
    RouteAlreadyExists = cef_media_route_create_result_t::CEF_MRCR_ROUTE_ALREADY_EXISTS as isize,
}

/// Connection state for a `MediaRoute` object.
#[repr(C)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum MediaRouteConnectionState {
    Unknown = cef_media_route_connection_state_t::CEF_MRCS_UNKNOWN as isize,
    Connecting = cef_media_route_connection_state_t::CEF_MRCS_CONNECTING as isize,
    Connected = cef_media_route_connection_state_t::CEF_MRCS_CONNECTED as isize,
    Closed = cef_media_route_connection_state_t::CEF_MRCS_CLOSED as isize,
    Terminated = cef_media_route_connection_state_t::CEF_MRCS_TERMINATED as isize,
}

/// Icon types for a `MediaSink` object.
#[repr(C)]
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum MediaSinkIconType {
    Cast = cef_media_sink_icon_type_t::CEF_MSIT_CAST as isize,
    CastAudioGroup = cef_media_sink_icon_type_t::CEF_MSIT_CAST_AUDIO_GROUP as isize,
    CastAudio = cef_media_sink_icon_type_t::CEF_MSIT_CAST_AUDIO as isize,
    Meeting = cef_media_sink_icon_type_t::CEF_MSIT_MEETING as isize,
    Hangout = cef_media_sink_icon_type_t::CEF_MSIT_HANGOUT as isize,
    Education = cef_media_sink_icon_type_t::CEF_MSIT_EDUCATION as isize,
    WiredDisplay = cef_media_sink_icon_type_t::CEF_MSIT_WIRED_DISPLAY as isize,
    Generic = cef_media_sink_icon_type_t::CEF_MSIT_GENERIC as isize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MediaSinkDeviceInfo {
    pub ip_address: String,
    pub port: i32,
    pub model_name: String,
}

impl MediaSinkDeviceInfo {
    pub(crate) unsafe fn from_raw(info: &cef_media_sink_device_info_t) -> MediaSinkDeviceInfo {
        MediaSinkDeviceInfo {
            ip_address: CefString::from_ptr_unchecked(&info.ip_address).into(),
            port: info.port,
            model_name: CefString::from_ptr_unchecked(&info.model_name).into(),
        }
    }
}

impl MediaRouteCreateResult {
    pub unsafe fn from_unchecked(raw: crate::CEnumType) -> Self {
        std::mem::transmute(raw)
    }
}

impl MediaRouteConnectionState {
    pub unsafe fn from_unchecked(raw: crate::CEnumType) -> Self {
        std::mem::transmute(raw)
    }
}

impl MediaSinkIconType {
    pub unsafe fn from_unchecked(raw: crate::CEnumType) -> Self {
        std::mem::transmute(raw)
    }
}

ref_counted_ptr!{
    /// Supports discovery of and communication with media devices on the local
    /// network via the Cast and DIAL protocols. The functions of this structure may
    /// be called on any browser process thread unless otherwise indicated.
    pub struct MediaRouter(*mut cef_media_router_t);
}

ref_counted_ptr!{
    /// Implemented by the client to observe `MediaRouter` events and registered via
    /// `MediaRouter::add_observer`. The functions of this structure will be
    /// called on the browser process UI thread.
    pub struct MediaObserver(*mut cef_media_observer_t);
}

ref_counted_ptr!{
    /// Represents a source from which media can be routed. Instances of this object
    /// are retrieved via `MediaRouter::get_source`. The functions of this
    /// structure may be called on any browser process thread unless otherwise
    /// indicated.
    pub struct MediaSource(*mut cef_media_source_t);
}

ref_counted_ptr!{
    /// Represents a sink to which media can be routed. Instances of this object are
    /// retrieved via `MediaObserver::on_sinks`. The functions of this structure
    /// may be called on any browser process thread unless otherwise indicated.
    pub struct MediaSink(*mut cef_media_sink_t);
}

ref_counted_ptr!{
    pub struct MediaRoute(*mut cef_media_route_t);
}

ref_counted_ptr!{
    struct MediaSinkDeviceInfoCallback(*mut cef_media_sink_device_info_callback_t);
}

ref_counted_ptr!{
    struct MediaRouteCreateCallback(*mut cef_media_route_create_callback_t);
}

/// Implemented by the client to observe `MediaRouter` events and registered via
/// `MediaRouter::add_observer`. The functions of this structure will be
/// called on the browser process UI thread.
pub trait MediaObserverCallbacks: 'static + Send {
    /// The list of available media sinks has changed or
    /// [`MediaRouter::notify_current_sinks`] was called.
    fn on_sinks(
        &mut self,
        sinks: &[MediaSink],
    ) {}
    /// The list of available media routes has changed or
    /// [`MediaRouter::notify_current_routes`] was called.
    fn on_routes(
        &mut self,
        routes: &[MediaRoute],
    ) {}
    /// The connection state of `route` has changed.
    fn on_route_state_changed(
        &mut self,
        route: MediaRoute,
        state: MediaRouteConnectionState,
    ) {}
    /// A message was recieved over `route`.
    fn on_route_message_received(
        &mut self,
        route: MediaRoute,
        message: &[u8],
    ) {}
}

impl MediaRouter {
    pub fn global() -> MediaRouter {
        unsafe{ Self::from_ptr_unchecked(cef_sys::cef_media_router_get_global()) }
    }
    /// Add an observer for `MediaRouter` events. The observer will remain registered
    /// until the returned `Registration` object is destroyed.
    pub fn add_observer(
        &self,
        observer: MediaObserver,
    ) -> Registration {
        unsafe {
            Registration::from_ptr_unchecked(
                (self.0.add_observer.unwrap())(
                    self.as_ptr(),
                    observer.into_raw()
                )
            )
        }
    }
    /// Returns a `MediaSource` object for the specified media source URN. Supported
    /// URN schemes include `cast:` and `dial:`, and will be already known by the
    /// client application (e.g. `cast:<appId>?clientId=<clientId>`).
    pub fn get_source(
        &self,
        urn: &str,
    ) -> Option<MediaSource> {
        unsafe {
            MediaSource::from_ptr(
                (self.0.get_source.unwrap())(
                    self.as_ptr(),
                    CefString::new(urn).as_ptr(),
                )
            )
        }
    }
    /// Trigger an asynchronous call to [`MediaObserver::on_sinks`] on all
    /// registered observers.
    pub fn notify_current_sinks(&self) {
        unsafe {
            (self.0.notify_current_sinks.unwrap())(self.as_ptr())
        }
    }
    /// Create a new route between `source` and `sink`. Source and sink must be
    /// valid, compatible (as reported by [`MediaSink::is_compatible_with`]), and
    /// a route between them must not already exist. `callback` will be executed on
    /// success or failure. If route creation succeeds it will also trigger an
    /// asynchronous call to [`MediaObserver::on_routes`] on all registered
    /// observers.
    ///
    /// # Method parameters
    /// `MediaRouteCreateResult` will be `Ok` if the route creation succeeded. `Option<&str>` will
    /// be a description of the error if the route creation failed. `Option<MediaRoute>` is the
    /// resulting route, or `None` if the route creation failed.
    pub fn create_route(
        &self,
        source: MediaSource,
        sink: MediaSink,
        callback: impl 'static + Send + FnOnce(MediaRouteCreateResult, Option<&str>, Option<MediaRoute>),
    ) {
        unsafe {
            (self.0.create_route.unwrap())(
                self.as_ptr(),
                source.into_raw(),
                sink.into_raw(),
                MediaRouteCreateCallback::new(callback).into_raw()
            )
        }
    }
    /// Trigger an asynchronous call to [`MediaObserver::on_routes`] on all
    /// registered observers.
    pub fn notify_current_routes(&self) {
        unsafe {
            (self.0.notify_current_routes.unwrap())(self.as_ptr())
        }
    }
}

impl MediaRoute {
    /// Returns the ID for this route.
    pub fn get_id(&self) -> String {
        unsafe {
            CefString::from_userfree_unchecked(
                (self.0.get_id.unwrap())(self.as_ptr())
            ).into()
        }
    }
    /// Returns the source associated with this route.
    pub fn get_source(&self) -> MediaSource {
        unsafe {
            MediaSource::from_ptr_unchecked(
                (self.0.get_source.unwrap())(self.as_ptr())
            )
        }
    }
    /// Returns the sink associated with this route.
    pub fn get_sink(&self) -> MediaSink {
        unsafe {
            MediaSink::from_ptr_unchecked(
                (self.0.get_sink.unwrap())(self.as_ptr())
            )
        }
    }
    /// Send a message over this route. `message` will be copied if necessary.
    pub fn send_route_message(&self, message: &[u8]) {
        unsafe {
            (self.0.send_route_message.unwrap())(
                self.as_ptr(),
                message.as_ptr() as *const c_void,
                message.len(),
            )
        }
    }
    /// Terminate this route. Will result in an asynchronous call to
    /// [`MediaObserver::on_routes`] on all registered observers.
    pub fn terminate(&self) {
        unsafe {
            (self.0.terminate.unwrap())(self.as_ptr())
        }
    }
}

impl MediaSink {
    /// Returns the ID for this sink.
    pub fn get_id(&self) -> String {
        unsafe {
            CefString::from_userfree_unchecked(
                (self.0.get_id.unwrap())(self.as_ptr())
            ).into()
        }
    }
    /// Returns `true` if this sink is valid.
    pub fn is_valid(&self) -> bool {
        unsafe {
            (self.0.is_valid.unwrap())(self.as_ptr()) != 0
        }
    }
    /// Returns the name of this sink.
    pub fn get_name(&self) -> String {
        unsafe {
            CefString::from_userfree_unchecked(
                (self.0.get_name.unwrap())(self.as_ptr())
            ).into()
        }
    }
    /// Returns the description of this sink.
    pub fn get_description(&self) -> String {
        unsafe {
            CefString::from_userfree_unchecked(
                (self.0.get_description.unwrap())(self.as_ptr())
            ).into()
        }
    }
    /// Returns the icon type for this sink.
    pub fn get_icon_type(&self) -> MediaSinkIconType {
        unsafe {
            MediaSinkIconType::from_unchecked(
                (self.0.get_icon_type.unwrap())(self.as_ptr())
            )
        }
    }
    pub fn get_device_info(&self, callback: impl 'static + Send + FnOnce(MediaSinkDeviceInfo)) {
        unsafe {
            self.0.get_device_info.unwrap()(
                self.as_ptr(),
                MediaSinkDeviceInfoCallback::new(callback).into_raw()
            )
        }
    }
    /// Returns `true` if this sink accepts content via Cast.
    pub fn is_cast_sink(&self) -> bool {
        unsafe {
            (self.0.is_cast_sink.unwrap())(self.as_ptr()) != 0
        }
    }
    /// Returns `true` if this sink accepts content via DIAL.
    pub fn is_dial_sink(&self) -> bool {
        unsafe {
            (self.0.is_dial_sink.unwrap())(self.as_ptr()) != 0
        }
    }
    /// Returns `true` if this sink is compatible with |source|.
    pub fn is_compatible_with(
        &self,
        source: MediaSource,
    ) -> bool {
        unsafe {
            (self.0.is_compatible_with.unwrap())(self.as_ptr(), source.into_raw()) != 0
        }
    }
}

impl MediaSource {
    /// Returns the ID (media source URN or URL) for this source.
    pub fn get_id(&self) -> String {
        unsafe {
            CefString::from_userfree_unchecked(
                (self.0.get_id.unwrap())(self.as_ptr())
            ).into()
        }
    }
    /// Returns `true` if this source is valid.
    pub fn is_valid(&self) -> bool {
        unsafe {
            (self.0.is_valid.unwrap())(self.as_ptr()) != 0
        }
    }
    /// Returns `true` if this source outputs its content via Cast.
    pub fn is_cast_source(&self) -> bool {
        unsafe {
            (self.0.is_cast_source.unwrap())(self.as_ptr()) != 0
        }
    }
    /// Returns `true` if this source outputs its content via DIAL.
    pub fn is_dial_source(&self) -> bool {
        unsafe {
            (self.0.is_dial_source.unwrap())(self.as_ptr()) != 0
        }
    }
}

struct MediaSinkDeviceInfoCallbackWrapper(SendProtectorMut<Option<Box<dyn 'static + Send + FnOnce(MediaSinkDeviceInfo)>>>);

impl MediaSinkDeviceInfoCallback {
    fn new(callback: impl 'static + Send + FnOnce(MediaSinkDeviceInfo)) -> MediaSinkDeviceInfoCallback {
        unsafe{ MediaSinkDeviceInfoCallback::from_ptr_unchecked(MediaSinkDeviceInfoCallbackWrapper(SendProtectorMut::new(Some(Box::new(callback)))).wrap().into_raw()) }
    }
}

impl Wrapper for MediaSinkDeviceInfoCallbackWrapper {
    type Cef = cef_media_sink_device_info_callback_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_media_sink_device_info_callback_t {
                base: unsafe { std::mem::zeroed() },
                on_media_sink_device_info: Some(Self::on_media_sink_device_info),
            },
            self,
        )
    }
}

cef_callback_impl!{
    impl for MediaSinkDeviceInfoCallbackWrapper: cef_media_sink_device_info_callback_t {
        fn on_media_sink_device_info(
            &self,
            info: MediaSinkDeviceInfo: *const cef_media_sink_device_info_t
        ) {
            unsafe {
                self.0.get_mut().take().unwrap()(
                    info
                )
            }
        }
    }
}



struct MediaRouteCreateCallbackWrapper(SendProtectorMut<Option<Box<dyn 'static + Send + FnOnce(MediaRouteCreateResult, Option<&str>, Option<MediaRoute>)>>>);

impl MediaRouteCreateCallback {
    fn new(callback: impl 'static + Send + FnOnce(MediaRouteCreateResult, Option<&str>, Option<MediaRoute>)) -> MediaRouteCreateCallback {
        unsafe{ MediaRouteCreateCallback::from_ptr_unchecked(MediaRouteCreateCallbackWrapper(SendProtectorMut::new(Some(Box::new(callback)))).wrap().into_raw()) }
    }
}

impl Wrapper for MediaRouteCreateCallbackWrapper {
    type Cef = cef_media_route_create_callback_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_media_route_create_callback_t {
                base: unsafe { std::mem::zeroed() },
                on_media_route_create_finished: Some(Self::on_media_route_create_finished),
            },
            self,
        )
    }
}

cef_callback_impl!{
    impl for MediaRouteCreateCallbackWrapper: cef_media_route_create_callback_t {
        fn on_media_route_create_finished(
            &self,
            result: MediaRouteCreateResult: cef_media_route_create_result_t::Type,
            error: Option<&CefString>: *const cef_string_t,
            route: Option<MediaRoute>: *mut cef_media_route_t,
        ) {
            unsafe {
                self.0.get_mut().take().unwrap()(
                    result,
                    error.map(String::from)
                        .as_ref()
                        .map(|e| &**e),
                    route,
                )
            }
        }
    }
}

struct MediaObserverWrapper(SendProtectorMut<Box<dyn MediaObserverCallbacks>>);

impl Wrapper for MediaObserverWrapper {
    type Cef = cef_media_observer_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_media_observer_t {
                base: unsafe { std::mem::zeroed() },
                on_sinks: Some(Self::on_sinks),
                on_routes: Some(Self::on_routes),
                on_route_state_changed: Some(Self::on_route_state_changed),
                on_route_message_received: Some(Self::on_route_message_received),
            },
            self,
        )
    }
}

cef_callback_impl!{
    impl for MediaObserverWrapper: cef_media_observer_t {
        fn on_sinks(
            &self,
            sinks_count: usize: usize,
            sinks: *const *mut cef_media_sink_t: *const *mut cef_media_sink_t,
        ) {
            let sinks = unsafe{ slice::from_raw_parts(sinks as *const MediaSink, sinks_count) };
            unsafe{ self.0.get_mut().on_sinks(sinks) };
        }
        fn on_routes(
            &self,
            routes_count: usize: usize,
            routes: *const *mut cef_media_route_t: *const *mut cef_media_route_t,
        ) {
            let routes = unsafe{ slice::from_raw_parts(routes as *const MediaRoute, routes_count) };
            unsafe{ self.0.get_mut().on_routes(routes) };
        }
        fn on_route_state_changed(
            &self,
            route: MediaRoute: *mut cef_media_route_t,
            state: MediaRouteConnectionState: cef_media_route_connection_state_t::Type,
        ) {
            unsafe{ self.0.get_mut().on_route_state_changed(route, state) };
        }
        fn on_route_message_received(
            &self,
            route: MediaRoute: *mut cef_media_route_t,
            message: *const c_void: *const c_void,
            message_size: usize: usize,
        ) {
            let message = unsafe{ slice::from_raw_parts(message as *const u8, message_size) };
            unsafe{ self.0.get_mut().on_route_message_received(route, message) };
        }
    }
}
