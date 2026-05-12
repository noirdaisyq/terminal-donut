use std::{
    env,
    f32::consts::TAU,
    io::{self, Write},
    thread,
    time::{Duration, Instant},
};

const SHADES: &[u8] = b" .,-~:;=!*#$@";

#[derive(Clone, Copy)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    fn normalized(self) -> Self {
        let len = self.dot(self).sqrt().max(0.0001);
        Self {
            x: self.x / len,
            y: self.y / len,
            z: self.z / len,
        }
    }

    fn rotate_x(self, angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self {
            x: self.x,
            y: self.y * c - self.z * s,
            z: self.y * s + self.z * c,
        }
    }

    fn rotate_y(self, angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self {
            x: self.x * c + self.z * s,
            y: self.y,
            z: -self.x * s + self.z * c,
        }
    }

    fn rotate_z(self, angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Self {
            x: self.x * c - self.y * s,
            y: self.x * s + self.y * c,
            z: self.z,
        }
    }
}

struct Screen {
    width: usize,
    height: usize,
    pixels: Vec<u8>,
    colors: Vec<(u8, u8, u8)>,
    depth: Vec<f32>,
}

impl Screen {
    fn new(width: usize, height: usize) -> Self {
        let area = width * height;
        Self {
            width,
            height,
            pixels: vec![b' '; area],
            colors: vec![(0, 0, 0); area],
            depth: vec![f32::NEG_INFINITY; area],
        }
    }

    fn clear(&mut self) {
        self.pixels.fill(b' ');
        self.colors.fill((0, 0, 0));
        self.depth.fill(f32::NEG_INFINITY);
    }

    fn plot(&mut self, x: isize, y: isize, z: f32, shade: u8, color: (u8, u8, u8)) {
        if x < 0 || y < 0 || x >= self.width as isize || y >= self.height as isize {
            return;
        }

        let idx = y as usize * self.width + x as usize;
        if z > self.depth[idx] {
            self.depth[idx] = z;
            self.pixels[idx] = shade;
            self.colors[idx] = color;
        }
    }

    fn render<W: Write>(&self, out: &mut W) -> io::Result<()> {
        write!(out, "\x1b[H")?;

        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let (r, g, b) = self.colors[idx];
                let ch = self.pixels[idx] as char;

                if ch == ' ' {
                    write!(out, " ")?;
                } else {
                    write!(out, "\x1b[38;2;{r};{g};{b}m{ch}\x1b[0m")?;
                }
            }
            write!(out, "\r\n")?;
        }

        out.flush()
    }
}

fn terminal_size() -> (usize, usize) {
    let width = env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(96)
        .clamp(40, 180);

    let height = env::var("LINES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(32)
        .saturating_sub(2)
        .clamp(20, 60);

    (width, height)
}

fn color_for(surface_angle: f32, light: f32, time: f32) -> (u8, u8, u8) {
    let pulse = (time * 1.7 + surface_angle * 2.5).sin() * 0.5 + 0.5;
    let glow = (80.0 + 175.0 * light).clamp(0.0, 255.0);

    let r = (30.0 + glow * (0.35 + pulse * 0.65)) as u8;
    let g = (90.0 + glow * (0.25 + (1.0 - pulse) * 0.45)) as u8;
    let b = (140.0 + glow * 0.55) as u8;

    (r, g, b)
}

fn main() -> io::Result<()> {
    let (width, height) = terminal_size();
    let mut screen = Screen::new(width, height);
    let mut out = io::stdout().lock();
    let start = Instant::now();

    write!(out, "\x1b[2J\x1b[?25l")?;

    let light = Vec3 {
        x: -0.35,
        y: 0.75,
        z: -0.55,
    }
    .normalized();

    for frame in 0..900 {
        let t = start.elapsed().as_secs_f32();
        let spin_x = t * 0.82;
        let spin_y = t * 0.37;
        let spin_z = t * 0.21;

        screen.clear();

        let major_radius = 1.35 + (t * 0.8).sin() * 0.08;
        let minor_radius = 0.55;
        let distance = 4.2;
        let scale = height as f32 * 0.82;

        let mut u = 0.0;
        while u < TAU {
            let mut v = 0.0;
            while v < TAU {
                let (su, cu) = u.sin_cos();
                let (sv, cv) = v.sin_cos();

                let ring = major_radius + minor_radius * cv;
                let point = Vec3 {
                    x: ring * cu,
                    y: ring * su,
                    z: minor_radius * sv,
                }
                .rotate_x(spin_x)
                .rotate_y(spin_y)
                .rotate_z(spin_z);

                let normal = Vec3 {
                    x: cv * cu,
                    y: cv * su,
                    z: sv,
                }
                .rotate_x(spin_x)
                .rotate_y(spin_y)
                .rotate_z(spin_z)
                .normalized();

                let z = point.z + distance;
                let inv_z = 1.0 / z;
                let x = (width as f32 * 0.5 + point.x * scale * inv_z * 1.85) as isize;
                let y = (height as f32 * 0.5 - point.y * scale * inv_z) as isize;

                let light_power = normal.dot(light).max(0.0);
                let rim = (1.0 - normal.z.abs()).powf(2.2) * 0.35;
                let intensity = (light_power * 0.85 + rim + 0.08).clamp(0.0, 1.0);
                let shade_idx = (intensity * (SHADES.len() - 1) as f32) as usize;
                let color = color_for(u + v, intensity, t);

                screen.plot(x, y, -z, SHADES[shade_idx], color);

                v += 0.045;
            }
            u += 0.018;
        }

        screen.render(&mut out)?;
        writeln!(
            out,
            "\x1b[38;2;180;220;255mterminal-donut\x1b[0m  frame {:03}  Ctrl+C to bail",
            frame
        )?;

        thread::sleep(Duration::from_millis(16));
    }

    write!(out, "\x1b[?25h\x1b[0m")?;
    Ok(())
}
