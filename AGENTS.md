# AGENTS.md — convenciones para agentes en InstinctRust

## Workspace

Tres crates: `instinct-core` (lenguaje y mundo, sin interfaz), `instinct-cli`
(terminal) e `instinct-gui` (ventana). La CLI y la GUI solo consumen el núcleo.

## Reglas del núcleo (no negociables)

- **Sin dependencias externas** en `instinct-core`.
- **Sin `f32`/`f64`**: valor = entero con signo de 64 bits o texto.
- **Sin `rand`**: un solo generador propio y sembrado, dentro del mundo.
- **El mundo no imprime**: emite eventos y registro que consumen CLI y GUI.
- **Ninguna entrada puede provocar un pánico** en el núcleo.
- **Diagnóstico único** con archivo y línea (desde 1) para toda la pila
  (lexer, parser, intérprete, mundo).
- **Entidades por identificador estable**: la rejilla guarda IDs, no referencias.
- Cada patrón de arquitectura se aplica una vez y solo cuando el juego de
  características que toca está completo; nada de refactors especulativos.

## Verificación esperada (antes de proponer cualquier PR)

```bash
cargo test --workspace                          # exit 0
cargo clippy --workspace --all-targets -- -D warnings   # exit 0
cargo fmt --all --check                         # exit 0
```

Para trabajo observable desde la terminal, añade el smoke del camino feliz
(`run --root examples --map "Plain with rocks" --place Dwarf@1,1 --ticks 5`).

## Flujo de trabajo

- **Un issue, una rama** `feat/<slug>` cortada del tip del stack; PR apilado.
- **Integración solo por PR**: ningún merge local y ningún merge a `main` sin
  revisión humana diurna.
- **Trabaja en un worktree aislado**, nunca sobre el checkout personal: un
  cambio de rama con archivos sin commit ya provocó pérdida de trabajo.
- **Nunca hagas push sin autorización humana explícita** (la autorización puede
  ser para una rama de PR concreta).
- **Planes nuevos** (planes de fase, issues, matrices, notas): en rutas `.local`
  ignoradas por git (p. ej. `plans.local/`), no en `plan/` rastreado.
- TDD estricto en issues de código: test que falla primero (RED), cambio mínimo
  (GREEN), refactor sin romper contratos.
- Commits modulares con Conventional Commits, separando código, docs y archivos
  de plan; `git status` debe quedar limpio.