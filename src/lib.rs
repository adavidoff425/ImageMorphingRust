#[allow(unused_imports)]
#[macro_use]
extern crate cgmath;
extern crate conv;
extern crate glium;
extern crate image;
extern crate imageproc;
extern crate num_traits;

use image::{ImageBuffer, Pixel, RgbaImage};

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f64; 2],
}

glium::implement_vertex!(Vertex, position);

pub struct Morph<'a>
{
    pub src: &'a RgbaImage,
    pub dst: &'a RgbaImage,
    pub src_lines: &'a Vec<Vec<Vertex>>,
    pub dst_lines: &'a Vec<Vec<Vertex>>,
    pub t: f64,
    pub p: f64,
    pub a: f64,
    pub b: f64,
}

impl<'a> Morph<'a> {
    pub fn new(
        src: &'a RgbaImage,
        dst: &'a RgbaImage,
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

    pub fn interpolate_lines(&self) -> Vec<Vec<Vertex>> {
        let mut inter_lines: Vec<Vec<Vertex>> = Vec::new();
        for i in 0..self.src_lines.len() {
            let mut new_inter_line: Vec<Vertex> = Vec::new();
            new_inter_line.push(Vertex {
                position: [
                    (1.0f64 - self.t) * self.src_lines[i][0].position[0]
                        + self.t * self.dst_lines[i][0].position[0],
                    (1.0f64 - self.t) * self.src_lines[i][0].position[1]
                        + self.t * self.dst_lines[i][0].position[1],
                ],
            });
            new_inter_line.push(Vertex {
                position: [
                    (1.0f64 - self.t) * self.src_lines[i][1].position[0]
                        + self.t * self.dst_lines[i][1].position[0],
                    (1.0f64 - self.t) * self.src_lines[i][1].position[1]
                        + self.t * self.dst_lines[i][1].position[1],
                ],
            });
            inter_lines.push(new_inter_line);
        }
        inter_lines
    }

    pub fn warp(
        &self,
        x: f64,
        y: f64,
        lines: &Vec<Vec<Vertex>>,
        src_lines: Vec<Vec<Vertex>>,
    ) -> (f64, f64) {
        let mut pd: Vec<f64> = Vec::new();
        let mut pq: Vec<f64> = Vec::new();
        let mut qd: Vec<f64> = Vec::new();
        let mut sum_x: f64 = 0.0;
        let mut sum_y: f64 = 0.0;
        let mut weight_sum: f64 = 0.0;

        for i in 0..self.src_lines.len() {
            pd.push(x - lines[i][0].position[0]);
            pd.push(y - lines[i][0].position[1]);
            pq.push(lines[i][1].position[0] - lines[i][0].position[0]);
            pq.push(lines[i][1].position[1] - lines[i][0].position[1]);
            let inter_len = pq[0] * pq[0] + pq[1] * pq[1];
            let u = (pd[0] * pq[0] + pd[1] * pq[1]) / inter_len;
            let inter_len = inter_len.sqrt();
            let v = (pd[0] * pq[1] - pd[1] * pq[0]) / inter_len;
            pq[0] = src_lines[i][1].position[0] - src_lines[i][0].position[0];
            pq[1] = src_lines[i][1].position[1] - src_lines[i][0].position[1];
            let src_len = (pq[0] * pq[0] + pq[1] * pq[1]).sqrt();
            let xx = src_lines[i][0].position[0] + u * pq[0] + v * pq[1] / src_len;
            let yy = src_lines[i][0].position[1] + u * pq[1] - v * pq[0] / src_len;
            let dist = if u < 0.0 {
                (pd[0] * pd[0] + pd[1] * pd[1]).sqrt()
            } else if u > 1.0 {
                qd.push(x - lines[i][1].position[0]);
                qd.push(y - lines[i][1].position[1]);
                (qd[0] * qd[0] + qd[1] * qd[1]).sqrt()
            } else {
                v.abs()
            };

            let weight = (inter_len.powf(self.p) / (self.a + dist)).powf(self.b);
            sum_x += xx * weight;
            sum_y += yy * weight;
            weight_sum += weight;
        }
        (sum_x/weight_sum, sum_y/weight_sum)
    }

    pub fn bilinear_interpolate(&self, img: &RgbaImage, x: f64, y: f64) -> (f64, f64, f64)
    {
        let (width, height) = img.dimensions();
        let i: f64 = 
          if x == 0.0 {
            1.0
          } else if x.ceil() == width as f64 {
            (width-1) as f64
          } else {
            x.ceil() 
          };

        let j: f64 = 
          if y == 0.0 {
            1.0
          } else if y.ceil() == height as f64 {
            (height-1) as f64
          } else {
            y.ceil() 
          };
        
        let alpha = i - x;
        let beta = j - y;
        let pix00 = img.get_pixel(i as u32 - 1, j as u32 - 1).to_rgba();
        let pix01 = img.get_pixel(i as u32, j as u32 - 1).to_rgba();
        let pix10 = img.get_pixel(i as u32 - 1, j as u32).to_rgba();
        let pix11 = img.get_pixel(i as u32, j as u32).to_rgba();

        let rgb0 = alpha * beta * pix00.0[0] as f64
            + (1.0f64 - alpha) * beta * pix01.0[0] as f64
            + alpha * (1.0f64 - beta) * pix10.0[0] as f64
            + (1.0f64 - alpha) * (1.0f64 - beta) * pix11.0[0] as f64;
        let rgb1 = alpha * beta * pix00.0[1] as f64
            + (1.0f64 - alpha) * beta * pix01.0[1] as f64
            + alpha * (1.0f64 - beta) * pix10.0[1] as f64
            + (1.0f64 - alpha) * (1.0f64 - beta) * pix11.0[1] as f64;
        let rgb2 = alpha * beta * pix00.0[2] as f64
            + (1.0f64 - alpha) * beta * pix01.0[2] as f64
            + alpha * (1.0f64 - beta) * pix10.0[2] as f64
            + (1.0f64 - alpha) * (1.0f64 - beta) * pix11.0[2] as f64;
      //  P::from_channels(
        (rgb0, rgb1, rgb2)
        /*    NumCast::from(rgb0).unwrap(),
            NumCast::from(rgb1).unwrap(),
            NumCast::from(rgb2).unwrap(),
            NumCast::from(255.0).unwrap(),
        )*/
    }

    pub fn interpolate_color(&self, src_pt: Vec<f64>, dst_pt: Vec<f64>) -> (f64, f64, f64)
    {
        let (src_r, src_g, src_b) = self
            .bilinear_interpolate(self.src, src_pt[0], src_pt[1]);
           // .to_rgba();
        let (dst_r, dst_g, dst_b) = self
            .bilinear_interpolate(self.dst, dst_pt[0], dst_pt[1]);
          /*  .to_rgba();*/
        let rgb0 = src_r * (1.0f64 - self.t) + dst_r * self.t; 
        let rgb1 = src_g * (1.0f64 - self.t) + dst_g * self.t;
        let rgb2 = src_b * (1.0f64 - self.t) + dst_b * self.t;
        (rgb0, rgb1, rgb2)
        /*Pixel::from_channels(
            NumCast::from(rgb0).unwrap(),
            NumCast::from(rgb1).unwrap(),
            NumCast::from(rgb2).unwrap(),
            NumCast::from(255.0).unwrap(),
        )*/
    }

    pub fn morph(&self) -> RgbaImage
    {
        let (src_h, src_w) = self.src.dimensions();
        let (dst_h, dst_w) = self.dst.dimensions();
        let width = 
          if src_w >= dst_w {
            (src_w as f32 * (src_w as f32 / dst_w as f32)) as u32
          } else {
            (dst_w as f32 * (dst_w as f32 / src_w as f32)) as u32
          };
        let height = 
          if src_h >= dst_h {
            (src_h as f32 * (src_h as f32 / dst_h as f32)) as u32
          } else {
            (dst_h as f32 * (dst_h as f32 / src_h as f32)) as u32
          };

        let morphed_img: RgbaImage = ImageBuffer::new(width, height);
        let mut src_map: RgbaImage = ImageBuffer::new(src_w, src_h);
        let mut dst_map: RgbaImage = ImageBuffer::new(dst_w, dst_h);
        let inter_lines = self.interpolate_lines();

        println!("src_h: {}, src_w: {}", src_h, src_w);
        println!("dst_h: {}, dst_w: {}", dst_h, dst_w);

        for y in 0..src_h-1 {
            for x in 0..src_w-1 {
                let (src_x, src_y) = self.warp(x as f64, y as f64, &inter_lines, self.src_lines.to_vec());
                let src_x = if src_x < 0.0 {
                    0.0
                } else if src_x > (src_w - 1).into() {
                    (src_w - 1) as f64
                } else {
                    src_x
                };
                let src_y = if src_y < 0.0 {
                    0.0
                } else if src_y > (src_h - 1).into() {
                    (src_h - 1) as f64
                } else {
                    src_y
                };
                let src_x = src_x as f32 * (src_w as f32 / 1024.0);
                let src_y = src_y as f32 * (src_h as f32 / 768.0);
                if src_x as u32 > src_w - 1 || src_y as u32 > src_h - 1 {
                  continue;
                }
                src_map.put_pixel(x, y, *self.src.get_pixel(src_x as u32, src_y as u32));

                //let src_pt: Vec<f64> = vec![src_x, src_y];
                //let dst_pt: Vec<f64> = vec![dst_x, dst_y];

                //let (r, g, b) = self.interpolate_color(src_pt, dst_pt);
                //let color = Pixel::from_channels(r as u8, g as u8, b as u8, 127);
                //morphed_img.put_pixel(x, y, color);
            }
        }

        for y in 0..dst_h-1 {
          for x in 0..dst_w-1 {
            let (dst_x, dst_y) = self.warp(x as f64, y as f64, &inter_lines, self.dst_lines.to_vec());
            let dst_x = if dst_x < 0.0 {
                0.0
            } else if dst_x > (dst_w - 1).into() {
                (dst_w - 1) as f64
            } else {
                dst_x
            };
            let dst_y = if dst_y < 0.0 {
                0.0
            } else if dst_y > (dst_h - 1).into() {
                (dst_h - 1) as f64
            } else {
                dst_y
            };
            let dst_x = dst_x as f32 * (dst_w as f32 / 1024.0);
            let dst_y = dst_y as f32 * (dst_h as f32 / 768.0);
            if dst_x > (dst_w - 1) as f32 || dst_y > (dst_h - 1) as f32 {
              continue;
            }
            dst_map.put_pixel(x, y, *self.dst.get_pixel(dst_x as u32, dst_y as u32));
          }
        }

        src_map.save("src_warp.png").unwrap();
        dst_map.save("dst_warp.png").unwrap();
        morphed_img
    }
}
