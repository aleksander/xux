use render::Event;
use driver;
use std::sync::mpsc::Sender;
use gfx_device_gl;
use gfx_window_glutin;
use gfx::traits::FactoryExt;
use gfx::{self, Device};
use gfx::format::*;
use glutin::{self, GlContext, MouseButton};
use glutin::WindowEvent::{KeyboardInput, Closed, Resized, MouseWheel, MouseInput, MouseMoved};
use cgmath::{Matrix2, Matrix3, Vector2, Vector3, SquareMatrix, Rad, Zero};
use imgui::{ImGui, Ui, GlyphRange};
use imgui_gfx_renderer::{Renderer, Shaders};
use image;
use proto::{ObjXY,ObjID,ResID};
use std::collections::{HashMap, BTreeMap};
use render::XUi;

mod delta {
    use std::time::Instant;

    pub struct Delta {
        last: Instant
    }

    impl Delta {
        pub fn new () -> Delta {
            Delta { last: Instant::now() }
        }

        pub fn tick (&mut self) -> f32 {
            let now = Instant::now();
            let delta = now - self.last;
            self.last = now;
            delta.as_secs() as f32 + delta.subsec_nanos() as f32 / 1_000_000_000.0
        }
    }
}

const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

gfx_defines! {
    vertex VertexCol {
        position: [f32; 2] = "position",
        color: [f32; 3] = "color",
        height: i32 = "height",
    }

    vertex VertexTex {
        position: [f32; 2] = "position",
        uv: [f32;2] = "uv",
    }

    pipeline pipe_col {
        vbuf: gfx::VertexBuffer<VertexCol> = (),
        transform: gfx::Global<[[f32; 3]; 3]> = "transform",
        threshold: gfx::Global<i32> = "threshold",
        //out: gfx::RenderTarget<Rgba8> = "Target0",
        out: gfx::BlendTarget<Rgba8> = ("Target0", gfx::state::MASK_ALL, gfx::preset::blend::ALPHA),
    }

    pipeline pipe_tex {
        vbuf: gfx::VertexBuffer<VertexTex> = (),
        tex: gfx::TextureSampler<[f32; 4]> = "sampler",
        transform: gfx::Global<[[f32; 3]; 3]> = "transform",
        //out: gfx::RenderTarget<Rgba8> = "Target0",
        out: gfx::BlendTarget<Rgba8> = ("Target0", gfx::state::MASK_ALL, gfx::preset::blend::ALPHA),
    }
}

//FIXME transparent color is BLACK for some reason
const VERTEX_SHADER_COL: &str = "
    #version 130

    in vec2 position;
    in vec3 color;
    in int height;
    out vec4 v_color;

    uniform mat3 transform;
    uniform int threshold;

    void main() {
        if (height > threshold) {
            v_color = vec4(color, 1.0);
        } else {
            v_color = vec4(0.0, 0.0, 0.0, 0.0);
        }
        gl_Position = vec4(transform * vec3(position, 1.0), 1.0);
    }
";

const FRAGMENT_SHADER_COL: &str = "
    #version 130

    in vec4 v_color;
    out vec4 Target0;

    void main() {
        Target0 = v_color;
    }
";

const VERTEX_SHADER_TEX: &str = "
    #version 130

    in vec2 position;
    in vec2 uv;
    out vec2 v_uv;

    uniform mat3 transform;

    void main() {
        v_uv = uv;
        gl_Position = vec4(transform * vec3(position, 1.0), 1.0);
    }
";

const FRAGMENT_SHADER_TEX: &str = "
    #version 130

    in vec2 v_uv;
    out vec4 Target0;

    uniform sampler2D sampler;

    void main() {
        Target0 = texture(sampler, v_uv);
    }
";

impl VertexCol {
    fn new (x: f32, y: f32, color: [f32; 3], height: i32) -> VertexCol {
        VertexCol {
            position: [x, y],
            color: color,
            height: height
        }
    }

    fn shift (&mut self, x: f32, y: f32) {
        self.position[0] += x;
        self.position[1] += y;
    }
}
impl VertexTex {
    fn shift (&mut self, x: f32, y: f32) {
        self.position[0] += x;
        self.position[1] += y;
    }
}

struct ObjCol {
    vertices: Vec<VertexCol>,
    indices: Vec<u32>,
}

impl ObjCol {

    fn new () -> ObjCol {
        ObjCol {
            vertices: Vec::new(),
            indices: Vec::new()
        }
    }

    fn with_capacity (vcap: usize, icap: usize) -> ObjCol {
        ObjCol {
            vertices: Vec::with_capacity(vcap),
            indices: Vec::with_capacity(icap)
        }
    }

    fn translate (&mut self, x: f32, y: f32) {
        for v in self.vertices.iter_mut() {
            v.shift(x,y);
        }
    }

    #[allow(dead_code)]
    fn grid_from_heights (size: usize, cell_size: f32, grid_x: i32, grid_y: i32, heights: &[i16], t/*thickness*/: f32) -> ObjCol {
        assert_eq!(heights.len(), size*size);
        const WARM: [f32; 3] = [1.0, 0.3, 0.3];
        const COLD: [f32; 3] = [0.3, 0.3, 1.0];
        let mut obj = ObjCol::new();
        let mut i = 0;
        for x in 0..size-1 {
            for y in 0..size-1 {
                let a = heights[y*size+x];
                let b = heights[y*size+x+1];
                let c = heights[(y+1)*size+x];
                if a != b {
                    let ax = x as f32 * cell_size;// + 0.5;
                    let ay = y as f32 * cell_size;
                    let bx = ax + cell_size;// - 1.0;
                    if a > b {
                        let height = (a - b) as i32;
                        obj.vertices.push(VertexCol::new(ax, ay+t, WARM, height));
                        obj.vertices.push(VertexCol::new(bx, ay,   WARM, height));
                        obj.vertices.push(VertexCol::new(ax, ay-t, WARM, height));
                    } else {
                        let height = (b - a) as i32;
                        obj.vertices.push(VertexCol::new(ax, ay,   COLD, height));
                        obj.vertices.push(VertexCol::new(bx, ay+t, COLD, height));
                        obj.vertices.push(VertexCol::new(bx, ay-t, COLD, height));
                    }
                    obj.indices.push(i); i+=1;
                    obj.indices.push(i); i+=1;
                    obj.indices.push(i); i+=1;
                }
                if a != c {
                    let ax = x as f32 * cell_size;
                    let ay = y as f32 * cell_size;// + 0.5;
                    let cy = ay + cell_size;// - 1.0;
                    if a > c {
                        let height = (a - b) as i32;
                        obj.vertices.push(VertexCol::new(ax-t, ay, WARM, height));
                        obj.vertices.push(VertexCol::new(ax, cy,   WARM, height));
                        obj.vertices.push(VertexCol::new(ax+t, ay, WARM, height));
                    } else {
                        let height = (b - a) as i32;
                        obj.vertices.push(VertexCol::new(ax, ay,   COLD, height));
                        obj.vertices.push(VertexCol::new(ax-t, cy, COLD, height));
                        obj.vertices.push(VertexCol::new(ax+t, cy, COLD, height));
                    }
                    obj.indices.push(i); i+=1;
                    obj.indices.push(i); i+=1;
                    obj.indices.push(i); i+=1;
                }
            }
        }
        obj.translate(size as f32 * cell_size * grid_x as f32, size as f32 * cell_size * grid_y as f32);
        obj
    }

    #[allow(dead_code)]
    fn grid_from_heights2 (size: usize, cell_size: f32, grid_x: i32, grid_y: i32, heights: &[i16]) -> ObjCol {
        use std::cmp::max;
        assert_eq!(heights.len(), size*size);
        let mut obj = ObjCol::new();
        for x in 0..size-1 {
            for y in 0..size-1 {
                let a = heights[y*size+x];
                let b = heights[y*size+x+1];
                let c = heights[(y+1)*size+x];
                let d = heights[(y+1)*size+x+1];
                let delta_ab = if a > b { a - b } else { b - a } as usize;
                let delta_ac = if a > c { a - c } else { c - a } as usize;
                let delta_bd = if b > d { b - d } else { d - b } as usize;
                let delta_cd = if c > d { c - d } else { d - c } as usize;
                let delta = max(max(delta_ab, delta_ac), max(delta_cd, delta_bd));
                if delta > 0 {
                    let ax = x as f32 * cell_size + 2.0;
                    let ay = y as f32 * cell_size + 2.0;
                    let dx = ax + cell_size - 4.0;
                    let dy = ay + cell_size - 4.0;
                    obj.add_quad(ax, ay, dx, dy, [0.1,0.1,0.1], delta as i32);
                } 
            }
        }
        obj.translate(size as f32 * cell_size * grid_x as f32, size as f32 * cell_size * grid_y as f32);
        obj
    }

    fn add_quad (&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: [f32; 3], height: i32) {
        let len = self.vertices.len() as u32;
        self.vertices.push(VertexCol::new(x1, y1, color, height));
        self.vertices.push(VertexCol::new(x1, y2, color, height));
        self.vertices.push(VertexCol::new(x2, y2, color, height));
        self.vertices.push(VertexCol::new(x2, y1, color, height));
        self.indices.push(len);
        self.indices.push(len+1);
        self.indices.push(len+2);
        self.indices.push(len);
        self.indices.push(len+2);
        self.indices.push(len+3);
    }

    fn from_objects (objects: &BTreeMap<ObjID,(ObjXY,ResID)>, hero_x: f32, hero_y: f32) -> ObjCol {
        let mut obj = ObjCol::with_capacity(objects.len()*4, objects.len()*6);
        for &(ObjXY(x,y),_resid) in objects.values() {
            obj.add_quad(x as f32 - 2.0, y as f32 - 2.0, x as f32 + 2.0, y as f32 + 2.0, [1.0,1.0,1.0], 1);
        }
        obj.add_quad(hero_x - 2.0, hero_y - 2.0, hero_x + 2.0, hero_y + 2.0, [0.1,1.0,0.1], 1);
        obj
    }

    fn bake (self, main_color: gfx::handle::RenderTargetView<gfx_device_gl::Resources, Rgba8>, factory: &mut gfx_device_gl::Factory, threshold: usize) -> BakedObjCol {
        let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&self.vertices, &self.indices[..]);
        let data = pipe_col::Data {
            vbuf: vertex_buffer,
            transform: Matrix3::identity().into(),
            threshold: threshold as i32,
            out: main_color
        };
        BakedObjCol {
            slice: slice,
            data: data
        }
    }
}

struct ObjTex {
    vertices: Vec<VertexTex>,
    indices: Vec<u32>,
    image: image::ImageBuffer<image::Rgba<u8>,Vec<u8>>,
}

impl ObjTex {
    fn with_capacity (vcap: usize, icap: usize, image_x: usize, image_y: usize) -> ObjTex {
        ObjTex {
            vertices: Vec::with_capacity(vcap),
            indices: Vec::with_capacity(icap),
            image: image::ImageBuffer::new(image_x as u32, image_y as u32),
        }
    }

    fn translate (&mut self, x: f32, y: f32) {
        for v in self.vertices.iter_mut() {
            v.shift(x,y);
        }
    }

    fn plane <F> (size: f32, color_fn: F) -> ObjTex where F: Fn(usize,usize)->[u8; 4] {
        let mut obj = ObjTex::with_capacity(4, 6, 100, 100);
        for y in 0..100 {
            for x in 0..100 {
                obj.image.put_pixel(x, y, image::Rgba(color_fn(x as usize, y as usize)));
            }
        }
        obj.vertices.push(VertexTex{position:[0.0, 0.0], uv:[0.0, 0.0] });
        obj.vertices.push(VertexTex{position:[size, 0.0], uv:[1.0, 0.0] });
        obj.vertices.push(VertexTex{position:[size, size], uv:[1.0, 1.0] });
        obj.vertices.push(VertexTex{position:[0.0, size], uv:[0.0, 1.0] });
        obj.indices.push(0);
        obj.indices.push(1);
        obj.indices.push(2);
        obj.indices.push(0);
        obj.indices.push(2);
        obj.indices.push(3);
        obj
    }

    fn plane_from_owning (size: f32, grid_x: i32, grid_y: i32, owning: &[u8]) -> ObjTex {
        assert_eq!(owning.len(), 100*100);
        let mut obj = ObjTex::plane(size, |x,y|{
            let owning = owning[y*100+x];
            if owning == 0 {
                [0,0,0,0]
            } else {
                //TODO different colors for personal and city claims
                [127*(owning&1)      + 127*((owning>>1)&1),
                 127*((owning>>2)&1) + 127*((owning>>3)&1),
                 127*((owning>>4)&1) + 127*((owning>>5)&1), 63]
            }
        });
        obj.translate(size as f32 * grid_x as f32, size as f32 * grid_y as f32);
        obj
            /*
               if owning[y*100+x] == 0 {
               palette[index]
               } else {
               let r: u8 = palette[index][0];
               let g = palette[index][1];
               let b = palette[index][2];
               let a = palette[index][3];
               [r.saturating_add(50u8),g,b,a]
               }
               */
    }

    fn plane_from_tiles (size: f32, grid_x: i32, grid_y: i32, tiles: &[u8], palette: &[[u8; 4]; 256]) -> ObjTex {
        assert_eq!(tiles.len(), 100*100);
        let mut obj = ObjTex::plane(size, |x,y|{
            let index = tiles[y*100+x] as usize;
            palette[index]
        });
        obj.translate(size as f32 * grid_x as f32, size as f32 * grid_y as f32);
        obj
    }

    fn bake (self, main_color: gfx::handle::RenderTargetView<gfx_device_gl::Resources, Rgba8>, factory: &mut gfx_device_gl::Factory) -> BakedObjTex {
        use gfx::Factory;
        use gfx::texture::{SamplerInfo, FilterMethod, WrapMode, Lod, PackedColor};
        use gfx::state::Comparison;
        let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&self.vertices, &self.indices[..]);
        //TODO find more convenient way to create "nearest" sampler
        let sampler = factory.create_sampler(
            SamplerInfo{
                filter: FilterMethod::Scale,
                wrap_mode: (WrapMode::Tile, WrapMode::Tile, WrapMode::Tile),
                lod_bias: Lod::from(0.0),
                lod_range: (Lod::from(0.0), Lod::from(0.0)),
                comparison: Some(Comparison::Never),
                border: PackedColor(0),
            }
            );
        let (width, height) = (100,100); //FIXME self.image.dimensions();
        let kind = gfx::texture::Kind::D2(width as u16, height as u16, gfx::texture::AaMode::Single);
        let (_, view) = factory.create_texture_immutable_u8::<Rgba8>(kind, &[&self.image]).unwrap();
        let data = pipe_tex::Data {
            vbuf: vertex_buffer,
            tex: (view, sampler),
            transform: Matrix3::identity().into(),
            out: main_color
        };
        BakedObjTex {
            slice: slice,
            data: data
        }
    }
}

struct BakedObjCol {
    slice: gfx::Slice<gfx_device_gl::Resources>,
    data: pipe_col::Data<gfx_device_gl::Resources>
}

struct BakedObjTex {
    slice: gfx::Slice<gfx_device_gl::Resources>,
    data: pipe_tex::Data<gfx_device_gl::Resources>
}

fn run_ui <'a> (ui: &Ui<'a>,
                fps: usize,
                threshold: usize,
                show_tiles: &mut bool,
                show_heights: &mut bool,
                show_owning: &mut bool,
                v1: &mut i32,
                objects: &BTreeMap<ObjID,(ObjXY,ResID)>,
                resources: &BTreeMap<ResID,String>) -> bool {
    use imgui::*;
    ui.window(im_str!("Клёцка"))
        .size((300.0, 600.0), ImGuiSetCond_FirstUseEver)
        .position((10.0, 10.0), ImGuiSetCond_FirstUseEver)
        .build(|| {
            ui.text(im_str!("Привет, Мир!!!"));
            ui.separator();
            let (x,y) = ui.imgui().mouse_pos();
            ui.text(im_str!("Mouse Position: ({:.1},{:.1})", x, y));
            ui.text(im_str!("FPS: {}", fps));
            ui.text(im_str!("threshold: {}", threshold));
            ui.checkbox(im_str!("tiles"), show_tiles);
            ui.checkbox(im_str!("heights"), show_heights);
            /*TODO indent*/ ui.radio_button(im_str!("quads"), v1, 0);
            /*TODO indent*/ ui.radio_button(im_str!("arrows"), v1, 1);
            /*TODO indent*/ ui.radio_button(im_str!("heatmap"), v1, 2);
            ui.checkbox(im_str!("owning"), show_owning);
        });
    ui.window(im_str!("Объекты"))
    //ui.window(im_str!("Объекты: {}", objects.len()))
        .size((300.0, 600.0), ImGuiSetCond_FirstUseEver)
        .position((300.0, 10.0), ImGuiSetCond_FirstUseEver)
        .build(|| {
            for ( &objid, &(ObjXY(_x,_y), ref resid) ) in objects.iter() {
                let resname = match resources.get(resid) {
                    Some(ref name) => name,
                    None => "???"
                };
                ui.text(im_str!("{} {}", objid, resname));
            }
        });

    true
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
struct MouseState {
    pos: (i32, i32),
    pressed: (bool, bool, bool),
    wheel: f32,
}

fn ui_handle_event (imgui: &mut ImGui, mouse_state: &mut MouseState, event: &glutin::Event) {
    use glutin::WindowEvent::*;
    use glutin::ElementState::Pressed;
    use glutin::{Event, MouseButton, MouseScrollDelta, TouchPhase};

    if let Event::WindowEvent { ref event, .. } = *event {
        match *event {
            //Resized(_, _) => {
            //gfx_window_glutin::update_views(&window, &mut main_color, &mut main_depth);
            //renderer.update_render_target(main_color.clone());
            //}
            //Closed => quit = true,
            KeyboardInput { input, .. } => {
                use glutin::VirtualKeyCode as Key;

                let pressed = input.state == Pressed;
                match input.virtual_keycode {
                    Some(Key::Tab) => imgui.set_key(0, pressed),
                    Some(Key::Left) => imgui.set_key(1, pressed),
                    Some(Key::Right) => imgui.set_key(2, pressed),
                    Some(Key::Up) => imgui.set_key(3, pressed),
                    Some(Key::Down) => imgui.set_key(4, pressed),
                    Some(Key::PageUp) => imgui.set_key(5, pressed),
                    Some(Key::PageDown) => imgui.set_key(6, pressed),
                    Some(Key::Home) => imgui.set_key(7, pressed),
                    Some(Key::End) => imgui.set_key(8, pressed),
                    Some(Key::Delete) => imgui.set_key(9, pressed),
                    Some(Key::Back) => imgui.set_key(10, pressed),
                    Some(Key::Return) => imgui.set_key(11, pressed),
                    Some(Key::Escape) => imgui.set_key(12, pressed),
                    Some(Key::A) => imgui.set_key(13, pressed),
                    Some(Key::C) => imgui.set_key(14, pressed),
                    Some(Key::V) => imgui.set_key(15, pressed),
                    Some(Key::X) => imgui.set_key(16, pressed),
                    Some(Key::Y) => imgui.set_key(17, pressed),
                    Some(Key::Z) => imgui.set_key(18, pressed),
                    Some(Key::LControl) | Some(Key::RControl) => imgui.set_key_ctrl(pressed),
                    Some(Key::LShift) | Some(Key::RShift) => imgui.set_key_shift(pressed),
                    Some(Key::LAlt) | Some(Key::RAlt) => imgui.set_key_alt(pressed),
                    Some(Key::LWin) | Some(Key::RWin) => imgui.set_key_super(pressed),
                    _ => {}
                }
            }
            MouseMoved { position: (x, y), .. } => mouse_state.pos = (x as i32, y as i32),
            MouseInput { state, button, .. } => {
                match button {
                    MouseButton::Left => mouse_state.pressed.0 = state == Pressed,
                    MouseButton::Right => mouse_state.pressed.1 = state == Pressed,
                    MouseButton::Middle => mouse_state.pressed.2 = state == Pressed,
                    _ => {}
                }
            }
            MouseWheel {
                delta: MouseScrollDelta::LineDelta(_, y),
                phase: TouchPhase::Moved,
                ..
            } |
            MouseWheel {
                delta: MouseScrollDelta::PixelDelta(_, y),
                phase: TouchPhase::Moved,
                ..
            } => mouse_state.wheel = y,
            ReceivedCharacter(c) => imgui.add_input_character(c),
            _ => (),
        }
    }

    update_mouse(imgui, mouse_state);
}

fn update_mouse(imgui: &mut ImGui, mouse_state: &mut MouseState) {
    let scale = imgui.display_framebuffer_scale();
    imgui.set_mouse_pos(
        mouse_state.pos.0 as f32 / scale.0,
        mouse_state.pos.1 as f32 / scale.1,
        );
    imgui.set_mouse_down(
        &[
        mouse_state.pressed.0,
        mouse_state.pressed.1,
        mouse_state.pressed.2,
        false,
        false,
        ],
        );
    imgui.set_mouse_wheel(mouse_state.wheel / scale.1);
    mouse_state.wheel = 0.0;
}



pub struct RenderImpl {
    tile_colors: HashMap<String,[u8;4]>,
    palette: [[u8; 4]; 256],
    events_loop: glutin::EventsLoop,
    window: glutin::GlWindow,
    device: gfx_device_gl::Device,
    factory: gfx_device_gl::Factory,
    main_color: gfx::handle::RenderTargetView<gfx_device_gl::Resources, Rgba8>,
    main_depth: gfx::handle::DepthStencilView<gfx_device_gl::Resources, DepthStencil>,
    encoder: gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,
    pso_col: gfx::PipelineState<gfx_device_gl::Resources, pipe_col::Meta>,
    pso_tex: gfx::PipelineState<gfx_device_gl::Resources, pipe_tex::Meta>,
    imgui: ImGui,
    imgui_renderer: Renderer<gfx_device_gl::Resources>,
    angle: f32, //TODO replace by camera.angle
    zoom: f32, //TODO replace by camera.zoom
    w: u32,
    h: u32,
    delta: delta::Delta,
    mouse_state: MouseState, //TODO move to struct Ui
    grids_tiles: Vec<BakedObjTex>,
    grids_owning: Vec<BakedObjTex>,
    grids_heights: Vec<BakedObjCol>,
    objects: BTreeMap<ObjID, (ObjXY, ResID)>,
    hero_x: f32,
    hero_y: f32,
    resources: BTreeMap<ResID, String>,
    dragging: bool,
    last_mouse_x: f64,
    last_mouse_y: f64,
    shift: Vector2<f32>, //TODO replace by camera.{x,y}
    ctrl_pressed: bool,
    threshold: usize,
    show_tiles: bool,
    show_heights: bool,
    show_owning: bool,
    v1: i32,
    widgets: BTreeMap<u16, (String, u16)>,
    xui: XUi,
}

impl RenderImpl {
    pub fn new () -> RenderImpl {

        let events_loop = glutin::EventsLoop::new();
        let context = glutin::ContextBuilder::new();
        let builder = glutin::WindowBuilder::new()
            .with_title("gfx 2d test".to_string())
            .with_dimensions(800, 600);

        let (window, device, mut factory, main_color, main_depth) =
            gfx_window_glutin::init::<Rgba8, DepthStencil>(builder, context, &events_loop);


        let (w, h) = window.get_inner_size_points().expect("get_inner_size_points failed");

        let shaders = {
            let version = device.get_info().shading_language;
            if version.is_embedded {
                if version.major >= 3 {
                    Shaders::GlSlEs300
                } else {
                    Shaders::GlSlEs100
                }
            } else {
                if version.major >= 4 {
                    Shaders::GlSl400
                } else if version.major >= 3 {
                    Shaders::GlSl130
                } else {
                    Shaders::GlSl110
                }
            }
        };

        let mut imgui = ImGui::init();
        imgui.set_font("DejaVuSansMono.ttf", 12.0, GlyphRange::Cyrillic).expect("Failed to imgui.set_font");
        RenderImpl {
            tile_colors: {
                use ron::de::from_reader;
                use std::io::BufReader;
                use std::fs::File;

                let f = File::open("tile_colors.ron").expect("unable to open tile_colors.ron");
                from_reader(BufReader::new(f)).expect("unable to deserialize")
            },
            palette: [[0,0,0,255]; 256],
            events_loop: events_loop,
            window: window,
            device: device,
            encoder: factory.create_command_buffer().into(),
            pso_col: factory.create_pipeline_simple(VERTEX_SHADER_COL.as_bytes(), FRAGMENT_SHADER_COL.as_bytes(), pipe_col::new()).expect("create_pipeline_simple failed"),
            pso_tex: factory.create_pipeline_simple(VERTEX_SHADER_TEX.as_bytes(), FRAGMENT_SHADER_TEX.as_bytes(), pipe_tex::new()).expect("create_pipeline_simple failed"),
            imgui_renderer: Renderer::init(&mut imgui, &mut factory, shaders, main_color.clone()).expect("Failed to initialize renderer"),
            imgui: imgui,
            factory: factory,
            main_color: main_color,
            main_depth: main_depth,
            angle: 0.0, //TODO replace by camera.angle
            zoom: 1.0, //TODO replace by camera.zoom
            w: w,
            h: h,
            delta: delta::Delta::new(),
            mouse_state: MouseState::default(), //TODO move to struct Ui
            grids_tiles: Vec::new(),
            grids_owning: Vec::new(),
            grids_heights: Vec::new(),
            objects: BTreeMap::new(),
            hero_x: 0.0,
            hero_y: 0.0,
            resources: BTreeMap::new(),
            dragging: false,
            last_mouse_x: 0.0,
            last_mouse_y: 0.0,
            shift: Vector2::zero(), //TODO replace by camera.{x,y}
            ctrl_pressed: false,
            threshold: 3,
            show_tiles: true,
            show_heights: true,
            show_owning: false,
            v1: 0,
            widgets: BTreeMap::new(),
            xui: XUi::new(),
        }
    }

    pub fn init (&self) {
    }

    pub fn event (&mut self, event: Event) {
        use render::Wdg;

        /*TODO app.event(event) */
        match event {
            Event::Tiles(tiles) => {
                for tile in tiles {
                    if let Some(color) = self.tile_colors.get(&tile.name) {
                        self.palette[tile.id as usize] = *color;
                    }
                }
            }
            Event::Grid(grid_x,grid_y,tiles,heights,owning) => {
                //println!("grid ({}, {})", grid_x, grid_y);
                //TODO app.rebuild_grid_cache(...)
                //XXX FIXME TODO one BIG mesh with all grids in it ?
                //or individual buffer+pipe.data for every grid ?
                //or use texture-per-grid and don't care at all
                let tiles = ObjTex::plane_from_tiles(1100.0, grid_x, grid_y, tiles.as_ref(), &self.palette)
                    .bake(self.main_color.clone(), &mut self.factory);
                self.grids_tiles.push(tiles);
                let owning = ObjTex::plane_from_owning(1100.0, grid_x, grid_y, owning.as_ref())
                    .bake(self.main_color.clone(), &mut self.factory);
                self.grids_owning.push(owning);
                //let heights = ObjCol::grid_from_heights(100, 11.0, grid_x, grid_y, heights.as_ref(), 1.0)
                //    .bake(main_color.clone(), &mut factory, threshold);
                let heights = ObjCol::grid_from_heights2(100, 11.0, grid_x, grid_y, heights.as_ref())
                    .bake(self.main_color.clone(), &mut self.factory, self.threshold);
                self.grids_heights.push(heights);
            }
            Event::Obj(id,xy,resid) => {
                //TODO ??? separate static objects like trees and
                //dynamic objects like rabbits to two
                //different caches
                self.objects.insert(id, (xy,resid));
            }
            Event::ObjRemove(ref id) => {
                self.objects.remove(id);
            }
            Event::Hero(ObjXY(x,y)) => {
                //TODO ??? add to objects
                self.hero_x = x as f32;
                self.hero_y = y as f32;
                //FIXME self.shift += Vector(-hero_x,-hero_y);
                self.shift[0] = -self.hero_x;
                self.shift[1] = -self.hero_y;
            }
            Event::Res(id, name) => {
                self.resources.insert(id, name);
            }
            Event::Wdg(Wdg::New(id,name,parent)) => {
                self.widgets.insert(id,(name.clone(),parent));
                self.xui.add_widget(id,name,parent).expect("unable to ui.add_widget");
            }
            Event::Wdg(Wdg::Msg(id,name)) => {
                self.xui.message(id,name).expect("unable to ui.message");
            }
            Event::Wdg(Wdg::Del(id)) => {
                self.widgets.remove(&id);
                self.xui.del_widget(id).expect("unable to ui.del_widget");
            }
            _ => {}
        }
    }

    //TODO FIXME split update() into update() and render() ???
    pub fn update (&mut self, render_tx: &Sender<driver::Event>) -> bool {
        use smallvec::SmallVec;
        // HANDLE EVENTS
        //TODO app.handle(...)
        //FIXME ugly hack !
        let mut events = SmallVec::<[glutin::Event; 64]>::new();
        self.events_loop.poll_events(|event| {
            events.push(event);
        });
        if events.spilled() {
            warn!("events smallvec spilled!");
        }
        for event in events.iter() {
            ui_handle_event(&mut self.imgui, &mut self.mouse_state, event);
            //TODO if ui.handle_event(event) == NOT_HANDLED {
            //         if app.handle_event(event) == NOT_HANDLED {
            //             match (event) {
            //                 ...
            //             }
            //         }
            //}
            match *event {
                glutin::Event::WindowEvent { ref event, .. } => {
                    //use glutin::KeyboardInput;
                    use glutin::ElementState::Pressed;
                    match *event {
                        KeyboardInput { input, .. } => {
                            use glutin::VirtualKeyCode as Key;
                            let pressed = input.state == Pressed;
                            match input.virtual_keycode {
                                Some(Key::Escape) => return false,
                                Some(Key::LControl) | Some(Key::RControl) => self.ctrl_pressed = pressed, //TODO use some kind of keys_state { ... }
                            Some(Key::Up) => { render_tx.send(driver::Event::Render(driver::RenderEvent::Up)).expect("unable to send Render::Up"); }
                            Some(Key::Down) => { render_tx.send(driver::Event::Render(driver::RenderEvent::Down)).expect("unable to send Render::Down"); }
                            Some(Key::Left) => { render_tx.send(driver::Event::Render(driver::RenderEvent::Left)).expect("unable to send Render::Left"); }
                            Some(Key::Right) => { render_tx.send(driver::Event::Render(driver::RenderEvent::Right)).expect("unable to send Render::Right"); }
                            _ => {}
                            }
                        }
                        Closed => return false,
                        Resized(width, height) => {
                            //TODO app.resize()
                            //TODO ui.resize()
                            self.w = width;
                            self.h = height;
                            gfx_window_glutin::update_views(&self.window, &mut self.main_color, &mut self.main_depth);
                            self.imgui_renderer.update_render_target(self.main_color.clone());
                            //TODO app.update_render_target(main_color.clone())
                            for t in self.grids_tiles.iter_mut() {
                                t.data.out = self.main_color.clone();
                            }
                            for t in self.grids_owning.iter_mut() {
                                t.data.out = self.main_color.clone();
                            }
                            for t in self.grids_heights.iter_mut() {
                                t.data.out = self.main_color.clone();
                            }
                        }
                        MouseWheel { delta: glutin::MouseScrollDelta::LineDelta(_, y), .. } => {
                            if self.ctrl_pressed {
                                if y < 0.0 { self.threshold += 1; } else { if self.threshold > 0 { self.threshold -= 1; } }
                            } else {
                                if y < 0.0 { self.zoom *= 1.1; } else { self.zoom *= 0.9; }
                            }
                        }
                        MouseInput {state, button: MouseButton::Left, ..} => self.dragging = state == Pressed,
                        MouseMoved {position: (x, y), ..} => {
                            let x = if x < 0.0 { 0.0 } else if x > self.w as f64 { self.w as f64 } else { x };
                            let y = if y < 0.0 { 0.0 } else if y > self.h as f64 { self.h as f64 } else { y };
                            let delta_x = x - self.last_mouse_x;
                            let delta_y = self.last_mouse_y - y; // y-axis is inverted
                            self.last_mouse_x = x;
                            self.last_mouse_y = y;
                            if self.dragging {
                                let rot = Matrix2::from_angle(Rad(-self.angle));
                                let vec = Vector2::new(delta_x as f32, delta_y as f32);
                                self.shift += rot * vec / self.zoom * 2.0;
                                //println!("shift ({} {})", shift[0], shift[1]);
                            }
                        }
                        _ => (),
                    }
                }
                _ => (),
            }
        }

        // UPDATE
        let delta_s = self.delta.tick();

        //TODO app.update(delta_s);
        //TODO camera.rotate(angle);
        //self.angle += delta_s * 0.1;

        // RENDER
        //TODO app.render(...)
        //FIXME recalc matrices only if something changed (w,h,angle,zoom)
        //TODO let transform = transform(w,h,camera)
        let translate = Matrix3::new(
            1.0,           0.0,           0.0,
            0.0,           1.0,           0.0,
            self.shift[0], self.shift[1], 1.0,
            );
        let rotate = Matrix3::from_angle_z(Rad(self.angle));
        let scale = Matrix3::from_diagonal(Vector3::new(self.zoom / self.w as f32, self.zoom / self.h as f32, 1.0));
        let transform = (scale * rotate * translate).into();

        if self.show_tiles {
            for t in self.grids_tiles.iter_mut() {
                t.data.transform = transform;
            }
        }
        if self.show_owning {
            for t in self.grids_owning.iter_mut() {
                t.data.transform = transform;
            }
        }
        if self.show_heights {
            for t in self.grids_heights.iter_mut() {
                t.data.transform = transform;
                t.data.threshold = self.threshold as i32;
            }
        }

        self.encoder.clear(&self.main_color, BLACK);

        if self.show_tiles {
            for t in self.grids_tiles.iter() {
                self.encoder.draw(&t.slice, &self.pso_tex, &t.data);
            }
        }

        if self.show_owning {
            for t in self.grids_owning.iter() {
                self.encoder.draw(&t.slice, &self.pso_tex, &t.data);
            }
        }

        if self.show_heights {
            for t in self.grids_heights.iter() {
                self.encoder.draw(&t.slice, &self.pso_col, &t.data);
            }
        }

        {
            let mut obj = ObjCol::from_objects(&self.objects, self.hero_x, self.hero_y).bake(self.main_color.clone(), &mut self.factory, self.threshold);
            obj.data.transform = transform;
            obj.data.threshold = 0;
            self.encoder.draw(&obj.slice, &self.pso_col, &obj.data);
        }

        let size_points = self.window.get_inner_size_points().unwrap();
        let size_pixels = self.window.get_inner_size_pixels().unwrap();
        let ui = self.imgui.frame(size_points, size_pixels, delta_s);

        let fps = (1.0 / delta_s) as usize;
        if !run_ui(&ui, fps, self.threshold, &mut self.show_tiles, &mut self.show_heights, &mut self.show_owning, &mut self.v1, &self.objects, &self.resources) {
            return false;
        }
        self.imgui_renderer.render(ui, &mut self.factory, &mut self.encoder).expect("IMGUI Rendering failed");

        self.encoder.flush(&mut self.device);
        self.window.swap_buffers().expect("window.swap_buffers() failed");
        self.device.cleanup();

        true
    }

    pub fn end (self) {
    }
}
