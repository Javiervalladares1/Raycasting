//! Módulo del “motor” con framebuffer, raycasting, colisiones, minimapa y texturas.

use raylib::prelude::*;

pub struct Framebuffer {
    pub w: u32,
    pub h: u32,
    pix: Vec<Color>,
}

impl Framebuffer {
    pub fn new(w: u32, h: u32) -> Self {
        Self { w, h, pix: vec![Color::BLACK; (w * h) as usize] }
    }
    pub fn limpiar(&mut self, c: Color) {
        self.pix.fill(c);
    }
    pub fn set(&mut self, x: i32, y: i32, c: Color) {
        if x >= 0 && y >= 0 && (x as u32) < self.w && (y as u32) < self.h {
            self.pix[(y as u32 * self.w + x as u32) as usize] = c;
        }
    }
    pub fn line_v(&mut self, x: i32, y0: i32, y1: i32, c: Color) {
        let a = y0.min(y1);
        let b = y0.max(y1);
        for y in a..=b { self.set(x, y, c); }
    }
    /// Placeholder (el texto real lo dibujamos con Raylib encima del framebuffer)
    pub fn texto(&mut self, _x: i32, _y: i32, _s: &str, _c: Color) {}
    pub fn pintar(&self, d: &mut RaylibDrawHandle, scale: i32) {
        for y in 0..self.h as i32 {
            for x in 0..self.w as i32 {
                let c = self.pix[(y as u32 * self.w + x as u32) as usize];
                if c.a > 0 {
                    d.draw_rectangle(x * scale, y * scale, scale, scale, c);
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct Jugador {
    pub x: f32,
    pub y: f32,
    pub ang: f32,
    pub vel: f32,
    pub rot: f32,
}

pub type Mapa = Vec<Vec<char>>;

pub fn celda(m: &Mapa, x: i32, y: i32) -> Option<char> {
    if y >= 0 && (y as usize) < m.len() && x >= 0 && (x as usize) < m[0].len() {
        Some(m[y as usize][x as usize])
    } else { None }
}

/// intenta mover con colisiones (pared si != caminable)
/// Caminable: ' ' (piso), 'E' (salida), 'A' (antorcha/sprite),
/// 'C' (moneda), 'P' (pozo → se pierde, pero se puede pisar).
pub fn mover_con_colision(j: &mut Jugador, dx_dir: f32, dy_dir: f32, mapa: &Mapa) {
    let dx = dx_dir * j.vel;
    let dy = dy_dir * j.vel;

    // Adelante/atrás con respecto a ángulo + strafe
    let dir_x = j.ang.cos() * dy + j.ang.sin() * dx;
    let dir_y = j.ang.sin() * dy - j.ang.cos() * dx;

    let nx = j.x + dir_x;
    let ny = j.y + dir_y;

    // X
    if let Some(c) = celda(mapa, nx.floor() as i32, j.y.floor() as i32) {
        if es_caminable(c) { j.x = nx; }
    }
    // Y
    if let Some(c) = celda(mapa, j.x.floor() as i32, ny.floor() as i32) {
        if es_caminable(c) { j.y = ny; }
    }
}

#[inline]
fn es_caminable(c: char) -> bool {
    matches!(c, ' ' | 'E' | 'A' | 'C' | 'P')
}

pub struct Texturas {
    pub tex: Vec<[Color; 64 * 64]>, // varias texturas 64x64 (una por pared id)
}

impl Texturas {
    pub fn nuevo() -> Self {
        // generamos 7 “slots” (0..6); usamos 1..6 para paredes distintas
        let mut v: Vec<[Color; 64 * 64]> = Vec::new();
        for i in 0..7 {
            let mut arr = [Color::BLACK; 64 * 64];
            for y in 0..64 {
                for x in 0..64 {
                    let c = match i {
                        1 => if (x / 8 + y / 8) % 2 == 0 { Color::DARKBLUE } else { Color::BLUE },
                        2 => if (x / 4) % 2 == 0 { Color::MAROON } else { Color::RED },
                        3 => if (y / 4) % 2 == 0 { Color::DARKGREEN } else { Color::GREEN },
                        4 => if ((x ^ y) & 16) == 0 { Color::BROWN } else { Color::BEIGE },
                        5 => if (x + y) % 10 < 5 { Color::PURPLE } else { Color::VIOLET },
                        6 => if (x * 3 + y * 5) % 37 < 18 { Color::GRAY } else { Color::LIGHTGRAY },
                        _ => Color::ORANGE,
                    };
                    arr[y * 64 + x] = c;
                }
            }
            v.push(arr);
        }
        Self { tex: v }
    }
    pub fn sample(&self, id: usize, u: f32, v: f32) -> Color {
        let tid = id.min(self.tex.len() - 1);
        // asegurar u,v en [0,1)
        let uu = {
            let f = u.fract();
            if f < 0.0 { f + 1.0 } else { f }
        };
        let vv = {
            let f = v.fract();
            if f < 0.0 { f + 1.0 } else { f }
        };
        let x = (uu * 64.0) as usize;
        let y = (vv * 64.0) as usize;
        self.tex[tid][y * 64 + x]
    }
}

/// Raycasting de muros con textura y cielo/piso simples, devuelve zbuffer por columna
pub fn dibujar_escena(fb: &mut Framebuffer, j: &Jugador, mapa: &Mapa, tex: &mut Texturas, z: &mut [f32]) {
    let w = fb.w as i32;
    let h = fb.h as i32;

    // cielo y piso
    for y in 0..h {
        let c = if y < h / 2 { Color::SKYBLUE } else { Color::BROWN };
        for x in 0..w { fb.set(x, y, c); }
    }

    let fov = 60.0_f32.to_radians();
    for x in 0..w {
        let cam_x = 2.0 * (x as f32 / w as f32) - 1.0;
        let ray_ang = j.ang + (fov / 2.0) * cam_x;

        let mut map_x = j.x.floor() as i32;
        let mut map_y = j.y.floor() as i32;

        let ray_dx = ray_ang.cos();
        let ray_dy = ray_ang.sin();

        let delta_x = if ray_dx == 0.0 { 1e30 } else { (1.0 / ray_dx).abs() };
        let delta_y = if ray_dy == 0.0 { 1e30 } else { (1.0 / ray_dy).abs() };

        let step_x: i32;
        let step_y: i32;
        let mut side_dist_x: f32;
        let mut side_dist_y: f32;

        if ray_dx < 0.0 {
            step_x = -1;
            side_dist_x = (j.x - map_x as f32) * delta_x;
        } else {
            step_x = 1;
            side_dist_x = ((map_x as f32 + 1.0) - j.x) * delta_x;
        }
        if ray_dy < 0.0 {
            step_y = -1;
            side_dist_y = (j.y - map_y as f32) * delta_y;
        } else {
            step_y = 1;
            side_dist_y = ((map_y as f32 + 1.0) - j.y) * delta_y;
        }

        let mut hit = false;
        let mut side = 0; // 0:x, 1:y
        let mut cell = ' ';
        while !hit {
            if side_dist_x < side_dist_y {
                side_dist_x += delta_x;
                map_x += step_x;
                side = 0;
            } else {
                side_dist_y += delta_y;
                map_y += step_y;
                side = 1;
            }
            if let Some(c) = celda(mapa, map_x, map_y) {
                // golpea si NO es caminable (o sea, es pared: 1..6)
                if !es_caminable(c) { hit = true; cell = c; }
            } else { break; }
        }

        if !hit { continue; }
        let mut perp_dist = if side == 0 {
            (map_x as f32 - j.x + (1 - step_x) as f32 / 2.0) / ray_dx
        } else {
            (map_y as f32 - j.y + (1 - step_y) as f32 / 2.0) / ray_dy
        };
        if perp_dist < 0.001 { perp_dist = 0.001; }
        z[x as usize] = perp_dist;

        // altura de pared
        let line_h = (h as f32 / perp_dist) as i32;
        let draw_start = (-line_h / 2 + h / 2).max(0);
        let draw_end   = ( line_h / 2 + h / 2).min(h - 1);

        // coordenada de textura (u)
        let mut wall_x = if side == 0 {
            j.y + perp_dist * ray_dy
        } else {
            j.x + perp_dist * ray_dx
        };
        wall_x -= wall_x.floor();

        // id de textura por tipo
        let id = match cell {
            '1' => 1, '2' => 2, '3' => 3, '4' => 4, '5' => 5, '6' => 6, _ => 1
        };

        // sombreado leve en caras Y
        let shade = if side == 1 { 0.8 } else { 1.0 };

        // pintar columna texturizada
        for y in draw_start..=draw_end {
            let v = (y - draw_start) as f32 / (draw_end - draw_start).max(1) as f32;
            let mut col = tex.sample(id, wall_x, v);
            col.r = ((col.r as f32) * shade) as u8;
            col.g = ((col.g as f32) * shade) as u8;
            col.b = ((col.b as f32) * shade) as u8;
            fb.set(x, y, col);
        }
    }
}

/// Minimap 2D en la esquina (escala 4 px por celda)
pub fn dibujar_minimapa(fb: &mut Framebuffer, j: &Jugador, mapa: &Mapa) {
    let s = 4; // px por celda
    let offx = 6;
    let offy = 6;
    for y in 0..mapa.len() as i32 {
        for x in 0..mapa[0].len() as i32 {
            let c = celda(mapa, x, y).unwrap_or('#');
            let col = match c {
                ' ' => Color::DARKGREEN, // piso
                'E' => Color::GOLD,      // salida
                'C' => Color::YELLOW,    // moneda
                'P' => Color::BLACK,     // pozo
                'A' => Color::ORANGE,    // antorcha/sprite
                _   => Color::DARKGRAY,  // pared
            };
            for yy in 0..s {
                for xx in 0..s {
                    fb.set(offx + x * s + xx, offy + y * s + yy, col);
                }
            }
        }
    }
    // jugador
    let px = offx + (j.x as i32) * s + s / 2;
    let py = offy + (j.y as i32) * s + s / 2;
    for yy in -1..=1 {
        for xx in -1..=1 {
            fb.set(px + xx, py + yy, Color::RED);
        }
    }
    // dirección
    let fx = (px as f32 + j.ang.cos() * 5.0) as i32;
    let fy = (py as f32 + j.ang.sin() * 5.0) as i32;
    linea_bresenham(fb, px, py, fx, fy, Color::YELLOW);
}

fn linea_bresenham(fb: &mut Framebuffer, x0: i32, y0: i32, x1: i32, y1: i32, c: Color) {
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let (mut x, mut y) = (x0, y0);
    loop {
        fb.set(x, y, c);
        if x == x1 && y == y1 { break; }
        let e2 = 2 * err;
        if e2 >= dy { err += dy; x += sx; }
        if e2 <= dx { err += dx; y += sy; }
    }
}
