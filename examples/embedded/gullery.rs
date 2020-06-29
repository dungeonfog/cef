use glutin::WindowedContext;
use glutin::PossiblyCurrent;
use glutin::event_loop::EventLoop;
use glutin::window::{Window, WindowBuilder};
use glutin::{ContextBuilder, GlRequest};
use gullery::{
    buffer::*,
    framebuffer::{render_state::*, *},
    geometry::{D2, GLSLFloat, GLVec2, Normalized},
    image_format::*,
    program::*,
    texture::*,
    vertex::VertexArrayObject,
    ContextState,
};
use cef::{
    browser_host::PaintElementType,
    values::Rect,
};
use gullery_macros::{Vertex, Uniforms};
pub type BGRA = Bgra;

pub struct GulleryRenderer {
    windowed_context: Option<WindowedContext<PossiblyCurrent>>,
    vao: VertexArrayObject<Vertex, u16>,
    texture: Texture<D2, Bgra>,
    program: Program<Vertex, Uniforms<'static>>,
    state: std::rc::Rc<ContextState>,
    buffer: Vec<Bgra>,
    default_framebuffer: FramebufferDefault,
    popup_rect: Option<Rect>,
}

unsafe impl Send for GulleryRenderer {}

impl crate::Renderer for GulleryRenderer {
    fn new<T>(window_builder: WindowBuilder, el: &EventLoop<T>) -> Self {
        unsafe {
            GulleryRenderer::new(window_builder, el)
        }
    }
    fn window(&self) -> &Window {
        self.windowed_context.as_ref().unwrap().window()
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
        let pixel_buffer = self;
        pixel_buffer.resize((width as _, height as _));

        match (element_type, pixel_buffer.popup_rect) {
            (PaintElementType::View, _) => {
                for rect in dirty_rects {
                    let row_span = rect.x as usize..rect.x as usize + rect.width as usize;
                    for row in (rect.y..rect.y+rect.height).map(|r| r as usize) {
                        let pixel_buffer_row =
                            &mut pixel_buffer.row_mut(row as u32).unwrap()
                                [row_span.clone()];
                        pixel_buffer_row.copy_from_slice(&buffer_row(row)[row_span.clone()]);
                    }
                }
                unsafe {pixel_buffer.blit()}
            },
            (PaintElementType::Popup, Some(rect)) => {
                let row_span = rect.x as usize..rect.x as usize + rect.width as usize;
                for row in (rect.y..rect.y+rect.height).map(|r| r as usize) {
                    let pixel_buffer_row =
                        &mut pixel_buffer.row_mut(row as u32).unwrap()
                            [row_span.clone()];
                    pixel_buffer_row.copy_from_slice(&buffer_row(row)[row_span.clone()]);
                }

                unsafe {pixel_buffer.blit()}
            },
            _ => (),
        }
    }
    fn set_popup_rect(&mut self, rect: Option<Rect>) {
        self.popup_rect = rect;
    }
}

#[derive(Vertex, Clone, Copy)]
struct Vertex {
    pos: GLVec2<f32>,
    tex_coord: GLVec2<u16, Normalized>,
}

#[derive(Clone, Copy, Uniforms)]
struct Uniforms<'a> {
    tex: &'a Texture<D2, dyn ImageFormat<ScalarType = GLSLFloat>>,
}

impl GulleryRenderer {
    unsafe fn new<T: 'static>(window: WindowBuilder, el: &EventLoop<T>) -> GulleryRenderer {
        let windowed_context = ContextBuilder::new()
            .with_depth_buffer(0)
            .with_stencil_buffer(0)
            .with_pixel_format(24, 8)
            .with_gl(GlRequest::Latest)
            .with_vsync(false)
            .build_windowed(window, &el).unwrap();

        let windowed_context = windowed_context.make_current().unwrap();
        let state = ContextState::new(|addr| windowed_context.get_proc_address(addr) as _);

        let vertex_buffer = Buffer::with_data(
            BufferUsage::StaticDraw,
            &[
                Vertex {
                    pos: GLVec2::new(-1.0, -1.0),
                    tex_coord: GLVec2::new(0, !0),
                },
                Vertex {
                    pos: GLVec2::new(-1.0, 1.0),
                    tex_coord: GLVec2::new(0, 0),
                },
                Vertex {
                    pos: GLVec2::new(1.0, 1.0),
                    tex_coord: GLVec2::new(!0, 0),
                },
                Vertex {
                    pos: GLVec2::new(1.0, -1.0),
                    tex_coord: GLVec2::new(!0, !0),
                },
            ],
            state.clone(),
        );
        let index_buffer = Buffer::with_data(
            BufferUsage::StaticDraw,
            &[0, 1, 2, 2, 3, 0u16],
            state.clone(),
        );
        let vao = VertexArrayObject::new(vertex_buffer, Some(index_buffer));
        let texture: Texture<D2, Bgra> = Texture::with_mip_count(
            GLVec2::new(1, 1),
            1,
            state.clone(),
        )
        .unwrap();
        let buffer = vec![Bgra::new(0, 0, 0, 255); 1];

        let vertex_shader = Shader::new(VERTEX_SHADER, state.clone()).unwrap();
        let fragment_shader = Shader::new(FRAGMENT_SHADER, state.clone()).unwrap();
        let (program, _): (Program<Vertex, Uniforms>, _) = Program::new(&vertex_shader, None, &fragment_shader).unwrap();
        let default_framebuffer = FramebufferDefault::new(state.clone()).unwrap();

        GulleryRenderer {
            windowed_context: Some(windowed_context),
            vao,
            texture,
            program,
            state,
            buffer,
            default_framebuffer,
            popup_rect: None,
        }
    }

    pub fn resize(&mut self, size: (u32, u32)) {
        let size = GLVec2::new(size.0, size.1);
        if size != self.texture.dims() {
            self.texture = Texture::with_mip_count(
                size,
                1,
                self.state.clone(),
            ).unwrap();
            self.buffer = vec![Bgra::new(0, 0, 0, 255); (size.x * size.y) as usize];
        }
    }

    pub fn row_mut(&mut self, row: u32) -> Option<&mut [Bgra]> {
        let pixel_len = self.texture.dims().x as usize;
        self.buffer[..]
            .chunks_mut(pixel_len)
            .nth(row as usize)
    }

    pub unsafe fn blit(&mut self) {
        self.windowed_context = Some(self.windowed_context.take().unwrap().make_current().unwrap());
        let dims = self.texture.dims();
        self.texture.sub_image(
            0,
            GLVec2::new(0, 0),
            dims,
            &*self.buffer,
        );
        let uniform = Uniforms {
            tex: self.texture.as_dyn(),
        };
        let render_state = RenderState {
            srgb: false,
            viewport: GLVec2::new(0, 0)..=self.texture.dims(),
            ..RenderState::default()
        };
        self.default_framebuffer.clear_depth(1.0);
        self.default_framebuffer.clear_color_all(Rgba::new(0.0, 0.0, 0.0, 1.0));
        self.default_framebuffer.draw(
            DrawMode::Triangles,
            ..,
            &self.vao,
            &self.program,
            uniform,
            &render_state,
        );

        self.windowed_context.as_ref().unwrap().swap_buffers().unwrap();
    }
}

const VERTEX_SHADER: &str = r#"
    #version 330

    in vec2 pos;
    in vec2 tex_coord;
    out vec2 tc;

    void main() {
        tc = tex_coord;
        gl_Position = vec4(pos, 0.0, 1.0);
    }
"#;

const FRAGMENT_SHADER: &str = r#"
    #version 330

    in vec2 tc;
    out vec4 color;

    uniform sampler2D tex;

    void main() {
        color = texture(tex, tc);
    }
"#;
