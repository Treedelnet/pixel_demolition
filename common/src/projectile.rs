pub struct ProjectileType {
    pub name: &'static str,
    pub init_vel: f32,
    pub damage_radius: f32,
    // Damage per pixel of overlap with player
    pub damage: f32,
}

pub struct Projectile {
    pub projectile_type: usize,
    pub x: f32,
    pub y: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub owner: usize,
}

impl Projectile {
    pub const TYPE_BULLET: usize = 0;
    pub const TYPE_GRENADE: usize = 1;

    pub const PROJECTILE_TYPES: [ProjectileType;2] = [
        ProjectileType {
            name: "bullet",
            init_vel: 1.7,
            damage_radius: 1.0,
            damage: 0.3,
        },
        ProjectileType {
            name: "grenade",
            init_vel: 0.7,
            damage_radius: 40.0,
            // Grenades do more damage in general, but the per-pixel damage is lower
            damage: 0.003,
        }
    ];

    pub fn draw_explosion(&self) -> Vec<(i32, i32)> {
        if Self::PROJECTILE_TYPES[self.projectile_type].damage_radius <= 1.0 {
            let pixels:Vec<(i32, i32)> = Vec::from([(self.x as i32, self.y as i32)]);
            return pixels;
        }

        let mut pixels:Vec<(i32, i32)> = Vec::new();

        let radius = Self::PROJECTILE_TYPES[self.projectile_type].damage_radius;

        for x in (-radius as i32)..=(radius as i32) {
            let angle = ((x as f32)/radius).acos();

            let y_top = -angle.sin() * radius;
            let y_top = y_top.round() as i32;

            let y_bottom = -y_top;

            for draw_y in y_top..y_bottom {
                pixels.push((x + self.x as i32, draw_y + self.y as i32));
            }
        }

        return pixels;
    }
}
