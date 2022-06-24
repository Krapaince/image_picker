pub struct Progression {
    current: usize,
    goal: usize,
}

impl Progression {
    pub fn new(goal: usize) -> Self {
        Self { current: 0, goal }
    }

    pub fn step(&mut self) {
        self.current = self.current.saturating_add(1);
    }

    pub fn compute_progress(&self) -> f32 {
        self.current as f32 / self.goal as f32
    }

    pub fn get_nb_remaining_step(&self) -> usize {
        self.goal - self.current
    }
}
