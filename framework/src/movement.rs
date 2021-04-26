use bevy::prelude::*;
use bevy::tasks::TaskPool;

pub struct Movement {
    pub speed: f32,
    pub direction: Vec2,
}

impl Movement {
    pub fn rotate(&mut self, angle_rad: f32) {
        let d = self.direction;
        self.direction = Vec2::new(
            d.x * angle_rad.cos() - d.y * angle_rad.sin(),
            d.x * angle_rad.sin() + d.y * angle_rad.cos(),
        );
    }
}

pub fn apply_movement(pool: Res<TaskPool>, mut query: Query<(&Movement, &mut Transform)>) {
    query.par_for_each_mut(&pool, 64, |(movement, mut tx)| {
        tx.translation += movement.direction.extend(0.0) * movement.speed;
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::f32::consts::PI;

    #[test]
    fn test_rotate() {
        let mut movement = Movement {
            speed: 0.0,
            direction: Vec2::X,
        };

        movement.rotate(PI);

        assert_eq!(movement.direction.x, -1.0);
        assert_eq!(movement.direction.y.abs() < f32::EPSILON, true);
        assert_eq!(movement.direction.length(), 1.0);

        movement.rotate(PI / 2.0);

        assert_eq!(movement.direction.length(), 1.0);
        assert_eq!(movement.direction.x.abs() < 0.000001, true);
        assert_eq!(movement.direction.y, -1.0);
    }
}
