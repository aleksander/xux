use std::sync::mpsc::Sender;
//use std::sync::mpsc::SendError;
use std::thread;
use driver;
use ncurses::*;
use deque::{self, Stolen};

#[derive(Debug)]
pub enum Event {
    Grid(i32, i32, Vec<u8>, Vec<i16>, Vec<u8>),
    Obj(u32, (i32, i32), u16),
    ObjRemove(u32),
    Res(u16, String),
    Hero((i32, i32)),
    // NewObj(i32,i32),
    // UpdObj(...),
    // AI(...ai desigions...),
    // AI: going to pick obj (ID)
    // AI: going by path (PATH CHAIN)
    Input,
}

//TODO implement as Render trait implementations
pub enum RenderKind {
    No,
    Tui,
    TwoD,
    ThreeD,
}

pub struct Render {
    kind: RenderKind,
    worker: deque::Worker<Event>,
}

impl Drop for Render {
    fn drop(&mut self) {
        match self.kind {
            RenderKind::No => {
            }
            RenderKind::Tui => {
                endwin();
            }
            RenderKind::TwoD => {
            }
            RenderKind::ThreeD => {
            }
        }
    }
}

impl Render {
    pub fn new(kind: RenderKind, render_tx: Sender<driver::Event>) -> Render {

        let (worker, stealer) = deque::new();

        match kind {
            RenderKind::No => {
                thread::spawn(move || {
                    loop {
                        match stealer.steal() {
                            Stolen::Data(event) => {
                                match event {
                                    _ => {}
                                }
                            }
                            Stolen::Empty => {}
                            Stolen::Abort => {}
                        }
                    }
                });
            }
            RenderKind::Tui => {
                //XXX could alternatively use: termion, rustbox, rustty
                initscr();

                if let None = curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE) {
                    warn!("set cursor failed");
                };

                thread::spawn(move || {
                    let mut counter = 0;
                    let mut last_event = "NONE".to_owned();
                    loop {
                        clear();
                        mvprintw(0, 0, &format!("counter: {} ", counter));
                        mvprintw(1, 0, &last_event);
                        refresh();
                        match stealer.steal() {
                            Stolen::Data(value) => {
                                counter += 1;
                                match value {
                                    Event::Grid(x, y, _tiles, _z, _ol) => {
                                        last_event = format!("GRID: {} {}", x, y);
                                    }
                                    Event::Obj(id, (x, y), _resid) => {
                                        last_event = format!("OBJ: {} {} {}", id, x, y);
                                    }
                                    Event::ObjRemove(_id) => {}
                                    Event::Hero((x, y)) => {
                                        last_event = format!("HERO: {} {}", x, y);
                                    }
                                    Event::Input => {
                                        //last_event = format!("INPUT");
                                        return;
                                    }
                                    _ => {}
                                }
                            }
                            Stolen::Empty => {
                                //info!("render: disconnected");
                                //return;
                            }
                            Stolen::Abort => {}
                        }
                    }
                });

                /*
                let input_tx = caller_tx.clone();
                thread::spawn(move || {
                    loop {
                        getch();
                        match input_tx.send(Event::Input) {
                            Ok(()) => {}
                            Err(_) => break,
                        }
                    }
                });
                */
            }
            RenderKind::TwoD => {
                thread::spawn(move || {
                    use piston_window::*;
                    //use std::sync::mpsc::TryRecvError;
                    use std::collections::BTreeMap;

                    let mut window: PistonWindow =
                        WindowSettings::new("Render", [800, 600])
                            .vsync(true)
                            .build().unwrap();
                    let font = "/usr/share/fonts/TTF/DejaVuSansMono.ttf";
                    let factory = window.factory.clone();
                    let mut glyphs = Glyphs::new(font, factory).unwrap();
                    let mut origin = None;
                    let mut objects = BTreeMap::new();
                    let mut zoom = 1.0;
                    let mut dragging = false;
                    let mut hero = (0,0);
                    let mut grids = BTreeMap::new();
                    let mut delta_height = 0;
                    let mut command_line = false;
                    let mut command = ":: ".to_string();
                    let mut show_objtypes = false;
                    let mut resources = BTreeMap::new();
                    let mut highlighted_tiles = 0u8;
                    'outer: while let Some(e) = window.next() {
                        match e {
                            Input::Update(_) => {
                                loop {
                                    match stealer.steal() {
                                        Stolen::Data(event) => {
                                            //println!("RENDER: {:?}", event);
                                            match event {
                                                Event::Grid(x,y,tiles,heights,owning) => { grids.insert((x,y), (tiles,heights,owning)); }
                                                Event::Obj(id,xy,resid) => { objects.insert(id, (xy,resid)); }
                                                Event::ObjRemove(ref id) => { objects.remove(id); }
                                                Event::Res(id,name) => { resources.insert(id, name); }
                                                Event::Hero(xy) => {
                                                    if origin.is_none() { origin = Some(xy); }
                                                    hero = xy;
                                                }
                                                Event::Input => break
                                            }
                                        }
                                        Stolen::Empty => break,
                                        Stolen::Abort => {}
                                    }
                                }
                            }
                            Input::Render(render) => {
                                window.draw_2d(&e, |c, g| {
                                    clear([0.0; 4], g);
                                    if let Some((ox,oy)) = origin {

                                        let t = c.transform.trans(render.width as f64 / 2.0, render.height as f64 / 2.0).zoom(zoom);

                                        let (gx,gy) = ::state::grid((hero.0,hero.1));
                                        for &(gridx,gridy) in [(gx-1,gy-1),(gx,gy-1),(gx+1,gy-1),
                                                               (gx-1,gy  ),(gx,gy  ),(gx+1,gy  ),
                                                               (gx-1,gy+1),(gx,gy+1),(gx+1,gy+1)].iter() {
                                            if let Some(&(ref tiles, ref heights, ref owning)) = grids.get(&(gridx,gridy)) {
                                                for y in 0..100 {
                                                    for x in 0..100 {
                                                        let i = y*100+x;
                                                        let tile = tiles[i];
                                                        let owning = owning[i];
                                                        //TODO do filter of what tile types is shown
                                                        if tile == 66 || tile == highlighted_tiles || owning > 0 {
                                                            let color =
                                                                if tile == 66 {
                                                                    let c = tile as f32 / 256.0;
                                                                    [c,c,c,1.0]
                                                                } else if tile == highlighted_tiles {
                                                                    [0.0,0.3,0.0,0.5]
                                                                } else {
                                                                    [0.6,0.0,0.0,0.5]
                                                                };
                                                            rectangle(
                                                                color,
                                                                [(gridx*1100+(x*11) as i32-ox) as f64,
                                                                 (gridy*1100+(y*11) as i32-oy) as f64,
                                                                 11.0, 11.0],
                                                            t, g);
                                                        }
                                                    }
                                                }
                                                for y in 0..99 {
                                                    for x in 0..99 {
                                                        use shift_to_unsigned::ShiftToUnsigned;

                                                        let i = y*100+x;
                                                        let z = heights[i].shift_to_unsigned();
                                                        let zx = heights[i+1].shift_to_unsigned();
                                                        let zy = heights[i+100].shift_to_unsigned();
                                                        let dx = if z > zx { z - zx } else { zx - z };
                                                        let dy = if z > zy { z - zy } else { zy - z };
                                                        if dx > delta_height && dy > delta_height {
                                                            let lx = (gridx * 1100 + (x * 11) as i32 - ox) as f64;
                                                            let ly = (gridy * 1100 + (y * 11) as i32 - oy) as f64;
                                                            let lcolor = [0.3, 0.3, 0.3, 1.0];
                                                            let lsize = 1.0 / zoom;
                                                            if dx > delta_height {
                                                                line(lcolor, lsize, [lx, ly, lx + 11.0, ly], t, g);
                                                            }
                                                            if dy > delta_height {
                                                                line(lcolor, lsize, [lx, ly, lx, ly + 11.0], t, g);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        for &((x,y),resid) in objects.values() {
                                            let (cx,cy) = (x-ox,y-oy);
                                            rectangle(if resid == 2951 {[1.0, 0.0, 0.0, 1.0]} else {[1.0, 1.0, 1.0, 1.0]}, [(cx - 2) as f64, (cy - 2) as f64, 5.0, 5.0], t, g);
                                        }
                                        rectangle([0.2, 0.2, 1.0, 1.0], [(hero.0-ox-2) as f64, (hero.1-oy-2) as f64, 5.0, 5.0], t, g);
                                        rectangle([1.0, 1.0, 1.0, 1.0], [(hero.0-ox) as f64, (hero.1-oy) as f64, 1.0, 1.0], t, g);

                                        if show_objtypes {
                                            let mut objtypes = BTreeMap::new();
                                            for &(_,resid) in objects.values() {
                                                let mut obj = objtypes.entry(resid).or_insert(0);
                                                *obj += 1;
                                            }

                                            let mut i = 0;
                                            for (resid,count) in objtypes.iter() {
                                                let res = if let Some(name) = resources.get(resid) { name } else { "???" };
                                                text::Text::new_color([0.3, 1.0, 0.4, 1.0], 9).draw(
                                                    &format!("{:6} {:6} {}", count, resid, res),
                                                    &mut glyphs,
                                                    &c.draw_state,
                                                    c.transform.trans(200.0, 20.0 + i as f64), g);
                                                i += 9; //TODO += font.height
                                            }
                                        }

                                        if command_line {
                                            text::Text::new_color([0.3, 1.0, 0.4, 1.0], 12).draw(
                                                &command,
                                                &mut glyphs,
                                                &c.draw_state,
                                                //TODO draw at the bottom of the window
                                                c.transform.trans(10.0, 20.0), g);
                                        }
                                    }
                                });
                            }
                            Input::Press(Button::Mouse(MouseButton::Left)) => dragging = true,
                            Input::Release(Button::Mouse(MouseButton::Left)) => dragging = false,
                            Input::Move(Motion::MouseRelative(x,y)) => if dragging {
                                if let Some((ox,oy)) = origin {
                                    origin = Some((ox - (x / zoom) as i32, oy - (y / zoom) as i32));
                                }
                            },
                            Input::Move(Motion::MouseScroll(_,y)) => zoom *= if y > 0.0 { 1.05 } else { 0.95 },
                            Input::Press(Button::Keyboard(key)) => {
                                match key {
                                    Key::A => if command_line { command += "a"; },
                                    Key::B => if command_line { command += "b"; },
                                    Key::C => if command_line { command += "c"; },
                                    Key::D => if command_line { command += "d"; },
                                    Key::E => if command_line { command += "e"; },
                                    Key::F => if command_line { command += "f"; },
                                    Key::G => if command_line { command += "g"; },
                                    Key::H => if command_line { command += "h"; },
                                    Key::I => if command_line { command += "i"; },
                                    Key::J => if command_line { command += "j"; },
                                    Key::K => if command_line { command += "k"; },
                                    Key::L => if command_line { command += "l"; },
                                    Key::M => if command_line { command += "m"; },
                                    Key::N => if command_line { command += "n"; },
                                    Key::O => if command_line { command += "o"; },
                                    Key::P => if command_line { command += "p"; },
                                    Key::Q => if command_line { command += "q"; },
                                    Key::R => if command_line { command += "r"; },
                                    Key::S => if command_line { command += "s"; },
                                    Key::T => if command_line { command += "t"; } else { show_objtypes = ! show_objtypes; },
                                    Key::U => if command_line { command += "u"; },
                                    Key::V => if command_line { command += "v"; },
                                    Key::W => if command_line { command += "w"; },
                                    Key::X => if command_line { command += "x"; },
                                    Key::Y => if command_line { command += "y"; },
                                    Key::Z => if command_line { command += "z"; },
                                    Key::D1 => if command_line { command += "1"; },
                                    Key::D2 => if command_line { command += "2"; },
                                    Key::D3 => if command_line { command += "3"; },
                                    Key::D4 => if command_line { command += "4"; },
                                    Key::D5 => if command_line { command += "5"; },
                                    Key::D6 => if command_line { command += "6"; },
                                    Key::D7 => if command_line { command += "7"; },
                                    Key::D8 => if command_line { command += "8"; },
                                    Key::D9 => if command_line { command += "9"; } else { highlighted_tiles = highlighted_tiles.wrapping_sub(1); },
                                    Key::D0 => if command_line { command += "0"; } else { highlighted_tiles = highlighted_tiles.wrapping_add(1); },
                                    Key::Space => if command_line { command += " "; },
                                    Key::PageUp => delta_height += 1,
                                    Key::PageDown => if delta_height > 0 { delta_height -= 1; },
                                    Key::Up    => { render_tx.send(driver::Event::Render(driver::RenderEvent::Up)).expect("unable to send Render::Up"); }
                                    Key::Down  => { render_tx.send(driver::Event::Render(driver::RenderEvent::Down)).expect("unable to send Render::Down"); }
                                    Key::Left  => { render_tx.send(driver::Event::Render(driver::RenderEvent::Left)).expect("unable to send Render::Left"); }
                                    Key::Right => { render_tx.send(driver::Event::Render(driver::RenderEvent::Right)).expect("unable to send Render::Right"); }
                                    //Key::PageUp => zoom *= 1.2,
                                    //Key::PageDown => zoom *= 0.8,
                                    Key::Return => {
                                        if command_line {
                                            //TODO execute command
                                        }
                                        command_line = !command_line;
                                        command = ":: ".to_string();
                                    }
                                    Key::Escape => if command_line { command_line = false; } else { break; },
                                    _ => {}
                                }
                            }
                            Input::Close(_) => break,
                            _ => {}
                        }
                    }
                    render_tx.send(driver::Event::Render(driver::RenderEvent::Quit)).expect("unable to send Render::Quit");
                });
            }
            RenderKind::ThreeD => {
                // 3D UI
                /*
                         thread::spawn(move || {
                                 use ::glium::DisplayBuild;
                                 use ::glium::Surface;
                                 use ::glium::glutin::WindowBuilder;
                                 use ::glium::index::NoIndices;
                                 use ::glium::VertexBuffer;
                                 use ::glium::index::PrimitiveType;
                                 use ::glium::glutin;
                                 use ::glium::Program;
                                 //use ::glium::uniforms::EmptyUniforms;
                                 use ::glium::draw_parameters::DrawParameters;
                                 //use ::glium::texture::/*TODO Compressed*/Texture2d;
                                 //use ::glium::texture::Texture2dArray;
                                 use ::glium::DepthTest;
                                 use std::sync::mpsc::TryRecvError;
                                 use cgmath;
                                 //use cgmath::Matrix;
                                 use cgmath::FixedArray;
                                 use camera_controllers as camera;

                                 let display = WindowBuilder::new()
                                         .with_dimensions(1024, 1024)
                                         .with_title(format!("render"))
                                         .with_depth_buffer(24)
                                         .build_glium()
                                         .unwrap();

                                 #[derive(Copy, Clone)]
                                 struct Vertex {
                                     v_pos: [f32; 3],
                                     v_col: [f32; 3],
                                 }

                                 implement_vertex!(Vertex, v_pos, v_col);

                                 let mut vertex_buffer: VertexBuffer<Vertex> = VertexBuffer::empty(&display, 0).unwrap();
                                 let indices = NoIndices(PrimitiveType::TrianglesList);

                                 let vertex_shader_src = r#"
                                     #version 140
                                     in vec3 v_pos;
                                     in vec3 v_col;
                                     flat out vec3 vv_col;

                                     uniform mat4 u_model;
                                     uniform mat4 u_view;
                                     uniform mat4 u_proj;

                                     void main() {
                                         vv_col = v_col;
                                         gl_Position = u_proj * u_view * u_model * vec4(v_pos, 1.0);
                                     }
                                 "#;

                                 let fragment_shader_src = r#"
                                     #version 140
                                     flat in vec3 vv_col;
                                     out vec4 color;
                                     void main() {
                                         //float c = float(vv_col) / 255.0;
                                         color = vec4(vv_col, 1.0);
                                     }
                                 "#;

                                 //FIXME don't do init here. move it to Render struct new()
                                 let program = match Program::from_source(&display, vertex_shader_src, fragment_shader_src, None) {
                                     Ok(program) => program,
                                     Err(error) => {
                                         info!("compile program ERROR: {:?}", error);
                                         return;
                                     }
                                 };

                                 let mut landscape = Vec::new();

                                 {
                                     let col = [1.0; 3];
                                     landscape.extend(&[Vertex{v_pos: [-300.0, 0.0,-300.0], v_col: col},
                                                        Vertex{v_pos: [-300.0, 0.0, 300.0], v_col: col},
                                                        Vertex{v_pos: [ 300.0, 0.0, 300.0], v_col: col},
                                                        Vertex{v_pos: [-300.0, 0.0,-300.0], v_col: col},
                                                        Vertex{v_pos: [ 300.0, 0.0, 300.0], v_col: col},
                                                        Vertex{v_pos: [ 300.0, 0.0,-300.0], v_col: col}]);
                                 }

                                 let mut grids_count = 0;

                                 //let mut camera_x = 1.0;
                                 //let mut camera_y = 1.0;
                                 //let mut camera_z = 1.0;

                                 let mut dragging = false;
                                 let mut dragging_xy = None;
                                 let mut zooming = false;
                                 let mut zooming_xy = None;

                                 let mut camera = camera::OrbitZoomCamera::new([0.0, 0.0, 0.0], camera::OrbitZoomCameraSettings::default().pitch_speed(1.0).orbit_speed(0.004));
                                 camera.distance = 5.0;

                                 let model_scale = 0.001;

                                 /*'ecto_loop:*/ loop {
                                     {
                                         let mut target = display.draw();
                                         target.clear_color_and_depth((0.1, 0.1, 0.1, 1.0), 1.0);

                                         let draw_params = DrawParameters {
                                             depth_test: DepthTest::IfLess,
                                             depth_write: true,
                                             //polygon_mode: ::glium::draw_parameters::PolygonMode::Line,
                                             .. Default::default()
                                         };

                                         /*
                                         let view: cgmath::AffineMatrix3<f32> = cgmath::Transform::look_at(
                                             &cgmath::Point3::new(camera_x, camera_y, camera_z),
                                             &cgmath::Point3::new(0.0, 0.0, 0.0),
                                             &cgmath::Vector3::unit_z(),
                                         );
                                         */

                                         let uniforms = uniform! {
                                             //u_model: cgmath::Matrix4::identity().into_fixed(),
                                             u_model: cgmath::Matrix4::new(model_scale, 0.0, 0.0, 0.0,
                                                                           0.0, model_scale, 0.0, 0.0,
                                                                           0.0, 0.0, model_scale, 0.0,
                                                                           0.0, 0.0, 0.0, 1.0).into_fixed(),
                                             //u_view: view.mat.into_fixed(),
                                             u_view: camera.camera(0.0).orthogonal(),
                                             u_proj: cgmath::perspective(cgmath::deg(60.0f32), 1.0/*stream.get_aspect_ratio()*/, 0.1, 1000.0).into_fixed(),
                                         };

                                         if let Err(e) = target.draw(&vertex_buffer, &indices, &program, &uniforms/*EmptyUniforms*/, &draw_params) {
                                             info!("target.draw ERROR: {:?}", e);
                                             return;
                                         }
                                         if let Err(e) = target.finish() {
                                             info!("target.finish ERROR: {:?}", e);
                                             return;
                                         }
                                     }

                                     for ev in display.poll_events() {
                                         match ev {
                                             glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) |
                                             glutin::Event::Closed => {
                                                 /*break 'ecto_loop;*/
                                                 return;
                                             }
                                             glutin::Event::MouseInput(glutin::ElementState::Pressed, glutin::MouseButton::Left) => {
                                                 dragging = true;
                                             }
                                             glutin::Event::MouseInput(glutin::ElementState::Released, glutin::MouseButton::Left) => {
                                                 dragging = false;
                                                 dragging_xy = None;
                                             }
                                             glutin::Event::MouseInput(glutin::ElementState::Pressed, glutin::MouseButton::Right) => {
                                                 zooming = true;
                                             }
                                             glutin::Event::MouseInput(glutin::ElementState::Released, glutin::MouseButton::Right) => {
                                                 zooming = false;
                                                 zooming_xy = None;
                                             }
                                             glutin::Event::MouseMoved((x,y)) => {
                                                 if dragging {
                                                     dragging_xy = match dragging_xy {
                                                         None => Some((x,y)),
                                                         Some((mx,my)) => {
                                                             //camera_x += ((x - mx) as f32) / 1000.0;
                                                             //camera_z += ((y - my) as f32) / 1000.0;
                                                             //camera.control_camera((x - mx) as f32, (y - my) as f32);
                                                             /*FIXME*/ //camera.control_camera(-(x - mx) as f32, -(y - my) as f32);
                                                             Some((x,y))
                                                         }
                                                     }
                                                 }
                                                 if zooming {
                                                     zooming_xy = match zooming_xy {
                                                         None => Some((x,y)),
                                                         Some((_mx,_my)) => {
                                                             //let dy = y - my;
                                                             //let factor = 1.0 + (dy as f32) / 100.0;
                                                             //camera_x *= factor;
                                                             //camera_y *= factor;
                                                             //camera_z *= factor;
                                                             Some((x,y))
                                                         }
                                                     }
                                                 }
                                             }
                                             _ => ()
                                         }
                                     }

                                     //loop {
                                         match rx.try_recv() {
                                             Ok(value) => {
                                                 match value {
                                                     Event::Grid(gridx,gridy,tiles,z) => {
                                                         //info!("render: received Grid ({},{})", gridx, gridy);
                                                         if grids_count == 0 {
                                                             camera.target = [0.0, z[0] as f32 * model_scale, 0.0];
                                                         }

                                                         let mut vertices = Vec::with_capacity(10_000);
                                                         for y in 0..100 {
                                                             for x in 0..100 {
                                                                 let index = y*100+x;
                                                                 let vx = (gridx + x as i32 * 11) as f32;
                                                                 let vy = (gridy + y as i32 * 11) as f32;
                                                                 let vz = z[index] as f32 * 4.0;
                                                                 //vertices.push([vx,vy,vz]);
                                                                 vertices.push([vx,vz,vy]);
                                                             }
                                                         }
                                                         let mut shape = Vec::with_capacity(60_000);
                                                         for y in 0..99 {
                                                             for x in 0..99 {
                                                                 let index = y*100+x;
                                                                 let color = [tiles[index] as f32 / 255.0; 3];
                                                                 shape.push(Vertex{v_pos: vertices[index+100], v_col: color});
                                                                 shape.push(Vertex{v_pos: vertices[index], v_col: color});
                                                                 shape.push(Vertex{v_pos: vertices[index+1], v_col: color});

                                                                 shape.push(Vertex{v_pos: vertices[index+100], v_col: color});
                                                                 shape.push(Vertex{v_pos: vertices[index+1], v_col: color});
                                                                 shape.push(Vertex{v_pos: vertices[index+101], v_col: color});
                                                             }
                                                         }
                                                         landscape.extend(&shape);
                                                         info!("render: vertices {}, faces {}, quads {}", landscape.len(), landscape.len()/3, landscape.len()/6);
                                                         vertex_buffer = VertexBuffer::new(&display, &landscape).unwrap();
                                                         grids_count += 1;
                                                     }
                                                     Event::Obj(x,y) => {
                                                         //info!("render: received Obj ({},{})", x, y);
                                                         let x = x as f32;
                                                         let y = y as f32;

                                                         let mut vertices = Vec::with_capacity(4);
                                                         vertices.push([x-3.0, 0.01, y-3.0]);
                                                         vertices.push([x-3.0, 0.01, y+3.0]);
                                                         vertices.push([x+3.0, 0.01, y+3.0]);
                                                         vertices.push([x+3.0, 0.01, y-3.0]);

                                                         let col = [0.0, 1.0, 0.0];
                                                         let mut mesh = Vec::with_capacity(6);
                                                         mesh.push(Vertex{v_pos: vertices[0], v_col: col});
                                                         mesh.push(Vertex{v_pos: vertices[1], v_col: col});
                                                         mesh.push(Vertex{v_pos: vertices[2], v_col: col});

                                                         mesh.push(Vertex{v_pos: vertices[2], v_col: col});
                                                         mesh.push(Vertex{v_pos: vertices[3], v_col: col});
                                                         mesh.push(Vertex{v_pos: vertices[0], v_col: col});

                                                         landscape.extend(&mesh);
                                                         vertex_buffer = VertexBuffer::new(&display, &landscape).unwrap();
                                                     }
                                                 }
                                             }
                                             Err(e) => {
                                                 if let TryRecvError::Disconnected = e {
                                                     info!("render: disconnected");
                                                     //break/* 'ecto_loop*/;
                                                     return;
                                                 }// else {
                                                 //    break;
                                                 //}
                                             }
                                         }
                                     //}

                                 }
                         });
                */
            }
        }

        Render {
            kind: kind,
            worker: worker,
        }
    }

    pub fn update(&mut self, event: Event) {
        self.worker.push(event)
    }
}
