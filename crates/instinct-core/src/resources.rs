//! Carga de recursos: terrenos (`.te`), objetos (`.ob`) y mapas (`.map`).
//!
//! Cada carpeta se lee en orden alfabético de nombre de archivo. Un recurso
//! que falla genera diagnósticos y se descarta, pero la carga continúa con los
//! demás. Una carpeta ausente se trata como vacía, sin diagnóstico.

use crate::diagnostic::Diagnostic;
use std::fs;
use std::path::Path;

/// Un terreno cargado. Su reserva empieza llena (igual a `resource_max`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Terrain {
    pub nombre: String,
    pub caracter: char,
    pub resource_max: i64,
    pub regen: i64,
    pub reserva: i64,
}

/// Un objeto cargado. Su reserva empieza en `resource_max` y no se regenera.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Objeto {
    pub nombre: String,
    pub caracter: char,
    pub resource_max: i64,
    pub reserva: i64,
}

/// Un mapa cargado: su nombre y sus filas de caracteres.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mapa {
    pub nombre: String,
    pub filas: Vec<Vec<char>>,
}

/// Resultado de cargar un directorio de recursos.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecursosCargados {
    pub terrenos: Vec<Terrain>,
    pub objetos: Vec<Objeto>,
    pub mapas: Vec<Mapa>,
    pub diagnosticos: Vec<Diagnostic>,
}

impl RecursosCargados {
    pub fn vacio() -> Self {
        RecursosCargados {
            terrenos: Vec::new(),
            objetos: Vec::new(),
            mapas: Vec::new(),
            diagnosticos: Vec::new(),
        }
    }
}

/// Par (resultado, diagnósticos) para un recurso individual.
type CargaRecurso<T> = (Option<T>, Vec<Diagnostic>);

// ---------------------------------------------------------------------------
// Parsing de líneas clave-valor (.te y .ob)
// ---------------------------------------------------------------------------

/// Divide una línea en (clave, valor) usando el primer espacio como separador.
/// Devuelve None si la línea no tiene clave.
fn separar_clave_valor(linea: &str) -> Option<(&str, &str)> {
    let linea = linea.trim();
    if linea.is_empty() {
        return None;
    }
    // Línea de comentario: su primer carácter no blanco es '#'.
    if linea.starts_with('#') {
        return None;
    }
    match linea.find(char::is_whitespace) {
        Some(i) => {
            let clave = &linea[..i];
            let valor = linea[i..].trim();
            if clave.is_empty() {
                None
            } else {
                Some((clave, valor))
            }
        }
        None => Some((linea, "")),
    }
}

/// Lee todas las parejas clave-valor de un contenido, en orden de aparición,
/// ignorando blancos y comentarios. Devuelve (pares, líneas_con_clave) donde
/// cada entrada guarda el número de línea (desde 1) para diagnósticos.
fn leer_claves(contenido: &str) -> Vec<(usize, String, String)> {
    let mut salida: Vec<(usize, String, String)> = Vec::new();
    for (idx, linea) in contenido.lines().enumerate() {
        let numero = idx + 1;
        if let Some((clave, valor)) = separar_clave_valor(linea) {
            salida.push((numero, clave.to_string(), valor.to_string()));
        }
    }
    salida
}

/// Valor de una clave buscada por nombre; devuelve el número de línea de su
/// primera aparición o None si no existe.
fn valor_clave<'a>(
    claves: &'a [(usize, String, String)],
    nombre: &'a str,
) -> Option<(usize, &'a str)> {
    claves
        .iter()
        .find(|(_, c, _)| c == nombre)
        .map(|(n, _, v)| (*n, v.as_str()))
}

/// Valida que `caracteres` tenga exactamente un carácter.
fn validar_un_caracter(
    numero: usize,
    valor: &str,
    archivo: &str,
    diagnosticos: &mut Vec<Diagnostic>,
) -> Option<char> {
    let mut chars = valor.chars();
    let primero = chars.next();
    let segundo = chars.next();
    match (primero, segundo) {
        (Some(c), None) => Some(c),
        _ => {
            diagnosticos.push(Diagnostic::new(
                archivo,
                numero,
                format!("el valor de 'char' debe ser un único carácter, obtuvo: '{valor}'"),
            ));
            None
        }
    }
}

/// Parsea un entero i64; falla con diagnóstico si no es numérico.
fn parsear_entero(
    numero: usize,
    valor: &str,
    clave: &str,
    archivo: &str,
    diagnosticos: &mut Vec<Diagnostic>,
) -> Option<i64> {
    match valor.trim().parse::<i64>() {
        Ok(v) => Some(v),
        Err(_) => {
            diagnosticos.push(Diagnostic::new(
                archivo,
                numero,
                format!("el valor de '{clave}' debe ser un número entero, obtuvo: '{valor}'"),
            ));
            None
        }
    }
}

/// Carga un terreno desde su contenido en memoria.
pub fn cargar_terreno(contenido: &str, archivo: &str) -> CargaRecurso<Terrain> {
    let mut diagnosticos: Vec<Diagnostic> = Vec::new();
    let claves = leer_claves(contenido);

    let nombre = match valor_clave(&claves, "terrain") {
        Some((n, v)) => {
            if v.trim().is_empty() {
                diagnosticos.push(Diagnostic::new(
                    archivo,
                    n,
                    "falta el valor de la clave 'terrain'".to_string(),
                ));
                return (None, diagnosticos);
            }
            v.trim().to_string()
        }
        None => {
            diagnosticos.push(Diagnostic::new(
                archivo,
                1,
                "falta la clave obligatoria 'terrain'".to_string(),
            ));
            return (None, diagnosticos);
        }
    };

    let caracter = match valor_clave(&claves, "char") {
        Some((n, v)) => match validar_un_caracter(n, v, archivo, &mut diagnosticos) {
            Some(c) => c,
            None => return (None, diagnosticos),
        },
        None => {
            diagnosticos.push(Diagnostic::new(
                archivo,
                1,
                "falta la clave obligatoria 'char'".to_string(),
            ));
            return (None, diagnosticos);
        }
    };

    let resource_max = match valor_clave(&claves, "resource_max") {
        Some((n, v)) => match parsear_entero(n, v, "resource_max", archivo, &mut diagnosticos) {
            Some(v) => v,
            None => return (None, diagnosticos),
        },
        None => {
            diagnosticos.push(Diagnostic::new(
                archivo,
                1,
                "falta la clave obligatoria 'resource_max'".to_string(),
            ));
            return (None, diagnosticos);
        }
    };

    let regen = match valor_clave(&claves, "regen") {
        Some((n, v)) => match parsear_entero(n, v, "regen", archivo, &mut diagnosticos) {
            Some(v) => v,
            None => return (None, diagnosticos),
        },
        None => {
            diagnosticos.push(Diagnostic::new(
                archivo,
                1,
                "falta la clave obligatoria 'regen'".to_string(),
            ));
            return (None, diagnosticos);
        }
    };

    if resource_max <= 0 {
        diagnosticos.push(Diagnostic::new(
            archivo,
            1,
            format!("'resource_max' debe ser mayor que 0, obtuvo {resource_max}"),
        ));
        return (None, diagnosticos);
    }
    if regen < 0 {
        diagnosticos.push(Diagnostic::new(
            archivo,
            1,
            format!("'regen' debe ser mayor o igual que 0, obtuvo {regen}"),
        ));
        return (None, diagnosticos);
    }

    (
        Some(Terrain {
            nombre,
            caracter,
            resource_max,
            regen,
            reserva: resource_max,
        }),
        diagnosticos,
    )
}

/// Carga un objeto desde su contenido en memoria.
pub fn cargar_objeto(contenido: &str, archivo: &str) -> CargaRecurso<Objeto> {
    let mut diagnosticos: Vec<Diagnostic> = Vec::new();
    let claves = leer_claves(contenido);

    let nombre = match valor_clave(&claves, "object") {
        Some((n, v)) => {
            if v.trim().is_empty() {
                diagnosticos.push(Diagnostic::new(
                    archivo,
                    n,
                    "falta el valor de la clave 'object'".to_string(),
                ));
                return (None, diagnosticos);
            }
            v.trim().to_string()
        }
        None => {
            diagnosticos.push(Diagnostic::new(
                archivo,
                1,
                "falta la clave obligatoria 'object'".to_string(),
            ));
            return (None, diagnosticos);
        }
    };

    let caracter = match valor_clave(&claves, "char") {
        Some((n, v)) => match validar_un_caracter(n, v, archivo, &mut diagnosticos) {
            Some(c) => c,
            None => return (None, diagnosticos),
        },
        None => {
            diagnosticos.push(Diagnostic::new(
                archivo,
                1,
                "falta la clave obligatoria 'char'".to_string(),
            ));
            return (None, diagnosticos);
        }
    };

    let resource_max = match valor_clave(&claves, "resource_max") {
        Some((n, v)) => match parsear_entero(n, v, "resource_max", archivo, &mut diagnosticos) {
            Some(v) => v,
            None => return (None, diagnosticos),
        },
        None => {
            diagnosticos.push(Diagnostic::new(
                archivo,
                1,
                "falta la clave obligatoria 'resource_max'".to_string(),
            ));
            return (None, diagnosticos);
        }
    };

    if resource_max <= 0 {
        diagnosticos.push(Diagnostic::new(
            archivo,
            1,
            format!("'resource_max' debe ser mayor que 0, obtuvo {resource_max}"),
        ));
        return (None, diagnosticos);
    }

    (
        Some(Objeto {
            nombre,
            caracter,
            resource_max,
            reserva: resource_max,
        }),
        diagnosticos,
    )
}

/// Carga un mapa desde su contenido en memoria. No hay comentarios en mapas.
/// `caracteres_validos` son los `char` declarados por terrenos y objetos;
/// cualquier otro carácter genera un diagnóstico con la línea de su fila
/// sin impedir la carga del mapa ni de los demás.
pub fn cargar_mapa(
    contenido: &str,
    archivo: &str,
    caracteres_validos: &[char],
) -> CargaRecurso<Mapa> {
    let mut diagnosticos: Vec<Diagnostic> = Vec::new();
    let lineas: Vec<&str> = contenido.lines().collect();

    if lineas.is_empty() {
        diagnosticos.push(Diagnostic::new(
            archivo,
            1,
            "el mapa debe tener al menos una fila (el nombre)".to_string(),
        ));
        return (None, diagnosticos);
    }

    let nombre = lineas[0].to_string();
    let filas_str = &lineas[1..];

    if filas_str.is_empty() {
        diagnosticos.push(Diagnostic::new(
            archivo,
            2,
            "el mapa debe tener al menos una fila de casillas".to_string(),
        ));
        return (None, diagnosticos);
    }

    let mut filas: Vec<Vec<char>> = Vec::new();
    let mut esperado_longitud: Option<usize> = None;

    for (idx, fila) in filas_str.iter().enumerate() {
        let numero = idx + 2; // línea 1 es el nombre
        let chars: Vec<char> = fila.chars().collect();
        if let Some(l) = esperado_longitud {
            if chars.len() != l {
                diagnosticos.push(Diagnostic::new(
                    archivo,
                    numero,
                    format!(
                        "la fila tiene {} caracteres, se esperaban {}",
                        chars.len(),
                        l
                    ),
                ));
                continue;
            }
        } else {
            esperado_longitud = Some(chars.len());
        }
        for c in &chars {
            if !caracteres_validos.contains(c) {
                diagnosticos.push(Diagnostic::new(
                    archivo,
                    numero,
                    format!("carácter desconocido en el mapa: '{c}'"),
                ));
            }
        }
        filas.push(chars);
    }

    if filas.is_empty() {
        diagnosticos.push(Diagnostic::new(
            archivo,
            2,
            "el mapa debe tener al menos una fila de casillas".to_string(),
        ));
        return (None, diagnosticos);
    }

    (Some(Mapa { nombre, filas }), diagnosticos)
}

// ---------------------------------------------------------------------------
// Carga por carpetas
// ---------------------------------------------------------------------------

fn listar_archivos_ordenados(dir: &Path, extension: &str) -> Vec<std::path::PathBuf> {
    let mut salida: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(lectura) = fs::read_dir(dir) {
        for entrada in lectura.flatten() {
            let path = entrada.path();
            if path.extension().is_some_and(|ext| ext == extension) {
                salida.push(path);
            }
        }
    }
    salida.sort();
    salida
}

/// Carga todos los terrenos de una carpeta, en orden alfabético.
pub fn cargar_terrenos_carpeta(ruta: &Path) -> (Vec<Terrain>, Vec<Diagnostic>) {
    let mut terrenos: Vec<Terrain> = Vec::new();
    let mut diagnosticos: Vec<Diagnostic> = Vec::new();
    for path in listar_archivos_ordenados(ruta, "te") {
        let archivo = path.to_string_lossy().to_string();
        let contenido = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                diagnosticos.push(Diagnostic::new(
                    &archivo,
                    1,
                    format!("no se pudo leer el archivo: {e}"),
                ));
                continue;
            }
        };
        match cargar_terreno(&contenido, &archivo) {
            (Some(t), mut d) => {
                diagnosticos.append(&mut d);
                terrenos.push(t);
            }
            (None, mut d) => {
                diagnosticos.append(&mut d);
            }
        }
    }
    (terrenos, diagnosticos)
}

/// Carga todos los objetos de una carpeta, en orden alfabético, verificando
/// que no repitan `char` ni `nombre` con terrenos u objetos ya cargados.
pub fn cargar_objetos_carpeta(ruta: &Path, terrenos: &[Terrain]) -> (Vec<Objeto>, Vec<Diagnostic>) {
    let mut objetos: Vec<Objeto> = Vec::new();
    let mut diagnosticos: Vec<Diagnostic> = Vec::new();

    let mut caracteres_usados: Vec<char> = terrenos.iter().map(|t| t.caracter).collect();
    let mut nombres_usados: Vec<String> =
        terrenos.iter().map(|t| nombre_clonado(&t.nombre)).collect();

    for path in listar_archivos_ordenados(ruta, "ob") {
        let archivo = path.to_string_lossy().to_string();
        let contenido = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                diagnosticos.push(Diagnostic::new(
                    &archivo,
                    1,
                    format!("no se pudo leer el archivo: {e}"),
                ));
                continue;
            }
        };
        match cargar_objeto(&contenido, &archivo) {
            (Some(o), mut d) => {
                // Verificar duplicados de char y nombre contra lo ya cargado.
                if caracteres_usados.contains(&o.caracter)
                    || objetos.iter().any(|obj| obj.caracter == o.caracter)
                {
                    d.push(Diagnostic::new(
                        &archivo,
                        1,
                        format!(
                            "el caracter '{}' ya está en uso por otro recurso",
                            o.caracter
                        ),
                    ));
                    diagnosticos.append(&mut d);
                    continue;
                }
                if nombres_usados.contains(&o.nombre)
                    || objetos.iter().any(|obj| obj.nombre == o.nombre)
                {
                    d.push(Diagnostic::new(
                        &archivo,
                        1,
                        format!("el nombre '{}' ya está en uso por otro recurso", o.nombre),
                    ));
                    diagnosticos.append(&mut d);
                    continue;
                }
                diagnosticos.append(&mut d);
                caracteres_usados.push(o.caracter);
                nombres_usados.push(o.nombre.clone());
                objetos.push(o);
            }
            (None, mut d) => {
                diagnosticos.append(&mut d);
            }
        }
    }
    (objetos, diagnosticos)
}

fn nombre_clonado(s: &str) -> String {
    s.to_string()
}

/// Carga todos los mapas de una carpeta, en orden alfabético. Los caracteres
/// desconocidos (que ningún terreno ni objeto declara) generan diagnóstico con
/// la línea de la fila y se descarta esa fila, pero no el mapa entero.
pub fn cargar_mapas_carpeta(
    ruta: &Path,
    terrenos: &[Terrain],
    objetos: &[Objeto],
) -> (Vec<Mapa>, Vec<Diagnostic>) {
    let mut mapas: Vec<Mapa> = Vec::new();
    let mut diagnosticos: Vec<Diagnostic> = Vec::new();

    let mut caracteres_validos: Vec<char> = terrenos.iter().map(|t| t.caracter).collect();
    caracteres_validos.extend(objetos.iter().map(|o| o.caracter));

    for path in listar_archivos_ordenados(ruta, "map") {
        let archivo = path.to_string_lossy().to_string();
        let contenido = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                diagnosticos.push(Diagnostic::new(
                    &archivo,
                    1,
                    format!("no se pudo leer el archivo: {e}"),
                ));
                continue;
            }
        };
        match cargar_mapa(&contenido, &archivo, &caracteres_validos) {
            (Some(m), mut d) => {
                diagnosticos.append(&mut d);
                mapas.push(m);
            }
            (None, mut d) => {
                diagnosticos.append(&mut d);
            }
        }
    }
    (mapas, diagnosticos)
}

/// Carga todos los recursos de un directorio raíz: `terrains/`, `objects/`,
/// `maps/` en ese orden. Las carpetas ausentes se tratan como vacías.
pub fn cargar_recursos(raiz: &Path) -> RecursosCargados {
    let mut diagnosticos: Vec<Diagnostic> = Vec::new();

    let dir_terrenos = raiz.join("terrains");
    let (terrenos, mut d) = cargar_terrenos_carpeta(&dir_terrenos);
    diagnosticos.append(&mut d);

    let dir_objetos = raiz.join("objects");
    let (objetos, mut d) = cargar_objetos_carpeta(&dir_objetos, &terrenos);
    diagnosticos.append(&mut d);

    let dir_mapas = raiz.join("maps");
    let (mapas, mut d) = cargar_mapas_carpeta(&dir_mapas, &terrenos, &objetos);
    diagnosticos.append(&mut d);

    RecursosCargados {
        terrenos,
        objetos,
        mapas,
        diagnosticos,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_terrain_example() {
        let contenido = "terrain Grass\nchar .\nresource_max 100\nregen 2\n";
        let (resultado, d) = cargar_terreno(contenido, "grass.te");
        assert!(d.is_empty(), "diagnósticos inesperados: {d:?}");
        let t = resultado.expect("debe cargar el terreno");
        assert_eq!(t.nombre, "Grass");
        assert_eq!(t.caracter, '.');
        assert_eq!(t.resource_max, 100);
        assert_eq!(t.regen, 2);
        assert_eq!(t.reserva, 100);
    }

    #[test]
    fn test_load_object_example() {
        let contenido = "object Rock\nchar #\nresource_max 100\n";
        let (resultado, d) = cargar_objeto(contenido, "rock.ob");
        assert!(d.is_empty(), "diagnósticos inesperados: {d:?}");
        let o = resultado.expect("debe cargar el objeto");
        assert_eq!(o.nombre, "Rock");
        assert_eq!(o.caracter, '#');
        assert_eq!(o.resource_max, 100);
        assert_eq!(o.reserva, 100);
    }

    #[test]
    fn test_char_hash_is_not_a_comment() {
        // Una línea que empieza por '#' es comentario, pero 'char #' no.
        let contenido = "object Rock\nchar #\nresource_max 100\n";
        let (resultado, d) = cargar_objeto(contenido, "rock.ob");
        assert!(d.is_empty(), "diagnósticos inesperados: {d:?}");
        let o = resultado.expect("debe cargar el objeto");
        assert_eq!(o.caracter, '#');
    }

    #[test]
    fn test_load_map_example() {
        // Mapa de 20 columnas x 7 filas con rocas en (3,1) y (4,1).
        let mut filas: Vec<Vec<char>> = Vec::new();
        for _ in 0..7 {
            filas.push(vec!['.'; 20]);
        }
        filas[1][3] = '#';
        filas[1][4] = '#';
        // Rocas adicionales del ejemplo para rellenar el mapa.
        filas[2][3] = '#';
        filas[2][4] = '#';
        filas[2][9] = '#';
        filas[3][10] = '#';
        filas[3][15] = '#';
        filas[3][16] = '#';
        filas[3][17] = '#';
        filas[4][17] = '#';
        filas[5][1] = '#';
        filas[6][1] = '#';
        filas[6][10] = '#';
        filas[6][11] = '#';

        let contenido = "Plain with rocks\n".to_owned()
            + &filas
                .iter()
                .map(|f| f.iter().collect::<String>())
                .collect::<Vec<_>>()
                .join("\n")
            + "\n";
        let (resultado, d) = cargar_mapa(&contenido, "plain.map", &['.', '#']);
        assert!(d.is_empty(), "diagnósticos inesperados: {d:?}");
        let m = resultado.expect("debe cargar el mapa");
        assert_eq!(m.nombre, "Plain with rocks");
        assert_eq!(m.filas.len(), 7);
        assert_eq!(m.filas[0].len(), 20);
        assert_eq!(m.filas[1][3], '#');
        assert_eq!(m.filas[1][4], '#');
        for (i, fila) in m.filas.iter().enumerate() {
            assert_eq!(fila.len(), 20, "fila {i} no tiene 20 columnas");
        }
    }

    #[test]
    fn test_map_unknown_char_reports_line() {
        // Solo '.' es un carácter válido; 'x' no lo está. Está en la línea 3
        // (fila índice 1 -> línea 3: 1 nombre + 1 fila + 1).
        let contenido = "Mapa raro\n..........\n...x......\n";
        let (_resultado, d) = cargar_mapa(contenido, "raro.map", &['.']);
        assert!(!d.is_empty(), "debe haber un diagnóstico");
        assert!(
            d.iter()
                .any(|diag| diag.linea == 3 && diag.mensaje.contains('x')),
            "diagnóstico en la línea 3 nombrando 'x', obtuvo: {d:?}"
        );
    }

    #[test]
    fn test_map_ragged_row_reports_line() {
        // Segunda fila más corta: línea 3.
        let contenido = "Mapa irregular\n..........\n....\n..........\n";
        let (_resultado, d) = cargar_mapa(contenido, "irregular.map", &['.']);
        assert!(!d.is_empty(), "debe haber un diagnóstico");
        assert!(
            d.iter().any(|diag| diag.linea == 3),
            "diagnóstico en la línea 3, obtuvo: {d:?}"
        );
    }

    #[test]
    fn test_resource_validation_diagnostics() {
        // Falta clave
        let (_, d) = cargar_terreno("terrain X\nchar .\nregen 2\n", "falta.ob");
        assert!(
            d.iter().any(|diag| diag.mensaje.contains("resource_max")),
            "falta resource_max: {d:?}"
        );
        // Valor no numérico
        let (_, d) = cargar_objeto("object X\nchar #\nresource_max abc\n", "numero.ob");
        assert!(
            d.iter().any(|diag| diag.mensaje.contains("resource_max")),
            "valor no numérico: {d:?}"
        );
        // char de más de un carácter
        let (_, d) = cargar_terreno("terrain X\nchar ab\nresource_max 10\nregen 1\n", "char.ob");
        assert!(
            d.iter().any(|diag| diag.mensaje.contains("char")),
            "char multi: {d:?}"
        );
        // resource_max <= 0
        let (_, d) = cargar_terreno("terrain X\nchar .\nresource_max 0\nregen 1\n", "max.ob");
        assert!(!d.is_empty(), "resource_max 0 debe fallar: {d:?}");
        // regen < 0
        let (_, d) = cargar_terreno("terrain X\nchar .\nresource_max 10\nregen -1\n", "regen.ob");
        assert!(!d.is_empty(), "regen < 0 debe fallar: {d:?}");
    }

    #[test]
    fn test_missing_directory_is_empty() {
        let (_t, dt) = cargar_terrenos_carpeta(Path::new("no_existe/terrains"));
        let (_o, do_) = cargar_objetos_carpeta(Path::new("no_existe/objects"), &[]);
        let (_m, dm) = cargar_mapas_carpeta(Path::new("no_existe/maps"), &[], &[]);
        assert!(dt.is_empty(), "carpeta ausente sin diagnósticos: {dt:?}");
        assert!(do_.is_empty(), "carpeta ausente sin diagnósticos: {do_:?}");
        assert!(dm.is_empty(), "carpeta ausente sin diagnósticos: {dm:?}");
    }

    #[test]
    fn test_char_repetido_descarta_archivo() {
        // Dos objetos con el mismo char '#': el primero se conserva, el
        // segundo se descarta con diagnóstico que nombra el char.
        use std::fs::{self as fs2};
        let temp = std::env::temp_dir().join("instinct_test_char_repetido");
        let _ = fs2::remove_dir_all(&temp);
        fs2::create_dir_all(&temp).expect("crear dir temp");
        fs2::write(temp.join("a.ob"), "object Rock\nchar #\nresource_max 100\n")
            .expect("escribir a.ob");
        fs2::write(temp.join("b.ob"), "object Otro\nchar #\nresource_max 50\n")
            .expect("escribir b.ob");
        let (objetos, d) = cargar_objetos_carpeta(&temp, &[]);
        assert_eq!(objetos.len(), 1, "solo el primero se conserva");
        assert_eq!(objetos[0].nombre, "Rock");
        assert!(
            d.iter().any(|diag| diag.mensaje.contains('#')),
            "diagnóstico nombra el char repetido: {d:?}"
        );
        let _ = fs2::remove_dir_all(&temp);
    }

    #[test]
    fn test_orden_alfabetico_primer_terreno() {
        // Dos terrenos: "b.te" declara "B", "a.te" declara "A". El primero en
        // orden alfabético de archivo es "a.te".
        let (resultado_a, _) =
            cargar_terreno("terrain A\nchar a\nresource_max 5\nregen 1\n", "a.te");
        let (_resultado_b, _) =
            cargar_terreno("terrain B\nchar b\nresource_max 5\nregen 1\n", "b.te");
        let ta = resultado_a.expect("a.te");
        assert_eq!(ta.nombre, "A");
        assert_eq!(ta.caracter, 'a');
    }
}
