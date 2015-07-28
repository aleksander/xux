use std::sync::mpsc::channel;
//use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::sync::mpsc::SendError;
use std::thread;
//use state::State;

#[derive(Debug)]
pub enum Event {
    Grid(i32,i32,Vec<u8>,Vec<i16>),
}

pub struct Render {
    tx: Sender<Event>,
}

impl Render {
    pub fn new () -> Render {
        let (tx,rx) = channel();
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
                        .with_dimensions(512, 512)
                        .with_title(format!("render"))
                        .with_depth_buffer(24)
                        .build_glium()
                        .unwrap();

                #[derive(Copy, Clone)]
                struct Vertex {
                    v_pos: [f32; 3],
                    v_col: u8,
                }

                implement_vertex!(Vertex, v_pos, v_col);

                let mut vertex_buffer: VertexBuffer<Vertex> = VertexBuffer::empty(&display, 0).unwrap();
                let indices = NoIndices(PrimitiveType::TrianglesList);

                let vertex_shader_src = r#"
                    #version 140
                    in vec3 v_pos;
                    in uint v_col;
                    flat out uint vv_col;

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
                    flat in uint vv_col;
                    out vec4 color;
                    void main() {
                        float c = float(vv_col) / 255.0;
                        color = vec4(c, c, c, 0.0);
                    }
                "#;

                //FIXME don't do init here. move it to Render struct new()
                let program = match Program::from_source(&display, vertex_shader_src, fragment_shader_src, None) {
                    Ok(program) => program,
                    Err(error) => {
                        println!("compile program ERROR: {:?}", error);
                        return;
                    }
                };

                let mut landscape = Vec::new();

                landscape.extend(&[Vertex{v_pos: [-300.0, 0.0,-300.0], v_col: 255},
                                   Vertex{v_pos: [-300.0, 0.0, 300.0], v_col: 255},
                                   Vertex{v_pos: [ 300.0, 0.0, 300.0], v_col: 255},
                                   Vertex{v_pos: [-300.0, 0.0,-300.0], v_col: 255},
                                   Vertex{v_pos: [ 300.0, 0.0, 300.0], v_col: 255},
                                   Vertex{v_pos: [ 300.0, 0.0,-300.0], v_col: 255}]);

                //let mut grids_count = 0;

                //let mut camera_x = 1.0;
                //let mut camera_y = 1.0;
                //let mut camera_z = 1.0;

                let mut dragging = false;
                let mut dragging_xy = None;
                let mut zooming = false;
                let mut zooming_xy = None;

                let mut camera = camera::OrbitZoomCamera::new([0.0, 0.0, 0.0], camera::OrbitZoomCameraSettings::default().pitch_speed(1.0).orbit_speed(0.004));
                camera.distance = 2.0;

                let model_scale = 0.005;

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
                            println!("target.draw ERROR: {:?}", e);
                            return;
                        }
                        if let Err(e) = target.finish() {
                            println!("target.finish ERROR: {:?}", e);
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
                                            camera.control_camera(-(x - mx) as f32, -(y - my) as f32);
                                            Some((x,y))
                                        }
                                    }
                                }
                                if zooming {
                                    zooming_xy = match zooming_xy {
                                        None => Some((x,y)),
                                        Some((/*mx*/_,/*my*/_)) => {
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

                    match rx.try_recv() {
                        Ok(value) => {
                            match value {
                                Event::Grid(gridx,gridy,tiles,z) => {
                                    println!("render: received Grid ({},{})", gridx, gridy);
                                    if gridx == 0 && gridy == 0 {
                                        //camera.target = [0.25, 0.25, z[0] as f32 * model_scale];
                                        camera.target = [0.25, z[0] as f32 * model_scale, 0.25];
                                    }

                                    let mut vertices = Vec::with_capacity(10_000);
                                    for y in 0..100 {
                                        for x in 0..100 {
                                            let index = y*100+x;
                                            let vx = (gridx as f32) * 100.0 + (x as f32);
                                            let vy = (gridy as f32) * 100.0 + (y as f32);
                                            let vz = z[index] as f32;
                                            //vertices.push([vx,vy,vz]);
                                            vertices.push([vx,vz,vy]);
                                        }
                                    }
                                    let mut shape = Vec::with_capacity(60_000);
                                    for y in 0..99 {
                                        for x in 0..99 {
                                            let index = y*100+x;
                                            let color = tiles[index];
                                            shape.push(Vertex{v_pos: vertices[index+100], v_col: color});
                                            shape.push(Vertex{v_pos: vertices[index], v_col: color});
                                            shape.push(Vertex{v_pos: vertices[index+1], v_col: color});

                                            shape.push(Vertex{v_pos: vertices[index+100], v_col: color});
                                            shape.push(Vertex{v_pos: vertices[index+1], v_col: color});
                                            shape.push(Vertex{v_pos: vertices[index+101], v_col: color});
                                        }
                                    }
                                    landscape.extend(&shape);
                                    println!("render: vertices {}, faces {}, quads {}", landscape.len(), landscape.len()/3, landscape.len()/6);
                                    vertex_buffer = VertexBuffer::new(&display, &landscape).unwrap();
                                    //grids_count += 1;
                                }
                            }
                        }
                        Err(e) => {
                            if let TryRecvError::Disconnected = e {
                                println!("render: disconnected");
                                break/* 'ecto_loop*/;
                            }
                        }
                    }
                }
        });
        Render{tx:tx}
    }

    pub fn update (&mut self, event: Event) -> Result<(), SendError<Event>> {
        self.tx.send(event)
    }
}
