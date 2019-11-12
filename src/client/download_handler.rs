use crate::{
    browser::{Browser},
    string::CefString,
    refcounted::{RefCountedPtr, Wrapper},
};
use cef_sys::{
    cef_string_t,
    cef_time_t,
    cef_browser_t,
    cef_download_handler_t,
    cef_download_item_callback_t,
    cef_download_item_t,
    cef_before_download_callback_t,
};
use std::os::raw::{c_int};
use parking_lot::Mutex;
use chrono::{DateTime, NaiveDateTime, NaiveDate, NaiveTime, Utc};

ref_counted_ptr!{
    /// Structure used to handle file downloads. The functions of this structure will
    /// called on the browser process UI thread.
    pub struct DownloadHandler(*mut cef_download_handler_t);
}

ref_counted_ptr!{
    /// Callback structure used to asynchronously cancel a download.
    pub struct DownloadItemCallback(*mut cef_download_item_callback_t);
}

ref_counted_ptr!{
    /// Callback structure used to asynchronously continue a download.
    pub struct BeforeDownloadCallback(*mut cef_before_download_callback_t);
}

ref_counted_ptr!{
    /// Structure used to represent a download item.
    pub struct DownloadItem(*mut cef_download_item_t);
}

impl DownloadHandler {
    pub fn new<C: DownloadHandlerCallbacks>(callbacks: C) -> DownloadHandler {
        unsafe{ DownloadHandler::from_ptr_unchecked(DownloadHandlerWrapper(Mutex::new(Box::new(callbacks))).wrap().into_raw()) }
    }
}

impl DownloadItemCallback {
    /// Call to cancel the download.
    pub fn cancel(&self) {
        unsafe{ self.0.cancel.unwrap()(self.as_ptr()) }
    }
    /// Call to pause the download.
    pub fn pause(&self) {
        unsafe{ self.0.pause.unwrap()(self.as_ptr()) }
    }
    /// Call to resume the download.
    pub fn resume(&self) {
        unsafe{ self.0.resume.unwrap()(self.as_ptr()) }
    }
}

impl BeforeDownloadCallback {
    /// Call to continue the download. Set `download_path` to the full file path
    /// for the download including the file name or leave blank to use the
    /// suggested name and the default temp directory. Set `show_dialog` to `true`
    /// if you do wish to show the default "Save As" dialog.
    pub fn cont(
        &self,
        download_path: &str,
        show_dialog: bool,
    ) {
        unsafe {
            self.0.cont.unwrap()(
                self.as_ptr(),
                CefString::new(download_path).as_ptr(),
                show_dialog as c_int,
            )
        }
    }
}

impl DownloadItem {
    /// Returns `true` if this object is valid. Do not call any other functions
    /// if this function returns false (0).
    pub fn is_valid(&self) -> bool {
        unsafe{ self.0.is_valid.unwrap()(self.as_ptr()) != 0 }
    }
    /// Returns `true` if the download is in progress.
    pub fn is_in_progress(&self) -> bool {
        unsafe{ self.0.is_in_progress.unwrap()(self.as_ptr()) != 0 }
    }
    /// Returns `true` if the download is complete.
    pub fn is_complete(&self) -> bool {
        unsafe{ self.0.is_complete.unwrap()(self.as_ptr()) != 0 }
    }
    /// Returns `true` if the download has been canceled or interrupted.
    pub fn is_canceled(&self) -> bool {
        unsafe{ self.0.is_canceled.unwrap()(self.as_ptr()) != 0 }
    }
    /// Returns a simple speed estimate in bytes/s.
    pub fn get_current_speed(&self) -> u64 {
        unsafe{ self.0.get_current_speed.unwrap()(self.as_ptr()) as u64 }
    }
    /// Returns the rough percent complete or `None` if the receive total size is
    /// unknown.
    pub fn get_percent_complete(&self) -> Option<u8> {
        let percent = unsafe{ self.0.get_percent_complete.unwrap()(self.as_ptr()) };
        match percent {
            -1 => None,
            _ => Some(percent as u8)
        }
    }
    /// Returns the total number of bytes.
    pub fn get_total_bytes(&self) -> u64 {
        unsafe{ self.0.get_total_bytes.unwrap()(self.as_ptr()) as u64 }
    }
    /// Returns the number of received bytes.
    pub fn get_received_bytes(&self) -> u64 {
        unsafe{ self.0.get_received_bytes.unwrap()(self.as_ptr()) as u64 }
    }
    /// Returns the time that the download started.
    pub fn get_start_time(&self) -> DateTime<Utc> {
        cef_time_to_system_time(unsafe{ self.0.get_start_time.unwrap()(self.as_ptr()) })
    }
    /// Returns the time that the download ended.
    pub fn get_end_time(&self) -> DateTime<Utc> {
        cef_time_to_system_time(unsafe{ self.0.get_end_time.unwrap()(self.as_ptr()) })
    }
    /// Returns the full path to the downloaded or downloading file.
    pub fn get_full_path(&self) -> String {
        String::from(unsafe{ CefString::from_userfree_unchecked(self.0.get_full_path.unwrap()(self.as_ptr())) })
    }
    /// Returns the unique identifier for this download.
    pub fn get_id(&self) -> u32 {
        unsafe{ self.0.get_id.unwrap()(self.as_ptr()) }
    }
    /// Returns the URL.
    pub fn get_url(&self) -> String {
        String::from(unsafe{ CefString::from_userfree_unchecked(self.0.get_url.unwrap()(self.as_ptr())) })
    }
    /// Returns the original URL before any redirections.
    pub fn get_original_url(&self) -> String {
        String::from(unsafe{ CefString::from_userfree_unchecked(self.0.get_original_url.unwrap()(self.as_ptr())) })
    }
    /// Returns the suggested file name.
    pub fn get_suggested_file_name(&self) -> String {
        String::from(unsafe{ CefString::from_userfree_unchecked(self.0.get_suggested_file_name.unwrap()(self.as_ptr())) })
    }
    /// Returns the content disposition.
    pub fn get_content_disposition(&self) -> String {
        String::from(unsafe{ CefString::from_userfree_unchecked(self.0.get_content_disposition.unwrap()(self.as_ptr())) })
    }
    /// Returns the mime type.
    pub fn get_mime_type(&self) -> String {
        String::from(unsafe{ CefString::from_userfree_unchecked(self.0.get_mime_type.unwrap()(self.as_ptr())) })
    }
}

fn cef_time_to_system_time(cef_time: cef_time_t) -> DateTime<Utc> {
    DateTime::from_utc(
        NaiveDateTime::new(
            NaiveDate::from_ymd(cef_time.year as i32, cef_time.month as u32, cef_time.day_of_month as u32),
            NaiveTime::from_hms_milli(cef_time.hour as u32, cef_time.minute as u32, cef_time.second as u32, cef_time.millisecond as u32),
        ),
        Utc,
    )
}

/// Trait used to handle file downloads. The functions of this structure will
/// called on the browser process UI thread.
pub trait DownloadHandlerCallbacks: 'static + Send {
    /// Called before a download begins. `suggested_name` is the suggested name for
    /// the download file. By default the download will be canceled. Execute
    /// `callback` either asynchronously or in this function to continue the
    /// download if desired. Do not keep a reference to `download_item` outside of
    /// this function.
    fn on_before_download(
        &mut self,
        browser: Browser,
        download_item: DownloadItem,
        suggested_name: &str,
        callback: BeforeDownloadCallback,
    );
    /// Called when a download's status or progress information has been updated.
    /// This may be called multiple times before and after `on_before_download()`.
    /// Execute `callback` either asynchronously or in this function to cancel the
    /// download if desired. Do not keep a reference to `download_item` outside of
    /// this function.
    fn on_download_updated(
        &mut self,
        browser: Browser,
        download_item: DownloadItem,
        callback: DownloadItemCallback,
    );
}

struct DownloadHandlerWrapper(Mutex<Box<dyn DownloadHandlerCallbacks>>);
impl Wrapper for DownloadHandlerWrapper {
    type Cef = cef_download_handler_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_download_handler_t {
                base: unsafe { std::mem::zeroed() },
                on_before_download: Some(Self::on_before_download),
                on_download_updated: Some(Self::on_download_updated),
            },
            self,
        )
    }
}

cef_callback_impl!{
    impl for DownloadHandlerWrapper: cef_download_handler_t {
        fn on_before_download(
            &self,
            browser: Browser: *mut cef_browser_t,
            download_item: DownloadItem: *mut cef_download_item_t,
            suggested_name: &CefString: *const cef_string_t,
            callback: BeforeDownloadCallback: *mut cef_before_download_callback_t
        ) {
            self.0.lock().on_before_download(
                browser,
                download_item,
                &*String::from(suggested_name),
                callback,
            );
        }
        fn on_download_updated(
            &self,
            browser: Browser: *mut cef_browser_t,
            download_item: DownloadItem: *mut cef_download_item_t,
            callback: DownloadItemCallback: *mut cef_download_item_callback_t
        ) {
            self.0.lock().on_download_updated(
                browser,
                download_item,
                callback,
            );
        }
    }
}
