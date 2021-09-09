#![allow(dead_code)]
use std::f32::consts::PI;

use bevy::prelude::*;

pub struct Arena {
    spawn_points: Vec<Vec3>,
    current_round: u32,
    total_rounds: u32,
}

impl Arena {
    pub fn spawn_points(&self) -> &Vec<Vec3> {
        &self.spawn_points
    }

    pub fn current_round(&self) -> u32 {
        self.current_round
    }

    pub fn is_last_round(&self) -> bool {
        self.current_round == self.total_rounds
    }

    pub fn next_round(&mut self) {
        self.current_round = self.total_rounds.min(self.current_round + 1);
    }

    pub fn total_rounds(&self) -> u32 {
        self.total_rounds
    }
}

pub struct SpawnPointsBuilder {
    points: Vec<Vec3>,
}

impl SpawnPointsBuilder {
    pub fn new() -> Self {
        Self { points: Vec::new() }
    }

    pub fn with_circle_points(mut self, count: u32, radius: f32) -> Self {
        self.points = (0..count)
            .map(|i| {
                let angle = (i as f32 / count as f32) * PI * 2.0;
                let x = angle.cos() * radius;
                let z = angle.sin() * radius;
                Vec3::new(x, 0.0, z)
            })
            .collect();

        self
    }

    pub fn build(self) -> Vec<Vec3> {
        self.points
    }
}

pub struct ArenaBuilder {
    spawn_points: Vec<Vec3>,
    total_rounds: u32,
}

impl ArenaBuilder {
    pub fn new() -> Self {
        Self {
            spawn_points: Vec::new(),
            total_rounds: 5,
        }
    }

    pub fn with_rounds(mut self, rounds: u32) -> Self {
        self.total_rounds = rounds;
        self
    }

    pub fn with_spawn_points(mut self, points: Vec<Vec3>) -> Self {
        self.spawn_points = points;

        self
    }

    pub fn build(self) -> Arena {
        Arena {
            spawn_points: self.spawn_points,
            current_round: 1,
            total_rounds: self.total_rounds,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn spawn_points_builder() {
        let builder = SpawnPointsBuilder::new();
        let points = builder.build();

        assert_eq!(points.len(), 0);
    }

    #[test]
    fn points_on_circle() {
        let count = 4;
        let radius = 10.0;
        let points = SpawnPointsBuilder::new()
            .with_circle_points(count, radius)
            .build();

        assert_eq!(points.len(), count as usize);
        assert_eq!(points[0], Vec3::new(radius, 0.0, 0.0));

        for point in points.iter() {
            assert!((point.distance(Vec3::ZERO) - radius).abs() < f32::EPSILON);
        }

        for window in points.windows(2) {
            let p1 = window[0];
            let p2 = window[1];
            assert!((p1.dot(p2)).abs() < 0.001);
        }
    }

    #[test]
    fn arena_builder_default_values() {
        let arena = ArenaBuilder::new().build();

        assert_eq!(arena.total_rounds, 5);
        assert_eq!(arena.current_round, 1);
        assert_eq!(arena.spawn_points.len(), 0);
    }

    #[test]
    fn arena_builder_with_rounds() {
        let expected_total_rounds = 10;
        let arena = ArenaBuilder::new()
            .with_rounds(expected_total_rounds)
            .build();

        assert_eq!(arena.total_rounds, expected_total_rounds);
        assert_eq!(arena.current_round, 1);
    }

    #[test]
    fn arena_next_round() {
        let total_rounds = 2;
        let mut arena = ArenaBuilder::new().with_rounds(total_rounds).build();
        assert_eq!(arena.current_round, 1);
        assert_eq!(arena.total_rounds, total_rounds);
        assert!(!arena.is_last_round());

        arena.next_round();
        assert_eq!(arena.current_round, 2);
        assert_eq!(arena.total_rounds, total_rounds);
        assert!(arena.is_last_round());

        arena.next_round();
        assert_eq!(arena.current_round, 2);
        assert_eq!(arena.total_rounds, total_rounds);
        assert!(arena.is_last_round());
    }

    #[test]
    fn arena_last_round() {
        let mut arena = ArenaBuilder::new().build();
        assert_eq!(arena.current_round, 1);
        assert_eq!(arena.total_rounds, 5);

        arena.next_round();
        assert_eq!(arena.current_round, 2);
        assert_eq!(arena.total_rounds, 5);
    }
}
