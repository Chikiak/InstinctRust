---
phase: F6
title: Banco de criaturas de prueba
status: todo
master: ../MASTER.md
fuentes:
  - ../../docs/enunciado.md
  - ../../docs/ambiguedades.md
issues: 6 desglosados en archivos propios (F6-01 a F6-06)
---

# F6 · Banco de criaturas de prueba (reuso de examples/)

## Objetivo y entregable

Un paquete autocontenido — carpeta `examples/` con `terrains/`, `objects/`,
`maps/` y `creatures/` (reusando lo que ya existe en `examples/`, sin crear una
raíz `bank/` nueva) y `examples/traps/` con los recursos que deben fallar de
forma controlada — que se puede copiar tal cual a cualquier proyecto de
estudiante y correr desde la terminal, y un documento (`examples/README.md`)
que por cada scenario dice qué regla del enunciado prueba, el comando exacto y
qué debe verse. Cada resultado esperado queda fijado por una prueba automática de
esta implementación de referencia: si la referencia se desvía del enunciado, el
scenario lo documenta en lugar de esconderlo.

Entregable observable: `cargo run -p instinct-cli -- run --root examples --map
Arena --seed 1 --ticks 5 --place Sprinter@2,2` corre un scenario del banco con
exit 0 y la salida documentada; `cargo test -p instinct-cli --test cli_bank` y
`cargo test -p instinct-core --test bank` verifican cada scenario del README.

## Estado de partida

Verificado el 2026-11-21 sobre `develop @ aa75bf0` (rama limpia; F5-01 a F5-09
mergeados).

- Puertas: `cargo test --workspace` → exit 0;
  `cargo clippy --workspace --all-targets -- -D warnings` → exit 0;
  `cargo fmt --all --check` → exit 0.
- La CLI (`crates/instinct-cli/src/`) ofrece `run` con `--root`, `--map`,
  `--seed`, `--ticks`, `--every`, `--place` repetible y `check <archivo.ins>`.
  `run` imprime `== tick N ==` y la rejilla ASCII en stdout; los eventos van a
  stdout por el puerto; los diagnósticos a stderr; sale 0. `check` imprime `OK`
  o los diagnósticos y sale 1.
- El núcleo ya implementa las siete acciones, las 31 percepciones, `see` y
  `name`, fin de tick, errores de ejecución con línea y sin pánicos, y
  determinismo con semilla.
- Huecos confirmados (no existen hoy): `examples/traps/`,
  `crates/instinct-cli/tests/cli_bank/`, `crates/instinct-core/tests/bank/`.
  (`examples/` ya existe con sus recursos base.)

## Contratos compartidos

Los issues los referencian; no los reexplican.

- **Raíz del banco**: `examples/` con `terrains/`, `objects/`, `maps/` y
  `creatures/`, autocontenida (incluye su propio `grass.te` y `rock.ob` con los
  valores del anexo), de modo que cargue con **cero diagnósticos**. No se crea
  `bank/` nuevo: se reusa `examples/` cambiando únicamente la raíz de `--root`
  y las referencias de nombre/directorio.
- **Raíz de trampas**: `examples/traps/` con la misma forma, autocontenida;
  contiene recursos inválidos más al menos una criatura y un mapa válidos, para
  demostrar 2.8 («sigue funcionando con las demás criaturas»). Los scenarios
  válidos nunca la cargan.
- **Scenario**: unidad del banco. En `examples/README.md`, una entrada por
  scenario agrupada por sección del enunciado, con: regla que prueba, archivos
  que utiliza, comando exacto de terminal y resultado esperado. Toda entrada
  cita por nombre el test que la verifica; ningún scenario sin test.
- **Determinismo**: todo scenario usa semilla fija, por defecto `--seed 1`.
  Los scenarios cuyo resultado depende de `random` se marcan como estadísticos.
- **Resultado esperado = comportamiento de esta referencia**, obtenido ejecutando
  el scenario de verdad. Si contradice el enunciado, se anota en la sección
  Desviaciones del README y se reporta a F7; **no se corrige el núcleo en esta
  fase**.
- **Dos hogares de pruebas**:
  `crates/instinct-cli/tests/cli_bank/` para lo observable desde la terminal
  (binario real, patrón `run_cli` de `cli_run/helpers.rs`) y
  `crates/instinct-core/tests/bank/` para el estado interno que la terminal no
  muestra (reservas, edades, variables de la cría, `pc`). Cada uno con su bloque
  `[[test]]` en el `Cargo.toml` de su crate.
- **La fase no cambia `crates/*/src/`**: solo datos, pruebas, `[[test]]` y
  documentación. `git diff develop -- crates/` debe quedar vacío salvo los dos
  `Cargo.toml`.
- **Puertas por issue**: `cargo test --workspace` exit 0 ·
  `cargo clippy --workspace --all-targets -- -D warnings` exit 0 ·
  `cargo fmt --all --check` exit 0 · `git status` sin cambios tras el commit.

## Decisiones de la fase

1. **Banco sobre `examples/` existente, no `bank/` nuevo** — se reusa `examples/`
   como raíz (cambiando nombre/dir respecto al planteamiento original con `bank/`).
   `load_resources` no recursa subcarpetas y `examples/` tiene catálogos afirmados
   por pruebas de CLI y ventana. Reusar `examples/` evita duplicar los catálogos
   de recursos y es la raíz que un estudiante puede copiar entero tal cual.
2. **Trampas en `examples/traps/`** — mantiene los scenarios válidos con stderr
   vacío y convierte la carga tolerante en un scenario observable.
3. **Dos hogares de pruebas** — resultados como reservas de terreno, variables de
   la cría o edades no se ven desde la terminal; el README documenta lo observable
   y la prueba del núcleo fija el estado interno.
4. **`--seed 1` por defecto** en los scenarios; los estadísticos se marcan como tales.
5. **Sin correcciones al núcleo** — un bug que el banco descubra se reporta en las
   notas del issue y en Desviaciones, y se trata en F7.

## Issues

| ID | Título | Cubre | Depende de | Tamaño | Estado |
|----|--------|-------|------------|--------|--------|
| F6-01 | Esqueleto del banco: movimiento, bloqueos y sueño de `wait` | 2.4 move/wait, 2.7 turno | — | M | done |
| F6-02 | Golpes, consumo y reservas | 2.4 attack/consume, 3.1, 3.4 | F6-01 | M | done |
| F6-03 | Reproducción, aborto natural y ciclo de turnos | 2.4 reproduce, 2.7 | F6-01 | M | done |
| F6-04 | Percepciones, `see` y `name` | 2.6 | F6-01 | M | done |
| F6-05 | Expresiones, tipos y errores de ejecución | 2.5, 2.8 ejecución | F6-01 | M | todo |
| F6-06 | Rugidos y `say`, trampas de compilación, mapas y cierre | 2.4 roar/say, 2.6 social, 2.8 compilación, 3.3 | F6-01 | M | todo |

### Orden y dependencias

F6-01 va primero porque fija todo lo que los demás reutilizan: la forma de
`examples/`, la convención de scenario del README, los helpers de `cli_bank`/`bank`
y las puertas verdes. Los F6-02 a F6-06 tocan archivos casi disjuntos y su orden
es libre.

## Cobertura

| Entregable / criterio de salida | Issues que lo cubren |
|---------------------------------|----------------------|
| Estructura de `examples/` y convención de scenario | F6-01 |
| `move` y `wait` con bloqueos | F6-01 |
| `attack`/`consume` contra criatura, objeto, terreno, casilla propia y fuera del mapa | F6-02 |
| Leyes de fin de tick: -1 vida, +1 edad, muerte, regeneración, objetos agotados | F6-02 |
| `reproduce`, aborto natural, cría con edad 0 sin variables | F6-03 |
| Ciclo de turnos: reanudación, variables que persisten, límite de 100 líneas | F6-03 |
| Percepciones propias, de mundo y de cercanía; `see` con 5 constantes; `name` | F6-04 |
| Expresiones: precedencia, división y resto con suelo, verdad de textos | F6-05 |
| Errores de ejecución: muerte con línea y el resto de criaturas sigue | F6-05 |
| `roar` disponible el tick siguiente y `say` que no consume turno | F6-06 |
| Trampas de compilación rechazadas con línea; mapas y carga tolerante | F6-06 |
| Desviaciones frente al enunciado documentadas | F6-06 |

## Verificación de salida de la fase

- `cargo test --workspace` → exit 0 (incluye `cli_bank` y `bank`).
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
- `cargo run -p instinct-cli -- run --root examples --map Arena --seed 1 --ticks 5 --place Sprinter@2,2` → exit 0 y salida idéntica a la documentada en el scenario del README.
- `cargo test -p instinct-cli --test cli_bank` y `cargo test -p instinct-core --test bank` → exit 0; cada scenario de `examples/README.md` aparece citado en al menos un test que pasa.
- `cargo run -p instinct-cli -- check examples/traps/creatures/<trampa>.ins` → exit 1 con `archivo:línea: mensaje` en stderr, una por trampa.
- Dos corridas del mismo scenario con la misma semilla dan salida byte a byte idéntica.
- `git diff develop -- crates/` → vacío salvo los dos `[[test]]` en los `Cargo.toml`.
- [ ] Sesión de verificación independiente al cierre (recomendada).

## Prompt de sesión

```text
Implementa el issue plan/F6-banco-criaturas/f6-01-esqueleto-banco.md (o el que te toque).
Léelo entero antes de tocar código y empieza por su sección "Leer primero".
Esta fase construye datos y pruebas, no cambia el núcleo: si un scenario revela
que la referencia contradice el enunciado, documéntalo en "Desviaciones" del
README y repórtalo, no lo arregles en caliente. Ejecuta los comandos de
"Verificación" y no lo des por terminado hasta cubrir cada criterio de
aceptación con evidencia. Al acabar: actualiza `status` y rellena
"Notas de implementación". Si el issue contradice el repo o te bloquea, para y
repórtalo en lugar de improvisar.
```

## Registro de ejecución

| Issue | Enfoque | Coste asumido | Estado | Medición |
|-------|---------|---------------|--------|----------|
|  |  |  |  |  |