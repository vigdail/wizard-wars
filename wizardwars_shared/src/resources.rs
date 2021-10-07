use bevy::prelude::*;

pub struct CharacterDimensions {
    width: f32,
    height: f32,
}

impl CharacterDimensions {
    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn radius(&self) -> f32 {
        self.width / 2.0
    }

    pub fn half_height(&self) -> f32 {
        self.height / 2.0
    }
}

impl Default for CharacterDimensions {
    fn default() -> Self {
        Self {
            width: 0.5,
            height: 1.8,
        }
    }
}

pub struct ArenaDimensions {
    pub radius: f32,
}

impl Default for ArenaDimensions {
    fn default() -> Self {
        Self { radius: 50.0 }
    }
}

pub const MAX_PLAYERS: usize = 8;

pub struct PlayerColors {
    pub colors: [Color; MAX_PLAYERS],
}

impl Default for PlayerColors {
    fn default() -> Self {
        let colors = [
            Color::RED,
            Color::BLUE,
            Color::GREEN,
            Color::PURPLE,
            Color::CYAN,
            Color::ORANGE,
            Color::OLIVE,
            Color::WHITE,
        ];
        Self { colors }
    }
}
