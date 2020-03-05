#[allow(unused_imports)]
#[macro_use]
extern crate cgmath;
extern crate glium;
extern crate image;
extern crate imageproc;

use image::{GenericImage, GenericImageView, ImageFormat};

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f64; 2],
}

glium::implement_vertex!(Vertex, position);

pub struct Morph<'a, T>
where
  T: GenericImage,  
{
    pub src: &'a T,
    pub dst: &'a T,
    pub src_lines: &'a Vec<Vec<Vertex>>,
    pub dst_lines: &'a Vec<Vec<Vertex>>,
    pub t: f64,
    pub p: f64,
    pub a: f64,
    pub b: f64,
}

impl<'a, T: GenericImage> Morph<'a, T> {
    pub fn new(
        src: &'a T,
        dst: &'a T,
        src_lines: &'a Vec<Vec<Vertex>>,
        dst_lines: &'a Vec<Vec<Vertex>>,
        t: f64,
        p: f64,
        a: f64,
        b: f64,
    ) -> Self {
        Morph {
            src,
            dst,
            src_lines,
            dst_lines,
            t,
            p,
            a,
            b,
        }
    }

    pub fn interpolate_lines(&self) -> Vec<Vec<Vertex>>
    {
      let mut inter_lines: Vec<Vec<Vertex>> = Vec::new();
      for i in 0..self.src_lines.len() {
        let mut new_inter_line: Vec<Vertex> = Vec::new();
        new_inter_line.push(Vertex {
          position: [(1.0f64-self.t)*self.src_lines[i][0].position[0] + self.t*self.dst_lines[i][0].position[0], (1.0f64-self.t)*self.src_lines[i][0].position[1] + self.t*self.dst_lines[i][0].position[1]]
        });
        new_inter_line.push(Vertex {
          position: [(1.0f64-self.t)*self.src_lines[i][1].position[0] + self.t*self.dst_lines[i][1].position[0], (1.0f64-self.t)*self.src_lines[i][1].position[1] + self.t*self.dst_lines[i][1].position[1]]
        });
        inter_lines.push(new_inter_line);
        }
      inter_lines
      }
/*
    pub fn warp<I, J>(
        mut self,
        pt: Vec<f64>,
        lines: Vec<Vec<Vertex>>,
        src_lines: Vec<Vec<Vertex>>,
        src_pt: Vec<f64>,
    ) -> J
    where
      I: GenericImage,
      J: GenericImageView<Pixel = I::Pixel>,
      {}

    pub fn bilinear_interpolate<I>(
        &self,
        img: I,
        (x, y): (f64, f64),
        pixel: &mut I::Pixel,
    ) -> &mut I::Pixel 
         where
            I: GenericImage,
            I::Pixel: 'static,
    {
    }

    pub fn interpolate_color<I>(
        &self,
        src_pt: Vec<f64>,
        dst_pt: Vec<f64>,
        pixel: &I::Pixel,
    ) -> &mut I::Pixel 
         where
            I: GenericImage,
            I::Pixel: 'static,
    { 
    }

    pub fn morph<I, J>(&self) -> J
    where
       I: GenericImage,
       J: GenericImageView<Pixel = I::Pixel>
    {
    }*/
  }
