use std::{
    env,
    io::{self, Write},
    thread,
    time::{Duration, Instant},
};

const PALETTE: [(u8, u8, u8); 37] = [
    (7, 7, 16),
    (18, 8, 24),
    (31, 9, 28),
    (45, 11, 26),
    (62, 13, 22),
    (78, 17, 18),
    (96, 22, 14),
    (115, 29, 11),
    (132, 38, 8),
    (149, 48, 7),
    (166, 61, 6),
    (181, 75, 7),
    (196, 91, 8),
    (210, 108, 11),
    (223, 126, 16),
    (234, 145, 23),
    (243, 163, 32),
    (250, 181, 45),
    (255, 198, 61),
    (255, 212, 79),
    (255, 224, 99),
    (255, 233, 120),
    (255, 240, 142),
    (255, 246, 164),
    (255, 250, 185),
    (255, 253, 205),
    (255, 255, 222),
    (255, 255, 236),
    (255, 255, 244),
    (255, 255, 250),
    (255, 250, 226),
    (255, 236, 184),
    (255, 220, 132),
    (255, 204, 86),
    (255, 190, 52),
    (255, 238, 160),
    (255, 255, 245),
];

const GLYPHS: &[u8] = b"  ..::-==+**##%@";

#[derive(Clone)]
struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self { state: seed | 1 }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.state >> 32) as u32
    }

    fn range(&mut self, max: u32) -> u32 {
        self.next_u32() % max.max(1)
    }
}

struct Fire {
    width: usize,
    height: usize,
    heat: Vec<u8>,
    rng: Rng,
}

impl Fire {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            heat: vec![0; width * height],
            rng: Rng::new(0xF1_A5_EED),
        }
    }

    fn feed_base(&mut self, frame: usize) {
        let base = self.height - 1;
        let wave = ((frame as f32 * 0.09).sin() * 0.5 + 0.5) * 7.0;

        for x in 0..self.width {
            let ember = self.rng.range(9) as i16;
            let lick = ((x as f32 * 0.19 + frame as f32 * 0.06).sin() * 0.5 + 0.5) * 10.0;
            let heat = 25 + wave as i16 + lick as i16 + ember;
            self.heat[base * self.width + x] = heat.clamp(0, 36) as u8;
        }

        for _ in 0..self.width / 10 {
            let x = self.rng.range(self.width as u32) as usize;
            self.heat[base * self.width + x] = 36;
        }
    }

    fn step(&mut self, frame: usize) {
        self.feed_base(frame);

        let wind = ((frame as f32 * 0.035).sin() * 1.5).round() as isize;

        for y in 1..self.height {
            for x in 0..self.width {
                let below_idx = y * self.width + x;
                let below = self.heat[below_idx] as i16;
                let decay = self.rng.range(4) as i16;
                let drift = self.rng.range(3) as isize - 1 + wind;
                let target_x = (x as isize + drift).rem_euclid(self.width as isize) as usize;
                let target_y = y - 1;
                let target_idx = target_y * self.width + target_x;

                self.heat[target_idx] = below.saturating_sub(decay).min(36) as u8;
            }
        }

        self.add_sparks(frame);
    }

    fn add_sparks(&mut self, frame: usize) {
        if frame % 2 != 0 {
            return;
        }

        for _ in 0..self.width / 18 {
            let x = self.rng.range(self.width as u32) as usize;
            let y = self.height - 2 - self.rng.range((self.height / 3).max(1) as u32) as usize;
            let idx = y * self.width + x;
            self.heat[idx] = self.heat[idx].saturating_add(8).min(36);
        }
    }

    fn render<W: Write>(&self, out: &mut W, elapsed: f32, frame: usize) -> io::Result<()> {
        write!(out, "\x1b[H")?;

        for y in 0..self.height {
            for x in 0..self.width {
                let heat = self.heat[y * self.width + x] as usize;
                let (r, g, b) = PALETTE[heat];
                let glyph_idx = heat * (GLYPHS.len() - 1) / (PALETTE.len() - 1);
                let glyph = GLYPHS[glyph_idx] as char;

                if heat == 0 {
                    write!(out, " ")?;
                } else {
                    write!(out, "\x1b[38;2;{r};{g};{b}m{glyph}\x1b[0m")?;
                }
            }
            write!(out, "\r\n")?;
        }

        write!(
            out,
            "\x1b[38;2;255;210;120mRust Fire Simulator\x1b[0m  frame {:04}  {:.1}s  Ctrl+C to quit\r\n",
            frame, elapsed
        )?;

        out.flush()
    }
}

fn terminal_size() -> (usize, usize) {
    let width = env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(100)
        .clamp(50, 180);

    let height = env::var("LINES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(34)
        .saturating_sub(2)
        .clamp(20, 70);

    (width, height)
}

fn main() -> io::Result<()> {
    let (width, height) = terminal_size();
    let mut fire = Fire::new(width, height);
    let mut out = io::stdout().lock();
    let start = Instant::now();

    write!(out, "\x1b[2J\x1b[?25l")?;

    for frame in 0..3_600 {
        let elapsed = start.elapsed().as_secs_f32();
        fire.step(frame);
        fire.render(&mut out, elapsed, frame)?;
        thread::sleep(Duration::from_millis(28));
    }

    write!(out, "\x1b[?25h\x1b[0m")?;
    Ok(())
}
