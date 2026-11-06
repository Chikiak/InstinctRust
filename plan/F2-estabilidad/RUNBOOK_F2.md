# Runbook — Sesión de implementación de la Fase 2 (Estabilidad: bugs, contratos y pruebas)

> Documento operativo para la ejecución desatendida de los issues de la fase F2 con
> un orquestador y subagentes. **No modifica el plan**: los issues, sus contratos
> (`plan\F2-estabilidad\PHASE.md`) y sus criterios de aceptación siguen siendo la
> fuente de verdad durante la implementación.

## 1. Objetivo

Ejecutar los ocho issues de F2 en el orden que marca su grafo de dependencias
(`PHASE.md` §Orden y dependencias), con un subagente por issue, sobre ramas por issue
mergeadas a `develop` al cerrar en verde. La sesión corre hasta terminar la fase,
hasta que un issue quede bloqueado o hasta que el usuario la pare. Sin hora de corte
fija.

A diferencia de F1 (cadena estricta), F2 tiene **dos raíces independientes**: F2-01 y
F2-02 no dependen de nada, y F2-06 tampoco; el resto se encadenan desde ellas. Un
issue bloqueado en una rama no invalida la otra.

## 2. Agentes

| Agente | Modo | Permisos clave | Se despacha para |
|---|---|---|---|
| `orquestador-f2` | primary | `task: allow`, `edit: allow`, sin web | — |
| `implementador-rust` | subagent | edición y ejecución totales, sin web ni task | implementar cada issue (intento 1) |
| `rescatista-rust` | subagent | igual que implementador | terminar un issue que el implementador dejó en rojo |
| `verificador-rust` | subagent | **sin `edit`** (solo lectura + bash) | verificación independiente final |
| `reviewer` | subagent | auditoría estática de pánicos, determinismo y semántica del lenguaje | auditoría final del diff |

Regla de contexto del orquestador: nunca lee código; solo ledger, estados por `grep`,
informes de subagente y salidas de comandos baratos.

## 3. Orden, rutas y ramas

Orden de ejecución propuesto (las tres raíces pueden ir en cualquier orden; aquí se
pone primero la que más desbloquea):

| # | Issue | Archivo | Rama | Depende de | Estimado |
|---|-------|---------|------|------------|----------|
| 1 | F2-02 Parser: argumentos propagados, asignación robusta, operadores y header | `plan\F2-estabilidad\F2-02-parser-args-asignacion-headers.md` | `feat/f2-02-parser-args` | — | 2–3 h |
| 2 | F2-03 Errores unificados y cero unwrap/expect | `plan\F2-estabilidad\F2-03-errores-unificados-sin-unwrap.md` | `feat/f2-03-errores-unificados` | F2-02 | 3–4 h |
| 3 | F2-01 Lexer: contrato de líneas, escapes y ASCII | `plan\F2-estabilidad\F2-01-lexer-lineas-escapes-ascii.md` | `feat/f2-01-lexer-contrato` | — | 2–3 h |
| 4 | F2-06 Mundo y tick: aritmética comprobada y coordenadas | `plan\F2-estabilidad\F2-06-mundo-tick-aritmetica-coordenadas.md` | `feat/f2-06-mundo-aritmetica` | — | 2–3 h |
| 5 | F2-04 Etiquetas deterministas y pruebas del núcleo | `plan\F2-estabilidad\F2-04-etiquetas-deterministas-nucleo-tipos.md` | `feat/f2-04-labels-deterministas` | F2-02, F2-03 | 2 h |
| 6 | F2-05 Intérprete: `pc` válido y límite de líneas | `plan\F2-estabilidad\F2-05-interprete-pc-y-limite.md` | `feat/f2-05-interp-pc-limite` | F2-03 | 2–3 h |
| 7 | F2-07 Catálogos con un solo dueño | `plan\F2-estabilidad\F2-07-catalogos-un-dueno.md` | `feat/f2-07-catalogos-dueno-unico` | F2-03, F2-06 | 4–5 h |
| 8 | F2-08 CLI: validación, flujos y pruebas | `plan\F2-estabilidad\F2-08-cli-validacion-y-flujos.md` | `feat/f2-08-cli-validacion` | F2-03, F2-07 | 3–4 h |

Grafo de dependencias (ver `PHASE.md` §Orden y dependencias):

```
F2-02 ──┬──> F2-03 ──┬──> F2-04
        │            ├──> F2-05
F2-01 ──┘            │
F2-06 ───────────────┴──> F2-07 ──> F2-08
```

Ningún ciclo. `git` fusión en el orden de la tabla; si el orquestador decide otra
raíz primero, basta con mantener el grafo.

## 4. Pre-vuelo del orquestador (una vez)

```powershell
git status --short                      # debe estar limpio
git checkout develop
cargo build --workspace                 # compila sin warnings
cargo test --workspace                  # suite verde (F1 ya entregada)
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

Después se crea el ledger (§5) con la hora de inicio. El "warm-up" es la suite del
workspace y la compilación de los dos crates.

## 5. Ledger (`plan\F2-estabilidad\NOCHE_F2.md`)

Único artefacto que el orquestador escribe durante la noche. Formato:

```markdown
# Ledger F2 — inicio HH:mm (<fecha>)
Pre-vuelo: OK | FALLO (<motivo>)

| Hora | Evento | Issue | Intento | Estado | Commit/rama |
|------|--------|-------|---------|--------|-------------|
| HH:mm | preflight | — | — | OK | rama base develop |
| HH:mm | dispatch | F2-02 | 1 rust | running | feat/f2-02-parser-args |
| HH:mm | done | F2-02 | 1 | 7/7 AC | a1b2c3d → merge d4e5f6g |
| HH:mm | contract-flag | F2-03 | 1 rust | firma de draw cambiada | misma sesión |
| HH:mm | blocked | F2-0X | — | <motivo> | rama descartada |
```

Eventos propios de esta fase, además de los de F1:

- `contract-flag`: el issue cambió una firma autorizada (`lex`, `LexError`, `draw`,
  catálogos). Se anota **qué** cambió, porque los issues siguientes la asumen.
- `tests-adjusted`: pruebas existentes ajustadas por un cambio de contrato; se anota
  cuáles, para que el verificador las revise y no las tome como regresión escondida.

**Regla de reanudación:** si la sesión del orquestador se compacta, relea este ledger
y el §6 antes de despachar nada.

## 6. Bucle orquestado

1. `git checkout -b feat/f2-0X-<slug>` (rama desde `develop`).
2. Despacho del implementador (§7.1).
3. Al volver el informe, verificación barata **en este orden**:
   `git log --oneline -2` (¿commit?) → `git status --short` (¿árbol limpio?) →
   `grep -n "^status:"` del issue (¿done?) → informe (¿todos los AC PASS?).
4. Verde: merge `--no-ff` a `develop`, borrar la rama, línea en el ledger → siguiente.
5. Rojo: clasificar el fallo (§8) y actuar. Nunca reintentar a ciegas.

## 7. Plantillas de despacho

### 7.1 Implementador (intento 1, todos los issues)

```text
Implementa el issue plan\F2-estabilidad\<ID>-<slug>.md (intento 1/1).
Léelo entero antes de tocar código y empieza por su sección "Leer primero".
Respeta los contratos de plan\F2-estabilidad\PHASE.md (el issue los referencia) y lee
también las "Notas de implementación" de los issues de los que depende: <DEPENDENCIAS>.
Esta fase corrige bugs y contratos: si un AC te obliga a cambiar una firma que PHASE.md
autoriza (lex, LexError, World::draw, end_of_tick_terrain_and_objects, Simulation::new),
hazlo y documéntalo en las notas; si te obliga a cambiar cualquier otro contrato, NO lo
hagas: repórtalo.
Aplica TDD: primero el test que falle, después el código.
Ejecuta TODOS los comandos de la sección "Verificación" desde la raíz del workspace; no
lo des por terminado hasta cubrir cada criterio de aceptación con evidencia real.
Al acabar en verde: commit ("<tipo>: <resumen> (<ID>)"), actualiza status en el issue y
en la tabla de PHASE.md, y rellena "Notas de implementación" con la evidencia AC por AC
más la lista de pruebas existentes ajustadas.
Si te bloqueas o el issue contradice el repo: NO comitas, deja el árbol como esté y
repórtalo. No cambies alcance ni criterios.
Devuelve MÁXIMO 20 líneas: AC por AC (PASS/FAIL + comando), commit (hash) o
"sin commit", bloqueos concretos, firmas cambiadas.
```

`<tipo>`: `fix` para correcciones de comportamiento, `test` si solo hay pruebas,
`feat` para la feature de escapes del lexer, `refactor` para cambios de firma sin
cambio de comportamiento.

**Restricciones críticas del núcleo InstinctRust:**

- `crates\instinct-core` no tiene dependencias externas ni referencias a la interfaz:
  nada de crates ajenos (el crate `rand` está prohibido), nada de `println!` (el mundo
  emite eventos), nada de `f32`/`f64` (el lenguaje solo tiene enteros `i64` o texto).
- Cero pánicos con entradas inválidas: ni `unwrap`/`expect` sobre datos del usuario ni
  sobre archivos, ni indexar fuera de rango. División/resto por cero y desbordamiento
  se detectan y se convierten en errores de ejecución. **Esta fase existe para
  eliminar los que quedan**: un issue que deje un `unwrap` sobre datos de entrada está
  en rojo aunque sus tests pasen.
- Estado del mundo: identificadores estables y almacenes; la rejilla guarda
  identificadores, no referencias. `Value` es entero `i64` o texto.
- Las pruebas no se suavizan: si un AC exige una aserción concreta, no se cambia por
  una más laxa para que pase.

### 7.2 Reanudación por fallo de infraestructura (mismo sessionID, mismo modelo)

```text
Reanuda: el intento previo se cortó por un fallo de infraestructura (<motivo:
build/toolchain/timeout>), no por el trabajo en sí. Relee el issue, comprueba el
estado del árbol (git status) y continúa donde lo dejaste. Mismas reglas de
cierre y mismo formato de informe.
```

### 7.3 Rescatista Rust (fallo del modelo, un solo intento)

```text
Retoma el issue plan\F2-estabilidad\<ID>-<slug>.md. Un intento previo con otro
modelo falló; tu misión es terminarlo, no rehacerlo desde cero si lo existente
aprovecha.
Resumen del fallo previo (≤10 líneas): <RESUMEN>.
Lee el issue entero + contratos de PHASE.md + las notas del intento previo; evalúa
qué AC están ya cubiertos y qué falta, incluida la ausencia de unwraps nuevos.
Ejecuta la verificación real de cada AC y cierra con las mismas reglas (commit,
status, notas). Informe ≤20 líneas.
```

### 7.4 Verificador independiente (al cierre)

```text
Verifica de forma independiente los issues marcados done de esta sesión:
<LISTA DE ARCHIVOS>. Para cada uno: lee el issue, contrasta cada AC contra el repo
(lectura + ejecución de sus comandos de "Verificación"), comprueba commit, status en
issue y PHASE.md, y notas rellenas. Presta atención especial a:
- las pruebas existentes ajustadas (deben estar citadas en las notas, no regresiones silenciosas);
- los unwraps/expect sobre datos de usuario o fichero en instinct-core/src e instinct-cli/src;
- que los cambios de firma autorizados se propagaron a todos los sitios de llamada.
Tabla final ≤20 líneas: issue | AC por AC PASS/FAIL con comando | commit sí/no | veredicto.
No modifiques nada.
```

### 7.5 Auditoría del reviewer (cierre obligatorio)

```text
Audita el diff completo de la sesión F2. Revisa: pánicos sobre entradas inválidas
(unwrap/expect/index fuera de rango), aritmética sin checked donde había desbordamiento
posible, casts a usize sin comprobar signo, orden de iteración dependiente de HashMap,
cadenas de if por nombre en lugar de catálogo, println! en el núcleo, aserciones de test
suavizadas, dependencias externas en instinct-core, f32/f64 en cualquier lado, y
cumplimiento del contrato zero-panic. Comprueba además que los cambios de firma
autorizados no dejaron sitios de llamada sin actualizar. Devuelve un veredicto
[APROBADO] o [RECHAZADO] con hallazgos concretos. No se da por terminada la tarea sin
este reporte.
```

## 8. Política de fallos

| Tipo de fallo | Cómo reconocerlo | Acción | Tope |
|---|---|---|---|
| **Infraestructura / toolchain / build** | Error al invocar el subagente, timeout, red, fallo de `cargo build`/`cargo test` | Reanudar la **misma sesión** del subagente (§7.2), mismo modelo | 5 reanudaciones → `blocked` |
| **Del modelo** | AC en rojo, trabajo incompleto, stubs o contradicción no reportada | Despachar **rescatista-rust** una vez (§7.3) sobre el mismo árbol | 1 intento → si falla: `blocked` y parar la cadena |
| **Contrato mal decidido** | El AC es inalcanzable sin romper un contrato que `PHASE.md` no autoriza | Parar ese issue, dejar `status: todo`, reportar al usuario con la contradicción concreta; no mergear | 1 reporte |
| **Auditoría reviewer** | Hallazgo de pánico en núcleo, cast sin signo, aritmética sin checked, tests suavizados, sitio de llamada sin actualizar | Corregir en rama del issue y re-despachar implementador | 1 intento de corrección |

Un issue `blocked` en una rama **no invalida la otra** (F2-01 y F2-06 son raíces), pero
sí detiene los issues que dependen de él. Nunca se cicla más allá de los topes.

## 9. Commits, ramas y rollback

- Una rama por issue desde `develop`; el subagente comita **solo** al cerrar en verde.
- El orquestador mergea `--no-ff` a `develop` únicamente con informe verde.
- Intento fallido → `git branch -D` de la rama del intento y se reintenta desde
  `develop` limpio. Nada de `push`. Nada de tocar `main`.
- Los issues que quedan `blocked` quedan con `status: todo`; el motivo queda en el
  ledger.
- Si un issue cambia una firma autorizada, el commit debe mencionarlo en el cuerpo
  (`firma de draw → Result`), para que el verificador lo encuentre con `git log`.

## 10. Cierre e informe de la mañana

1. Verificador independiente (§7.4) sobre los issues done.
2. Auditoría del reviewer (§7.5) sobre el diff completo de la sesión.
3. `plan\F2-estabilidad\INFORME_NOCHE_F2.md`:
   - Estado por issue (done / blocked + motivo).
   - AC verdes por issue (copiados de las Notas de implementación).
   - Desviaciones aplicadas (§11) y su justificación.
   - Cambios de firma realizados y pruebas existentes ajustadas, con su razón.
   - Verificación independiente: resultado de la tabla.
   - Resultado de la auditoría del reviewer: [APROBADO] / [RECHAZADO] + hallazgos.
   - Siguiente paso recomendado (normalmente: abrir F3 con el procedimiento estándar).
4. Respuesta final al usuario en ≤25 líneas.

## 11. Particularidades por issue

- **F2-02** (raíz, desbloquea F2-03): al propagar el error de un argumento de acción,
  decide el retorno de `parse_action_call` dentro del issue y documéntalo; no pierdas
  el diagnóstico ni cambies el orden por línea del resultado.
- **F2-03** (contrato transversal): cambia cuatro firmas de golpe (`lex`, `LexError`,
  `World::draw`, comportamiento de `execute_turn`). Verifica **todos** los sitios de
  llamada, incluidos los tests, antes de commitear. El `Result` de `draw` obliga a
  decidir qué hace la CLI ante el error: abortar la impresión con mensaje por `stderr`
  es aceptable.
- **F2-01**: solo toca el lexer y sus pruebas; la feature de escapes cambia el
  contenido de los textos, revisa que ninguna prueba afirme un texto con barra.
- **F2-06**: el camino del cast negativo en `Move::apply` es defensivo (la dirección ya
  se valida en `{-1,0,1}`): el AC se cumple con un helper comprobable y usado. Para el
  desbordamiento de fin de tick, elegir error de ejecución y documentarlo.
- **F2-07** (el más grande): dueño único de catálogos. Si el borrow checker aprieta, la
  solución es el dueño único, **no** `Rc<RefCell>`; si aun así hiciera falta compartir,
  parar y reportar. Tras este issue, la CLI deja de pasar catálogos a `draw`.
- **F2-08**: depende de las firmas que dejaron F2-03 y F2-07; adapta la impresión del
  mapa **antes** de separar flujos, y ajusta las pruebas que comparan salida combinada.
  No cambies los códigos de salida que ya fija `cli_usage.rs`.

## 12. Verificación de salida de la fase

- `cargo test --workspace` → todo en verde.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
- `cargo run -p instinct-cli -- run --root examples --map "Plain with rocks" --seed 1 --ticks 10 --place Walker@0,0` → exit 0, misma salida que F1 salvo el cambio de flujos de F2-08.
- [ ] Cero `unwrap`/`expect`/`panic!` sobre datos de usuario o fichero en `crates\instinct-core\src` y `crates\instinct-cli\src`.
- [ ] Cada problema de la tabla de cobertura de `PHASE.md` tiene issue; los de comportamiento tienen prueba.
- [ ] Dos corridas con la misma semilla dan la misma historia.
- [ ] Los escenarios E2E de F1 siguen pasando.
