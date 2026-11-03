---
id: F0-04
title: Colocar criaturas y aplicar acciones
phase: F0
status: todo
depends_on: [F0-02]
size: M
---

# F0-04 · Colocar criaturas y aplicar acciones

## Objetivo
Que el mundo pueda tener criaturas colocadas sobre el terreno y que las acciones `wait`, `move` y `say` actúen sobre él con sus reglas exactas. Fija el trait `Action` y el catálogo, de modo que añadir una acción sea escribir un tipo más.

## Alcance
**Haz**
- `Species` (sin programa aún) y almacén de criaturas con identificador estable; casilla ocupada por objeto o criatura, nunca ambos.
- Colocar y quitar criaturas con validación; dibujo ASCII con la criatura como primera letra de su especie; contador de ticks en el mundo con su avance.
- Trait `Action`, `Catalog` de acciones por nombre y aridad, y los tipos `wait`, `move` y `say`.
- Tipo `Event` con `Said` y lista de eventos en el mundo.

**No hagas**
- Percepciones, variables ni intérprete (F0-05 y F0-07).
- Resto de acciones (F1).
- Lectura de especies desde archivos (F0-06).

### Archivos previstos
- `crates/instinct-core/src/world.rs` — modificar — criaturas, ocupación, ticks, eventos.
- `crates/instinct-core/src/species.rs` — nuevo — `Species`.
- `crates/instinct-core/src/actions.rs` — nuevo — trait, catálogo y las tres acciones.
- `crates/instinct-core/src/event.rs` — nuevo — `Event`.

## Criterios de aceptación
- [ ] **AC1** Colocar una criatura en una casilla libre funciona; colocarla fuera del mapa, sobre un objeto o sobre otra criatura devuelve un `Diagnostic` que nombra las coordenadas y no cambia el mundo. Verif: `test_place_rejects_invalid_cells`
- [ ] **AC2** El dibujo ASCII muestra la primera letra de la especie en la casilla de la criatura y el resto igual que antes. Verif: `test_render_shows_creature`
- [ ] **AC3** `move` avanza una casilla por vez hasta `speed` y se detiene antes de la primera casilla fuera del mapa, con objeto o con criatura; cubre derecha, diagonal, bloqueo inmediato, borde y velocidad mayor que el recorrido libre. Verif: `test_move_stops_before_obstacles` (tabla de casos)
- [ ] **AC4** `move` con dirección (0, 0) o `speed` menor o igual que 0 no mueve y no falla; con dx o dy fuera de {-1, 0, 1}, o con algún argumento de texto, falla con un mensaje que nombra el argumento. Verif: `test_move_argument_errors`
- [ ] **AC5** `wait` con n devuelve n-1 turnos de sueño; con n menor que 1 cuenta como 1 (cero de sueño); con texto falla. Verif: `test_wait_sleep_turns`
- [ ] **AC6** `say` emite `Said` con el tick actual, la especie, la posición y el texto (un número se muestra como texto) y su propiedad `consume turno` es falsa; las otras dos acciones la tienen verdadera. Verif: `test_say_emits_event_and_is_free`
- [ ] **AC7** El catálogo resuelve las tres acciones por nombre con su aridad (`wait` 1, `move` 3, `say` 1) y no conoce otras; un test registra una acción extra definida solo en el test sin tocar los tipos existentes. Verif: `test_catalog_lookup_and_extension`
- [ ] **AC8** Suite completa, clippy y fmt en verde. Verif: los comandos de Verificación

## Contexto
**Ya existe** (F0-02): `World` en `crates/instinct-core/src/world.rs` con rejilla, terreno y objetos y su dibujo ASCII; `Diagnostic`; `Rng` en `crates/instinct-core/src/rng.rs`.
**Contratos que debes respetar**: acciones, eventos, fin de tick y semántica de `move` y `say` en `../PHASE.md#contratos-compartidos`. Almacén por identificador, no por referencias: es decisión fijada del maestro.
**Ojo**: la acción recibe argumentos ya evaluados como `Value` y decide su validez; el intérprete añadirá la línea a los errores más adelante. Aplicar una acción necesita modificar el mundo y la criatura que la ejecuta: pasa el identificador de la criatura, no una referencia a ella, para no chocar con el préstamo del mundo. Añadir una variante de ocupante obliga a revisar los `match` existentes: es esperado. El mundo posee también el `Rng`, aunque aquí aún no se use.
**Rust que se practica**: identificadores estables en lugar de referencias, traits con `Rc<dyn ...>`, `match` exhaustivo, `&mut` sobre una sola estructura.

### Leer primero
- `crates/instinct-core/src/world.rs` — estructura de la rejilla y su dibujo.
- `crates/instinct-core/src/diagnostic.rs` — errores con coordenadas.
- `crates/instinct-core/src/rng.rs` — generador que el mundo debe poseer.
- `docs/ambiguedades.md` — A13.

## Especificación
- Las acciones son tipos sin estado; el catálogo las guarda como referencias compartidas de un solo hilo.
- Propiedades de `Action`: nombre, aridad, consume turno, y aplicar con los valores recibidos sobre el mundo y una criatura, devolviendo los turnos de sueño (cero en `move` y `say`).
- `say` pone en el evento la posición de la criatura en ese momento.
- La casilla del `move` solo se pisa si es terreno sin objeto ni criatura; el borde del mapa cuenta como bloqueo.

## Tests
Unitarios sobre un mundo pequeño construido en el test (puede usar un terreno y una roca declarados en el test). Prueba movimiento con tabla de casos. No mockees el mundo: es la pieza que se verifica.

## Verificación
- `cargo test -p instinct-core actions` y `cargo test -p instinct-core world` → esperado: AC1 a AC7 en verde.
- `cargo test --workspace` → esperado: sin regresiones.
- `cargo clippy --workspace --all-targets -- -D warnings` y `cargo fmt --all --check` → esperado: exit 0.

**Terminado** = todos los AC cumplidos con evidencia + comandos en verde (suite completa) + sin stubs/TODOs/código muerto + sin cambios fuera de alcance.

## Si te bloqueas
Si no ves cómo aplicar una acción sin dos préstamos mutables simultáneos, para y repórtalo con el mensaje del compilador; no uses `Rc<RefCell>` por todo el mundo ni cambies a referencias dentro de la rejilla: es decisión fijada. Tampoco decidas tú qué hace `move` ante casos no listados.

## Cierre
- Actualiza `status` aquí y en la tabla de `PHASE.md`.
- No edites Alcance ni Criterios de aceptación (si están mal, repórtalo abajo).
- Añade a `docs/ambiguedades.md` cualquier decisión nueva.
- Termina con un resumen: qué comandos ejecutaste y su resultado, AC por AC.

### Notas de implementación
