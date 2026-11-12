# CLAUDE.md

Guía para trabajar en este repositorio: qué es el proyecto, cómo está organizado y qué reglas seguir.

## Qué es

Deepslate Studio es un editor de skins de escritorio para un juego de bloques, hecho en Rust. Nombre del crate y del binario: `deepslate-studio`.

- **Licencia:** GPLv3. Todo código y todo recurso que entre debe ser compatible con GPLv3.
- **Plataformas:** Linux (la principal) y Windows. Las dos tienen que compilar y pasar las pruebas.
- **Idiomas:** español (base), inglés, portugués de Brasil y catalán.
- **Objetivo:** editor rápido, sin cuenta ni publicidad, con identidad visual propia.

## Comandos

```sh
cargo run                                   # abrir la app
cargo test                                  # pruebas
cargo test -- --ignored                     # también las que necesitan GPU
cargo clippy --all-targets -- -D warnings   # sin avisos
cargo fmt --check
```

La CI (`.github/workflows/ci.yml`) ejecuta `fmt`, `clippy` y `test` en Linux y en Windows. Un cambio está terminado cuando pasa los tres.

## Stack

| Uso | Librería |
|---|---|
| Interfaz | `eframe` / `egui` |
| Vista 3D | `wgpu` (a través de eframe) |
| Imágenes | `image` (PNG y JPEG) y `png` (este último para los bloques de texto del PNG) |
| Ajustes | `serde` con RON |
| Traducciones | `fluent-bundle`, y `sys-locale` para detectar el idioma del sistema |
| Diálogos de archivo | `rfd` |
| Rutas de datos | `directories` |
| Registro | `log` |

Rust estable, edición 2024, versión mínima 1.95.

## Estructura

```
src/
  main.rs      arranque: registro, ícono, ventana
  core/        modelo de skin: partes, capas, Steve/Alex, conversión 64×32 → 64×64
  editor/      herramientas, deshacer/rehacer, selección, espejo, colores, plantillas
  project/     formato de proyecto: guardar y abrir
  render3d/    geometría del personaje, poses, dibujo con wgpu (shader en skin.wgsl)
  library/     lectura de la carpeta de la biblioteca, búsqueda, fondos de tarjeta
  settings/    ajustes, atajos y registro
  i18n/        carga de traducciones
  ui/          ventana, pestañas, paneles, diálogos, biblioteca (library_view.rs),
               ajustes (settings_window.rs) y estilo visual (style.rs)
assets/
  i18n/<idioma>/main.ftl   textos de la interfaz (es, en, pt, ca)
  fonts/                   fuente pixel de los títulos (SIL OFL, con su licencia)
  icon.png                 ícono de la app
  deepslate-studio.desktop entrada de menú para Linux
```

Las dependencias entre módulos van en un solo sentido: `ui` usa todo lo demás; `editor` y `project` usan `core` y el tipo `Pose` de `render3d`; `core` no depende de ningún otro módulo. La lógica va en `core` y `editor`, que se pueden probar sin ventana. `ui` solo dibuja y llama a esas funciones.

## Conceptos clave

- **Skin:** textura RGBA de 64×64 más su modelo (Steve, brazos de 4 px, o Alex, de 3 px). Tiene seis partes: cabeza, torso, dos brazos y dos piernas. Cada parte tiene capa base y capa exterior. `core/layout.rs` sabe dónde cae cada cara de cada parte en la textura.
- **Skins antiguas de 64×32:** al abrirlas se convierten a 64×64. El brazo y la pierna derechos se copian espejados al lado izquierdo.
- **Deshacer:** el `Editor` guarda copias completas de la skin. Cada operación que cambia píxeles pasa por `Editor::step`, que registra un paso solo si algo cambió. Los trazos de lápiz usan `begin_stroke` / `end_stroke` para que un trazo entero sea un solo paso.
- **Espejo:** `mirror_pixel` lleva un píxel al lugar equivalente en la parte opuesta (izquierda ↔ derecha). Aplicarlo dos veces devuelve el píxel original.
- **Formato de proyecto:** un PNG de skin normal con los datos extra (versión, modelo, pose y tarjeta de biblioteca) en un bloque iTXt. Cualquier visor lo abre como imagen. Un PNG sin ese bloque se importa como skin nueva. Si el archivo trae una versión más nueva que la de la app, se rechaza con un aviso, no se lee a medias.
- **Ajustes:** `settings.ron` en el directorio de configuración del sistema. Si el archivo está dañado, se guarda como `.ron.bak` y se usan los valores por defecto.
- **Pestañas:** la pestaña activa vive en los campos de `App` (`editor`, `path`, `camera`, `selected`, `card`); las demás, en `tabs`. `switch_tab` las intercambia. Todo lo que es propio de una skin abierta va en `Doc` y en `swap_active`.
- **Biblioteca:** cada PNG de la carpeta es una tarjeta. Las miniaturas se dibujan con la vista 3D y se guardan en la caché del sistema, con la huella del archivo como nombre.
- **Textos:** `I18n::tr("clave")` y `I18n::tr_args("clave", &[...])`. Si falta una clave, la interfaz muestra la clave misma. La prueba `every_used_key_exists` busca en el código las claves usadas y falla si alguna no está en `es/main.ftl`.

## Reglas

**Textos**
- Ningún texto visible queda fijo en el código: todo sale de `assets/i18n/`.
- Al agregar una clave, agrégala en `es`, `en`, `pt` y `ca`. La prueba `languages_match_spanish` falla si algún idioma no tiene exactamente las mismas claves que el español. Si no tienes la traducción, pon el texto en español con un comentario `# pendiente` arriba; nunca dejes la clave vacía.
- Pasa las claves literales a `tr("…")`, no las armes en tiempo de ejecución: si no, la prueba de claves no las encuentra.

**Marcas y recursos de terceros**
- No usar el nombre del juego ni logos o marcas de terceros en la interfaz, el nombre de la app ni los fondos.
- No incluir texturas, capas oficiales ni skins de otras personas en `assets/`, en las pruebas ni en los instaladores. El usuario puede cargarlas para la vista previa, pero la app no las guarda ni las redistribuye. Las pruebas generan sus propias imágenes.
- Los fondos y los íconos se dibujan desde cero.

**Datos del usuario**
- 64×64 es el formato estándar. Cualquier cambio en el modelo de skin o en el formato de proyecto se consulta antes, porque puede romper archivos ya guardados.
- Si cambia el formato de proyecto: sube `project::VERSION`, sigue leyendo las versiones anteriores y agrega pruebas que lo demuestren.
- La conversión de 64×32 a 64×64 está cubierta por pruebas. No se cambia sin pruebas que respalden el cambio.
- Toda operación que modifica la skin pasa por el `Editor` y queda en la pila de deshacer. La interfaz nunca escribe píxeles directamente.

**Código**
- Nada de dependencias nuevas sin preguntar. Si una es necesaria, justifícala en el commit.
- Código multiplataforma: rutas con `std::path` y directorios de datos con `directories`. Nada de `/` fijas ni de comandos propios de un sistema.
- Las pruebas de `core/`, `editor/` y `project/` van primero. La interfaz se prueba a mano.
- Los comentarios del código van en español y los identificadores en inglés, igual que el código existente.

## Cómo trabajar

- **Por hitos.** El alcance y el orden de los hitos están en `SPEC.md`. No adelantes funciones de un hito posterior.
- **Plan antes que código.** Antes de un cambio grande (un módulo nuevo, un cambio de formato o una dependencia), propón el plan y espera confirmación.
- **Commits** en español, en tiempo presente, cortos y descriptivos. Por ejemplo: «Agrega espejo por parte».
- **Ramas:** el trabajo de cada hito va en su propia rama y se une a `master` cuando el usuario lo aprueba.
