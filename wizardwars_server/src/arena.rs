#![allow(dead_code)]
use std::f32::consts::PI;

use bevy::prelude::*;

pub struct Arena {
    spawn_points: Vec<Vec3>,
}

pub struct SpawnPointsBuilder {
    count: u32,
    points: Vec<Vec3>,
}

impl SpawnPointsBuilder {
    pub fn new(count: u32) -> Self {
        let points = (0..count).map(|_| Vec3::ZERO).collect();
        Self { count, points }
    }

    pub fn with_circle_points<'a>(&'a mut self, radius: f32) -> &'a mut Self {
        self.points = (0..self.count)
            .map(|i| {
                let angle = (i as f32 / self.count as f32) * PI * 2.0;
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
}

impl ArenaBuilder {
    pub fn with_spawn_points(&mut self, points: SpawnPointsBuilder) -> &mut Self {
        self.spawn_points = points.build();

        self
    }

    pub fn build(self) -> Arena {
        Arena {
            spawn_points: self.spawn_points,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn spawn_points_builder() {
        let count = 5;
        let builder = SpawnPointsBuilder::new(count);
        let points = builder.build();

        assert_eq!(points.len(), count as usize);

        for point in points.iter() {
            assert_eq!(point, &Vec3::ZERO);
        }
    }

    #[test]
    fn points_on_circle() {
        let count = 4;
        let radius = 10.0;
        let mut builder = SpawnPointsBuilder::new(count);
        builder.with_circle_points(radius);
        let points = builder.build();

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
}
