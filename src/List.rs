use tui::widgets::ListState;

pub struct List {
    pub branches: Vec<String>,
    pub filtered_branches: Vec<String>,
    pub state: ListState,
    pub focused: bool,
    pub size: usize,
    pub position: usize,
}

impl List {
    // TODO: Don't hardcode the list
    pub fn new() -> Self {
        let words = vec![
            "main".to_string(),
            "task/ABK-12-Create-simulated-sensors".to_string(),
            "task/ABK-19-Setup-Communication-IoT-Hub".to_string(),
            "task/ABK-20-IoT-Hub-Msg-Handling-Pi".to_string(),
            "task/ABK-23-Create-Azure-Function-Read-Grow-Chamber".to_string(),
            "task/ABK-24-Create-Azure-Function-Write-Grow-Chamber".to_string(),
            "task/ABK-30-Create-graph-components".to_string(),
            "task/ABK-46-Integrate-backend-with-devices".to_string(),
            "task/abk-11-create-sensor-and-actuator-routines".to_string(),
            "task/abk-17-raspberry-pi-interfacing".to_string(),
            "task/abk-42-create-non-blocking-arduino-routine".to_string(),
            "task/abk-9-create-motr-and-servo-routine".to_string(),
            "topic/ABK-47-Integrate-backend-frontend".to_string(),
        ];

        Self {
            branches: words.clone(),
            filtered_branches: words.clone(),
            state: ListState::default(),
            focused: true,
            size: words.len(),
            position: 0,
        }
    }

    pub fn set_size(&mut self, size: usize) {
        self.size = size;
    }

    pub fn get_position(&self) -> usize {
        self.position
    }

    pub fn increment_position(&mut self) {
        if self.get_position() != 0 {
            self.position -= 1;
            self.state.select(Some(self.position));
        }
    }

    pub fn decrement_position(&mut self) {
        if self.position < self.size - 1 {
            self.position += 1;
            self.state.select(Some(self.position));
        }
    }

    pub fn reset_state(&mut self) {
        self.set_size(self.filtered_branches.len());
        self.position = 0;
        self.state.select(Some(0));
    }
}
