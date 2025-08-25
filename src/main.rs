//! Ray-caster simple – Javier
//! Ahora con monedas (C) y pozos (P). Si pisas P, pierdes.
//! Monedas se muestran como sprites y se recolectan al pasar por la celda.

mod motor;
mod mapas;
mod sprites;

use motor::*;
use mapas::*;
use sprites::*;

use raylib::prelude::*;

const W: u32 = 320;   // resolución lógica (ancha)
const H: u32 = 200;   // resolución lógica (alta)
const SCALE: i32 = 3; // factor de escala a la ventana
const BLOQUE: usize = 1;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Estado {
    Menu,
    Juego,
    Exito,
    Perdio,
}

// --- helpers de spawn seguro ---
fn es_walkable(mapa: &Mapa, x: i32, y: i32) -> bool {
    if let Some(c) = celda(mapa, x, y) {
        // Piso libre o celdas caminables (incluye monedas y pozos)
        c == ' ' || c == 'E' || c == 'A' || c == 'C' || c == 'P'
    } else {
        false
    }
}

fn spawn_mas_cercano(mapa: &Mapa, sx: i32, sy: i32) -> (i32, i32) {
    if es_walkable(mapa, sx, sy) {
        return (sx, sy);
    }
    let max_r = (mapa.len() + mapa[0].len()) as i32;
    for r in 1..=max_r {
        for dy in -r..=r {
            for dx in -r..=r {
                let nx = sx + dx;
                let ny = sy + dy;
                if es_walkable(mapa, nx, ny) {
                    return (nx, ny);
                }
            }
        }
    }
    (1, 1)
}

fn main() {
    // --- ventana ---
    let (mut rl, thread) = raylib::init()
        .size((W as i32) * SCALE, (H as i32) * SCALE)
        .title("Raycaster – Javier")
        .resizable()
        .build();

    rl.set_target_fps(60);
    rl.set_mouse_scale(1.0, 1.0);
    rl.set_mouse_cursor(raylib::consts::MouseCursor::MOUSE_CURSOR_CROSSHAIR);

    // --- framebuffer lógico ---
    let mut fb = Framebuffer::new(W, H);

    // --- estados ---
    let mut estado = Estado::Menu;
    let mut idx_nivel = 0usize;
    let niveles = niveles();

    // --- jugador ---
    let mut jug = Jugador {
        x: 2.5,
        y: 2.5,
        ang: 0.0,
        vel: 2.0 / 60.0,
        rot: 2.2 / 60.0,
    };

    // --- texturas y sprites ---
    let mut tex = Texturas::nuevo();
    let mut spr = Sprites::nuevo();       // frames y lista vacía; se llena al entrar al nivel

    // --- monedas ---
    let mut coins_total: usize = 0;
    let mut coins_taken: usize = 0;

    // --- mouse look ---
    let mut mouse_on = true;
    rl.set_mouse_position((
        (W as f32 * SCALE as f32) / 2.0,
        (H as f32 * SCALE as f32) / 2.0,
    ));
    let mut prev_mouse_x: i32 = (W as i32 * SCALE) / 2;

    // --- gamepad ---
    let mut gamepad_id = None;
    for id in 0..4 {
        if rl.is_gamepad_available(id) {
            gamepad_id = Some(id);
            break;
        }
    }

    // --- bucle principal ---
    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        fb.limpiar(Color::BLACK);

        match estado {
            Estado::Menu => {
                if d.is_key_pressed(KeyboardKey::KEY_DOWN) {
                    idx_nivel = (idx_nivel + 1) % niveles.len();
                }
                if d.is_key_pressed(KeyboardKey::KEY_UP) {
                    idx_nivel = (idx_nivel + niveles.len() - 1) % niveles.len();
                }
                if d.is_key_pressed(KeyboardKey::KEY_ENTER) {
                    // Spawn seguro
                    let mapa = &niveles[idx_nivel].mapa;
                    let (sx, sy, ang0) = niveles[idx_nivel].inicio;
                    let (fx, fy) = spawn_mas_cercano(mapa, sx, sy);
                    jug.x = fx as f32 + 0.5;
                    jug.y = fy as f32 + 0.5;
                    jug.ang = ang0;

                    // Rellenar sprites desde mapa (A = antorcha, C = coin, P = pozo)
                    coins_total = spr.rellenar_desde_mapa(mapa);
                    coins_taken = 0;

                    estado = Estado::Juego;
                }
            }

            Estado::Juego => {
                let mapa = &niveles[idx_nivel].mapa;

                // --- teclado ---
                let mut dir_x = 0.0;
                let mut dir_y = 0.0;

                if d.is_key_down(KeyboardKey::KEY_W) { dir_y += 1.0; }
                if d.is_key_down(KeyboardKey::KEY_S) { dir_y -= 1.0; }
                if d.is_key_down(KeyboardKey::KEY_A) { dir_x -= 1.0; }
                if d.is_key_down(KeyboardKey::KEY_D) { dir_x += 1.0; }
                if d.is_key_down(KeyboardKey::KEY_Q) { jug.ang -= jug.rot; }
                if d.is_key_down(KeyboardKey::KEY_E) { jug.ang += jug.rot; }

                // --- mouse toggle ---
                if d.is_key_pressed(KeyboardKey::KEY_M) {
                    mouse_on = !mouse_on;
                    prev_mouse_x = d.get_mouse_x();
                }

                // --- mouse look horizontal ---
                if mouse_on {
                    let mx = d.get_mouse_x();
                    let dx = mx - prev_mouse_x;
                    jug.ang += (dx as f32) * 0.0035;
                    prev_mouse_x = mx;
                }

                // --- gamepad ---
                if let Some(id) = gamepad_id {
                    let lx = d.get_gamepad_axis_movement(id, GamepadAxis::GAMEPAD_AXIS_LEFT_X);
                    let ly = d.get_gamepad_axis_movement(id, GamepadAxis::GAMEPAD_AXIS_LEFT_Y);
                    dir_x += lx as f32;
                    dir_y += -ly as f32;

                    let rx = d.get_gamepad_axis_movement(id, GamepadAxis::GAMEPAD_AXIS_RIGHT_X);
                    jug.ang += (rx as f32) * 0.04;
                }

                // normalizar input
                let len = (dir_x * dir_x + dir_y * dir_y).sqrt();
                if len > 0.01 { dir_x /= len; dir_y /= len; }

                // mover con colisiones (C y P son caminables)
                mover_con_colision(&mut jug, dir_x, dir_y, &mapa);

                // ¿cayó en pozo?
                if let Some(c) = celda(mapa, jug.x as i32, jug.y as i32) {
                    if c == 'P' {
                        estado = Estado::Perdio;
                    }
                }

                // ¿recogió moneda(s) en la celda?
                let recogidas = spr.recolectar_monedas_en(jug.x, jug.y);
                if recogidas > 0 { coins_taken += recogidas; }

                // raycasting paredes + zbuffer
                let mut zbuf = vec![f32::INFINITY; W as usize];
                dibujar_escena(&mut fb, &jug, &mapa, &mut tex, &mut zbuf);

                // sprites (antorcha/monedas/pozos)
                spr.actualizar();
                dibujar_sprites(&mut fb, &jug, &mapa, &spr, &zbuf);

                // minimapa
                dibujar_minimapa(&mut fb, &jug, &mapa);

                // éxito si toca 'E' (no depende de las monedas, pero podés exigir todas si querés)
                if let Some(c) = celda(&mapa, jug.x as i32, jug.y as i32) {
                    if c == 'E' {
                        estado = Estado::Exito;
                    }
                }
            }

            Estado::Exito => { /* UI abajo */ }
            Estado::Perdio => { /* UI abajo */ }
        }

        // pintar framebuffer
        fb.pintar(&mut d, SCALE);

        // HUD/UI
        match estado {
            Estado::Menu => {
                let centro_x = (W as i32 * SCALE) / 2;
                let mut y = 70;
                d.draw_text("RAYCASTER – BIENVENIDO", centro_x - 140, 20, 24, Color::RAYWHITE);
                d.draw_text("Usa ↑/↓ para elegir nivel y ENTER para iniciar",
                            centro_x - 190, 45, 12, Color::LIGHTGRAY);

                for (i, n) in niveles.iter().enumerate() {
                    let marca = if i == idx_nivel { "> " } else { "  " };
                    let txt = format!("{marca}{}", n.nombre);
                    d.draw_text(&txt, centro_x - 120, y, 20,
                                if i==idx_nivel { Color::YELLOW } else { Color::GRAY });
                    y += 22;
                }

                d.draw_text("Mouse: mirar | WSAD: mover | Q/E: rotar | M: toggle mouse",
                            10, H as i32*SCALE - 30, 12, Color::GRAY);
                d.draw_text("Gamepad: stick izq mover, stick der rotar",
                            10, H as i32*SCALE - 16, 12, Color::GRAY);
            }
            Estado::Juego => {
                d.draw_text(&format!("FPS: {}", d.get_fps()), 6, 6, 14, Color::WHITE);
                d.draw_text(&format!("Coins: {}/{}", coins_taken, coins_total),
                            6, 24, 14, Color::YELLOW);
            }
            Estado::Exito => {
                let cx = (W as i32 * SCALE) / 2;
                d.draw_text("¡ÉXITO!", cx - 60, 40, 30, Color::LIME);
                d.draw_text("Has llegado a la salida.", cx - 120, 80, 20, Color::RAYWHITE);
                d.draw_text("ENTER: volver al menú", cx - 120, 110, 18, Color::LIGHTGRAY);
                if d.is_key_pressed(KeyboardKey::KEY_ENTER) {
                    estado = Estado::Menu;
                }
            }
            Estado::Perdio => {
                let cx = (W as i32 * SCALE) / 2;
                d.draw_text("¡PERDISTE!", cx - 80, 40, 30, Color::RED);
                d.draw_text("Caíste en un pozo.", cx - 90, 80, 20, Color::RAYWHITE);
                d.draw_text("ENTER: volver al menú", cx - 120, 110, 18, Color::LIGHTGRAY);
                if d.is_key_pressed(KeyboardKey::KEY_ENTER) {
                    estado = Estado::Menu;
                }
            }
        }
    }
}
