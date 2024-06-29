use image::ImageBuffer;
use piston_window::{
    EventLoop, EventSettings, Events, OpenGL, PistonWindow, Texture, TextureContext,
    TextureSettings, WindowSettings,
};
use rug::{Complex, Float};
use std::{f64::consts::LN_2, vec};

mod constants;
use constants::{CENTER, HEIGHT, ITERATIONS, PALETTE, WIDTH, ZOOM_FACTOR};

fn scale_x(x: u32, width: u32, min_re: f64, max_re: f64, zoom: f64, center_re: f64) -> f64 {
    (center_re
        + (x as f64 - width as f64 / 2.0)
            * Float::with_val(128, (max_re - min_re) / (width as f64 * zoom)))
    .to_f64()
}

fn scale_y(y: u32, height: u32, min_im: f64, max_im: f64, zoom: f64, center_im: f64) -> f64 {
    center_im
        - (y as f64 - height as f64 / 2.0)
            * Float::with_val(128, (max_im - min_im) / (height as f64 * zoom)).to_f64()
}

fn interpolate_color(color1: (u8, u8, u8), color2: (u8, u8, u8), t: f64) -> (u8, u8, u8) {
    let r = (color1.0 as f64 * (1.0 - t) + color2.0 as f64 * t) as u8;
    let g = (color1.1 as f64 * (1.0 - t) + color2.1 as f64 * t) as u8;
    let b = (color1.2 as f64 * (1.0 - t) + color2.2 as f64 * t) as u8;
    (r, g, b)
}

fn mandelbrot_zoom_frame(
    width: u32,
    height: u32,
    iterations: u32,
    zoom: f64,
    center_x: f64,
    center_y: f64,
) -> ImageBuffer<image::Rgba<u8>, Vec<u8>> {
    let palette_size = PALETTE.len() as u32;

    // refernce generation with just center
    let z = Complex::with_val(
        128,
        (
            Float::with_val(128, center_x),
            Float::with_val(128, center_y),
        ),
    );

    let mut reference_points: Vec<(Float, Float)> = vec![z.clone().into_real_imag()];
    let mut cz = z.clone();
    for _ in 0..iterations {
        cz = cz.clone().square() + z.clone();
        reference_points.push(cz.clone().into_real_imag());
    }

    ImageBuffer::from_par_fn(width, height, |x, y| {
        let x0 = scale_x(x, width, -2.5, 1.0, zoom, center_x);
        let y0 = scale_y(y, height, -1.0, 1.0, zoom, center_y);
        let mut color = (iterations % palette_size) as f64;

        if (x0 - center_x) * (x0 - center_x) + (y0 - center_y) * (y0 - center_y) > 1e-15_f64
            || zoom > 1e8_f64
        {
            let (mut z_x, mut z_y, mut x2, mut y2) = (0.0, 0.0, 0.0, 0.0);
            for i in 0..iterations {
                z_y = (z_x + z_x) * z_y + y0;
                z_x = x2 - y2 + x0;
                x2 = z_x * z_x;
                y2 = z_y * z_y;
                if x2 + y2 > (1 << 16) as f64 {
                    let log_zn = (x2 + y2).log((1 << 8) as f64) / 2.0;
                    let nu = (log_zn / LN_2).log((1 << 8) as f64) / LN_2;
                    color = (i as f64 + 1.0 - nu) % palette_size as f64;
                    break;
                }
            }
        } else {
            let d_z_x: f64 = x0 - center_x;
            let d_z_y: f64 = y0 - center_y;
            let mut e_x: f64 = 0.0;
            let mut e_y: f64 = 0.0;
            let mut d_cz_x;
            let mut d_cz_y;

            for (i, ((z_n_x, z_n_y), (z_n_1_x, z_n_1_y))) in reference_points
                .iter()
                .cloned()
                .zip(reference_points.iter().skip(1).cloned())
                .enumerate()
            {
                let t1_x = 2.0 * (z_n_x.clone() * e_x - z_n_y.clone() * e_y).to_f64();
                let t1_y = 2.0 * (z_n_x * e_y + z_n_y * e_x).to_f64();

                let t2_x = e_x * e_x - e_y * e_y;
                let t2_y = 2.0 * e_x * e_y;

                e_x = t1_x + t2_x + d_z_x;
                e_y = t1_y + t2_y + d_z_y;
                d_cz_x = z_n_1_x + e_x;
                d_cz_y = z_n_1_y + e_y;

                if d_cz_x.clone().square() + d_cz_y.clone().square() > (1 << 16) as f64 {
                    let log_zn =
                        ((d_cz_x.square() + d_cz_y.square()).to_f64()).log((1 << 8) as f64) / 2.0;
                    let nu = (log_zn / LN_2).log((1 << 8) as f64) / LN_2;
                    color = (i as f64 + 1.0 - nu) % palette_size as f64;
                    break;
                }
            }
        }

        let i1 = color.floor() as usize;
        let i2 = (i1 + 1) % palette_size as usize;
        let (r, g, b) = interpolate_color(PALETTE[i1], PALETTE[i2], color - i1 as f64);

        image::Rgba([r, g, b, 255])
    })
}

fn main() {
    let mut zoom = 1.0;
    let center = 2;
    let (center_x, center_y) = CENTER[center];

    let mut window: PistonWindow = WindowSettings::new("mandelbrot set", [WIDTH, HEIGHT])
        .graphics_api(OpenGL::V3_2)
        .build()
        .unwrap();

    let mut texture_context = TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into(),
    };

    let mut events = Events::new(EventSettings::new().lazy(false));
    let settings = TextureSettings::new();
    let mut texture = Texture::from_image(
        &mut texture_context,
        &image::ImageBuffer::new(WIDTH, HEIGHT),
        &settings,
    )
    .unwrap();
    while let Some(e) = events.next(&mut window) {
        window.draw_2d(&e, |c, g, device| {
            zoom += ZOOM_FACTOR;
            let data = mandelbrot_zoom_frame(WIDTH, HEIGHT, ITERATIONS, zoom, center_x, center_y);
            // As we zoom in, this gets faster but we lose precision due to f64
            texture.update(&mut texture_context, &data).unwrap();
            piston_window::image(&texture, c.transform, g);
            texture_context.encoder.flush(device);
        });
    }
}
