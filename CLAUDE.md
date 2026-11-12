# CLAUDE.md

Instrucciones para Claude Code en este repositorio.

## Proyecto

Editor de skins de Minecraft de escritorio, hecho en Rust.
Nombre: Deepslate Studio (nombre del crate y del binario: `deepslate-studio`).

- Licencia: GPLv3. Todo código nuevo debe ser compatible con GPLv3.
- Plataformas objetivo: Linux (principal) y Windows.
- Idiomas de la interfaz: español (base), inglés, portugués y catalán.
- Interfaz: egui. Vista 3D con wgpu.

Ver `SPEC.md` para el alcance completo, los hitos y las decisiones.

## Stack

- Rust estable, edición 2024, mínimo 1.95.
- `eframe` / `egui` para la interfaz.
- `wgpu` para la vista 3D.
- `image` para PNG.
- `serde` con RON o TOML para ajustes y formato de proyecto.
- `fluent-bundle` para traducciones.

## Estructura prevista

```
src/
  core/        modelo de skin, capas, conversión Steve/Alex, conversión 64x32 -> 64x64
  editor/      herramientas, pila de deshacer/rehacer, selección, espejo
  render3d/    modelo 3D, poses (ángulos por articulación), miniaturas
  library/     biblioteca de tarjetas, caché de miniaturas
  project/     formato de proyecto propio, guardado y carga
  settings/    ajustes, atajos
  i18n/        carga de traducciones
  ui/          paneles, pestañas, tarjetas
assets/
  i18n/        es, en, pt, ca
  backgrounds/ fondos predefinidos (originales, no copiados de Minecraft)
tests/
```

## Reglas de trabajo

- **Nada de texto fijo en la interfaz.** Todo texto visible sale de `assets/i18n/`. Si agregas un texto, agrega la clave en los cuatro idiomas (o marca como pendiente la traducción faltante, nunca dejarla vacía).
- **Español es el idioma base.** Si falta una traducción, el fallback es español.
- **No usar "Minecraft" ni logos o marcas de terceros** en la interfaz, el nombre de la app ni los fondos. Los fondos predefinidos se dibujan desde cero.
- **Sin texturas de terceros en el repositorio.** No incluir capas oficiales de Minecraft ni skins de otras personas en `assets/`, en pruebas ni en instaladores. El usuario las carga localmente para previsualizar, y la app no las guarda ni las redistribuye.
- **Formato 64×64 es el estándar.** Cualquier cambio que afecte el modelo de skin o el formato de proyecto debe preguntarse antes.
- **Compatibilidad:** la conversión de 64×32 a 64×64 debe tener pruebas unitarias. No cambiar esa lógica sin tests que la respalden.
- **Deshacer:** toda operación que modifique la skin debe registrarse en la pila de comandos. No modificar píxeles directamente desde la interfaz.
- **Sin dependencias nuevas sin preguntar,** salvo que sea necesario para el hito en curso y se justifique en el commit.
- **Windows y Linux:** no usar rutas con `/` fijas ni comandos específicos de un sistema. Usar `std::path` y directorios de datos estándar.

## Cómo trabajar

- Antes de implementar algo grande, proponer el plan y esperar confirmación.
- Trabajar por hitos, según `SPEC.md`. No adelantar funciones de hitos posteriores.
- Escribir pruebas para `core/` y `project/` antes que para la interfaz.
- Mensajes de commit en español, en tiempo presente, cortos y descriptivos.

## Decisiones tomadas

- Versión mínima de Rust: 1.95 (edición 2024).
- Traducciones: `fluent-bundle`, archivos en `assets/i18n/<idioma>/main.ftl`.
