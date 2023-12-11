use rayon::prelude::*;
use std::io::Write;

const FRAME_DELAY: u32 = 5;
const WINDOW_NAME: &str = "Motion Extraction";

fn blend_image(
    image1: &image::ImageBuffer<image::Rgb<u8>, Vec<u8>>,
    image2: &image::ImageBuffer<image::Rgb<u8>, Vec<u8>>,
) -> image::ImageBuffer<image::Rgb<u8>, Vec<u8>> {
    let mut image = image1.clone();
    image
        .par_iter_mut()
        .zip(image2.par_iter())
        .for_each(|(pixel1, pixel2)| *pixel1 = ((*pixel1 as u16 + !*pixel2 as u16) / 2) as u8);
    image
}

#[show_image::main]
fn main() -> std::io::Result<()> {
    // first camera in system
    let index = nokhwa::utils::CameraIndex::Index(0);
    let requested = nokhwa::utils::RequestedFormat::new::<nokhwa::pixel_format::RgbFormat>(
        nokhwa::utils::RequestedFormatType::AbsoluteHighestFrameRate,
        // nokhwa::utils::RequestedFormatType::AbsoluteHighestResolution,
    );
    // make the camera
    let mut camera = nokhwa::Camera::new(index, requested).unwrap();
    let resolution = camera.resolution();
    // create window
    let window = show_image::create_window(WINDOW_NAME, Default::default()).unwrap();
    // create buffer of frames of size frame_delay
    let mut buffer: Vec<image::ImageBuffer<image::Rgb<u8>, Vec<u8>>> =
        Vec::with_capacity(FRAME_DELAY as usize);

    loop {
        let start = std::time::Instant::now();
        // get a frame
        let frame: nokhwa::Buffer = camera.frame().unwrap();
        // decode into an ImageBuffer
        let decoded = frame
            .decode_image::<nokhwa::pixel_format::RgbFormat>()
            .unwrap();

        // add the oldest image on top of the newest image at 50% opacity
        if buffer.len() > 1 {
            let image = blend_image(&buffer[0], &decoded);

            // save the image
            window
                .set_image(
                    WINDOW_NAME,
                    show_image::ImageView::new(
                        show_image::ImageInfo::rgb8(resolution.width(), resolution.height()),
                        &image,
                    ),
                )
                .unwrap();
        }

        // add frame to buffer
        if buffer.len() == FRAME_DELAY as usize {
            buffer.remove(0);
        }
        buffer.push(decoded);

        print!("FPS: {}\r", 1.0 / start.elapsed().as_secs_f64());
        std::io::stdout().flush().unwrap();
    }
}
