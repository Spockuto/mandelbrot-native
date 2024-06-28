use image::ImageBuffer;
use piston_window::{
    EventLoop, EventSettings, Events, PistonWindow, Texture, TextureContext, TextureSettings,
    WindowSettings,
};
use std::f64::consts::LN_2;

mod constants;
use constants::{CENTER, HEIGHT, ITERATIONS, PALETTE, WIDTH, ZOOM_FACTOR};

fn scale_x(x: u32, width: u32, min_re: f64, max_re: f64, zoom: f64, center_re: f64) -> f64 {
    center_re + (x as f64 - width as f64 / 2.0) * (max_re - min_re) / (width as f64 * zoom)
}

fn scale_y(y: u32, height: u32, min_im: f64, max_im: f64, zoom: f64, center_im: f64) -> f64 {
    center_im - (y as f64 - height as f64 / 2.0) * (max_im - min_im) / (height as f64 * zoom)
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

    ImageBuffer::from_par_fn(width, height, |x, y| {
        let x0 = scale_x(x, width, -2.5, 1.0, zoom, center_x);
        let y0 = scale_y(y, height, -1.0, 1.0, zoom, center_y);
        let (mut z_x, mut z_y, mut x2, mut y2) = (0.0, 0.0, 0.0, 0.0);
        let mut color = (iterations % palette_size) as f64;

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

        let i1 = color.floor() as usize;
        let i2 = (i1 + 1) % palette_size as usize;
        let (r, g, b) = interpolate_color(PALETTE[i1], PALETTE[i2], color - i1 as f64);

        image::Rgba([r, g, b, 128])
    })
}

fn main() {
    let mut zoom = 1.0;
    let center = 3;
    let (center_x, center_y) = CENTER[center];

    let mut window: PistonWindow = WindowSettings::new("mandlebrot set", [WIDTH, HEIGHT])
        .build()
        .unwrap();

    let mut texture_context = TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into(),
    };

    let mut events = Events::new(EventSettings::new().lazy(false));
    let settings = TextureSettings::new();
    while let Some(e) = events.next(&mut window) {
        window.draw_2d(&e, |c, g, device| {
            zoom += ZOOM_FACTOR;
            let data = mandelbrot_zoom_frame(WIDTH, HEIGHT, ITERATIONS, zoom, center_x, center_y);

            let texture = Texture::from_image(&mut texture_context, &data, &settings).unwrap();
            piston_window::image(&texture, c.transform, g);
            texture_context.encoder.flush(device);
        });
    }
}
