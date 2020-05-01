use winit::{
    window::{Window, WindowBuilder},
    event_loop::EventLoop,
};
use cef::{
    browser_host::PaintElementType,
    values::Rect,
};
use winit_blit::{PixelBufferTyped, BGRA};

pub struct WinitBlitRenderer {
    window: Window,
    pixel_buffer: PixelBufferTyped<BGRA>,
    popup_rect: Option<Rect>,
}

impl crate::Renderer for WinitBlitRenderer {
    fn new<T>(window_builder: WindowBuilder, el: &EventLoop<T>) -> Self {
        let window = window_builder.build(el).unwrap();
        let width = window.inner_size().width;
        let height = window.inner_size().height;
        WinitBlitRenderer {
            pixel_buffer: PixelBufferTyped::new_supported(width, height, &window),
            window,
            popup_rect: None,
        }
    }
    fn window(&self) -> &Window {
        &self.window
    }
    fn on_paint(
        &mut self,
        element_type: PaintElementType,
        dirty_rects: &[Rect],
        buffer: &[u8],
        width: i32,
        height: i32,
    ) {
        println!("paint {:?}", dirty_rects);
        let buffer = BGRA::from_raw_slice(buffer);
        assert_eq!(buffer.len(), (width * height) as usize);
        let buffer_row = |row: usize| {
            &buffer[row as usize * width as usize..(1 + row) as usize * width as usize]
        };
        let pixel_buffer = &mut self.pixel_buffer;
        if pixel_buffer.width() != width as u32 || pixel_buffer.height() != height as u32 {
            *pixel_buffer = PixelBufferTyped::new_supported(width as u32, height as u32, &self.window);
        }
        match (element_type, self.popup_rect) {
            (PaintElementType::View, _) => {
                for rect in dirty_rects {
                    let row_span = rect.x as usize..rect.x as usize + rect.width as usize;
                    for row in (rect.y..rect.y+rect.height).map(|r| r as usize) {
                        let pixel_buffer_row =
                            &mut pixel_buffer.row_mut(row as u32).unwrap()
                                [row_span.clone()];
                        pixel_buffer_row.copy_from_slice(&buffer_row(row)[row_span.clone()]);
                    }

                    pixel_buffer.blit_rect(
                        (rect.x as u32, rect.y as u32),
                        (rect.x as u32, rect.y as u32),
                        (rect.width as u32, rect.height as u32),
                        &self.window
                    ).unwrap();
                }
            },
            (PaintElementType::Popup, Some(rect)) => {
                let row_span = rect.x as usize..rect.x as usize + rect.width as usize;
                for row in (rect.y..rect.y+rect.height).map(|r| r as usize) {
                    let pixel_buffer_row =
                        &mut pixel_buffer.row_mut(row as u32).unwrap()
                            [row_span.clone()];
                    pixel_buffer_row.copy_from_slice(&buffer_row(row)[row_span.clone()]);
                }

                pixel_buffer.blit_rect(
                    (rect.x as u32, rect.y as u32),
                    (rect.x as u32, rect.y as u32),
                    (rect.width as u32, rect.height as u32),
                    &self.window
                ).unwrap();
            },
            _ => (),
        }
    }
    fn set_popup_rect(&mut self, rect: Option<Rect>) {
        self.popup_rect = rect;
    }
}
