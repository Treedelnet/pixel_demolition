pub struct Flash {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub ttl: f32
}

impl Flash {
    // Flashes live 100ms
    pub const TTL: f32 = 100.0;

    pub fn new(x: f32, y: f32, radius: f32) -> Flash {
        Flash {
            x,
            y,
            radius,
            ttl: Self::TTL,
        }
    }
}
