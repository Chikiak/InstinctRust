//! Marcadores de rugido: función pura para saber qué celdas tienen un rugido
//! activo (este tick o el anterior) y qué texto mostrar.
//! Sin dependencia de eframe/egui: totalmente testeable sin ventana.

use instinct_core::world::Roar;

/// Devuelve una entrada por celda con rugido activo: `(x, y, text)`.
///
/// `rows`/`cols` se usan para validación opcional de bounds; los rugidos
/// fuera de la malla se ignoran silenciosamente.
pub fn roar_markers(roars: &[Roar], rows: usize, cols: usize) -> Vec<(usize, usize, String)> {
    let mut markers: Vec<(usize, usize, String)> = Vec::new();
    for roar in roars {
        if roar.pos.x < cols && roar.pos.y < rows {
            markers.push((roar.pos.x, roar.pos.y, roar.text.clone()));
        }
    }
    markers
}
