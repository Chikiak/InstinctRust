# Ambigüedades iniciales de Instinct

Esta tabla recoge las ambigüedades detectadas en `plan/F0/PHASE.md#contratos-compartidos`
(sección "Ambigüedades iniciales", filas A1 a A13). Cada fila registra la
decisión tomada, la alternativa considerada y si debe confirmarse con los
profesores. La columna "confirmar con profesores" empieza vacía.

| # | Tema | Decisión tomada | Alternativa | Confirmar con profesores |
|---|------|-----------------|-------------|--------------------------|
| A1 | Orden de lectura dentro de una carpeta | Alfabético; define cuál es el "primer terreno" | Por fecha de modificación | |
| A2 | Carpeta ausente | Se trata como vacía | Error de carga | |
| A3 | Valor de `tick` en el primer tick | 0 | 1 | |
| A4 | Llegar al final del archivo | Termina el turno; reanuda en `start:` el siguiente | Termina la simulación de la criatura | |
| A5 | Qué cuenta para el límite de 100 líneas | Toda línea ejecutada, etiquetas incluidas | Solo líneas con acción | |
| A6 | Posición de `not` | Más fuerte que las comparaciones, como lista el enunciado | Como Python (por debajo de comparaciones) | |
| A7 | Comparaciones encadenadas | De izquierda a derecha sin encadenar como Python | Encadenamiento estilo Python | |
| A8 | `and` y `or` | Cortocircuito | Evaluación completa de ambos operandos | |
| A9 | Desbordamiento de enteros | Error de ejecución | Envoltura modular (Rust por defecto) | |
| A10 | Asignar a nombre de acción o función | Error de compilación | Permitido (sobrescribe) | |
| A11 | Errores por ausencia | Línea 1 | Línea de la clave faltante | |
| A12 | Comentarios en `.te`, `.ob`, mapas | Solo línea completa en `.te` y `.ob`; ninguno en mapas | Comentarios al final de línea también | |
| A13 | `move` con speed menor o igual que 0 | No se mueve, sin error | Error de ejecución | |
| A14 | Rugidos propios en `roars_near`/`last_roar` | Excluidos por `creature_id` del emisor; una criatura no oye su propio rugido | Incluirlos si coinciden especie/posición | |
| A15 | Empate en "más cercano" a misma distancia | Gana el primero del recorrido fijo (criaturas por id ascendente, objetos por filas/columnas ascendentes) | Orden aleatorio o por semilla | |
| A16 | Casilla propia en conteos de cercanía | Fuera de todos los conteos y de "más cercanos" por simetría con `total_*` | Incluirla si hay aliado/enemigo/objeto en la casilla | |
