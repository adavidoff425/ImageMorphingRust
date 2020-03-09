#[allow(unused_imports)]
#[macro_use]
extern crate glium;
extern crate cgmath;
extern crate image;
extern crate imageproc;
extern crate imagemorph;
extern crate winit;

use imagemorph::*;
use cgmath::{Matrix4, Vector2};
use glium::glutin::{dpi, event, event_loop, window, ContextBuilder};
use glium::{index, texture, DrawParameters, IndexBuffer, Surface, VertexBuffer};
use image::{DynamicImage, GenericImageView, ImageFormat};
use std::fs::File;
use std::io::Cursor;
use std::path::Path;
use std::time::{Duration, Instant};


const VERTEX_SHADER: &'static str = r#"
    #version 140

    in vec2 position;
    uniform mat4 projection;
    out vec2 v_tex_coords;

    void main() {
      if (gl_VertexID % 4 == 0) {
        v_tex_coords = vec2(0.0, 1.0);
      } else if (gl_VertexID % 4 == 1) {
        v_tex_coords = vec2(1.0, 1.0);
      } else if (gl_VertexID % 4 == 2) {
        v_tex_coords = vec2(0.0, 0.0);
      } else {
        v_tex_coords = vec2(1.0, 0.0);
      }
      gl_Position = projection * vec4(position, 0.0, 1.0);
    }
"#;

const FRAGMENT_SHADER: &'static str = r#"
    #version 140

    in vec2 v_tex_coords;
    out vec4 color;
    uniform sampler2D tex;

    void main() {
      color = texture(tex, v_tex_coords);
    }
"#;

fn main() {
    //  assert_eq!(env::args().count(), 3);
    let src = std::env::args().nth(1).unwrap();
    let dst = std::env::args().nth(2).unwrap();

    // Boilerplate code for initilizing glium display window
    // Adapted from tutorial at docs.rs/glium/0.26.0/glium
    let events_loop = event_loop::EventLoop::new();
    let wb = window::WindowBuilder::new()
        .with_inner_size(dpi::LogicalSize::new(1024.0, 768.0))
        .with_title("Image Morphing Tool");
    let cb = ContextBuilder::new();
    let display_src = glium::Display::new(wb, cb, &events_loop).unwrap();
    let mut src_img = image::open(&Path::new(&src)).unwrap().to_rgba();

    let src = {
        let src_img = image::open(&Path::new(&src)).unwrap().to_rgba();
        let src_dim = src_img.dimensions();
        let src_img = texture::RawImage2d::from_raw_rgba_reversed(&src_img.into_raw(), src_dim);
        texture::SrgbTexture2d::new(&display_src, src_img).unwrap()
    };


    let (vertices, indices) = {
        let data: Vec<u16> = vec![0, 1, 2, 1, 3, 2];
        let vertex_buf = VertexBuffer::empty_dynamic(&display_src, 4).unwrap();
        let index_buf =
            IndexBuffer::new(&display_src, index::PrimitiveType::TrianglesList, &data).unwrap();
        (vertex_buf, index_buf)
    };

    let program =
        glium::Program::from_source(&display_src, VERTEX_SHADER, FRAGMENT_SHADER, None).unwrap();

    let perspective = {
        let matrix: Matrix4<f32> = cgmath::ortho(0.0, 1024.0, 768.0, 0.0, -1.0, 1.0);
        Into::<[[f32; 4]; 4]>::into(matrix)
    };

    let size = Vector2 { x: 768.0, y: 576.0 };
    let mut position = Vector2 { x: 512.0, y: 384.0 };
    let mut x_pos: f64 = 0.0;
    let mut y_pos: f64 = 0.0;
    let mut is_src = 1;
    let mut line_seg_pt = 0;
    let mut src_lines: Vec<VertexBuffer<Vertex>> = Vec::new();
    let mut src_lines_ref: Vec<Vec<Vertex>> = Vec::new();
    let line_idx = index::NoIndices(index::PrimitiveType::LineStrip);
    let line_params = DrawParameters {
        line_width: Some(2.0),
        ..Default::default()
    };
    let mut new_line: Vec<Vertex> = Vec::new();

    events_loop.run(move |event, _, control_flow| {
        let next_frame_time = Instant::now() + Duration::from_nanos(16_666_667);
        *control_flow = event_loop::ControlFlow::WaitUntil(next_frame_time);
        if line_seg_pt == 2 {
            new_line.clear();
            line_seg_pt = 0;
        }

        match event {
            event::Event::WindowEvent { event, .. } => match event {
                event::WindowEvent::CloseRequested => {
                            let image: texture::RawImage2d<u8> =
                                display_src.read_front_buffer().unwrap();
                            src_img = image::ImageBuffer::from_raw(
                                image.width,
                                image.height,
                                image.data.into_owned(),
                            )
                            .unwrap();
                    if is_src == 1 {
                        is_src = 0;
                    } else {
//                        *control_flow = event_loop::ControlFlow::Exit;
                        return;
                    };
                }
                // Tracks position of cursor
                event::WindowEvent::CursorMoved {
                    position: PhysicalPosition,
                    ..
                } => {
                    x_pos = PhysicalPosition.x;
                    y_pos = PhysicalPosition.y;
                }
                event::WindowEvent::MouseInput { state, button, .. } => match (state, button) {
                    (Pressed, Left) => {
                        new_line.push(Vertex {
                            position: [x_pos, y_pos],
                        });
                        if line_seg_pt == 0 {
                            line_seg_pt = 1;
                        } else {
                            src_lines
                                .push(VertexBuffer::immutable(&display_src, &new_line).unwrap());
                            src_lines_ref.push(new_line.clone());
                            /*     println!(
                                "Start: ({}, {}), End: ({}, {})",
                                new_line[0].position[0],
                                new_line[0].position[1],
                                new_line[1].position[0],
                                new_line[1].position[1]
                            );*/
                            line_seg_pt = 2;
                        };
                    }
                    _ => return,
                },
                _ => return,
            },
            event::Event::NewEvents(cause) => match cause {
                event::StartCause::ResumeTimeReached { .. } => (),
                event::StartCause::Init => (),
                _ => return,
            },
            _ => return,
        }

        if is_src == 1 {
            let mut target = display_src.draw();
            target.clear_color(1.0, 1.0, 1.0, 1.0);

            {
                let left = position.x - size.x / 2.0;
                let right = position.x + size.x / 2.0;
                let bottom = position.y + size.y / 2.0;
                let top = position.y - size.y / 2.0;
                let mut vertex_buf = vec![
                    Vertex {
                        position: [left, top],
                    },
                    Vertex {
                        position: [right, top],
                    },
                    Vertex {
                        position: [left, bottom],
                    },
                    Vertex {
                        position: [right, bottom],
                    },
                ];

                vertices.write(&vertex_buf);
            }

            {
                let uniforms = uniform! {
                  projection: perspective,
                  tex: &src,
                };

                target
                    .draw(
                        &vertices,
                        &indices,
                        &program,
                        &uniforms,
                        &Default::default(),
                    )
                    .unwrap();

                for line in &src_lines[..] {
                    target
                        .draw(line, &indices, &program, &uniforms, &line_params)
                        .unwrap();
                }
            }
            target.finish().unwrap();
        } else {
            let events_loop_dst = event_loop::EventLoop::new();
            let wb = window::WindowBuilder::new()
                .with_inner_size(dpi::LogicalSize::new(1024.0, 768.0))
                .with_title("Image Morphing Tool");
            let cb = ContextBuilder::new();
            let display_dst = glium::Display::new(wb, cb, &events_loop_dst).unwrap();
            let dst = {
                let dst_img = image::open(&Path::new(&dst)).unwrap().to_rgba();
                let dst_dim = dst_img.dimensions();
                let dst_img =
                    texture::RawImage2d::from_raw_rgba_reversed(&dst_img.into_raw(), dst_dim);
                texture::SrgbTexture2d::new(&display_dst, dst_img).unwrap()
            };
            let mut new_line: Vec<Vertex> = Vec::new();
            let mut dst_lines: Vec<VertexBuffer<Vertex>> = Vec::new();
            let mut dst_lines_ref: Vec<Vec<Vertex>> = Vec::new();
            let src_img = src_img.clone();
            let src_lines_ref = src_lines_ref.clone();

            events_loop_dst.run(move |event, _, control_flow| {
                let next_frame_time = Instant::now() + Duration::from_nanos(16_666_667);
                *control_flow = event_loop::ControlFlow::WaitUntil(next_frame_time);
                if line_seg_pt == 2 {
                    new_line.clear();
                    line_seg_pt = 0;
                }

                match event {
                    event::Event::WindowEvent { event, .. } => match event {
                        event::WindowEvent::CloseRequested => {
                            let image: texture::RawImage2d<u8> =
                                display_dst.read_front_buffer().unwrap();
                            let mut image: image::RgbaImage = image::ImageBuffer::from_raw(
                                image.width,
                                image.height,
                                image.data.into_owned(),
                            )
                            .unwrap();
                            //let image = DynamicImage::ImageRgba8(image).flipv();
                            //    image.save("dst-lines.png").unwrap();
                            let morph = Morph::new(
                              &image, &src_img, &src_lines_ref, &dst_lines_ref, 0.0, 0.0, 0.0, 0.0);
                            for line in &src_lines_ref {
                              println!("{}", line);
                            }
                            morph.interpolate_lines();
                            *control_flow = event_loop::ControlFlow::Exit;
                            return;
                        }
                        event::WindowEvent::CursorMoved {
                            position: PhysicalPosition,
                            ..
                        } => {
                            x_pos = PhysicalPosition.x;
                            y_pos = PhysicalPosition.y;
                        }
                        event::WindowEvent::MouseInput { state, button, .. } => {
                            match (state, button) {
                                (Pressed, Left) => {
                                    //      println!("mouse pressed at ({}, {})", x_pos, y_pos);
                                    new_line.push(Vertex {
                                        position: [x_pos, y_pos],
                                    });
                                    if line_seg_pt == 0 {
                                        line_seg_pt = 1;
                                    } else {
                                        dst_lines.push(
                                            VertexBuffer::immutable(&display_dst, &new_line)
                                                .unwrap(),
                                        );
                                        dst_lines_ref.push(new_line.clone());
                                        /*         println!(
                                            "Start: ({}, {}), End: ({}, {})",
                                            new_line[0].position[0],
                                            new_line[0].position[1],
                                            new_line[1].position[0],
                                            new_line[1].position[1]
                                        );*/
                                        line_seg_pt = 2;
                                    };
                                }
                                _ => return,
                            }
                        }
                        _ => return,
                    },
                    event::Event::NewEvents(cause) => match cause {
                        event::StartCause::ResumeTimeReached { .. } => (),
                        event::StartCause::Init => (),
                        _ => return,
                    },
                    _ => return,
                }

                let (vertices, indices, line_params) = {
                    let data: Vec<u16> = vec![0, 1, 2, 1, 3, 2];
                    let vertex_buf = VertexBuffer::empty_dynamic(&display_dst, 4).unwrap();
                    let index_buf =
                        IndexBuffer::new(&display_dst, index::PrimitiveType::TrianglesList, &data)
                            .unwrap();
                    let line_params = DrawParameters {
                        line_width: Some(2.0),
                        ..Default::default()
                    };
                    (vertex_buf, index_buf, line_params)
                };

                let program =
                    glium::Program::from_source(&display_dst, VERTEX_SHADER, FRAGMENT_SHADER, None)
                        .unwrap();

                let mut target = display_dst.draw();
                target.clear_color(1.0, 1.0, 1.0, 1.0);

                {
                    let left = position.x - size.x / 2.0;
                    let right = position.x + size.x / 2.0;
                    let bottom = position.y + size.y / 2.0;
                    let top = position.y - size.y / 2.0;
                    let mut vertex_buf = vec![
                        Vertex {
                            position: [left, top],
                        },
                        Vertex {
                            position: [right, top],
                        },
                        Vertex {
                            position: [left, bottom],
                        },
                        Vertex {
                            position: [right, bottom],
                        },
                    ];

                    vertices.write(&vertex_buf);
                }

                {
                    let uniforms = uniform! {
                      projection: perspective,
                      tex: &dst,
                    };

                    target
                        .draw(
                            &vertices,
                            &indices,
                            &program,
                            &uniforms,
                            &Default::default(),
                        )
                        .unwrap();

                    for line in &dst_lines[..] {
                        target
                            .draw(line, &indices, &program, &uniforms, &line_params)
                            .unwrap();
                    }
                }
                target.finish().unwrap();
            })
        }
    });
}