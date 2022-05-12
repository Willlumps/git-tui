pub struct ListWindow {
    pub min: usize,
    pub max: usize,
    pub position: usize,
    pub size: usize,
    pub height: usize,
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
        self.max = self.height;
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
        self.min = self.min.saturating_sub(distance);
        self.position = self.min;

        if self.position > 0 {
            self.max -= distance;
        }
    }

    fn scroll_down(&mut self, distance: usize) {
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
