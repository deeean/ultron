#![deny(clippy::all)]

use napi::*;
use napi::tokio::{spawn};
use napi::bindgen_prelude::*;
use napi_derive::*;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

#[napi]
pub enum ColorType {
  RGBA,
  BGRA,
}

#[napi(object)]
pub struct Point {
  pub x: u32,
  pub y: u32,
}

#[napi]
#[derive(Clone)]
pub struct ImageData {
  pub data: Vec<u8>,
  pub width: u32,
  pub height: u32,
  pub color_type: ColorType,
  pub pixel_width: u8,
}

#[napi]
pub async fn save_image_data(image_data: &ImageData, path: String) -> Result<()> {
  let image_data = image_data.clone();
  let path = path.clone();

  match spawn(async move {
    let pixel_width = image_data.pixel_width as usize;
    let buffer = match image_data.color_type {
      ColorType::RGBA => image_data.data.clone(),
      ColorType::BGRA => {
        let mut buf = image_data.data.clone();
        for i in (0..buf.len()).step_by(pixel_width) {
          let b = buf[i];
          let r = buf[i + 2];
          buf[i] = r;
          buf[i + 2] = b;
        }

        buf
      },
    };

    let image_buffer = match image::RgbaImage::from_raw(image_data.width, image_data.height, buffer) {
      Some(buffer) => buffer,
      None => return Err(Error::from_reason("Failed to create image buffer")),
    };

    match image::DynamicImage::ImageRgba8(image_buffer).save(path) {
      Ok(_) => Ok(()),
      Err(_) => Err(Error::from_reason("Failed to save image")),
    }
  }).await {
    Ok(result) => result,
    Err(_) => Err(Error::from_reason("Failed to spawn task")),
  }
}

#[napi]
pub async fn read_image_data(path: String) -> Result<ImageData> {
  let path = path.clone();

  match spawn(async move {
    let dynamic_image = match image::open(path) {
      Ok(dynamic_image) => dynamic_image,
      Err(_) => return Err(Error::from_reason("Failed to open image")),
    };

    let width = dynamic_image.width();
    let height = dynamic_image.height();
    let pixel_width = dynamic_image.color().bytes_per_pixel();
    let data = dynamic_image.as_bytes().to_vec();

    Ok(ImageData {
      data,
      width,
      height,
      pixel_width,
      color_type: ColorType::RGBA,
    })
  }).await {
    Ok(result) => result,
    Err(_) => Err(Error::from_reason("Failed to spawn task")),
  }
}

fn image_search_inner<'a>(source: &'a ImageData, target: &'a ImageData, variant: i32) -> Option<Point> {
  let source_pixels = source.data.as_slice();
  let target_pixels = target.data.as_slice();

  let source_width = source.width;
  let source_height = source.height;

  let target_width = target.width;
  let target_height = target.height;

  let source_pixel_width = source.pixel_width as u32;
  let target_pixel_width = target.pixel_width as u32;

  let source_pixel_count = source_width * source_height;
  let target_pixel_count = target_width * target_height;

  if variant == 0 {
    for i in 0..source_pixel_count {
      let sx = i % source_width;
      let sy = i / source_width;

      if sx + target_width > source_width || sy + target_height > source_height {
        continue;
      }

      let mut is_found = true;

      for j in 0..target_pixel_count {
        let tx = j % target_width;
        let ty = j / target_width;

        let x = sx + tx;
        let y = sy + ty;

        let source_index = ((y * source_width + x) * source_pixel_width) as usize;
        let source_red = source_pixels[source_index + 2];
        let source_green = source_pixels[source_index + 1];
        let source_blue = source_pixels[source_index];

        let target_index = (j * target_pixel_width) as usize;

        let red = target_pixels[target_index];
        let green = target_pixels[target_index + 1];
        let blue = target_pixels[target_index + 2];

        is_found = source_red == red && source_green == green && source_blue == blue;

        if !is_found {
          break;
        }
      }

      if is_found {
        return Some(Point {
          x: sx,
          y: sy,
        });
      }
    }
  } else {
    for i in 0..source_pixel_count {
      let sx = i % source_width;
      let sy = i / source_width;

      if sx + target_width > source_width || sy + target_height > source_height {
        continue;
      }

      let mut is_found = true;

      for j in 0..target_pixel_count {
        let tx = j % target_width;
        let ty = j / target_width;

        let x = sx + tx;
        let y = sy + ty;

        let source_index = ((y * source_width + x) * source_pixel_width) as usize;
        let source_red = source_pixels[source_index + 2] as i32;
        let source_green = source_pixels[source_index + 1] as i32;
        let source_blue = source_pixels[source_index] as i32;

        let target_index = (j * target_pixel_width) as usize;

        let red = target_pixels[target_index] as i32;
        let green = target_pixels[target_index + 1] as i32;
        let blue = target_pixels[target_index + 2] as i32;

        let red_low = if source_red < variant { 0 } else { source_red - variant };
        let red_high = if source_red + variant > 255 { 255 } else { source_red + variant };

        let green_low = if source_green < variant { 0 } else { source_green - variant };
        let green_high = if source_green + variant > 255 { 255 } else { source_green + variant };

        let blue_low = if source_blue < variant { 0 } else { source_blue - variant };
        let blue_high = if source_blue + variant > 255 { 255 } else { source_blue + variant };

        is_found = red >= red_low
          && red <= red_high
          && green >= green_low
          && green <= green_high
          && blue >= blue_low
          && blue <= blue_high;

        if !is_found {
          break;
        }
      }

      if is_found {
        return Some(Point {
          x: sx,
          y: sy,
        });
      }
    }
  }

  None
}

#[napi]
pub async fn image_search(source: &ImageData, target: &ImageData, variant: Option<i32>) -> Result<Option<Point>> {
  let source = source.clone();
  let target = target.clone();
  let variant = variant.unwrap_or(0);

  match spawn(async move {
    image_search_inner(&source, &target, variant)
  })
    .await {
    Ok(result) => Ok(result),
    Err(err) => Err(Error::from_reason(err.to_string())),
  }
}