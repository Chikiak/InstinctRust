# Runbook — Sesión de implementación de la Fase 3 (Percepciones, funciones y texto)

> Documento operativo para la ejecución desatendida de los issues de la fase F3 con
> un orquestador y subagentes. **No modifica el plan**: los issues, sus contratos
> (`plan\F3-percepciones-funciones-texto\PHASE.md`) y sus criterios de aceptación
> siguen siendo la fuente de verdad durante la implementación.

## 1. Objetivo

Ejecutar los cuatro issues de F3 en el orden que marca su grafo de dependencias
(`PHASE.md` §Orden y dependencias), con un subagente por issue, sobre ramas por issue
mergeadas a `develop` al cerrar en verde. La sesión corre hasta terminar la fase, hasta
que un issue quede bloqueado o hasta que el usuario la pare. Sin hora de corte fija.

A diferencia de F2 (que tenía tres raíces), F3 es una **cadena estricta**:
F3-01 → F3-02 → F3-03 → F3-04. No hay raíces independientes y no hay paralelismo
autorizado: los tres primeros issues editan `perception.rs` (el trait crece), `world.rs`
(la instantánea y las consultas) y `actions.rs` (los registros), y encadenarlos evita que
dos sesiones se pisen en el mismo sitio. La cadena es de solapamiento de archivos, no de
dependencia semántica: si un issue queda `blocked`, se para la cadena y se reporta.

## 2. Agentes

| Agente | Modo | Permisos clave | Se despacha para |
|---|---|---|---|
| `orquestador-f3` | primary | `task: allow`, `edit: allow`, sin web | — |
| `implementador-rust` | subagent | edición y ejecución totales, sin web ni task | implementar cada issue (intento 1) |
| `rescatista-rust` | subagent | igual que implementador | terminar un issue que el implementador dejó en rojo |
| `verificador-rust` | subagent | **sin `edit`** (solo lectura + bash) | verificación independiente final |
| `reviewer` | subagent | auditoría estática de pánicos, determinismo y semántica del lenguaje | auditoría final del diff |

Regla de contexto del orquestador: nunca lee código; solo ledger, estados por `grep`,
informes de subagente y salidas de comandos baratos.

## 3. Orden, rutas y ramas

| # | Issue | Archivo | Rama | Depende de | Estimado |
|---|-------|---------|------|------------|----------|
| 1 | F3-01 Percepciones sobre sí misma y sobre el mundo | `plan\F3-percepciones-funciones-texto\F3-01-percepciones-propia-y-mundo.md` | `feat/f3-01-percepciones-propia-mundo` | — | 3–4 h |
| 2 | F3-02 Percepciones de cercanía, más cercano estable y rugidos | `plan\F3-percepciones-funciones-texto\F3-02-percepciones-cercania.md` | `feat/f3-02-percepciones-cercania` | F3-01 | 4–5 h |
| 3 | F3-03 `see` y `name` con las cinco constantes | `plan\F3-percepciones-funciones-texto\F3-03-see-name-y-constantes.md` | `feat/f3-03-see-name-constantes` | F3-02 | 4–5 h |
| 4 | F3-04 Los cuatro ejemplos del anexo | `plan\F3-percepciones-funciones-texto\F3-04-ejemplos-del-anexo.md` | `feat/f3-04-ejemplos-anexo` | F3-03 | 3–4 h |

Grafo de dependencias (ver `PHASE.md` §Orden y dependencias):

```
F3-01 --> F3-02 --> F3-03 --> F3-04
```

Ningún ciclo. `git` fusión en el orden de la tabla.

## 4. Pre-vuelo del orquestador (una vez)

```powershell
git status --short                      # debe estar limpio
git checkout develop
cargo build --workspace                 # compila sin warnings
cargo test --workspace                  # suite verde (F2 ya entregada)
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

Después se crea el ledger (§5) con la hora de inicio. El "warm-up" es la suite del
workspace y la compilación de los dos crates.

## 5. Ledger (`plan\F3-percepciones-funciones-texto\NOCHE_F3.md`)

Único artefacto que el orquestador escribe durante la noche. Formato:

```markdown
# Ledger F3 — inicio HH:mm (<fecha>)
Pre-vuelo: OK | FALLO (<motivo>)

| Hora | Evento | Issue | Intento | Estado | Commit/rama |
|------|--------|-------|---------|--------|-------------|
| HH:mm | preflight | — | — | OK | rama base develop |
| HH:mm | dispatch | F3-01 | 1 rust | running | feat/f3-01-percepciones-propia-mundo |
| HH:mm | done | F3-01 | 1 | 9/9 AC | a1b2c3d → merge d4e5f6g |
| HH:mm | contract-flag | F3-02 | 1 rust | CreatureView creció, Roar con emisor | misma sesión |
| HH:mm | tests-adjusted | F3-02 | 1 | EmptyView ampliado | notas del issue |
| HH:mm | blocked | F3-0X | — | <motivo> | rama descartada |
```

Eventos propios de esta fase, además de los de F1 y F2:

- `contract-flag`: el issue cambió una firma autorizada (`CreatureView`, `expr::parse`,
  `EvalContext`, `Roar`, `World::set_variable`). Se anota **qué** cambió, porque los
  issues siguientes la asumen.
- `tests-adjusted`: pruebas existentes ajustadas por un cambio de contrato; se anota
  cuáles, para que el verificador las revise y no las tome como regresión escondida.
  En F3 las esperadas son el doble `EmptyView` (crece con el trait), las llamadas a
  `expr::parse` en `tests/perception_tests.rs` y la prueba
  `test_function_in_expression_reports_on_its_line`, que deja de usar `see`.

**Regla de reanudación:** si la sesión del orquestador se compacta, relea este ledger
y el §6 antes de despachar nada.

## 6. Bucle orquestado

1. `git checkout -b feat/f3-0X-<slug>` (rama desde `develop`).
2. Despacho del implementador (§7.1).
3. Al volver el informe, verificación barata **en este orden**:
   `git log --oneline -2` (¿commit?) → `git status --short` (¿árbol limpio?) →
   `grep -n "^status:"` del issue (¿done?) → informe (¿todos los AC PASS?).
4. Verde: merge `--no-ff` a `develop`, borrar la rama, línea en el ledger → siguiente.
5. Rojo: clasificar el fallo (§8) y actuar. Nunca reintentar a ciegas.

Al terminar la cadena, los pasos de cierre del §10 (verificador, reviewer, informe).

## 7. Plantillas de despacho

### 7.1 Implementador (intento 1, todos los issues)

```text
Implementa el issue plan\F3-percepciones-funciones-texto\<ID>-<slug>.md (intento 1/1).
Léelo entero antes de tocar código y empieza por su sección "Leer primero".
Respeta los contratos de plan\F3-percepciones-funciones-texto\PHASE.md (el issue los
referencia) y lee también las "Notas de implementación" de los issues de los que
depende: <DEPENDENCIAS>.
Esta fase completa el catálogo del lenguaje: si un AC te obliga a cambiar una firma que
PHASE.md autoriza (CreatureView, expr::parse, EvalContext, Roar, World::set_variable),
hazlo, propágalo a todos los sitios de llamada en el mismo cambio y documéntalo en las
notas; si te obliga a cambiar cualquier otro contrato, NO lo hagas: repórtalo.
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

`<tipo>`: `feat` para las percepciones, `see`/`name` y las constantes, `fix` si un AC
corrige comportamiento, `test` si solo hay pruebas, `refactor` para cambios de firma sin
cambio de comportamiento (los de `CreatureView` y `expr::parse` lo son).

**Restricciones críticas del núcleo InstinctRust:**

- `crates\instinct-core` no tiene dependencias externas ni referencias a la interfaz:
  nada de crates ajenos (el crate `rand` está prohibido), nada de `println!` (el mundo
  emite eventos), nada de `f32`/`f64` (el lenguaje solo tiene enteros `i64` o texto).
- Cero pánicos con entradas inválidas: ni `unwrap`/`expect` sobre datos del usuario ni
  sobre archivos, ni indexar fuera de rango. División/resto por cero y desbordamiento
  se detectan y se convierten en errores de ejecución. Una percepción o función nuevas
  que lean fuera de la rejilla devuelven su valor por contrato (`NONE`, texto vacío), no
  pánico.
- Estado del mundo: identificadores estables y almacenes; la rejilla guarda
  identificadores, no referencias. `Value` es entero `i64` o texto.
- **Las treinta y una percepciones y las dos funciones se resuelven por catálogo**, con
  un trait y un registro por nombre: una cadena de `if` por nombre es fallo del issue
  aunque sus tests pasen.
- Determinismo: el recorrido de lo cercano es fijo (criaturas por identificador, rejilla
  por filas y columnas, rugidos en orden). Nada de `HashMap` que ordene por azar.
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
Retoma el issue plan\F3-percepciones-funciones-texto\<ID>-<slug>.md. Un intento previo
con otro modelo falló; tu misión es terminarlo, no rehacerlo desde cero si lo existente
aprovecha.
Resumen del fallo previo (≤10 líneas): <RESUMEN>.
Lee el issue entero + contratos de PHASE.md + las notas del intento previo; evalúa
qué AC están ya cubiertos y qué falta, incluida la ausencia de unwraps nuevos y de
cadenas de if por nombre.
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
- que cada percepción y función nuevas se resuelven por catálogo, sin if por nombre;
- que los recorridos de cercanía y de see/name son deterministas y no envuelven coordenadas;
- que los cambios de firma autorizados se propagaron a todos los sitios de llamada.
Tabla final ≤20 líneas: issue | AC por AC PASS/FAIL con comando | commit sí/no | veredicto.
No modifiques nada.
```

### 7.5 Auditoría del reviewer (cierre obligatorio)

```text
Audita el diff completo de la sesión F3. Revisa: pánicos sobre entradas inválidas
(unwrap/expect/index fuera de rango/conversión de coordenada sin comprobar signo),
orden de iteración dependiente de HashMap, cadenas de if por nombre en lugar de catálogo,
println! en el núcleo, aserciones de test suavizadas, dependencias externas en
instinct-core, f32/f64 en cualquier lado, y cambios de comportamiento en percepciones que
ya existían. Comprueba además que los cambios de firma autorizados (CreatureView,
expr::parse, EvalContext, Roar, World::set_variable) no dejaron sitios de llamada sin
actualizar, y que los cuatro ejemplos del anexo son literales del enunciado. Devuelve un
veredicto [APROBADO] o [RECHAZADO] con hallazgos concretos. No se da por terminada la
tarea sin este reporte.
```

## 8. Política de fallos

| Tipo de fallo | Cómo reconocerlo | Acción | Tope |
|---|---|---|---|
| **Infraestructura / toolchain / build** | Error al invocar el subagente, timeout, red, fallo de `cargo build`/`cargo test` | Reanudar la **misma sesión** del subagente (§7.2), mismo modelo | 5 reanudaciones → `blocked` |
| **Del modelo** | AC en rojo, trabajo incompleto, stubs o contradicción no reportada | Despachar **rescatista-rust** una vez (§7.3) sobre el mismo árbol | 1 intento → si falla: `blocked` y parar la cadena |
| **Contrato mal decidido** | El AC es inalcanzable sin romper un contrato que `PHASE.md` no autoriza | Parar ese issue, dejar `status: todo`, reportar al usuario con la contradicción concreta; no mergear | 1 reporte |
| **Auditoría reviewer** | Hallazgo de pánico en núcleo, if por nombre, recorrido no determinista, tests suavizados, sitio de llamada sin actualizar | Corregir en rama del issue y re-despachar implementador | 1 intento de corrección |

Un issue `blocked` en F3 **detiene la cadena**: no hay raíces alternativas que puedan
avanzar mientras se resuelve. Nunca se cicla más allá de los topes.

**Lección de F2 aplicada:** la reanudación por `sessionID` no es ejecutable con este
harness (el parámetro llega truncado). Cuando falle la infraestructura, se abre una
sesión nueva que parte del estado real del árbol y de las notas del intento previo, y se
registra como `reanudacion-desviada`; nunca es un reintento a ciegas.

## 9. Commits, ramas y rollback

- Una rama por issue desde `develop`; el subagente comita **solo** al cerrar en verde.
- El orquestador mergea `--no-ff` a `develop` únicamente con informe verde.
- Intento fallido → `git branch -D` de la rama del intento y se reintenta desde
  `develop` limpio. Nada de `push`. Nada de tocar `main`.
- Los issues que quedan `blocked` quedan con `status: todo`; el motivo queda en el
  ledger.
- Si un issue cambia una firma autorizada, el commit debe mencionarlo en el cuerpo
  (`CreatureView +nearby`), para que el verificador lo encuentre con `git log`.
- **Cada subagente trabaja en su propia rama**, nunca sobre `develop` ni sobre el árbol
  principal: en F2 hubo fugas de ediciones al árbol principal durante las sesiones en
  paralelo. En F3 no hay paralelismo, así que el riesgo es menor, pero la rama por issue
  es obligatoria igual.

## 10. Cierre e informe de la mañana

1. Verificador independiente (§7.4) sobre los issues done.
2. Auditoría del reviewer (§7.5) sobre el diff completo de la sesión.
3. `plan\F3-percepciones-funciones-texto\INFORME_NOCHE_F3.md`:
   - Estado por issue (done / blocked + motivo).
   - AC verdes por issue (copiados de las Notas de implementación).
   - Desviaciones aplicadas (§11) y su justificación.
   - Cambios de firma realizados y pruebas existentes ajustadas, con su razón.
   - Verificación independiente: resultado de la tabla.
   - Resultado de la auditoría del reviewer: [APROBADO] / [RECHAZADO] + hallazgos.
   - Ambigüedades A14 a A17 anotadas en `docs/ambiguedades.md`.
   - Siguiente paso recomendado (normalmente: abrir F4 con el procedimiento estándar).
4. Respuesta final al usuario en ≤25 líneas.

## 11. Particularidades por issue

- **F3-01** (raíz de la cadena): crece `CreatureView` con diez accesores. Propaga el
  cambio a los dos dobles de prueba (`FixedView` en las pruebas de `src/perception.rs` y
  `EmptyView` en `tests/perception_tests.rs`) en el mismo commit; un doble sin los
  métodos nuevos es el fallo más probable de este issue. `width`, `height` y `total_*`
  pasan por `i64`: usa la conversión con vuelta a comprobar que ya existe, sin `as`.
- **F3-02** (el más grande): un solo recorrido para las catorce percepciones. Si el
  borrow checker aprieta al recorrer criaturas y rejilla desde la instantánea, la
  solución es que `World` calcule y devuelva la estructura `Nearby`, **no** `RefCell` en
  el mundo; si aun así hiciera falta compartir, parar y reportar. El `id` del emisor en
  `Roar` cambia `add_roar` y `Roar::apply`: propaga ambos. Las ambigüedades A14–A16 se
  anotan en `docs/ambiguedades.md` antes de cerrar.
- **F3-03** (el más transversal): cambia `expr::parse`, añade `EvalContext::view` y el
  nodo `Call`, y crea el módulo de funciones. Verifica **todos** los sitios de llamada,
  incluidos los tests, antes de commitear. La prueba
  `test_function_in_expression_reports_on_its_line` deja de usar `see` (ahora válido) y
  afirma el mismo contrato con un nombre desconocido: no se suaviza. Si `view()` no
  encaja en `EvalContext` sin romper los contextos de prueba, parar y reportar antes de
  cambiar el trait de otra forma.
- **F3-04** (integración): los cuatro `.ins` son literales del anexo. Si uno no se
  comporta como describe, es un fallo de F3-01 a F3-03: reportar con el escenario
  concreto, no ajustar la semilla hasta que pase ni cambiar la semántica del mundo.
  Las pruebas leen los archivos de `examples/creatures/` con ruta relativa al manifiesto
  del crate, como hace `cli_run.rs`.

## 12. Verificación de salida de la fase

- `cargo test --workspace` → todo en verde.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
- `cargo run -p instinct-cli -- check examples/creatures/hobbit.ins` (y `ent`, `dwarf`,
  `uruk`) → imprime `OK` y exit 0.
- `cargo run -p instinct-cli -- run --root examples --map "Plain with rocks" --seed 7
  --ticks 30 --place Dwarf@0,1 --place Ent@5,5 --place Hobbit@10,10 --place Uruk@15,3` →
  exit 0, stderr vacío, misma salida en dos ejecuciones.
- [ ] Las 31 percepciones y las 2 funciones resuelven por catálogo, sin cadena de `if`
      por nombre.
- [ ] Cero `unwrap`/`expect`/`panic!` sobre datos de usuario o fichero en
      `crates\instinct-core\src` y `crates\instinct-cli\src`.
- [ ] Las ambigüedades A14 a A17 quedan anotadas en `docs/ambiguedades.md`.
- [ ] Los escenarios E2E de F0, F1 y F2 siguen dando la misma salida para la misma
      semilla.
