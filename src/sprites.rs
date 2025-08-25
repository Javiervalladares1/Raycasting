//! Sprites: Antorcha (A), Moneda (C), Pozo (P)

use crate::motor::*;
use raylib::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SpriteKind { Torch, Coin, Pit }

#[derive(Clone, Copy)]
pub struct Sprite {
    pub x: f32,
    pub y: f32,
    pub kind: SpriteKind,
}

pub struct Sprites {
    pub lista: Vec<Sprite>,
    anim_torch: Vec<[Color; 32*32]>, // frames 32x32
    anim_coin:  Vec<[Color; 32*32]>, // frames 32x32
    img_pit:    [Color; 32*32],      // estático
    f_torch: usize,
    f_coin:  usize,
    t: usize,
}

impl Sprites {
    pub fn nuevo() -> Self {
        // antorcha procedural
        let mut torch:Vec<[Color; 32*32]> = Vec::new();
        for f in 0..4 {
            let mut img = [Color::BLANK; 32*32];
            for y in 0..32 {
                for x in 0..32 {
                    let v = ((x*3 + y*5 + f*11) % 32) as i32;
                    let c = if v < 10 { Color::GOLD }
                            else if v < 18 { Color::ORANGE }
                            else if v < 26 { Color::RED }
                            else { Color::BLANK };
                    img[y*32 + x] = c;
                }
            }
            torch.push(img);
        }

        // moneda (borde dorado, “brillo” animado)
        let mut coin_anim:Vec<[Color; 32*32]> = Vec::new();
        for f in 0..4 {
            let mut img = [Color::BLANK; 32*32];
            for y in 0..32 {
                for x in 0..32 {
                    let cx = 16.0; let cy = 16.0;
                    let dx = x as f32 - cx;
                    let dy = y as f32 - cy;
                    let r2 = dx*dx + dy*dy;
                    let inside = r2 <= 14.5*14.5;
                    if inside {
                        // brillo gira
                        let ang = (f as f32)*0.8;
                        let shine = ((dx*ang.cos() + dy*ang.sin()).abs() < 2.0) as i32;
                        img[y*32 + x] = if shine==1 { Color::YELLOW } else { Color::GOLD };
                    } else if (r2 - 14.5*14.5).abs() < 2.5 {
                        img[y*32 + x] = Color::ORANGE;
                    }
                }
            }
            coin_anim.push(img);
        }

        // pozo (círculo negro con borde gris)
        let mut pit = [Color::BLANK; 32*32];
        for y in 0..32 {
            for x in 0..32 {
                let cx = 16.0; let cy = 16.0;
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                let r2 = dx*dx + dy*dy;
                if r2 <= 14.5*14.5 {
                    pit[y*32 + x] = Color::BLACK;
                } else if (r2 - 14.5*14.5).abs() < 2.5 {
                    pit[y*32 + x] = Color::DARKGRAY;
                }
            }
        }

        Self {
            lista: Vec::new(),
            anim_torch: torch,
            anim_coin: coin_anim,
            img_pit: pit,
            f_torch: 0,
            f_coin: 0,
            t: 0,
        }
    }

    /// Rellena sprites leyendo el mapa. Devuelve cuántas monedas hay.
    pub fn rellenar_desde_mapa(&mut self, mapa: &Mapa) -> usize {
        self.lista.clear();
        let mut coins = 0usize;
        for y in 0..mapa.len() as i32 {
            for x in 0..mapa[0].len() as i32 {
                match super::motor::celda(mapa, x, y).unwrap_or('#') {
                    'A' => self.lista.push(Sprite { x: x as f32 + 0.5, y: y as f32 + 0.5, kind: SpriteKind::Torch }),
                    'C' => { self.lista.push(Sprite { x: x as f32 + 0.5, y: y as f32 + 0.5, kind: SpriteKind::Coin }); coins += 1; }
                    'P' => self.lista.push(Sprite { x: x as f32 + 0.5, y: y as f32 + 0.5, kind: SpriteKind::Pit }),
                    _ => {}
                }
            }
        }
        coins
    }

    /// Elimina monedas en la celda actual del jugador. Devuelve cuántas recogió.
    pub fn recolectar_monedas_en(&mut self, px: f32, py: f32) -> usize {
        let cx = px.floor() as i32;
        let cy = py.floor() as i32;
        let mut count = 0usize;
        self.lista.retain(|s| {
            let scx = s.x.floor() as i32;
            let scy = s.y.floor() as i32;
            let same_cell = scx == cx && scy == cy;
            if same_cell && matches!(s.kind, SpriteKind::Coin) {
                count += 1;
                false // quitar
            } else {
                true
            }
        });
        count
    }

    pub fn actualizar(&mut self) {
        self.t += 1;
        if self.t % 12 == 0 { self.f_torch = (self.f_torch + 1) % self.anim_torch.len(); }
        if self.t % 10 == 0 { self.f_coin  = (self.f_coin  + 1) % self.anim_coin.len(); }
    }
}

pub fn dibujar_sprites(fb:&mut Framebuffer, j:&Jugador, mapa:&Mapa, spr:&Sprites, z:&[f32]) {
    let w = fb.w as i32;
    let h = fb.h as i32;

    // ordenar por distancia (lejos->cerca)
    let mut orden:Vec<(usize, f32)> = spr.lista.iter()
        .enumerate()
        .map(|(i,s)| (i, ((s.x - j.x).powi(2) + (s.y - j.y).powi(2)).sqrt()))
        .collect();
    orden.sort_by(|a,b| b.1.partial_cmp(&a.1).unwrap());

    for (idx, _dist) in orden {
        let s = spr.lista[idx];

        // seguridad: no dentro de pared
        if let Some(c) = super::motor::celda(mapa, s.x as i32, s.y as i32) {
            if c != ' ' && c != 'A' && c != 'C' && c != 'P' { continue; }
        }

        let dx = s.x - j.x;
        let dy = s.y - j.y;

        // Transformación a espacio de cámara (sin vector 'plane', usando ortogonal al 'look')
        let inv_det = 1.0 / (j.ang.cos() * (j.ang + std::f32::consts::FRAC_PI_2).sin()
                           - j.ang.sin() * (j.ang + std::f32::consts::FRAC_PI_2).cos());

        let trans_x = inv_det * ((j.ang + std::f32::consts::FRAC_PI_2).sin()*dx - (j.ang + std::f32::consts::FRAC_PI_2).cos()*dy);
        let trans_y = inv_det * (-j.ang.sin()*dx + j.ang.cos()*dy);
        if trans_y <= 0.01 { continue; }

        let sprite_screen_x = ((w as f32 / 2.0) * (1.0 + trans_x / trans_y)) as i32;

        // Tamaño proporcional a distancia, pero con mínimo para que se vean mejor
        let sprite_h = ((h as f32 / trans_y) as i32).max(14);
        let sprite_w = sprite_h;

        let draw_start_y = (-sprite_h/2 + h/2).max(0);
        let draw_end_y   = ( sprite_h/2 + h/2).min(h-1);
        let draw_start_x = (-sprite_w/2 + sprite_screen_x).max(0);
        let draw_end_x   = ( sprite_w/2 + sprite_screen_x).min(w-1);

        // elegir frame por tipo
        let (frame_opt, one_img) = match s.kind {
            SpriteKind::Torch => (Some(&spr.anim_torch[spr.f_torch]), None),
            SpriteKind::Coin  => (Some(&spr.anim_coin[spr.f_coin]), None),
            SpriteKind::Pit   => (None, Some(&spr.img_pit)),
        };

        for stripe in draw_start_x..=draw_end_x {
            let tex_x = ((stripe - (-sprite_w/2 + sprite_screen_x)) * 32 / sprite_w).clamp(0,31);
            if (trans_y as f32) < z[stripe as usize] {
                for y in draw_start_y..=draw_end_y {
                    let tex_y = ((y - (-sprite_h/2 + h/2)) * 32 / sprite_h).clamp(0,31);
                    let col = if let Some(fr) = frame_opt {
                        fr[tex_y as usize * 32 + tex_x as usize]
                    } else {
                        one_img.unwrap()[tex_y as usize * 32 + tex_x as usize]
                    };
                    if col.a > 0 { fb.set(stripe, y, col); }
                }
            }
        }
    }
}
