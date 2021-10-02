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
}

impl Attack {
    pub fn new(damage: u32) -> Self {
        Attack { damage }
    }

    pub fn damage(&self) -> Damage {
        Damage::new(self.damage)
    }
}

pub struct FireBall {
    pub attack: Attack,
}
