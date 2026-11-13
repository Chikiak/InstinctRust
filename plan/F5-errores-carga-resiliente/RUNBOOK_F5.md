# InstinctRust — Runbook de F5: Errores y carga resiliente

**Fase:** F5 · Errores y carga resiliente
**Carpeta:** `plan\F5-errores-carga-resiliente\`
**Dependencia:** F4 cerrada (arquitectura y patrones sin cambio de comportamiento).
**Línea base:** `cargo test --workspace` (exit 0), `cargo clippy --workspace --all-targets -- -D warnings` (exit 0), `cargo fmt --all --check` (exit 0).

---

## 1. Objetivo de F5

Cerrar el catálogo de errores de la sección 2.8 del enunciado con mensaje y línea, y demostrar que
la carga sobrevive a los archivos inválidos. Al terminar:

- El mapa con carácter desconocido queda rechazado con carácter y línea.
- Existe una batería de archivos inválidos, uno por error de compilación, con su `README` de
  mensajes y líneas esperados.
- Una carpeta mixta (válidos + inválidos) carga informando cada error y la simulación corre con los
  recursos válidos.
- Los errores de ejecución nombran el `.ins` real, matan solo a su criatura y quedan en el evento
  `RuntimeError`.
- `check` acumula todos los errores de un archivo en lugar de cortar en el primero.

**Regla de oro:** los mensajes existentes no cambian; la fase añade cobertura y precisión, no
reescribe el catálogo. La única excepción de comportamiento es F5-01 (rechazo del mapa), que
corrige una discrepancia con el enunciado.

## 2. Los 5 issues de F5

| ID | Título | Depende de | Tamaño | Archivo |
|----|--------|------------|--------|---------|
| F5-01 | Mapa con carácter desconocido rechazado, con carácter y línea | F4 | M | F5-01-mapa-caracter-desconocado-rechazado.md |
| F5-02 | Batería de archivos inválidos, uno por error de compilación de 2.8 | F5-01 | M | F5-02-bateria-errores-compilacion.md |
| F5-03 | Carga resiliente de punta a punta sobre una carpeta mixta | F5-01 | M | F5-03-carga-resiliente-punta-a-punta.md |
| F5-04 | Errores de ejecución con archivo real y registro del tick | F4 | M | F5-04-errores-ejecucion-registro.md |
| F5-05 | Validación acumulativa en `check` y limpieza del compilador | F5-02 | M | F5-05-validacion-acumulativa-check.md |

### Orden sugerido

1. **F5-01** — único cambio de comportamiento; primero para encontrar sorpresas pronto.
2. **F5-02** y **F5-03** — batería y carga mixta; orden libre (fixtures disjuntos).
3. **F5-04** — núcleo (interp/sim/event/species); independiente.
4. **F5-05** — cierra sobre la batería creada.

> Cada issue es una sesión. Un issue por sesión.

## 3. Cómo empezar un issue (TDD obligatorio)

1. **Lee el issue entero**, empezando por "Leer primero".
2. **Escribe primero el test que falle** y comprueba que falla.
3. **Aplica la edición quirúrgica** con `edit`.
4. **Ejecuta las pruebas afectadas** (sección 5 del runbook).
5. Resuelve errores de compilador o clippy sin `#[allow]` injustificado.
6. Antes de cerrar: suite completa en verde + clippy + fmt.

## 4. Comandos (Windows / PowerShell 5.1, sin `&&`)

Línea base de sesión:
- `cargo test --workspace`

Pruebas afectadas:
- `cargo test -p instinct-core`
- `cargo test -p instinct-cli`
- `cargo test -p instinct-core --test resources_tests`
- `cargo test -p instinct-cli --test cli_check_errors` (nuevo en F5-02; no existe antes)

Puerta de salida por issue:
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --all --check`

Comprobaciones manuales útiles:
- `cargo run -p instinct-cli -- check examples\creatures\hobbit.ins` → `OK`
- `cargo run -p instinct-cli -- run --root examples --map "Plain with rocks" --seed 7 --ticks 30 --place Dwarf@0,1` (dos corridas idénticas)

## 5. Qué verificar antes de marcar un issue `done`

- Todos los AC cumplidos con evidencia (comando + resultado), AC por AC.
- Suite completa, clippy y fmt en verde.
- Cero `unwrap`/`expect`/`panic!` nuevos sobre datos de usuario o archivo en núcleo y CLI.
- Cero `println!` en `crates\instinct-core\src`; cero `f32`/`f64` en el núcleo.
- Los mensajes de error existentes no cambiaron (salvo autorización expresa del issue).
- Los tests existentes no se modificaron salvo que el issue lo autorice (F5-01 autoriza dos).
- `status` actualizado en el issue y en la tabla de `PHASE.md`; "Notas de implementación" rellenas.

## 6. Bloqueos conocidos

| Bloqueo | Qué hacer |
|---------|-----------|
| Rechazar el mapa completo rompe un escenario E2E más allá de los dos tests autorizados | Parar y reportar: un ejemplo dependía del comportamiento incorrecto |
| La batería encuentra un error de 2.8 sin mensaje | Cubrirlo en el núcleo dentro de F5-02 y anotarlo en Notas |
| Propagar el archivo real hasta `interp` exige cambios de firma amplios | Reportar; alternativa válida: `sim.rs` construye el diagnóstico final con el origen de la especie, siempre explícito y probado |
| Un test existente choca con lo pedido | Los tests existentes no se tocan salvo autorización del issue; si chocan, es bloqueo y se reporta |

## 7. Criterios de salida de la fase

- Los 5 issues en `done`.
- Suite completa, clippy y fmt en verde.
- Batería completa: cada archivo de `fixtures/errors/` falla con el mensaje y línea de su `README`.
- Carga mixta: `run --root fixtures/mixed_root` informa cada error y corre con los válidos.
- Determinismo: misma semilla, misma historia en los cuatro ejemplos del anexo.
- `MASTER.md` con línea de cierre de F5 y planeación de F6.

## 8. Archivos de referencia

- `plan\F5-errores-carga-resiliente\PHASE.md` — contratos, decisiones, cobertura.
- `plan\MASTER.md` — contrato entre fases y señales de replanificación.
- `docs\enunciado.md` — secciones 2.8 (errores) y 3.x/4.1 (carga de mapas y recursos).
- `docs\PATRONES_DISENO_ARQUITECTURA.md` — sección 6.3 (validación acumulativa).
- `crates\instinct-core\src\diagnostic.rs` — el tipo `Diagnostic` único.
- `crates\instinct-core\src\resources.rs` — carga fail-soft y `load_map`.
