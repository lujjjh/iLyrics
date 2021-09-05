use bindings::Windows::Win32::Graphics::Direct2D::D2D_RECT_F;

use super::Size;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Rect {
    pub fn width(&self) -> f32 {
        self.right - self.left
    }

    pub fn height(&self) -> f32 {
        self.bottom - self.top
    }

    pub fn size(&self) -> Size {
        Size {
            width: self.width(),
            height: self.height(),
        }
    }

    pub fn inset(&self, left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self {
            left: self.left + left,
            top: self.top + top,
            right: self.right - right,
            bottom: self.bottom - bottom,
        }
    }

    pub fn inset_dx_dy(&self, dx: f32, dy: f32) -> Self {
        self.inset(dx, dy, dx, dy)
    }

    pub fn inset_all(&self, value: f32) -> Self {
        self.inset_dx_dy(value, value)
    }
}

impl Into<D2D_RECT_F> for Rect {
    fn into(self) -> D2D_RECT_F {
        let Self {
            left,
            top,
            right,
            bottom,
        } = self;
        D2D_RECT_F {
            left,
            top,
            right,
            bottom,
        }
    }
}

impl Into<Rect> for D2D_RECT_F {
    fn into(self) -> Rect {
        let Self {
            left,
            top,
            right,
            bottom,
        } = self;
        Rect {
            left,
            top,
            right,
            bottom,
        }
    }
}
