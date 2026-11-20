# DEMO_MANUAL — Flujo 4.1 de `instinct-gui` (verificación a mano)

Guía corta para comprobar con la ventana real el flujo completo de la sección 4.1
del enunciado (`docs/enunciado.md:380-388`). La lógica que ves aquí está cubierta
por tests headless (`--smoke` y `tests/reinicio`, `tests/determinismo`); esta guía
solo verifica la traducción a widgets de la capa `eframe`.

## Comando

- Demostración visual (abre ventana):

  ```powershell
  cargo run -p instinct-gui
  ```

- Verificación headless del mismo flujo (sin ventana; debe terminar en exit 0):

  ```powershell
  cargo run -p instinct-gui -- --smoke
  ```

  Imprime el registro completo del flujo 4.1 y termina en `== N entries ==`
  (más `== --smoke OK: flujo 4.1 completo verificado ==`). Es la puerta
  automatizable del entregable; la ventana real se verifica con los 4 pasos de
  abajo.

 Convenio de símbolos: cada celda es `CELL_SIZE_PX`; criatura > objeto > terreno
 (una criatura tapa el objeto/terreno de su celda). El registro del riel derecho
 ("Registro de carga") solo muestra diagnósticos de carga; los eventos de la
 simulación van al registro que imprime `--smoke` (y se pintan los rugidos sobre
 la rejilla).

---

## Paso 1 · Carga (4.1 paso 1)

- **Acción:** arrancar `cargo run -p instinct-gui`.
- **Esperado (observable):** el panel derecho "Catálogos" lista los mapas
  (`Plain with rocks`) y las especies de `examples/` (Dwarf, Ent, Hobbit, Uruk,
  Talker, Roarer, …). Aparece el botón **"Entrar en preparación"** (hay mapa y
  especies). Si algún `.te/.ob/.map/.ins` fallara, su error saldría con archivo y
  línea bajo "Registro de carga" y la app seguiría funcionando (no aborta).
- **Éxito:** mapas y especies listados; botón de preparación visible.
- **Fallo:** panel vacío, mensaje "sin mapas o sin especies", o la app no abre.

## Paso 2 · Preparación y colocación de las 4 criaturas (4.1 paso 2)

- **Acción:** pulsar **"Entrar en preparación"** (mapa `Plain with rocks`). En el
  desplegable de especies, elegir especie y **hacer clic en una casilla** para
  colocarla. Colocar las 4 del baseline: **Dwarf @ (1,0)**, **Ent @ (5,1)**,
  **Hobbit @ (10,0)**, **Uruk @ (12,3)**.
- **Esperado (observable):** la cabecera muestra `Mapa: Plain with rocks …
  Colocadas: 4`; la lista "Criaturas colocadas" enumera las 4 con su `especie @
  x,y`; la rejilla dibuja un símbolo por criatura en esas celdas. Se puede
  *quitar* una con el botón **"Quitar"** (o haciendo clic sobre la criatura) y
  volver a colocarla; no se puede colocar sobre un objeto ni sobre otra criatura.
- **Éxito:** exactamente 4 criaturas en las coordenadas indicadas, todas
  removibles y recolocables.
- **Fallo:** no se coloca al hacer clic, se permite colocar sobre otra criatura,
  o el contador no llega a 4.

## Paso 3 · Tiempo: play / pausa / paso / velocidades (4.1 paso 3)

- **Acción:** pulsar **"Play"**; observar el contador de tick subir. Cambiar la
  **Velocidad** entre **2 / 10 / 30 tps**. Pulsar **"Pausa"** (el tick se detiene)
  y **"Paso"** (avanza exactamente un tick).
- **Esperado (observable):** el rótulo de estado refleja `Running N tps` /
  `Paused N tps`; el **Tick** aumenta con el tiempo y se detiene en pausa; con
  "Paso" suma 1. Las criaturas actúan (minan, huyen, cazan, hablan) y los
  **rugidos** se marcan en ámbar sobre la rejilla. En marcha no se pueden añadir
  ni quitar criaturas (los clics sobre la rejilla no colocan).
- **Éxito:** sube/baja el tick según play/pausa/paso; las 3 velocidades cambian
  el ritmo visible; los rugidos aparecen marcados.
- **Fallo:** el tick no avanza, no se puede pausar, o al pausar sigue corriendo.

## Paso 4 · Reinicio (4.1 paso 4)

- **Acción:** con la simulación a mitad de ticks, pulsar **"Reiniciar"**.
- **Esperado (observable):** se vuelve a la fase de **preparación**: estado
  `Idle`, **Tick 0**, las **4 criaturas del baseline re-colocadas** en sus
  celdas y el mapa/semilla preservados (la siguiente partida reproduce la misma
  historia). El registro deja constancia con una línea que contiene `[reinicio]`
  (visible en la salida de `--smoke`).
- **Éxito:** de vuelta en preparación, tick 0, 4 criaturas en su sitio; al dar
  "Play" otra vez la historia se reproduce igual.
- **Fallo:** vuelve con tick > 0, pierde criaturas, o se queda en estado
  `Running`.

---

## Notas

- La verificación headless equivalente (`cargo run -p instinct-gui -- --smoke`)
  recorre los 4 pasos sobre la máquina de estados y sale con 0; si algo falla,
  imprime la causa en el registro y sale con 1.
- Determinismo intacto: la GUI con `--seed 7` produce la misma historia que la CLI
  (línea base SHA256 `BF6DE0F5EAB874DF037CE3B8AC9B996B4115F55FAF6D77379531E88F00656D09`).