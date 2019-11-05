use glutin::WindowedContext;
use glutin::PossiblyCurrent;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::{Window, WindowBuilder};
use glutin::ContextBuilder;
use gullery::{
    buffer::*,
    framebuffer::{render_state::*, *},
    geometry::D2,
    glsl::{GLSLFloat, GLVec2, Normalized},
    image_format::*,
    program::*,
    texture::*,
    vertex::VertexArrayObject,
    ContextState,
};
use gullery_macros::{Vertex, Uniforms};
pub type BGRA = Bgra;

pub struct QuickBlit {
    pub windowed_context: Option<WindowedContext<PossiblyCurrent>>,
    vao: VertexArrayObject<Vertex, u16>,
    texture: Texture<D2, Bgra>,
    program: Program<Vertex, Uniforms<'static>>,
    state: std::rc::Rc<ContextState>,
    buffer: Vec<Bgra>,
    default_framebuffer: FramebufferDefault,
}

unsafe impl Send for QuickBlit {}

#[derive(Vertex, Clone, Copy)]
struct Vertex {
    pos: GLVec2<f32>,
    tex_coord: GLVec2<u16, Normalized>,
}

#[derive(Clone, Copy, Uniforms)]
struct Uniforms<'a> {
    tex: &'a Texture<D2, dyn ImageFormat<ScalarType = GLSLFloat>>,
}

impl QuickBlit {
    pub unsafe fn new<T: 'static>(window: WindowBuilder, el: &EventLoop<T>) -> QuickBlit {
        let windowed_context =
            ContextBuilder::new().build_windowed(window, &el).unwrap();

        let windowed_context = windowed_context.make_current().unwrap();
        let state = ContextState::new(|addr| windowed_context.get_proc_address(addr));

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

        QuickBlit {
            windowed_context: Some(windowed_context),
            vao,
            texture,
            program,
            state,
            buffer,
            default_framebuffer,
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
