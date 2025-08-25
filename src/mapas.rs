//! Mapas + selector de nivel. Símbolos:
//! '1'..'6' = paredes
//! ' ' = piso
//! 'E' = salida
//! 'A' = antorcha (sprite)
//! 'C' = moneda (sprite)
//! 'P' = pozo (caminable pero si lo pisas, pierdes)

use super::motor::Mapa;

pub struct Nivel {
    pub nombre: &'static str,
    pub mapa: Mapa,
    pub inicio: (i32, i32, f32), // x, y, ang
}

pub fn niveles() -> Vec<Nivel> {
    vec![
        Nivel {
            nombre: "Nivel 1 – Pasillos",
            mapa: parse_mapa(&[
                "111111111111111111",
                "1 C 2   P  3   C E1",
                "1 111  33  3  1111",
                "1   C 22   P  4  1",
                "1  444   11   C  1",
                "1 C 1   1   6    1",
                "1   1   1111111111",
                "1   1     P      1",
                "1 C 1   A   C    1",
                "111111111111111111",
            ]),
            inicio: (2, 1, 0.0),
        },
        Nivel {
            nombre: "Nivel 2 – Patio",
            mapa: parse_mapa(&[
                "111111111111111111",
                "1 C 2    P    A  1",
                "1 1   1111  6  C 1",
                "1 1   C    1   P E1",
                "1 1  3333  1     1",
                "1  C 444   1   C 1",
                "1   6   P  1     1",
                "1   C  A   1     1",
                "1 P        2   C 1",
                "111111111111111111",
            ]),
            inicio: (2, 1, 0.0),
        },
    ]
}

/// Parser robusto: usa el **máximo ancho** entre todas las filas y rellena con ' '
/// cuando una fila es más corta. Si una fila es más larga, se trunca al máximo.
fn parse_mapa(lines: &[&str]) -> Mapa {
    let h = lines.len();
    let w = lines.iter().map(|s| s.chars().count()).max().unwrap_or(0);

    let mut m = vec![vec![' '; w]; h];
    for (y, row) in lines.iter().enumerate() {
        // Convertimos la fila a Vec<char> una vez para indexar por posición
        let chars: Vec<char> = row.chars().collect();
        for x in 0..w {
            let ch = if x < chars.len() { chars[x] } else { ' ' };
            m[y][x] = match ch {
                '0' | ' ' => ' ',                              // piso
                '1' | '2' | '3' | '4' | '5' | '6' => ch,       // paredes
                'E' | 'A' | 'C' | 'P' => ch,                   // especiales
                _ => '1',                                      // cualquier otro símbolo lo tratamos como pared
            };
        }
    }
    m
}
