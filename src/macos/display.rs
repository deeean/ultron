use crate::{ColorType, ImageData};
use core_graphics::display::{CGDirectDisplayID, CGGetActiveDisplayList, CGMainDisplayID};
use core_graphics::geometry::CGRect;
use core_graphics::image::CGImage;
use core_graphics::sys::CGImageRef;
use napi::*;
use napi_derive::*;
use foreign_types::{ForeignType};

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
  fn CGDisplayCreateImageForRect(display: CGDirectDisplayID, rect: CGRect) -> CGImageRef;
}

#[napi]
pub fn get_active_display_count() -> Result<u32> {
  unsafe {
    let mut count = 0;
    let err = CGGetActiveDisplayList(0, 0 as *mut CGDirectDisplayID, &mut count);
    if err != 0 {
      return Err(Error::new(Status::GenericFailure, "CGGetActiveDisplayList failed".to_string()));
    }

    Ok(count)
  }
}

#[napi]
pub fn take_screenshot(x: f64, y: f64, width: f64, height: f64, id: Option<u32>) -> Result<ImageData> {
  unsafe {
    let id = match id {
      Some(id) => {
        let mut count = get_active_display_count()?;
        let mut active_displays: Vec<u32> = Vec::with_capacity(count as usize);
        active_displays.set_len(count as usize);

        let err = CGGetActiveDisplayList(count, active_displays.as_mut_ptr(), &mut count);
        if err != 0 {
          return Err(Error::new(Status::GenericFailure, "CGGetActiveDisplayList failed".to_string()));
        }

        active_displays[id as usize]
      },
      None => CGMainDisplayID(),
    };


    let image_ref = CGDisplayCreateImageForRect(id, CGRect {
      origin: core_graphics::geometry::CGPoint {
        x,
        y,
      },
      size: core_graphics::geometry::CGSize {
        width,
        height,
      },
    });

    if image_ref.is_null() {
      return Err(Error::new(Status::GenericFailure, "CGDisplayCreateImage failed".to_string()));
    }

    let image = CGImage::from_ptr(image_ref);

    Ok(ImageData {
      data: image.data().to_vec(),
      width: image.width() as u32,
      height: image.height() as u32,
      pixel_width: (image.bits_per_pixel() / 8) as u8,
      color_type: ColorType::BGRA,
    })
  }
}