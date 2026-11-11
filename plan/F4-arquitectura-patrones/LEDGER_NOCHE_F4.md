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
