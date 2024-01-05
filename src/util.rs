pub struct Rectangle<T> {
    pub x: T,
    pub y: T,
    pub w: T,
    pub h: T,
}

impl<T> Rectangle<T> {
    pub fn new(x: T, y: T, w: T, h: T) -> Self {
        Self { x, y, w, h }
    }
}
