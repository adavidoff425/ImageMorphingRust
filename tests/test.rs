extern crate imagemorph;
extern crate image;

use imagemorph::*;
use image::{ImageBuffer, RgbaImage};

#[test]
fn test_inter_lines() {
    let src_line = vec![Vertex { position: [0f64, 100f64], }, Vertex { position: [100f64, 100f64], }];
    let dst_line = vec![Vertex { position: [0f64, 50f64], }, Vertex { position: [100f64, 50f64], }];
    let src_lines = vec![src_line];
    let dst_lines = vec![dst_line];
    let src = ImageBuffer::new(200, 200);
    let dst = ImageBuffer::new(200, 200);
    let morph = Morph::new(
        &src,
        &dst,
        &src_lines,
        &dst_lines,
        0.5f64, 0.0f64, 0.0f64, 0.0f64
    );
    let inter_line_morph = morph.interpolate_lines();
    let inter_line_actual = vec![vec![Vertex { position: [0f64, 75f64], }, Vertex { position: [100f64, 75f64], }]];
    assert_eq!(inter_line_morph[0][0].position[0], inter_line_actual[0][0].position[0]);
    assert_eq!(inter_line_morph[0][0].position[1], inter_line_actual[0][0].position[1]);
    assert_eq!(inter_line_morph[0][1].position[0], inter_line_actual[0][1].position[0]);
    assert_eq!(inter_line_morph[0][1].position[1], inter_line_actual[0][1].position[1]);
}

#[test]
fn test_warp() {
    let src_line = vec![Vertex { position: [0f64, 100f64], }, Vertex { position: [100f64, 100f64], }];
    let dst_line = vec![Vertex { position: [0f64, 50f64], }, Vertex { position: [100f64, 50f64], }];
    let src_lines = vec![src_line];
    let dst_lines = vec![dst_line];
    let src = ImageBuffer::new(200, 200);
    let dst = ImageBuffer::new(200, 200);
    let morph = Morph::new(
        &src,
        &dst,
        &src_lines,
        &dst_lines,
        0.5f64, 0.0f64, 0.0f64, 0.0f64
    );
    let inter_line_morph = morph.interpolate_lines();
    let lines = vec![vec![Vertex { position: [inter_line_morph[0][0].position[0], inter_line_morph[0][0].position[1]], }, Vertex { position: [inter_line_morph[0][1].position[0], inter_line_morph[0][1].position[1]], }]];
    let x: f64 = 50.0;
    let y: f64 = 80.0;
    let new_pt = morph.warp(x, y, &lines, morph.src_lines.to_vec());
    println!("{}, {}", new_pt.0, new_pt.1);
}
