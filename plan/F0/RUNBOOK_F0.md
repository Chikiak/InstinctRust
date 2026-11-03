# Runbook — Sesión de implementación de la Fase 0 (Esqueleto andante)

> Documento operativo para la ejecución desatendida de los issues pendientes de la
> fase F0 con un orquestador y subagentes. **No modifica el plan**: los issues, sus
> contratos (`plan\F0\PHASE.md`) y sus criterios de aceptación siguen siendo la
> fuente de verdad durante la implementación.

## 1. Objetivo

Ejecutar **F0-01 → F0-02 → F0-03 → F0-04 → F0-05 → F0-06 → F0-07 → F0-08** (orden
estricto por dependencias; F0-03 es paralelizable con F0-02 pero se ejecuta en
cadena para simplicidad) con un subagente por issue, sobre ramas por issue
mergeadas a `develop` al cerrar en verde. La sesión corre hasta terminar la fase,
hasta que un issue quede bloqueado o hasta que el usuario la pare. Sin hora de
corte fija.

## 2. Agentes

| Agente (categoría del tool task) | Modo | Modelo | Permisos clave | Se despacha para |
|---|---|---|---|---|
| `orquestador-f0` | primary | — (es el agente de la sesión) | `task: allow`, `edit: allow`, sin web | — |
| `implementador-rust` | subagent | — | edición y ejecución totales, sin web ni task | implementar cada issue (intento 1) |
| `rescatista-rust` | subagent | — | igual que implementador | terminar un issue que el implementador dejó en rojo |
| `verificador-rust` | subagent | — | **sin `edit`** (solo lectura + bash) | verificación independiente final |
| `reviewer` | subagent | — | auditoría estática de pánicos, determinismo y semántica del lenguaje | auditoría final del diff |

Regla de contexto del orquestador (ver ficha del agente): nunca lee código; solo
ledger, estados por `grep`, informes de subagente y salidas de comandos baratos.

## 3. Orden, rutas y ramas

| # | Issue | Archivo | Rama | Depende de | Estimado |
|---|-------|---------|------|------------|----------|
| 1 | F0-01 Arnés, diagnósticos y generador sembrado | `plan\F0\F0-01-arnes-diagnosticos-y-generador.md` | `feat/f0-01-arnes-diagnosticos-generador` | — | 2–4 h |
| 2 | F0-02 Cargar recursos y mostrar un mapa | `plan\F0\F0-02-cargar-recursos-y-mostrar-mapa.md` | `feat/f0-02-cargar-recursos-mapa` | F0-01 | 3–5 h |
| 3 | F0-03 Evaluar expresiones sobre enteros y texto | `plan\F0\F0-03-evaluar-expresiones.md` | `feat/f0-03-evaluar-expresiones` | F0-01 | 3–5 h |
| 4 | F0-04 Colocar criaturas y aplicar acciones | `plan\F0\F0-04-colocar-criaturas-y-aplicar-acciones.md` | `feat/f0-04-criaturas-acciones` | F0-02 | 3–5 h |
| 5 | F0-05 Leer percepciones y variables de una criatura | `plan\F0\F0-05-percepciones-y-variables.md` | `feat/f0-05-percepciones-variables` | F0-03, F0-04 | 3–5 h |
| 6 | F0-06 Compilar y comprobar una criatura `.ins` | `plan\F0\F0-06-compilar-y-comprobar-criatura-ins.md` | `feat/f0-06-compilar-criatura` | F0-05 | 3–5 h |
| 7 | F0-07 Turnos y ticks con sus leyes | `plan\F0\F0-07-turnos-y-ticks-con-sus-leyes.md` | `feat/f0-07-turnos-ticks` | F0-06 | 4–6 h |
| 8 | F0-08 Simular desde la terminal | `plan\F0\F0-08-simular-desde-la-terminal.md` | `feat/f0-08-simular-terminal` | F0-07 | 2–4 h |

La cadena es estricta: si un issue queda `blocked`, los siguientes no tienen
sentido y la sesión pasa a cierre (§10).

Grafo de dependencias (ver `PHASE.md` §162):
```
F0-01 --> F0-02 --> F0-04 --> F0-05 --> F0-06 --> F0-07 --> F0-08
      F0-01 --> F0-03 ------------------------------^
```

## 4. Pre-vuelo del orquestador (una vez)

```powershell
git status --short                      # debe estar limpio
git checkout develop
cargo build --workspace                 # compila sin warnings
cargo test --workspace                  # suite verde (si ya hay código)
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check
```

Después se crea el ledger (§5) con la hora de inicio. No hay servicios Docker ni
Playwright en este proyecto; el "warm-up" es la suite de pruebas del workspace y
la compilación de los dos crates (`instinct-core` e `instinct-cli`).

## 5. Ledger (`plan\F0\NOCHE_F0.md`)

Único artefacto que el orquestador escribe durante la noche. Formato:

```markdown
# Ledger nocturno F0 — inicio HH:mm (<fecha>)
Pre-vuelo: OK | FALLO (<motivo>)

| Hora | Evento | Issue | Intento | Estado | Commit/rama |
|------|--------|-------|---------|--------|-------------|
| HH:mm | preflight | — | — | OK | rama base develop |
| HH:mm | dispatch | F0-01 | 1 rust | running | feat/f0-01-arnes-diagnosticos-generador |
| HH:mm | done | F0-01 | 1 | 8/8 AC | a1b2c3d → merge d4e5f6g |
| HH:mm | infra-fail | F0-02 | 1 rust | reanudado (build) | misma sesión |
| HH:mm | model-fail | F0-02 | 2 rescatista | running | mismo árbol |
| HH:mm | blocked | F0-0X | — | <motivo> | rama descartada |
```

**Regla de reanudación:** si la sesión del orquestador se compacta, relee este
ledger y el §6 antes de despachar nada.

## 6. Bucle orquestado

1. `git checkout -b feat/f0-0X-<slug>` (rama desde `develop`).
2. Despacho del implementador (§7.1).
3. Al volver el informe, verificación barata **en este orden**:
   `git log --oneline -2` (¿commit?) → `git status --short` (¿árbol limpio?) →
   `grep -n "^status:"` del issue (¿done?) → informe (¿todos los AC PASS?).
4. Verde: `git checkout develop && git merge --no-ff feat/f0-0X-<slug>` +
   `git branch -d feat/f0-0X-<slug>` + línea en el ledger → siguiente issue.
5. Rojo: clasificar el fallo (§8) y actuar. Nunca reintentar a ciegas.

## 7. Plantillas de despacho

### 7.1 Implementador (intento 1, todos los issues)

```text
Implementa el issue plan\F0\<ID>-<slug>.md (intento 1/1).
Léelo entero antes de tocar código y empieza por su sección "Leer primero".
Respeta los contratos de plan\F0\PHASE.md (el issue los referencia) y lee
también las "Notas de implementación" de los issues de los que depende:
<DEPENDENCIAS>.
Ejecuta TODOS los comandos de la sección "Verificación" desde la raíz del
workspace; no lo des por terminado hasta cubrir cada criterio de aceptación con
evidencia real.
Al acabar en verde: commit ("<tipo>: <resumen> (<ID>)"), actualiza status en el
issue y en la tabla de PHASE.md, y rellena "Notas de implementación" con la
evidencia AC por AC.
Si te bloqueas o el issue contradice el repo: NO comitas, deja el árbol como
esté y repórtalo. No cambies contratos ni alcance.
Devuelve MÁXIMO 20 líneas: AC por AC (PASS/FAIL + comando), commit (hash) o
"sin commit", bloqueos concretos.
```

`<tipo>`: `feat` para funcionalidad nueva, `test` si solo hay tests, `chore`/`docs`
según el caso. Conventional Commits.

**Restricciones críticas del núcleo InstinctRust:**

- `crates\instinct-core` no tiene dependencias externas ni referencias a la
  interfaz: nada de crates ajenos (el crate `rand` está prohibido), nada de
  `println!` (el mundo emite eventos), nada de `f32`/`f64` (el lenguaje solo
  tiene enteros `i64` o texto).
- Cero pánicos con entradas inválidas: ni `unwrap`/`expect` sobre datos del
  usuario ni sobre archivos, ni indexar fuera de rango. División/resto por cero
  y desbordamiento se detectan y se convierten en errores de ejecución.
- Estado del mundo: identificadores estables y almacenes; la rejilla guarda
  identificadores, no referencias. `Value` es entero `i64` o texto.
- El ciclo de ejecución es TDD: primero el test que falle, después el código.

### 7.2 Reanudación por fallo de infraestructura (mismo sessionID, mismo modelo)

```text
Reanuda: el intento previo se cortó por un fallo de infraestructura (<motivo:
build/toolchain/timeout>), no por el trabajo en sí. Relee el issue, comprueba el
estado del árbol (git status) y continúa donde lo dejaste. Mismas reglas de
cierre y mismo formato de informe.
```

### 7.3 Rescatista Rust (fallo del modelo, un solo intento)

```text
Retoma el issue plan\F0\<ID>-<slug>.md. Un intento previo con otro modelo falló;
tu misión es terminarlo, no rehacerlo desde cero si lo existente aprovecha.
Resumen del fallo previo (≤10 líneas): <RESUMEN>.
Lee el issue entero + contratos de PHASE.md + las notas del intento previo; evalúa
qué AC están ya cubiertos y qué falta. Ejecuta la verificación real de cada AC y
cierra con las mismas reglas (commit, status, notas). Informe ≤20 líneas.
```

### 7.4 Verificador independiente (al cierre)

```text
Verifica de forma independiente los issues marcados done de esta noche:
<LISTA DE ARCHIVOS>. Para cada uno: lee el issue, contrasta cada AC contra el
repo (lectura + ejecución de sus comandos de "Verificación"), comprueba commit,
status en issue y PHASE.md, y notas rellenas. Tabla final ≤20 líneas: issue | AC
por AC PASS/FAIL con comando | commit sí/no | veredicto. No modifiques nada.
```

### 7.5 Auditoría del reviewer (cierre obligatorio)

```text
Audita el diff completo de la sesión F0. Revisa: pánicos sobre entradas
inválidas (unwrap/expect/index), división truncada en vez de semántica hacia
menos infinito, orden de iteración dependiente de HashMap, cadenas de if por
nombre en lugar de catálogo, println! en el núcleo, aserciones de test
suavizadas, dependencias externas en instinct-core, f32/f64 en cualquier lado,
y cumplimiento del contrato zero-panic. Devuelve un veredicto [APROBADO] o
[RECHAZADO] con hallazgos concretos. No se da por terminada la tarea sin este
reporte.
```

## 8. Política de fallos

| Tipo de fallo | Cómo reconocerlo | Acción | Tope |
|---|---|---|---|
| **Infraestructura / toolchain / build** | Error al invocar el subagente, timeout, red, fallo de `cargo build`/`cargo test`, SDK de Rust no instalado | Reanudar la **misma sesión** del subagente (§7.2), mismo modelo | 5 reanudaciones → `blocked` |
| **Del modelo** | El subagente respondió con AC en rojo, trabajo incompleto, stubs o contradicción no reportada | Despachar **rescatista-rust** una vez (§7.3) sobre el mismo árbol | 1 intento → si falla: `blocked` y parar la cadena |
| **Mixto** | Primero infra, y el resultado final sigue rojo | Reanudar por infra; si persiste, rescatista | Los topes de arriba |
| **Auditoría reviewer** | Hallazgo de pánico en núcleo, división truncada, HashMap no ordenado, `if` por nombre, `println!` en core, tests suavizados | Corregir en rama del issue y re-despachar implementador | 1 intento de corrección |

Un issue `blocked` **detiene la cadena**: los issues posteriores dependen de él.
Nunca se cicla más allá de los topes: a la tercera señal de que el issue no sale,
se documenta y se para.

## 9. Commits, ramas y rollback

- Una rama por issue desde `develop`; el subagente comita **solo** al cerrar en
  verde.
- El orquestador mergea `--no-ff` a `develop` únicamente con informe verde.
- Intento fallido → `git branch -D` de la rama del intento y se reintenta desde
  `develop` limpio. Nada de `push`. Nada de tocar `main`.
- Los issues que quedan `blocked` quedan con `status: todo` (los subagentes no
  deben marcarlos done); el motivo queda en el ledger.

## 10. Cierre e informe de la mañana

1. Verificador independiente (§7.4) sobre los issues done.
2. Auditoría del reviewer (§7.5) sobre el diff completo de la sesión.
3. `plan\F0\INFORME_NOCHE_F0.md`:
   - Estado por issue (done / blocked + motivo).
   - AC verdes por issue (copiados de las Notas de implementación, no re-leídos).
   - Desviaciones aplicadas (§11) y su justificación.
   - Verificación independiente: resultado de la tabla.
   - Resultado de la auditoría del reviewer: [APROBADO] / [RECHAZADO] + hallazgos.
   - Siguiente paso recomendado (normalmente: ejecutar el issue bloqueado o el
     siguiente con el prompt de sesión estándar de `PHASE.md`).
4. Respuesta final al usuario en ≤25 líneas.

## 11. Particularidades por issue

- **F0-01** (crítico, del que depende todo): fija el arnés antes de cualquier
  lógica de dominio. Requiere workspace con `instinct-core` e `instinct-cli`,
  sin dependencias externas, `cargo test --workspace`, clippy y fmt en verde.
- **F0-02** (mayor riesgo): formatos `.te`, `.ob`, `.map`; el `#` es a la vez
  carácter de roca y inicio de comentario; el mapa no tiene comentarios.
- **F0-03**: división hacia menos infinito y resto con signo del divisor (ajustar
  la semántica de Rust); desbordamiento y división por cero son errores de
  ejecución, nunca pánicos.
- **F0-04**: acciones son tipos sin estado; aplicar usa el identificador de la
  criatura, no una referencia ("el intérprete lee, el mundo escribe").
- **F0-05**: `random` avanza el generador aunque el mundo se lea inmutable;
  mutabilidad interior acotada al generador, no al mundo entero.
- **F0-06**: `check` reporta todos los errores ordenados por línea, no se para en
  el primero; errores de ausencia en línea 1.
- **F0-07**: orden de turnos sorteado con semilla; límite de 100 líneas por
  turno; fin de tick (-1 vida, +1 edad, muertes, regeneración, objetos agotados);
  eventos `Said`, `Died`, `RuntimeError`.
- **F0-08**: `run --root --map --seed --ticks --every --place`; salida 0 si todo
  bien, 1 si falla carga, mapa o colocación; mapa inexistente lista los
  disponibles por stderr y sale con 1; dos corridas misma semilla, misma salida
  byte a byte.

## 12. Verificación de salida de la fase

- `cargo test --workspace` → todo en verde.
- `cargo clippy --workspace --all-targets -- -D warnings` → exit 0.
- `cargo fmt --all --check` → exit 0.
- `cargo run -p instinct-cli -- run --root examples --map "Plain with rocks" --seed 1 --ticks 10 --place Walker@0,0` → muestra el mapa en el tick 0 y en el 10, la línea de muerte por vida del Walker y exit 0.
- Dos corridas del comando anterior con la misma semilla dan la misma salida byte a byte.
- Todas las filas de `docs\ambiguedades.md` tienen decisión; las que decidieron los issues están añadidas.
