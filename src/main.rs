use colors_transform::Color;
use colors_transform::{Hsl, Rgb};
use image::ImageBuffer;
use num_complex::Complex;

fn mandelbrot_zoom_frame(
    width: u32,
    height: u32,
    iterations: u32,
    hue: f32,
    color: [u8; 4],
    zoom: f64,
    center_x: f64,
    center_y: f64,
) -> ImageBuffer<image::Rgba<u8>, Vec<u8>> {
    let scale_x = 5.0 / width as f64 / zoom;
    let scale_y = 5.0 / height as f64 / zoom;

    ImageBuffer::from_par_fn(width, height, |x, y| {
        let temp_x = (x as f64 * scale_x) - 2.5 / zoom + center_x;
        let temp_y = (y as f64 * scale_y) - 2.5 / zoom + center_y;

        let z = Complex::new(temp_x, temp_y);
        let mut cz = z;

        for i in 0..iterations {
            cz = cz * cz + z;

            if cz.norm_sqr() > 5.0 {
                let color_string = Hsl::from(
                    hue,
                    100_f32,
                    ((i as f32) / (iterations as f32) * 100.0) as f32,
                );
                return image::Rgba([
                    color_string.get_red().round() as u8,
                    color_string.get_green().round() as u8,
                    color_string.get_blue().round() as u8,
                    255,
                ]);
            }
        }

        return image::Rgba(color);
    })
}

fn main() {
    use piston_window::*;

    let height: u32 = 1080;
    let width: u32 = 1920;
    let color_string = "#EFEFEF";
    let iterations = 500;
    let hue = 30.0;
    let mut zoom = 1.0;
    let center = 6;

    let (center_x, center_y) = match center {
        1 => (-0.75, 0.1),
        2 => (0.339410819995598, -0.050668285162643),
        3 => (-0.10109636384562, 0.95628651080914),
        4 => (-0.77568377, 0.13646737),
        5 => ( 0.272149607027528, 0.005401159465460),
        6 => (-1.7492046334590113301, 0.00028684660234660531403),
        7 => (0.2925755, -0.0149977),
        8 => (-0.814158841137593, 0.189802029306573),
        9 => (-0.1182402951560276787014475129283, 0.64949165134945441813936036487738),
        _ => (1.0, 1.0),
    };

    let mut window: PistonWindow = WindowSettings::new("mandlebrot set", [width, height])
        .build()
        .unwrap();

    let mut texture_context = TextureContext {
        factory: window.factory.clone(),
        encoder: window.factory.create_command_buffer().into(),
    };

    let mut default: [u8; 4] = [0; 4];
    let default_color = Rgb::from_hex_str(color_string).unwrap();
    default[0] = default_color.get_red() as u8;
    default[1] = default_color.get_green() as u8;
    default[2] = default_color.get_blue() as u8;
    default[3] = 255;

    let mut events = Events::new(EventSettings::new().lazy(false));
    while let Some(e) = events.next(&mut window) {
        window.draw_2d(&e, |c, g, device| {
            zoom += 10000.0;
            let data = mandelbrot_zoom_frame(
                width, height, iterations, hue, default, zoom, center_x, center_y,
            );

            let texture =
                Texture::from_image(&mut texture_context, &data, &TextureSettings::new()).unwrap();
            image(&texture, c.transform, g);
            texture_context.encoder.flush(device);
        });
    }
}
