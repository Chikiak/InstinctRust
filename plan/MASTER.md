# Instinct (Rust) — Plan general

## Resumen
- **Objetivo**: construir en Rust una implementación de referencia del proyecto "Instinto": compilador e intérprete del lenguaje Instinct, simulador de mundo por ticks que corre sin interfaz y, después, una ventana para verlo. Sirve para dos cosas: (1) crear y probar criaturas, terrenos, objetos y mapas con los que comprobar los proyectos de los estudiantes, y (2) aprender Rust.
- **Éxito del proyecto**:
  1. Cualquier criatura, terreno, objeto o mapa conforme a las secciones 2 a 4 del enunciado carga y corre, incluidos los cuatro ejemplos del anexo (Hobbit, Ent, Dwarf y el Uruk de la sección 2.1). Así la implementación sirve de referencia: lo que haga aquí es lo que debe hacer un proyecto de estudiante.
  2. Existe un banco de criaturas y mapas de prueba, cada uno dirigido a una regla concreta de la base, con el resultado esperado documentado, listo para correrlo en los proyectos de los estudiantes.
  3. La simulación es reproducible con semilla y no aborta ante entradas inválidas.
  4. El código está organizado de modo que cada fase deja conceptos de Rust practicados y entendidos, no solo código que funciona.
- **Punto de partida**: greenfield; solo existe el enunciado.
- **Estrategia**: B · Funcionando primero. Para probar criaturas basta con el simulador por terminal, así que lo útil llega pronto; la ventana es una mejora posterior.
- **Camino feliz mínimo**: desde la terminal, cargar el terreno Grass, el objeto Rock, un mapa y una criatura mínima, colocarla, correr N ticks con una semilla y ver el mapa ASCII cambiar mientras la criatura se mueve, esquiva rocas y muere por vida o por edad.

## Alcance
**Incluye**: la base de las secciones 2 a 4 (lenguaje, mundo, aplicación), el banco de criaturas de prueba y su documentación.

**No incluye**: máquina virtual con bytecode, extensiones del lenguaje (salvo que más adelante interesen), sprites y animaciones, defensa y evaluación (el proyecto no se entrega).

## Supuestos y preguntas abiertas
| # | Supuesto / pregunta | Impacto si es falso | Estado |
|---|---------------------|---------------------|--------|
| 1 | Windows con toolchain estable de Rust instalado y funcionando (compilador y enlazador) | El primer issue de F0 queda bloqueado | asumido |
| 2 | Sin internet durante las sesiones: el núcleo no usa dependencias externas, así que F0 a F4 no necesitan descargar nada; la ventana sí | La fase gráfica no compilaría | asumido; ver atajo 4 y F5 |
| 3 | Trabaja el autor, usando una IA como tutor y revisora, para aprender; no se delega todo en un agente | Con un agente autónomo se subiría el dial y se perdería el objetivo de aprender | asumido |
| 4 | Las ambigüedades del enunciado se resuelven con lo más simple que no aborte, y cada una se anota para contrastarla con los profesores | Un criterio distinto al de los profesores daría "referencia" equivocada | asumido; confirmar las anotadas con los profesores |
| 5 | Los formatos y carpetas son los del enunciado (`creatures/`, `terrains/`, `objects/`, `maps/`) | Habría que reabrir F3 | asumido |
| 6 | La referencia debe comportarse como pide el enunciado; si un estudiante difiere, hay que poder decir quién tiene razón | Falsos fallos en proyectos de estudiantes | asumido |

## Decisiones fijadas
- **Workspace de tres crates: núcleo, CLI y GUI.** El núcleo contiene lenguaje y mundo, sin interfaz y sin dependencias externas; CLI y GUI solo lo consumen. Descartado: un único crate con módulos.
- **Polimorfismo con traits y catálogos de nombres.** Acciones, funciones reservadas, percepciones y nodos de expresión son tipos que implementan un trait común; el compilador resuelve cada nombre contra un catálogo y valida la aridad al compilar. Es el diseño que pide el enunciado y a la vez enseña traits, objetos de trait y registros en Rust. Descartado: un enum cerrado con un `match` enorme.
- **Intérprete de árbol, sin bytecode.** El programa es una lista plana de sentencias con las etiquetas resueltas a posiciones; cada criatura guarda su punto de ejecución entre turnos.
- **El intérprete lee y el mundo escribe.** Ejecutar un turno devuelve como mucho una acción pendiente y el mundo la aplica. Evita los choques de préstamos mutables, y es la lección central de propiedad en este proyecto. Descartado: acceso mutable al mundo desde el intérprete y `Rc<RefCell>` por todas partes.
- **Entidades por identificador estable.** Criaturas, objetos y terrenos viven en un almacén; la rejilla guarda identificadores, no referencias.
- **Valor = entero con signo de 64 bits o texto.** División y resto con semántica de Python (hacia menos infinito, resto con el signo del divisor), porque lo pide el enunciado y Rust por defecto trunca.
- **Un solo generador pseudoaleatorio propio y sembrado**, dentro del mundo, para `random` y para el orden de turnos. Sin el crate `rand`. Nota: la secuencia no coincidirá con la de los proyectos de los estudiantes, por lo que el banco de pruebas evita depender de `random` salvo donde se compare solo el comportamiento estadístico.
- **Diagnóstico único**: mensaje, archivo y línea (desde 1), usado en compilación, carga y ejecución.
- **El mundo no imprime**: emite eventos y registro que consumen CLI y GUI.
- **Ninguna entrada puede provocar un pánico** en el núcleo.

## Atajos asumidos
| # | Atajo (qué se simplifica) | Riesgo | Fase que lo salda |
|---|---------------------------|--------|-------------------|
| 1 | F0 solo implementa las acciones wait, move y say | Criaturas del anexo aún no corren | F1 |
| 2 | F0 solo tiene siete percepciones básicas, pero ya dentro del catálogo | Ejemplos con percepciones de cercanía no compilan | F2 |
| 3 | Errores de F0 como mensajes simples con línea; solo los de ejecución que evitan abortar | Mensajes poco uniformes | F3 |
| 4 | Salida solo en ASCII y las dependencias de la ventana sin descargar | Descubrir tarde que no compila sin conexión | F5 (descargarlas en la primera conexión antes de empezar) |

## Contratos entre fases
- **API del núcleo** (única superficie para CLI y GUI): cargar recursos en el orden terrenos, objetos, mapas, criaturas, devolviendo catálogo y diagnósticos; crear una simulación desde mapa, colocaciones y semilla; avanzar un tick; consultar una instantánea del estado y recoger los eventos nuevos.
- **Catálogo de nombres**: resuelve acciones, funciones, percepciones y constantes.
- **Turno**: percepciones refrescadas al empezar, ejecución hasta la primera acción distinta de say, máximo 100 líneas, reanudación donde quedó.
- **Fin de tick**: -1 de vida, +1 de edad, muertes, regeneración de terrenos, retirada de objetos agotados.

## Verificación global
- `cargo test --workspace` — pruebas unitarias e integración.
- `cargo clippy --workspace --all-targets -- -D warnings` — lint.
- `cargo fmt --all --check` — formato.
- Escenario de punta a punta por terminal: el comando exacto lo fija F0.
- El arnés lo crea F0 como parte de su primer issue.

## Perfil de implementación y dimensionado
- **Implementador**: el autor, aprendiendo Rust, con una IA como tutor y revisora; ejecuta comandos y tests en su laptop Windows, sin internet la mayor parte del día.
- **Dial de dimensionado**: conservador. Se aprende mientras se construye, las sesiones son más cortas y conviene entender cada pieza antes de seguir.
- **Un issue por sesión**: sí. Cada issue incluye una nota "concepto de Rust que se practica" (propiedad y préstamos, enums, traits, `Result`, etc.) y qué conviene reescribir sin ayuda para fijarlo.
- **Formato de artefactos**: markdown en `plan/`.

## Fases
| Fase | Tramo | Objetivo | Entregable observable | Depende de | Issues previstos |
|------|-------|----------|-----------------------|------------|------------------|
| F0 | Esqueleto | Camino feliz de punta a punta por terminal | Simulación ASCII de una criatura mínima en un mapa con rocas | — | 6–8 |
| F1 | Completar | Acciones y leyes del mundo completas | Cada acción probada contra criatura, objeto, terreno y borde | F0 | 4–6 |
| F2 | Completar | Percepciones, see/name y texto | Hobbit, Ent, Dwarf y Uruk corren y se comportan como describe el enunciado | F1 | 4–6 |
| F3 | Completar | Errores y carga resiliente | Batería de archivos inválidos con mensaje y línea esperados | F2 | 3–5 |
| F4 | Completar | Banco de criaturas de prueba | Colección de criaturas y mapas, cada una probando una regla, con resultado esperado documentado | F3 | 4–6 |
| F5 | Completar | Ventana | Cargar, elegir mapa, colocar, play/pausa/paso/velocidades, reiniciar, registro | F3 | 5–7 |
| F6 | Endurecer | Conformidad y robustez | Una prueba por afirmación del enunciado; cero pánicos en simulaciones largas; misma semilla, misma historia | F4 | 3–4 |
| F7 | Endurecer | Rendimiento (si hace falta) | Tick fluido con muchas criaturas, con línea base medida primero | F5 | 2–3 |
| F8+ | Extensiones | Opcionales | Se decide después | F6 | por definir |

F4 va antes que la ventana porque es lo que cubre el primer objetivo (probar proyectos de estudiantes) y solo necesita la terminal. F5 sirve sobre todo para ver las criaturas en acción y para aprender; si el tiempo escasea, es la primera candidata a recortar.

### F0 · Esqueleto andante
- **Objetivo**: tener el camino feliz corriendo con piezas reales y con el diseño final, aunque estrecho.
- **Entregable observable**: un comando que carga un directorio de ejemplo (Grass, Rock, un mapa pequeño, una criatura mínima), coloca criaturas en las coordenadas pedidas, corre N ticks con una semilla y muestra el mapa en ASCII y el registro.
- **Incluye**: workspace con arnés (tests, lint, formato); cargadores reales de terrenos, objetos y mapas; cabecera de cinco claves; comentarios, etiquetas, goto, if con goto y asignación; expresiones enteras con la precedencia completa y literales de texto solo como argumento de say; acciones wait, move y say; percepciones health, age, lifespan, x, y, tick y random; orden de turnos sorteado con la semilla; límite de 100 líneas por turno; fin de tick completo; división por cero y variable sin asignar como errores de ejecución que matan a la criatura.
- **Excluye**: attack, consume, reproduce y roar; see y name; percepciones de cercanía y de mundo; operaciones con texto; catálogo completo de errores de compilación.
- **Riesgos y mitigación**: el borrow checker en el ciclo turno-mundo, mitigado por la decisión "el intérprete lee, el mundo escribe"; que F0 pase de 8 issues, en cuyo caso se estrecha el camino y lo sobrante pasa a F1.
- **Criterios de salida**: test de integración que corre la criatura mínima con semilla fija y compara el mapa final; dos corridas con la misma semilla dan el mismo resultado; la criatura muere por vida 0 y por edad en el tick esperado; move se detiene ante roca y borde; los tres comandos de verificación global pasan; el comando de terminal muestra la simulación.
- **Decisiones abiertas para el plan específico**: comportamiento ante desbordamiento de enteros (no puede abortar), argumentos y formato de salida del comando, cómo se indican las colocaciones, estructura interna de módulos.

### F1 · Acciones y leyes del mundo
- **Objetivo**: completar las siete acciones y las reglas de la sección 3.4.
- **Entregable observable**: escenarios mínimos, uno por acción, ejecutables desde pruebas y desde la terminal.
- **Incluye**: attack y consume sobre criatura, objeto, terreno, la propia casilla con (0, 0) y fuera del mapa (no hace nada); criatura muerta por golpe desaparece en el acto y deja solo terreno; objeto con reserva 0 desaparece al final del tick; reproduce con sus condiciones y su "aborto natural", cría con edad 0, sin variables y arrancando en start:, y primer turno en el tick siguiente; wait con n menor que 1 como 1 y sueño de n-1 turnos mientras se sigue envejeciendo; roar con el mensaje disponible el tick siguiente; criaturas muertas antes de su turno no juegan.
- **Excluye**: lectura de rugidos desde el lenguaje (F2).
- **Riesgos y mitigación**: orden de turnos con nacimientos y muertes en medio del tick, mitigado con escenarios deterministas por semilla.
- **Criterios de salida**: cada caso anterior tiene una prueba con resultado observable; las pruebas de F0 siguen en verde.
- **Decisiones abiertas para el plan específico**: representación interna de los rugidos y de la cola de eventos, cómo comparten código attack y consume.

### F2 · Percepciones, funciones y texto
- **Objetivo**: catálogo completo de percepciones, see y name, constantes y operaciones con texto.
- **Entregable observable**: los cuatro ejemplos del anexo corren en la terminal y muestran los comportamientos descritos.
- **Riesgos**: elegir el "más cercano" de forma estable cuando hay empates (mitigación: orden de recorrido fijo).
- **Criterios de salida**: pruebas por percepción y por función; los cuatro ejemplos compilan y ejecutan; resolver las treinta percepciones no usa una cadena de `if` por nombre.

### F3 · Errores y carga resiliente
- **Objetivo**: catálogo completo de errores de compilación y de ejecución, con línea.
- **Entregable observable**: una carpeta de archivos inválidos (uno por error de la sección 2.8) con el mensaje y línea esperados, y una carga que sigue funcionando con los demás recursos.
- **Criterios de salida**: todos los errores de la sección 2.8 cubiertos; mapa con carácter desconocido rechazado con carácter y línea; un error de ejecución mata solo a esa criatura y queda en el registro.

### F4 · Banco de criaturas de prueba
- **Objetivo**: tener un paquete que se pueda llevar a cualquier proyecto de estudiante y que permita decir, regla por regla, si lo cumple.
- **Entregable observable**: una carpeta con criaturas, terrenos, objetos y mapas, y un documento con, por cada escenario, qué regla prueba y qué debe verse (estado final, eventos o mensajes), obtenido de esta implementación de referencia.
- **Incluye**: un escenario por regla de las secciones 2.4 a 3.4 (movimiento y bloqueos, golpes a cada tipo de objetivo, reproducción y aborto, sueño de wait, límite de 100 líneas, orden de turnos y nacimientos, leyes de fin de tick, precedencia y tipos, see y name con borde y visión, percepciones de cercanía); criaturas "trampa" que deben fallar de forma controlada (archivos con error de compilación, división por cero, mezcla de tipos); mapas con carácter desconocido.
- **Riesgos**: diferencias legítimas por aleatoriedad, mitigadas con criaturas deterministas que no leen `random`, o con escenarios cuyo resultado es un rango.
- **Criterios de salida**: cada escenario tiene resultado esperado verificado por una prueba automática en esta implementación; el documento se puede seguir sin leer el código.

### F5 · Ventana
- **Objetivo**: cumplir la sección 4.1 con una ventana.
- **Entregable observable**: arrancar, ver errores de carga en el registro, elegir mapa, colocar y quitar criaturas en la preparación, play, pausa, avanzar un tick, al menos tres velocidades y reiniciar.
- **Riesgos**: la biblioteca gráfica y su compilación sin internet. Su primer issue es un spike que compila sin conexión, dibuja la rejilla y recibe un clic; en ese issue se elige la biblioteca (decisión abierta con dueño: F5).
- **Criterios de salida**: flujo completo de la sección 4.1 demostrado a mano y con el núcleo sin cambios de API.

### F6 · Conformidad y robustez (endurecer)
- **Atributo**: corrección frente al enunciado.
- **Criterio de salida**: una prueba por cada afirmación de las secciones 2 a 4 (las que F4 no cubrió); simulaciones largas (decenas de miles de ticks) con los cuatro ejemplos y mapas variados sin un solo pánico; misma semilla produce la misma historia comparando el estado tick a tick.

### F7 · Rendimiento (endurecer)
- **Atributo**: fluidez de la simulación.
- **Criterio de salida**: su primer issue mide la línea base con un mapa grande y muchas criaturas; la cifra objetivo se confirma entonces (decisión abierta con dueño: esta fase). Si la base ya es fluida a la velocidad más alta de la ventana, la fase se elimina.

### F8+ · Extensiones (opcionales)
Candidatas si interesan: estructuras de control de alto nivel traducidas a saltos, terrenos y objetos con comportamiento propio, inspector de criatura, gráficas de población. Se deciden después de F6.

## Señales de replanificación
- F0 pasa de 8 issues: estrechar el camino feliz.
- Una fase tiene más de 8 issues: dividirla.
- Un profesor o un proyecto de estudiante contradice un criterio de la referencia: corregir la referencia y rehacer los resultados esperados de F4.
- El spike de F5 no compila sin conexión: cambiar de biblioteca o resolver la descarga antes de seguir.
- La línea base de F7 ya cumple el objetivo: eliminar la fase.
- El borrow checker obliga a romper "el intérprete lee, el mundo escribe": volver a las decisiones fijadas y rehacer F1 y F2.

## Registro de cambios
| Fecha | Cambio | Motivo |
|-------|--------|--------|
| 2026-11-01 | Versión inicial | — |
| 2026-11-01 | Objetivo reorientado a referencia de pruebas y aprendizaje; se añade F4 (banco de criaturas), la ventana pasa a F5, se elimina la fase de defensa, dial conservador | El autor es Alumno Ayudante y el proyecto no se evalúa |
