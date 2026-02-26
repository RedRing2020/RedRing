#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SelectionRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl SelectionRect {
    pub fn from_points(start: (f32, f32), end: (f32, f32)) -> Self {
        let x = start.0.min(end.0);
        let y = start.1.min(end.1);
        let width = (end.0 - start.0).abs();
        let height = (end.1 - start.1).abs();

        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains(&self, point: (f32, f32)) -> bool {
        point.0 >= self.x
            && point.0 <= self.x + self.width
            && point.1 >= self.y
            && point.1 <= self.y + self.height
    }

    pub fn is_empty(&self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_points() {
        let rect = SelectionRect::from_points((100.0, 200.0), (50.0, 120.0));
        assert_eq!(rect.x, 50.0);
        assert_eq!(rect.y, 120.0);
        assert_eq!(rect.width, 50.0);
        assert_eq!(rect.height, 80.0);
    }

    #[test]
    fn contains_point() {
        let rect = SelectionRect::from_points((10.0, 10.0), (30.0, 20.0));
        assert!(rect.contains((10.0, 10.0)));
        assert!(rect.contains((30.0, 20.0)));
        assert!(!rect.contains((31.0, 20.0)));
    }
}
