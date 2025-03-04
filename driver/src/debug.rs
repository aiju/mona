use crate::hw::*;

pub struct TextDisplay {
    min_x: u16,
    max_x: u16,
    min_y: u16,
    max_y: u16,
}

impl TextDisplay {
    pub fn new() -> Self {
        TextDisplay {
            min_x: 80,
            max_x: 0,
            min_y: 30,
            max_y: 0,
        }
    }
    pub fn init(&mut self, hw: &mut Hw) {
        let font = include_bytes!("VGA-ROM.F14");
        for (addr, &data) in font.iter().enumerate() {
            hw.set_reg(
                R_TEXT_ACCESS,
                B_TEXT_ACCESS_FONT | (addr as u32) << 16 | data as u32,
            );
        }
        for i in 0..80 * 30 {
            hw.set_reg(
                R_TEXT_ACCESS,
                B_TEXT_ACCESS_TEXT | (i as u32) << 16 | 0xf0 << 8 | 0x20,
            );
        }
        hw.set_reg(R_TEXT_TRANSPARENT, 15);
        hw.set_reg(R_TEXT_EN, B_TEXT_EN);
    }
    pub fn clear(&mut self, hw: &mut Hw, x: u16, y: u16, w: u16, h: u16) {
        for j in y..y + h {
            for i in x..x + w {
                hw.set_reg(
                    R_TEXT_ACCESS,
                    B_TEXT_ACCESS_TEXT | ((j * 80 + i) as u32) << 16 | 0xf0 << 8 | 0x20,
                );
            }
        }
    }
    pub fn clear_all(&mut self, hw: &mut Hw) {
        if self.min_x < self.max_x && self.min_y < self.max_y {
            self.clear(
                hw,
                self.min_x,
                self.min_y,
                self.max_x - self.min_x + 1,
                self.max_y - self.min_y + 1,
            );
            self.min_x = 80;
            self.max_x = 0;
            self.min_y = 30;
            self.max_y = 0;
        }
    }
    pub fn print(&mut self, hw: &mut Hw, mut x: u16, mut y: u16, str: &str) {
        let x_left = x;
        assert!(x < 80 && y < 30);
        for ch in str.chars() {
            if ch != '\n' {
                hw.set_reg(
                    R_TEXT_ACCESS,
                    B_TEXT_ACCESS_TEXT
                        | ((y * 80 + x) as u32) << 16
                        | 0x0d << 8
                        | (ch as u32) & 255,
                );
                self.min_x = self.min_x.min(x);
                self.max_x = self.max_x.max(x);
                self.min_y = self.min_y.min(y);
                self.max_y = self.max_y.max(y);
            }
            if x == 79 || ch == '\n' {
                y += 1;
                x = x_left;
                if y == 30 {
                    break;
                }
            } else {
                x += 1;
            }
        }
    }
}

const STATS_ROWS: [&'static str; 7] = [
    "starter->",
    "coarse->",
    "fine->",
    "depth->",
    "pixel->",
    "uv->",
    "texture->",
];

pub struct StatsCollection {
    stats: Vec<u32>,
}

impl StatsCollection {
    pub fn new() -> Self {
        StatsCollection {
            stats: Vec::with_capacity(STATS_ROWS.len() * 4),
        }
    }
    pub fn reset(&mut self, hw: &mut Hw) {
        self.stats.clear();
        self.stats
            .extend((0..7 * 4).map(|i| hw.get_reg(0x800 + 4 * i)));
    }
    pub fn start(&mut self, hw: &mut Hw) {
        hw.set_reg(R_STATS_ENABLED, 1);
    }
    pub fn stop(&mut self, hw: &mut Hw) {
        hw.set_reg(R_STATS_ENABLED, 0);
    }
    pub fn print(&mut self, hw: &mut Hw) {
        let stats = (0..7 * 4)
            .map(|i| {
                hw.get_reg(0x800 + 4 * i)
                    .wrapping_sub(self.stats[i as usize])
            })
            .collect::<Vec<_>>();
        println!(
            "{:10}{:>12}{:>12}{:>12}{:>12}",
            "left", "empty", "empty", "not empty", "not empty"
        );
        println!(
            "{:10}{:>12}{:>12}{:>12}{:>12}",
            "right", "full", "not full", "full", "not full"
        );
        for row in 0..7 {
            print!("{:10}", STATS_ROWS[row]);
            for col in 0..4 {
                print!("{:12}", stats[row * 4 + col]);
            }
            print!("\n");
        }
        print!("\n");
    }
}
