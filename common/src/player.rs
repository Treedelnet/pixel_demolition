use crate::weapon::Weapon;

pub struct Player {
    pub name: String,

    pub weapon_type: Option<usize>,
    pub ammo: i32,

    pub x_last: f32,
    pub y_last: f32,
    pub x: f32,
    pub y: f32,
    pub x_new: f32,
    pub y_new: f32,

    pub vel_y: f32,
    pub angle: f32,
    pub ready: bool,
    pub jumps: i32,
    pub health: f32,
    pub trigger_pulled: bool,
    pub ticks_since_last_fire: i32,

    pub alive: bool,
    // In ms
    pub time_to_respawn: i32,

    pub kills: i32,
    pub deaths: i32,
}

impl Player {
    pub const PLAYER_HEIGHT:u32 = 64;
    pub const PLAYER_WIDTH:u32 = 24;

    #[cfg(target_family = "wasm")]
    pub const COLOR_NAMES:[&str;4] = ["red", "blue", "green", "purple"];

    #[cfg(target_family = "wasm")]
    pub const COLORS:[[u8;3];4] = [
        [150, 50, 50],
        [50, 50, 150],
        [50, 150, 50],
        [150, 50, 150],
    ];

    pub const AIR_JUMPS: i32 = 3;

    #[cfg(target_family = "wasm")]
    pub const JUMP_VEL: f32 = -0.5;

    pub const MAX_HEALTH: f32 = 10.0;

    // Move 100 pixels per second
    #[cfg(target_family = "wasm")]
    pub const MOVE_SPEED: f32 = 100.0/1000.0;

    // Respawn after 5 seconds
    pub const TIME_TO_RESPAWN: i32 = 5*1000;

    pub fn new(name: String) -> Player {
        return Player {
            name,
            weapon_type: None,
            ammo: 0,
            x_last:800.0,
            y_last:420.0,
            x:800.0,
            y:420.0,
            x_new:800.0,
            y_new:420.0,
            vel_y: 0.0,
            angle: 0.0,
            ready: false,
            jumps: Self::AIR_JUMPS,
            health: Self::MAX_HEALTH,
            trigger_pulled: false,
            ticks_since_last_fire: -1,
            alive: true,
            time_to_respawn: 0,
            kills: 0,
            deaths: 0,
        }
    }

    pub fn assign_weapon(&mut self, weapon_type: usize) {
        self.weapon_type = Some(weapon_type);
        self.ammo = Weapon::WEAPON_TYPES[weapon_type].ammo_count;
        self.ticks_since_last_fire = -1;
    }

    pub fn kill(&mut self) {
        self.alive = false;
        self.ammo = 0;
        self.weapon_type = None;
        self.deaths += 1;
        self.time_to_respawn = Self::TIME_TO_RESPAWN;
    }

    pub fn respawn(&mut self, x: f32, y: f32) {
        self.alive = true;
        self.health = Self::MAX_HEALTH;
        self.x_last = x;
        self.y_last = y;
        self.x = x;
        self.y = y;
        self.x_new = x;
        self.y_new = y;
    }
}
