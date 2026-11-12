# Ledger de noche — F4 (arquitectura y patrones)

Rama base: `develop` (commit `4dbdbd9`). Pre-vuelo: `cargo build --workspace` OK; `cargo test --workspace` 0 fallos; clippy `-- -D warnings` OK; `fmt --all --check` OK. Árbol limpio.

Orden de ejecucion (RUNBOOK_F4 seccion 3): raices F4-01, F4-02, F4-03 (cualquier orden); F4-04 (dep F4-02); F4-05 (dep F4-04); F4-06 (dep F4-02); F4-07 (condicional, al final); F4-08 (cierra, dep F3, F4-04).

| Hora | Issue | Veredicto | Decision |
|------|-------|-----------|----------|
| (pre-vuelo) | — | verde | Ledger creado; listo para despachar F4-01 |
| ~21:45 | F4-01 | verde | Commit 09f2441, merge en develop, rama borrada. Todos los AC PASS. |
| ~22:05 | F4-02 | verde | Commit c70617e, merge en develop, rama borrada. Todos los AC PASS. Desviación menor en test de roar (move(0,0,0) en lugar de wait). Siguiente: F4-03 (raiz independiente). |
| ~22:30 | F4-03 | verde | Commit a265116+b739f5b, merge en develop, rama borrada. Todos los AC PASS. Desviacion documentada: shuffle sobre &mut [usize> no &mut [T] (dyn-compat). Siguiente: F4-04 (dep F4-02). |
| ~22:55 | F4-04 | verde | Commit f24f252, merge en develop, rama borrada. Todos los AC PASS. Siguiente: F4-05 (dep F4-04). |
| ~23:15 | F4-05 | verde | Commit acd16ac+21b0189, merge en develop, rama borrada. Todos los AC PASS. Artefacto stray `null` borrado. Siguiente: F4-06 (dep F4-02). |
| ~23:35 | F4-06 | verde | Commit b27458d, merge en develop, rama borrada. Todos los AC PASS. Siguiente: F4-07 (Visitor AST, condicional: hay consumidor real?). |
| ~23:50 | F4-07 | verde (descarte) | Commit 3048748, merge en develop, rama borrada. No hay consumidor real de Visitor; descarte documentado (decisión 4 fase + MASTER dec. 40). Siguiente: F4-08 (cierra la fase, dep F3, F4-04). |
| ~00:15 | F4-08 | verde | Commits d712f81+0e45293, merge en develop, rama borrada. Todos los AC PASS. Cierra F4 (8/8). Siguiente: verificación final + informe. |
| ~00:30 | verificación | APROBADO CON OBSERVACIONES (asesor) | verificador-nemotron independiente: suite/clippy/fmt/determinismo/API CLI verdes; observaciones documentales (estados en PHASE.md, nombres de test en AC de F4-02, escenario fuera de mapa). Juicio final: revisión matutina. Informe en INFORME_NOCHE_F4.md. |
| ~01:10 | F4-07 reapertura | verde | Usuario decide Camino B (Visitor como único recorrido; intérprete como ExprVisitor). Commit 822d1c5, merge en develop, rama borrada. Todos los AC PASS (visitor demo NodeCounter). Verificación final en develop pendiente. |
| ~01:40 | cierre documental | verde | Observaciones del verificador resueltas: PHASE.md con F4-08 done en la tabla de issues, registro de ejecución sin "pendiente" (F4-01/03/04/05 → done), casillas de salida marcadas, nota de F4-07 actualizada (Camino B); MASTER.md con línea de cierre de F4 (8/8) y planeación de F5. Pendiente conocido, no bloqueante: AC de F4-02 citan nombres de test que no existen en el repo; los equivalentes pasan (no se editan AC de issues). |
