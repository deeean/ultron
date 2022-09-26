use core_graphics::display::CGWarpMouseCursorPosition;
use napi::*;
use napi_derive::*;

#[napi]
pub fn mouse_move(x: u32, y: u32) {
  unsafe {
    CGWarpMouseCursorPosition(core_graphics::geometry::CGPoint {
      x: x as f64,
      y: y as f64,
    });
  }
}