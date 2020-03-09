#[allow(unused_imports)]
#[macro_use]
extern crate cgmath;
extern crate conv;
extern crate glium;
extern crate image;
extern crate imageproc;
extern crate num_traits;

use image::{GenericImage, GenericImageView, Pixel, Primitive};
use num_traits::NumCast;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f64; 2],
}

glium::implement_vertex!(Vertex, position);

pub struct Morph<'a, T>
where
    T: GenericImageView,
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

impl<'a, T: GenericImageView> Morph<'a, T> {
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
      for i in 0..self.src_lines.len()-1 {
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

    pub fn warp<I, J>(
        self,
        pt: Vec<f64>,
        lines: Vec<Vec<Vertex>>,
        src_lines: Vec<Vec<Vertex>>,
    ) -> (f64, f64)
    where
        I: GenericImage,
        J: GenericImageView<Pixel = I::Pixel>,
    {
        let mut pd: Vec<f64> = Vec::new();
        let mut pq: Vec<f64> = Vec::new();
        let mut qd: Vec<f64> = Vec::new();
        let mut sum_x: f64 = 0.0;
        let mut sum_y: f64 = 0.0;
        let mut weight_sum: f64 = 0.0;

        for i in 0..self.src_lines.len() {
            pd.push(pt[0] - lines[i][0].position[0]);
            pd.push(pt[1] - lines[i][0].position[1]);
            pq.push(lines[i][1].position[0] - lines[i][0].position[0]);
            pq.push(lines[i][1].position[1] - lines[i][0].position[1]);
            let inter_len: f64 = pq[0] * pq[0] + pq[1] * pq[1];
            let u = (pd[0] * pq[0] + pd[1] * pq[1]) / inter_len;
            let inter_len = inter_len.sqrt();
            let v = (pd[0] * pq[1] - pd[1] * pq[0]) / inter_len;
            pq[0] = src_lines[i][1].position[0] - src_lines[i][0].position[0];
            pq[1] = src_lines[i][1].position[1] - src_lines[i][0].position[1];
            let src_len = (pq[0] * pq[0] + pq[1] * pq[1]).sqrt();
            let x = src_lines[i][0].position[0] + u * pq[0] + v * pq[1] / src_len;
            let y = src_lines[i][0].position[1] + u * pq[1] - v * pq[0] / src_len;
            let dist: f64 = if u < 0.0 {
                (pd[0] * pd[0] + pd[1] * pd[1]).sqrt()
            } else if u > 1.0 {
                qd.push(pt[0] - lines[i][1].position[0]);
                qd.push(pt[1] - lines[i][1].position[1]);
                (qd[0] * qd[0] + qd[1] * qd[1]).sqrt()
            } else {
                v.abs()
            };

            let weight: f64 = (inter_len.powf(self.p) / (self.a + dist)).powf(self.b);
            sum_x += x * weight;
            sum_y += y * weight;
            weight_sum += weight;
        }
        /*  src[0] = sum_x / weight_sum;
        src[1] = sum_y / weight_sum;*/
        (sum_x / weight_sum, sum_y / weight_sum)
    }

    pub fn bilinear_interpolate<I, P, S>(&self, img: &I, x: f64, y: f64) -> P 
    where
        I: GenericImageView<Pixel = P>,
        P: Pixel<Subpixel = S> + 'a,
        S: Primitive + 'a,
    {
        let i = x.ceil() as u32;
        let j = y.ceil() as u32;
        let alpha: f64 = i as f64 - x;
        let beta: f64 = j as f64 - y;
        let pix00 = img.get_pixel(i - 1, j - 1).to_rgba();
        let pix01 = img.get_pixel(i, j - 1).to_rgba();
        let pix10 = img.get_pixel(i - 1, j).to_rgba();
        let pix11 = img.get_pixel(i, j).to_rgba();

        let rgb0 = alpha * beta * pix00.0[0].to_f64().unwrap()
            + (1.0f64 - alpha) * beta * pix01.0[0].to_f64().unwrap()
            + alpha * (1.0f64 - beta) * pix10.0[0].to_f64().unwrap()
            + (1.0f64 - alpha) * (1.0f64 - beta) * pix11.0[0].to_f64().unwrap(); 
        let rgb1 = alpha * beta * pix00.0[1].to_f64().unwrap()
            + (1.0f64 - alpha) * beta * pix01.0[1].to_f64().unwrap()
            + alpha * (1.0f64 - beta) * pix10.0[1].to_f64().unwrap()
            + (1.0f64 - alpha) * (1.0f64 - beta) * pix11.0[1].to_f64().unwrap(); 
        let rgb2 = alpha * beta * pix00.0[2].to_f64().unwrap()
            + (1.0f64 - alpha) * beta * pix01.0[2].to_f64().unwrap()
            + alpha * (1.0f64 - beta) * pix10.0[2].to_f64().unwrap()
            + (1.0f64 - alpha) * (1.0f64 - beta) * pix11.0[2].to_f64().unwrap(); 
        P::from_channels(
            NumCast::from(rgb0).unwrap(),
            NumCast::from(rgb1).unwrap(),
            NumCast::from(rgb2).unwrap(),
            NumCast::from(255.0).unwrap(),
        )
    }

    pub fn interpolate_color<I, P, S>(
        &self,
        src_pt: Vec<f64>,
        dst_pt: Vec<f64>,
    ) -> P
    where
        I: GenericImageView<Pixel = P>,
        P: Pixel<Subpixel = S> + 'static,
        S: Primitive + 'static,
    {
        let src_color = self
            .bilinear_interpolate(self.src, src_pt[0], src_pt[1])
            .to_rgba();
        let dst_color = self
            .bilinear_interpolate(self.dst, dst_pt[0], dst_pt[1])
            .to_rgba();
        let src_r: f64 = NumCast::from(src_color.0[0]).unwrap();
        let src_g: f64 = NumCast::from(src_color.0[1]).unwrap();
        let src_b: f64 = NumCast::from(src_color.0[2]).unwrap();
        let dst_r: f64 = NumCast::from(dst_color.0[0]).unwrap();
        let dst_g: f64 = NumCast::from(dst_color.0[1]).unwrap();
        let dst_b: f64 = NumCast::from(dst_color.0[2]).unwrap();
        let rgb0 = src_r * (1.0f64 - self.t) + dst_r * self.t; 
        let rgb1 = src_g * (1.0f64 - self.t) + dst_g * self.t;
        let rgb2 = src_b * (1.0f64 - self.t) + dst_b * self.t;
        P::from_channels(
            NumCast::from(rgb0).unwrap(),
            NumCast::from(rgb1).unwrap(),
            NumCast::from(rgb2).unwrap(),
            NumCast::from(255.0).unwrap(),
        )
    }

    /* pub fn morph<I, J>(&self) -> J
    where
       I: GenericImage,
       J: GenericImageView<Pixel = I::Pixel>
    {

    }*/
}
