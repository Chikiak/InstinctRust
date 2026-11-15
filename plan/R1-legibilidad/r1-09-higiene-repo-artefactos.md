---
id: R1-09
title: Higiene del repo — artefactos `*_out.txt` fuera de git
phase: R1
status: done
depends_on: []
size: S
---

# R1-09 · Higiene del repo — artefactos `*_out.txt` fuera de git

## Objetivo

En la raíz están versionados cuatro archivos que son salida de comandos (`cli_out.txt`,
`clippy_out.txt`, `fmt_out.txt`, `test_out.txt`). Dejan de ensuciar el repo y `git status`.

## Alcance

**Haz**
- `git rm --cached cli_out.txt clippy_out.txt fmt_out.txt test_out.txt` (los archivos pueden
  quedarse en disco).
- Añade a `.gitignore` los patrones de salida (p. ej. `*_out.txt`, `cli_out.txt`, `clippy_out.txt`,
  `fmt_out.txt`, `test_out.txt`) sin romper el `.gitignore` actual.
- Commit con mensaje `chore: untrack command output artifacts`.

**No hagas**
- No borres los archivos del disco si alguien los está mirando (solo dejan de estar trackeados).
- No toques `.gitignore` de `.opencode/` (los ledgers ya están excluidos vía `.git/info/exclude`).
- No cambies código ni pruebas.

### Archivos previstos
- Raíz: `cli_out.txt`, `clippy_out.txt`, `fmt_out.txt`, `test_out.txt` — untrack.
- `.gitignore` — modificar.

## Criterios de aceptación

- [ ] **AC1** `git status --short` limpio y los cuatro archivos sin trackear. Verif: comando.
- [ ] **AC2** `git ls-files | Select-String "_out.txt"` → 0 coincidencias. Verif: comando.
- [ ] **AC3** Clippy/fmt/tests siguen verdes (no deberían entender este cambio). Verif: comandos de
  la fase.

## Contexto

Higiene detectada en el reconocimiento; no es lógica.

## Notas de implementación

Patrones añadidos al .gitignore raíz: *_out.txt, cli_out.txt, clippy_out.txt, fmt_out.txt, test_out.txt. Los 4 archivos siguen en disco, solo dejan de estar trackeados.