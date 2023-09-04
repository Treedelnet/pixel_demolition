pub struct Sounds {}

impl Sounds {
    pub const MINIGUN: usize = 0;
    pub const GLAUNCHER: usize = 1;
    pub const BULLET: usize = 2;
    pub const GRENADE: usize = 3;

    pub const SOUNDS: [&'static str; 4]
        = ["minigun.mp3", "glauncher.mp3", "bullet.mp3", "grenade.mp3"];
}
