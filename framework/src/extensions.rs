use bevy::prelude::*;

pub trait TransformExtensions {
    fn position(&self) -> Vec2;
}

impl TransformExtensions for Transform {
    fn position(&self) -> Vec2 {
        self.translation.truncate()
    }
}
