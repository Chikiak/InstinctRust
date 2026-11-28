# InstinctRust

Implementación de referencia en Rust del proyecto **Instinto**: un compilador e
intérprete del lenguaje Instinct (archivos `.ins`), un simulador de mundo por
ticks que corre sin interfaz, y una ventana para observarlo. Sirve para crear y
probar criaturas, terrenos, objetos y mapas con los que comprobar proyectos de
estudiantes, y para aprender Rust.

## Requisitos

- Toolchain estable de Rust (el workspace usa `edition = 2021` y `resolver = "2"`).

## Compilar

```bash
cargo build --release
```

## Estructura

Workspace de tres crates:

| Crate | Qué contiene |
|---|---|
| `instinct-core` | Lenguaje (lexer, parser/AST, intérprete) y mundo/simulador. Sin dependencias externas, sin `f32/f64` (solo `i64` y texto), sin `rand`, sin `println!` (el mundo emite eventos) y sin pánicos con entrada inválida. |
| `instinct-cli` | Terminal: `run` y `check`. |
| `instinct-gui` | Ventana (`eframe`/`egui`) que consume el núcleo. |

## Correr una simulación desde la terminal

```bash
cargo run -p instinct-cli -- run --root examples --map "Plain with rocks" --place Dwarf@1,1 --ticks 5 --seed 7
```

Opciones de `run`: `--root <dir>` (raíz de recursos), `--map <nombre>`,
`--place <Especie>@<x>,<y>` (repetible), `--seed <n>`, `--ticks <n>`,
`--every <n>`. Salida de ejemplo (primer tick):

```text
== tick 0 ==
....................
.D.##...............
...##.....#.........
..........#....###..
...............#....
.#..................
.#........##........
```

Para comprobar una criatura sin simular:

```bash
cargo run -p instinct-cli -- check examples/creatures/dwarf.ins
# OK
```

## Ventana

```bash
cargo run -p instinct-gui
```

## Banco de pruebas

`examples/` es la raíz del banco: 49 criaturas, 3 mapas, 2 objetos y 2
terrenos, más `examples/traps/` con recursos que deben fallar de forma
controlada. Cada scenario de `examples/README.md` dice qué regla del enunciado
prueba, el comando exacto y qué debe verse; las suites
`cargo/instinct-core/tests/bank/` y `cargo/instinct-cli/tests/cli_bank/` los
verifican.

## Pruebas

```bash
cargo test --workspace
```

## Estado

Referencia en desarrollo (v0.1.0 en preparación). Limitaciones conocidas y
desviaciones documentadas, en su caso, en `examples/README.md` y en el plan
vigente.