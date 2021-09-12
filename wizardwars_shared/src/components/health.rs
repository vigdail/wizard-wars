use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Health {
    current: u32,
    maximum: u32,
}

impl Health {
    pub fn new(amount: u32) -> Self {
        Self {
            current: amount,
            maximum: amount,
        }
    }

    pub fn current(&self) -> u32 {
        self.current
    }

    pub fn maximum(&self) -> u32 {
        self.maximum
    }

    pub fn set_to(&mut self, amount: u32) {
        self.current = amount.min(self.maximum);
    }

    pub fn change_by(&mut self, amount: i32) {
        self.current = ((self.current as i32 + amount).max(0) as u32).min(self.maximum);
    }

    pub fn set_maximum(&mut self, amount: u32) {
        self.maximum = amount;
        self.current = self.current.min(self.maximum);
    }

    pub fn should_die(&self) -> bool {
        self.current == 0
    }

    pub fn fraction(&self) -> f32 {
        (self.current as f32) / (self.maximum as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_health_value() {
        let mut health = Health::new(100);

        assert_eq!(health.current(), 100);
        assert_eq!(health.maximum(), 100);

        health.set_to(50);
        assert_eq!(health.current(), 50);
        assert_eq!(health.maximum(), 100);

        health.set_to(150);
        assert_eq!(health.current(), 100);
        assert_eq!(health.maximum(), 100);

        health.set_to(0);
        assert_eq!(health.current(), 0);
        assert_eq!(health.maximum(), 100);
    }

    #[test]
    fn change_health() {
        let mut health = Health::new(100);

        assert_eq!(health.current(), 100);
        assert_eq!(health.maximum(), 100);

        health.change_by(50);
        assert_eq!(health.current(), 100);
        assert_eq!(health.maximum(), 100);

        health.change_by(-50);
        assert_eq!(health.current(), 50);
        assert_eq!(health.maximum(), 100);

        health.change_by(10);
        assert_eq!(health.current(), 60);
        assert_eq!(health.maximum(), 100);

        health.change_by(-70);
        assert_eq!(health.current(), 0);
        assert_eq!(health.maximum(), 100);
    }

    #[test]
    fn should_die() {
        let mut health = Health::new(10);
        assert!(!health.should_die());

        health.change_by(-10);
        assert!(health.should_die());
    }

    #[test]
    fn set_maximum() {
        let mut health = Health::new(10);

        health.set_maximum(20);
        assert_eq!(health.current, 10);
        assert_eq!(health.maximum, 20);

        health.set_maximum(5);
        assert_eq!(health.current, 5);
        assert_eq!(health.maximum, 5);
    }
}
