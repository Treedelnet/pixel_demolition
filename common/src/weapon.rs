use crate::projectile::Projectile;
use crate::server_tick::ServerTick;

pub struct Weapon {
    pub weapon_type: usize,
    pub x: f32,
    pub y: f32,
}

pub struct WeaponType {
    pub name: &'static str,
    // Server ticks elapsed in between fires
    pub ticks_per_fire: i32,
    pub ammo_count: i32,
    // Pixels/ms
    pub projectile_type: usize,
}

impl Weapon {
    #[cfg(not(target_family = "wasm"))]
    pub const WEAPON_WIDTH: u32 = 42;

    pub const WEAPON_TYPES:[WeaponType;2] = [
        WeaponType {
            name: "minigun",
            // Fire 5 times per second
            ticks_per_fire: (5.0/(1000.0/ServerTick::SERVER_TICK as f32)) as i32,
            ammo_count: 50,
            projectile_type: Projectile::TYPE_BULLET,
        },
        WeaponType {
            name: "glauncher",
            // Fire every 2 seconds
            ticks_per_fire: (0.5*(1000.0/ServerTick::SERVER_TICK as f32)) as i32,
            ammo_count: 10,
            projectile_type: Projectile::TYPE_GRENADE,
        },
    ];

    pub fn new(weapon_type: usize, x: f32, y: f32) -> Weapon {
        return Weapon {
            weapon_type,
            x,
            y,
        }
    }
}
