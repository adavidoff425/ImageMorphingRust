extern crate glium;
extern crate image;
extern crate imageproc;
extern crate winit;

use glium::Surface;
use glium::glutin::{dpi, event, event_loop, window, ContextBuilder};
use std::time::{Duration, Instant};

fn main() {
  // Boilerplate code for initilizing glium display window
  // Taken from docs.rs/glium/0.26.0/glium
  let mut events_loop = event_loop::EventLoop::new();
  let wb = window::WindowBuilder::new()
    .with_inner_size(dpi::LogicalSize::new(1024.0, 768.0))
    .with_title("Image Morphing Tool");
  let cb = ContextBuilder::new();
  let display = glium::Display::new(wb, cb, &events_loop).unwrap();

  events_loop.run(move |event, _, control_flow| {
    let next_frame_time = Instant::now() + Duration::from_nanos(16_666_667);
    *control_flow = event_loop::ControlFlow::WaitUntil(next_frame_time);

    match event {
      event::Event::WindowEvent { event, .. } => match event {
          event::WindowEvent::CloseRequested => {
            *control_flow = event_loop::ControlFlow::Exit;
            return;
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

    let mut target = display.draw();
    target.clear_color(0.0, 0.0, 1.0, 1.0);
    target.finish().unwrap();
  });
}
