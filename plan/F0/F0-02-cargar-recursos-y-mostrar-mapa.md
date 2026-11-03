---
id: F0-02
title: Cargar recursos y mostrar un mapa
phase: F0
status: done
depends_on: [F0-01]
size: M
---

# F0-02 · Cargar recursos y mostrar un mapa

## Objetivo
Que el usuario pueda cargar los terrenos, objetos y mapas de una carpeta de recursos y ver un mapa en ASCII desde la terminal. Fija el modelo de mundo (rejilla, terrenos, objetos) y el comportamiento de carga resiliente sobre el que se construye todo lo demás.

## Alcance
**Haz**
- Cargadores de `.te`, `.ob` y `.map` según el formato de la fase, devolviendo recursos cargados y una lista de diagnósticos.
- `World` con rejilla de terreno (con su reserva) y objetos (con su reserva), y una función que lo dibuja en ASCII.
- Subcomando `run` del CLI solo con `--root`, `--map` y `--ticks` (aceptado pero sin efecto aún).
- Recursos de ejemplo: `grass.te`, `rock.ob` y `plain.map`.

**No hagas**
- Criaturas, ticks ni percepciones (F0-04 en adelante).
- Cargar la carpeta `creatures/` (F0-06).
- Dependencias externas.

### Archivos previstos
- `crates/instinct-core/src/resources.rs` — nuevo — lectura de los tres formatos y carga por carpetas.
- `crates/instinct-core/src/world.rs` — nuevo — rejilla, catálogo de terrenos y objetos, dibujo ASCII.
- `crates/instinct-cli/src/main.rs` — modificar — subcomando `run`.
- `crates/instinct-cli/tests/cli_run.rs` — nuevo.
- `examples/terrains/grass.te`, `examples/objects/rock.ob`, `examples/maps/plain.map` — nuevos — copiados del enunciado.

## Criterios de aceptación
- [ ] **AC1** `grass.te` carga como terreno `Grass` con carácter `.`, reserva máxima 100 y regeneración 2. Verif: `test_load_terrain_example`
- [ ] **AC2** `rock.ob` carga como objeto `Rock` con carácter `#` y reserva máxima 100; el `#` como valor de `char` no se toma por comentario, y una línea que empieza por `#` sí lo es. Verif: `test_load_object_example`, `test_char_hash_is_not_a_comment`
- [ ] **AC3** `plain.map` carga con nombre "Plain with rocks", 20 columnas y 7 filas, con rocas en (3,1) y (4,1) y terreno `Grass` debajo de todas las casillas. Verif: `test_load_map_example`
- [ ] **AC4** Un mapa con un carácter que ningún terreno ni objeto declara, o con una fila de largo distinto, produce un diagnóstico con la línea de esa fila (el del carácter nombra el carácter) y no impide cargar los demás recursos. Verif: `test_map_unknown_char_reports_line`, `test_map_ragged_row_reports_line`
- [ ] **AC5** Un `.te` o `.ob` al que le falta una clave, con valor no numérico donde se espera número, con `char` de más de un carácter, o con un `char` repetido por otro recurso ya cargado produce un diagnóstico con su línea y ese archivo se descarta. Verif: `test_resource_validation_diagnostics`
- [ ] **AC6** Una carpeta ausente se carga como vacía sin diagnóstico. Verif: `test_missing_directory_is_empty`
- [ ] **AC7** `instinct-cli run --root examples --map "Plain with rocks" --ticks 0` imprime `== tick 0 ==` y las siete filas del mapa idénticas al archivo, y sale con 0; un nombre de mapa inexistente escribe por stderr los mapas disponibles y sale con 1. Verif: `test_cli_prints_map`, `test_cli_unknown_map_exits_1`
- [ ] **AC8** La suite completa, clippy y fmt terminan en verde. Verif: los tres comandos de la sección Verificación

## Contexto
**Ya existe** (F0-01): workspace, `Diagnostic` en `crates/instinct-core/src/diagnostic.rs`, CLI con uso y `--help` en `crates/instinct-cli/src/main.rs`.
**Contratos que debes respetar**: formatos de archivo, orden de carga y comandos en `../PHASE.md#contratos-compartidos`. Las ambigüedades A1, A2 y A12 se aplican aquí.
**Ojo**: el `#` es a la vez el carácter de la roca y el inicio de comentario; el mapa no tiene comentarios. El enunciado muestra los ejemplos con una barra invertida antes del `#` inicial por el formato Markdown: los archivos reales no la llevan. El primer terreno declarado es el primero en orden alfabético por nombre de archivo.
**Rust que se practica**: lectura de archivos con `?`, `Result`, `String` y `&str`, iteradores sobre líneas, `Vec` de `Vec`, propiedad de datos cargados.

### Leer primero
- `crates/instinct-core/src/diagnostic.rs` — cómo reportar errores con línea.
- `crates/instinct-cli/src/main.rs` — cómo se organiza el CLI para añadir `run`.
- `docs/enunciado.md` — secciones 3.1 a 3.3.
- `docs/ambiguedades.md` — filas A1, A2 y A12.

## Especificación
| Recurso | Claves | Validación |
|---|---|---|
| Terreno | `terrain`, `char`, `resource_max`, `regen` | `resource_max` mayor que 0, `regen` mayor o igual que 0 |
| Objeto | `object`, `char`, `resource_max` | `resource_max` mayor que 0 |
| Mapa | primera línea nombre; filas | Todas las filas del mismo largo; al menos una fila |

- El terreno empieza con la reserva llena; el objeto, con `resource_max`.
- El carácter de dibujo de una casilla es el del objeto si lo hay, y si no, el del terreno.
- Si dos archivos declaran el mismo nombre, se conserva el primero y el segundo genera diagnóstico.
- `run` con un mapa válido imprime el tick 0; `--ticks` se lee y se valida como entero pero aún no simula.

## Tests
Unitarios en `resources.rs` y `world.rs` sobre cadenas en memoria y sobre los archivos de `examples/`; integración del CLI contra el binario. No mockees el sistema de archivos para los ejemplos reales. Escribe primero los tests de AC2 y AC4.

## Verificación
- `cargo test -p instinct-core resources` → esperado: tests de AC1 a AC6 en verde.
- `cargo test -p instinct-cli` → esperado: tests de AC7 en verde.
- `cargo test --workspace` → esperado: sin regresiones.
- `cargo clippy --workspace --all-targets -- -D warnings` y `cargo fmt --all --check` → esperado: exit 0.

**Terminado** = todos los AC cumplidos con evidencia + comandos en verde (suite completa) + sin stubs/TODOs/código muerto + sin cambios fuera de alcance.

## Si te bloqueas
Si `Diagnostic` no encaja con lo que necesitas, repórtalo en las notas en lugar de cambiar su contrato. No decidas tú el algoritmo de colisión de nombres más allá de lo escrito, ni añadas formatos nuevos.

## Cierre
- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Añade a `docs/ambiguedades.md` cualquier decisión nueva.
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación

**AC1 PASS** — `cargo test -p instinct-core resources` → `test_load_terrain_example` ok. `grass.te` carga como `Terrain { nombre: "Grass", caracter: '.', resource_max: 100, regen: 2, reserva: 100 }`. Archivos: `crates/instinct-core/src/resources.rs` (`cargar_terreno`), `examples/terrains/grass.te`.

**AC2 PASS** — `test_load_object_example` + `test_char_hash_is_not_a_comment` ok. `rock.ob` carga como `Objeto { nombre: "Rock", caracter: '#', resource_max: 100 }`. `char #` no se toma por comentario (la línea no inicia por `#`); una línea cuyo primer no-blanco es `#` sí es comentario (`leer_claves`/`separar_clave_valor`).

**AC3 PASS** — `test_load_map_example` ok. `plain.map` → nombre "Plain with rocks", 7 filas × 20 columnas, rocas en (3,1) y (4,1). `World::desde_mapa` resuelve objeto sobre el primer terreno (`crates/instinct-core/src/world.rs`).

**AC4 PASS** — `test_map_unknown_char_reports_line` + `test_map_ragged_row_reports_line` ok. `cargar_mapa(contenido, archivo, caracteres_validos)` reporta diagnóstico con la línea de la fila para carácter desconocido o fila de largo distinto; no impide cargar el resto.

**AC5 PASS** — `test_resource_validation_diagnostics` + `test_char_repetido_descarta_archivo` ok. Clave faltante, valor no numérico, `char` >1 carácter, `resource_max` <= 0, `regen` < 0 y `char` repetido por otro recurso → diagnóstico con línea y el archivo se descarta (primero se conserva en orden alfabético).

**AC6 PASS** — `test_missing_directory_is_empty` ok. Carpeta ausente (`terrains/`, `objects/`, `maps/`) se carga como vacía sin diagnóstico (`listar_archivos_ordenados` ignora directorios ausentes).

**AC7 PASS** — `cargo test -p instinct-cli` → `test_cli_prints_map` (`run --root examples --map "Plain with rocks" --ticks 0` imprime `== tick 0 ==` y las 7 filas, salida 0) + `test_cli_unknown_map_exits_1` (mapa inexistente escribe mapas disponibles por stderr, salida 1). CLI: `crates/instinct-cli/src/main.rs` (subcomando `run`, flags `--root/--map/--seed/--ticks/--every/--place`, `--ticks` validado como entero) + `crates/instinct-cli/tests/cli_run.rs`.

**AC8 PASS** — `cargo test --workspace` (26 tests ok), `cargo clippy --workspace --all-targets -- -D warnings` (exit 0), `cargo fmt --all --check` (exit 0).

Tipos nuevos: `Terrain`, `Objeto`, `Mapa`, `RecursosCargados` en `resources.rs`; `World`, `ContenidoCasilla` en `world.rs`. Funciones clave: `cargar_recursos`, `cargar_terrenos_carpeta`, `cargar_objetos_carpeta`, `cargar_mapas_carpeta`, `cargar_terreno`, `cargar_objeto`, `cargar_mapa`, `World::desde_mapa`, `World::dibujar`. Orden de carga: terrenos, objetos, mapas; alfabético por archivo (A1). Carpeta ausente = vacía (A2). Sin `unwrap`/`expect` sobre datos de usuario; sin `rand`; sin `println!` en el núcleo; sin floats.
