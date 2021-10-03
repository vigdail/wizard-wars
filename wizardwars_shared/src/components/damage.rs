#[derive(Debug)]
pub struct Damage {
    amount: u32,
}

impl Damage {
    pub fn new(amount: u32) -> Self {
        Self { amount }
    }

    pub fn amount(&self) -> u32 {
        self.amount
    }
}

pub struct Attack {
    damage: u32,
    knockback_force: f32,
}

impl Attack {
    pub fn new(damage: u32, knockback_force: f32) -> Self {
        Attack {
            damage,
            knockback_force,
        }
    }

    pub fn damage(&self) -> Damage {
        Damage::new(self.damage)
    }

    pub fn knockback_force(&self) -> f32 {
        self.knockback_force
    }
}

pub struct FireBall {
    pub attack: Attack,
}
