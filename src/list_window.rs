pub struct ListWindow {
    min: usize,
    max: usize,
    position: usize,
    size: usize,
    height: usize,
}

pub enum ScrollDirection {
    Up,
    Down,
}

impl ListWindow {
    pub fn new(min: usize, max: usize, position: usize, size: usize, height: usize) -> Self {
        Self {
            min,
            max,
            position,
            size,
            height,
        }
    }

    pub fn reset(&mut self) {
        self.min = 0;
        self.max = self.size.min(self.height).saturating_sub(1);
        self.position = 0;
    }

    pub fn scroll(&mut self, direction: ScrollDirection, distance: usize) {
        match direction {
            ScrollDirection::Up => self.scroll_up(distance),
            ScrollDirection::Down => self.scroll_down(distance),
        }
    }

    pub fn set_height(&mut self, height: usize) {
        self.height = height;
    }

    pub fn set_size(&mut self, size: usize) {
        self.size = size;
    }

    pub fn height(&mut self) -> usize {
        self.height
    }

    pub fn position(&self) -> Option<usize> {
        Some(self.position)
    }

    fn scroll_up(&mut self, distance: usize) {
        self.position = self.min;
        self.min = self.min.saturating_sub(distance);
        self.position = self.position.saturating_sub(distance);

        if self.position == 0 {
            self.max = self.height - 1;
        } else {
            self.max -= distance;
        }
    }

    fn scroll_down(&mut self, distance: usize) {
        if self.size == 0 {
            return;
        }

        self.position = self.max;
        self.position += distance;

        if self.position > self.size - 1 {
            self.position = self.size - 1;
            self.max = self.size - 1;
            self.min = self.size.saturating_sub(self.height);
        } else {
            self.max += distance;
            self.min += distance;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn reset_smaller_size() {
        let mut window = ListWindow::new(0, 0, 0, 10, 25);
        window.reset();

        assert_eq!(window.max, 9);
        assert_eq!(window.min, 0);
        assert_eq!(window.position, 0);
    }

    #[test]
    fn reset_larger_size() {
        let mut window = ListWindow::new(0, 0, 0, 100, 25);
        window.reset();

        assert_eq!(window.max, 24);
        assert_eq!(window.min, 0);
        assert_eq!(window.position, 0);
    }

    #[test]
    fn scroll_up_single_top() {
        let mut window = ListWindow::new(0, 24, 0, 100, 25);
        window.scroll_up(1);

        assert_eq!(window.min, 0);
        assert_eq!(window.max, 24);
        assert_eq!(window.position, 0);
    }

    #[test]
    fn scroll_up_single_not_top() {
        let mut window = ListWindow::new(5, 29, 29, 100, 25);
        window.scroll_up(1);

        assert_eq!(window.min, 4);
        assert_eq!(window.max, 28);
        assert_eq!(window.position, 4);
    }

    #[test]
    fn scroll_up_page_top() {
        let mut window = ListWindow::new(0, 24, 0, 100, 25);
        window.scroll_up(12);

        assert_eq!(window.min, 0);
        assert_eq!(window.max, 24);
        assert_eq!(window.position, 0);
    }

    #[test]
    fn scroll_up_page_middle() {
        let mut window = ListWindow::new(10, 34, 10, 100, 25);
        window.scroll_up(12);

        assert_eq!(window.min, 0);
        assert_eq!(window.max, 24);
        assert_eq!(window.position, 0);
    }

    #[test]
    fn scroll_up_page_bottom() {
        let mut window = ListWindow::new(30, 54, 54, 100, 25);
        window.scroll_up(12);

        assert_eq!(window.min, 18);
        assert_eq!(window.max, 42);
        assert_eq!(window.position, 18);
    }

    #[test]
    fn scroll_down_single_bottom() {
        let mut window = ListWindow::new(10, 19, 19, 20, 10);
        window.scroll_down(1);

        assert_eq!(window.min, 10);
        assert_eq!(window.max, 19);
        assert_eq!(window.position, 19);
    }

    #[test]
    fn scroll_down_single_not_bottom() {
        let mut window = ListWindow::new(0, 9, 0, 20, 10);
        window.scroll_down(1);

        assert_eq!(window.min, 1);
        assert_eq!(window.max, 10);
        assert_eq!(window.position, 10);
    }

    #[test]
    fn scroll_down_page_bottom() {
        let mut window = ListWindow::new(75, 99, 75, 100, 25);
        window.scroll_down(12);

        assert_eq!(window.min, 75);
        assert_eq!(window.max, 99);
        assert_eq!(window.position, 99);
    }

    #[test]
    fn scroll_down_page_middle() {
        let mut window = ListWindow::new(70, 94, 70, 100, 25);
        window.scroll_down(12);

        assert_eq!(window.min, 75);
        assert_eq!(window.max, 99);
        assert_eq!(window.position, 99);
    }

    #[test]
    fn scroll_down_page_top() {
        let mut window = ListWindow::new(30, 54, 54, 100, 25);
        window.scroll_down(12);

        assert_eq!(window.min, 42);
        assert_eq!(window.max, 66);
        assert_eq!(window.position, 66);
    }
}
