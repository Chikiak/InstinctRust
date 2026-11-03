//! Modelo de mundo: rejilla de terreno, objetos y dibujo ASCII.

use crate::diagnostic::Diagnostic;
use crate::resources::{Mapa, Objeto, Terrain};

/// Un elemento que ocupa una casilla sobre el terreno.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContenidoCasilla {
    /// Solo terreno.
    Vacio,
    /// Hay un objeto sobre el terreno.
    Objeto(usize), // índice en el catálogo de objetos del mundo
}

/// El mundo: rejilla de terrenos y objetos sobre ellos.
#[derive(Debug, Clone)]
pub struct World {
    pub nombre_mapa: String,
    pub columnas: usize,
    pub filas: usize,
    /// Terreno de cada casilla, indexado como [fila][columna].
    pub terreno: Vec<Vec<usize>>, // índice en el catálogo de terrenos
    /// Contenido opcional (objeto) de cada casilla.
    pub contenido: Vec<Vec<ContenidoCasilla>>,
    pub diagnosticos: Vec<Diagnostic>,
}

impl World {
    /// Construye un mundo a partir de un mapa cargado y los catálogos.
    /// Los caracteres que no coincidan con ningún terreno u objeto generan
    /// diagnóstico; el terreno debajo de una casilla con objeto es el primer
    /// terreno cargado.
    pub fn desde_mapa(mapa: &Mapa, terrenos: &[Terrain], objetos: &[Objeto]) -> Self {
        let mut diagnosticos: Vec<Diagnostic> = Vec::new();

        if terrenos.is_empty() {
            diagnosticos.push(Diagnostic::new(
                mapa.nombre.as_str(),
                1,
                "no hay terrenos cargados para dibujar el mapa".to_string(),
            ));
            return World {
                nombre_mapa: mapa.nombre.clone(),
                columnas: 0,
                filas: 0,
                terreno: Vec::new(),
                contenido: Vec::new(),
                diagnosticos,
            };
        }

        let primer_terreno = 0usize;
        let columnas = mapa.filas[0].len();
        let filas = mapa.filas.len();

        let mut terreno: Vec<Vec<usize>> = Vec::with_capacity(filas);
        let mut contenido: Vec<Vec<ContenidoCasilla>> = Vec::with_capacity(filas);

        for (fila_idx, fila) in mapa.filas.iter().enumerate() {
            let mut fila_terreno: Vec<usize> = Vec::with_capacity(columnas);
            let mut fila_contenido: Vec<ContenidoCasilla> = Vec::with_capacity(columnas);
            for c in fila.iter() {
                // Buscar el carácter entre objetos primero, luego terrenos.
                let objeto_idx = objetos.iter().position(|o| o.caracter == *c);
                if let Some(idx) = objeto_idx {
                    fila_terreno.push(primer_terreno);
                    fila_contenido.push(ContenidoCasilla::Objeto(idx));
                    continue;
                }
                let terreno_idx = terrenos.iter().position(|t| t.caracter == *c);
                match terreno_idx {
                    Some(idx) => {
                        fila_terreno.push(idx);
                        fila_contenido.push(ContenidoCasilla::Vacio);
                    }
                    None => {
                        diagnosticos.push(Diagnostic::new(
                            mapa.nombre.as_str(),
                            fila_idx + 2,
                            format!("carácter desconocido en el mapa: '{c}'"),
                        ));
                        fila_terreno.push(primer_terreno);
                        fila_contenido.push(ContenidoCasilla::Vacio);
                    }
                }
            }
            terreno.push(fila_terreno);
            contenido.push(fila_contenido);
        }

        World {
            nombre_mapa: mapa.nombre.clone(),
            columnas,
            filas,
            terreno,
            contenido,
            diagnosticos,
        }
    }

    /// Dibuja el mundo en ASCII: el carácter del objeto si lo hay, si no el del
    /// terreno. Las criaturas se añadirán en F0-04.
    pub fn dibujar(&self, terrenos: &[Terrain], objetos: &[Objeto]) -> String {
        let mut salida = String::new();
        for fila in 0..self.filas {
            for columna in 0..self.columnas {
                let c = match &self.contenido[fila][columna] {
                    ContenidoCasilla::Objeto(idx) => objetos[*idx].caracter,
                    ContenidoCasilla::Vacio => terrenos[self.terreno[fila][columna]].caracter,
                };
                salida.push(c);
            }
            salida.push('\n');
        }
        salida
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn terreno_grass() -> Terrain {
        Terrain {
            nombre: "Grass".to_string(),
            caracter: '.',
            resource_max: 100,
            regen: 2,
            reserva: 100,
        }
    }

    fn objeto_rock() -> Objeto {
        Objeto {
            nombre: "Rock".to_string(),
            caracter: '#',
            resource_max: 100,
            reserva: 100,
        }
    }

    #[test]
    fn test_dibujo_mapa_ejemplo() {
        let mapa = Mapa {
            nombre: "Plain with rocks".to_string(),
            filas: vec![
                "....................".chars().collect(),
                "...##...............".chars().collect(),
            ],
        };
        let terrenos = vec![terreno_grass()];
        let objetos = vec![objeto_rock()];
        let mundo = World::desde_mapa(&mapa, &terrenos, &objetos);
        assert!(mundo.diagnosticos.is_empty(), "{:?}", mundo.diagnosticos);
        let dibujo = mundo.dibujar(&terrenos, &objetos);
        let lineas: Vec<&str> = dibujo.lines().collect();
        assert_eq!(lineas.len(), 2);
        assert_eq!(lineas[0], "....................");
        assert_eq!(lineas[1], "...##...............");
    }

    #[test]
    fn test_caracter_desconocido_genera_diagnostico() {
        let mapa = Mapa {
            nombre: "Raro".to_string(),
            filas: vec!["....x....".chars().collect()],
        };
        let terrenos = vec![terreno_grass()];
        let objetos = vec![objeto_rock()];
        let mundo = World::desde_mapa(&mapa, &terrenos, &objetos);
        assert!(
            mundo
                .diagnosticos
                .iter()
                .any(|d| d.linea == 2 && d.mensaje.contains('x')),
            "{:?}",
            mundo.diagnosticos
        );
    }
}
