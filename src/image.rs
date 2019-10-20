use cef_sys::{cef_image_t, cef_image_create, cef_color_type_t, cef_alpha_type_t};

use crate::{
    values::BinaryValue,
};

/// Describes how to interpret the components of a pixel.
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ColorType {
    /// RGBA with 8 bits per pixel (32bits total).
    Rgba8888 = cef_color_type_t::CEF_COLOR_TYPE_RGBA_8888,
    /// BGRA with 8 bits per pixel (32bits total).
    Bgra8888 = cef_color_type_t::CEF_COLOR_TYPE_BGRA_8888,
}

/// Describes how to interpret the alpha component of a pixel.
#[repr(i32)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AlphaType {
    /// No transparency. The alpha component is ignored.
    Opaque = cef_alpha_type_t::CEF_ALPHA_TYPE_OPAQUE,
    /// Transparency with pre-multiplied alpha component.
    Premultiplied = cef_alpha_type_t::CEF_ALPHA_TYPE_PREMULTIPLIED,
    /// Transparency with post-multiplied alpha component.
    Postmultiplied = cef_alpha_type_t::CEF_ALPHA_TYPE_POSTMULTIPLIED,
}

ref_counted_ptr! {
    /// Container for a single image represented at different scale factors. All
    /// image representations should be the same size in density independent pixel
    /// (DIP) units. For example, if the image at scale factor 1.0 is 100x100 pixels
    /// then the image at scale factor 2.0 should be 200x200 pixels -- both images
    /// will display with a DIP size of 100x100 units. The functions of this
    /// structure must be called on the browser process UI thread.
    pub struct Image(*mut cef_image_t);
}

/// Returned by [Image::get_representation_info]
pub struct RepresentationInfo {
    /// The actual scale factor for the representation.
    actual_scale_factor: f32,
    /// The representation width in pixel coordinates.
    pixel_width: i32,
    /// The representation height in pixel coordinates.
    pixel_height: i32,
}

/// Returned by [Image::get_as_bitmap], [Image::get_as_png] and [Image::get_as_jpeg].
pub struct BinaryImage {
    /// The output representation width in pixel coordinates.
    pixel_width: i32,
    /// The output representation height in pixel coordinates.
    pixel_height: i32,
    /// A vector containing the pixel data.
    data: Vec<u8>,
}

impl Image {
    /// Create a new [Image]. It will initially be empty. Use the add_*() functions
    /// to add representations at different scale factors.
    pub fn new() -> Self {
        unsafe { Self::from_ptr(cef_image_create()).unwrap() }
    }

    /// Returns true if this [Image] is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty.map(|is_empty| {
            unsafe { is_empty(self.as_ptr()) != 0 }
        }).unwrap_or(true)
    }
    // Returns true if this [Image] and `that` [Image] share the same underlying
    // storage. Will also return true if both images are empty.
    pub fn is_same(&self, that: &Self) -> bool {
        self.0.is_same.map(|is_same| {
            unsafe { is_same(self.as_ptr(), that.as_ptr()) != 0 }
        }).unwrap_or(false)
    }
    /// Add a bitmap image representation for `scale_factor`. Only 32-bit RGBA/BGRA
    /// formats are supported. `pixel_width` and `pixel_height` are the bitmap
    /// representation size in pixel coordinates. `pixel_data` is the array of
    /// pixel data and should be `pixel_width` x `pixel_height` x 4 bytes in size.
    /// `color_type` and `alpha_type` values specify the pixel format.
    pub fn add_bitmap(&mut self, scale_factor: f32, pixel_width: i32, pixel_height: i32, color_type: ColorType, alpha_type: AlphaType, pixel_data: &[u8]) -> bool {
        self.0.add_bitmap.map(|add_bitmap| {
            unsafe { add_bitmap(self.as_ptr(), scale_factor, pixel_width, pixel_height, color_type as cef_color_type_t::Type, alpha_type as cef_alpha_type_t::Type, pixel_data.as_ptr() as *const std::ffi::c_void, pixel_data.len()) != 0 }
        }).unwrap_or(false)
    }
    /// Add a PNG image representation for `scale_factor`. `png_data` is the image
    /// data. Any alpha transparency in the PNG data will
    /// be maintained.
    pub fn add_png(&mut self, scale_factor: f32, png_data: &[u8]) -> bool {
        self.0.add_png.map(|add_png| {
            unsafe { add_png(self.as_ptr(), scale_factor, png_data.as_ptr() as *const std::ffi::c_void, png_data.len()) != 0 }
        }).unwrap_or(false)
    }
    /// Create a JPEG image representation for `scale_factor`. `jpeg_data` is the
    /// image data. The JPEG format does not support
    /// transparency so the alpha byte will be set to `0xFF` for all pixels.
    pub fn add_jpeg(&mut self, scale_factor: f32, jpeg_data: &[u8]) -> bool {
        self.0.add_jpeg.map(|add_jpeg| {
            unsafe { add_jpeg(self.as_ptr(), scale_factor, jpeg_data.as_ptr() as *const std::ffi::c_void, jpeg_data.len()) != 0 }
        }).unwrap_or(false)
    }
    /// Returns the image width in density independent pixel (DIP) units.
    pub fn get_width(&self) -> usize {
        self.0.get_width.map(|get_width| {
            unsafe { get_width(self.as_ptr()) }
        }).unwrap_or(0)
    }
    /// Returns the image height in density independent pixel (DIP) units.
    pub fn get_height(&self) -> usize {
        self.0.get_height.map(|get_height| {
            unsafe { get_height(self.as_ptr()) }
        }).unwrap_or(0)
    }
    /// Returns true if this image contains a representation for
    /// `scale_factor`.
    pub fn has_representation(&self, scale_factor: f32) -> bool {
        self.0.has_representation.map(|has_representation| {
            unsafe { has_representation(self.as_ptr(), scale_factor) != 0 }
        }).unwrap_or(false)
    }
    /// Removes the representation for `scale_factor`. Returns true on success.
    pub fn remove_representation(&mut self, scale_factor: f32) -> bool {
        self.0.remove_representation.map(|remove_representation| {
            unsafe { remove_representation(self.as_ptr(), scale_factor) != 0 }
        }).unwrap_or(false)
    }
    /// Returns information for the representation that most closely matches
    /// `scale_factor`.
    pub fn get_representation_info(&self, scale_factor: f32) -> Option<RepresentationInfo> {
        self.0.get_representation_info.and_then(|get_representation_info| {
            let mut actual_scale_factor = 0.0;
            let mut pixel_width = 0;
            let mut pixel_height = 0;
            if unsafe { get_representation_info(self.as_ptr(), scale_factor, &mut actual_scale_factor, &mut pixel_width, &mut pixel_height) } != 0 {
                Some(RepresentationInfo {
                    actual_scale_factor,
                    pixel_width,
                    pixel_height,
                })
            } else {
                None
            }
        })
    }
    /// Returns the bitmap representation that most closely matches `scale_factor`.
    /// Only 32-bit RGBA/BGRA formats are supported. `color_type` and `alpha_type`
    /// values specify the desired output pixel format.
    pub fn get_as_bitmap(&self, scale_factor: f32, color_type: ColorType, alpha_type: AlphaType) -> Option<BinaryImage> {
        self.0.get_as_bitmap.and_then(|get_as_bitmap| {
            let mut pixel_width = 0;
            let mut pixel_height = 0;
            let binary = unsafe { get_as_bitmap(self.as_ptr(), scale_factor, color_type as cef_color_type_t::Type, alpha_type as cef_alpha_type_t::Type, &mut pixel_width, &mut pixel_height) };
            BinaryValue::from_ptr(binary).map(|data| BinaryImage {
                pixel_width,
                pixel_height,
                data: data.into(),
            })
        })
    }
    /// Returns the PNG representation that most closely matches `scale_factor`. If
    /// `with_transparency` is true any alpha transparency in the image will be
    /// represented in the resulting PNG data.
    pub fn get_as_png(&self, scale_factor: f32, with_transparency: bool) -> Option<BinaryImage> {
        self.0.get_as_png.and_then(|get_as_png| {
            let mut pixel_width = 0;
            let mut pixel_height = 0;
            let binary = unsafe { get_as_png(self.as_ptr(), scale_factor, with_transparency as i32, &mut pixel_width, &mut pixel_height) };
            BinaryValue::from_ptr(binary).map(|data| BinaryImage {
                pixel_width,
                pixel_height,
                data: data.into(),
            })
        })
    }
    /// Returns the JPEG representation that most closely matches `scale_factor`.
    /// `quality` determines the compression level with 0 == lowest and 100 ==
    /// highest. The JPEG format does not support alpha transparency and the alpha
    /// channel, if any, will be discarded.
    pub fn get_as_jpeg(&self, scale_factor: f32, quality: u8) -> Option<BinaryImage> {
        self.0.get_as_jpeg.and_then(|get_as_jpeg| {
            let mut pixel_width = 0;
            let mut pixel_height = 0;
            let binary = unsafe { get_as_jpeg(self.as_ptr(), scale_factor, std::cmp::min(100, quality) as i32, &mut pixel_width, &mut pixel_height) };
            BinaryValue::from_ptr(binary).map(|data| BinaryImage {
                pixel_width,
                pixel_height,
                data: data.into(),
            })
        })
    }
}
