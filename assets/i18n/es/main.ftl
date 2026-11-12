# Español: idioma base de Deepslate Studio.

menu-file = Archivo
menu-edit = Editar
menu-settings = Ajustes
file-new = Nuevo
file-open = Abrir…
file-save = Guardar
file-save-as = Guardar como…
file-export = Exportar PNG…
edit-undo = Deshacer
edit-redo = Rehacer
edit-copy-selection = Copiar selección al sello
edit-erase-selection = Borrar selección
edit-deselect = Quitar selección
edit-copy-parts = Copiar partes…
edit-paste-parts = Pegar partes…

tools = Herramientas
tool-pencil = Lápiz
tool-eraser = Borrador
tool-fill = Bote de pintura
tool-picker = Cuentagotas
tool-gradient = Degradado
tool-stamp = Sello
tool-select = Selección
stamp-empty = El sello está vacío. Selecciona un área y usa «Copiar selección al sello».
mirror = Espejo:
mirror-off = No
mirror-all = Todo
mirror-part = Parte seleccionada
mirror-pick-part = Elige una parte en la lista de abajo o con un clic en la vista 3D.
action-toggle-mirror = Activar o desactivar espejo
action-swap-colors = Intercambiar colores
action-toggle-view = Cambiar entre 2D y 3D
color = Color
color-primary = Color principal
color-secondary = Color secundario (final del degradado)
color-hex = Código hex: #RRGGBB o #RRGGBBAA
color-recent = Usados hace poco
palette = Paleta
palette-name = Nombre
palette-add = Agregar color actual
palette-remove = Quitar de la paleta
palette-import = Importar desde imagen o skin…
palette-hint = Clic derecho en un color para renombrarlo o quitarlo.
status-palette-imported =
    { $count ->
        [one] Se agregó 1 color a la paleta.
       *[other] Se agregaron { $count } colores a la paleta.
    }
model-current = Modelo: { $model }
model-convert = Convertir a { $model }…

layers = Visibilidad
layer-base = Base
layer-overlay = Capa exterior
part-head = Cabeza
part-body = Torso
part-right-arm = Brazo derecho
part-left-arm = Brazo izquierdo
part-right-leg = Pierna derecha
part-left-leg = Pierna izquierda
part-label-overlay = { $part } · ext.
part-select-hint = Clic para seleccionar la parte (sirve para el espejo y para copiar partes).

untitled = Sin título
status-opened = Abierto: { $file }
status-imported = Importada: { $file }. Usa «Guardar como…» para crear el proyecto.
status-saved = Guardado: { $file }
status-exported = Exportado: { $file }
status-error = Error: { $detail }

error-not-png = El archivo no es un PNG válido.
error-size = La skin mide { $w }×{ $h }. Solo se aceptan 64×64 y 64×32.
error-newer = Este proyecto es de una versión más nueva de la app (formato { $version }). Actualiza la app para abrirlo.
error-corrupt = Los datos del proyecto están dañados.
error-io = No se pudo acceder al archivo: { $detail }

discard-question = Hay cambios sin guardar. ¿Qué quieres hacer?
discard-save = Guardar
discard-discard = Descartar cambios
cancel = Cancelar
ok = Aceptar

convert-title = Convertir a { $model }
convert-note = Los brazos cambian de ancho. Al pasar a Alex se quita una columna por cara y al pasar a Steve se duplica una. Puedes deshacerlo.
convert-before = Antes
convert-after = Después
convert-apply = Aplicar

export-translucent =
    { $count ->
        [one] Hay 1 píxel de la capa base que no es opaco.
       *[other] Hay { $count } píxeles de la capa base que no son opacos.
    } El juego los mostrará opacos.
export-anyway = Exportar igual

settings-title = Ajustes
settings-theme = Tema
theme-system = Sistema
theme-dark = Oscuro
theme-light = Claro
settings-pixel-size = Tamaño de píxel
settings-grid = Mostrar cuadrícula
settings-labels = Mostrar nombres de partes
settings-undo = Niveles de deshacer
settings-shortcuts = Atajos
shortcut-press = Pulsa una combinación… (Esc cancela)
shortcut-none = Sin atajo
shortcut-clear = Quitar
shortcut-reset = Restablecer atajos
shortcut-clash = Esa combinación ya la usa «{ $action }». Quítasela primero.
key-ctrl = Ctrl
key-shift = Mayús
key-alt = Alt

view-2d = 2D
view-3d = 3D
view-reset = Restablecer vista
view3d-unavailable = La vista 3D no está disponible en este equipo.
view3d-hint = Clic izquierdo: pinta con lápiz, borrador, bote o cuentagotas, y selecciona la parte. Arrastra fuera del personaje: lo mueve. Clic derecho y arrastra: gira. Rueda: acerca o aleja.
capture = Capturar imagen…
status-captured = Captura guardada: { $file }
error-capture = No se pudo obtener la imagen de la vista 3D.

pose = Pose
pose-custom = Personalizada
pose-standing = De pie
pose-walking = Caminando
pose-waving = Saludando
pose-arm-up = Brazo arriba
pose-crouching = Agachado
pose-running = Corriendo
pose-combat = Combate
pose-neutral = Neutra

parts-copy-title = Copiar partes
parts-copy-note = Marca las partes que quieres copiar. Puedes abrir otra skin y pegarlas allí.
parts-copy = Copiar
parts-paste-title = Pegar partes
parts-paste-note = Marca las partes que entran. Reemplazan lo que haya en esas partes; puedes deshacerlo.
parts-paste = Pegar
status-parts-copied = Partes copiadas.
status-stamp-copied = Selección copiada al sello.

file-quick-export = Exportar rápido
export-overwrite = Ya existe «{ $file }». ¿Quieres reemplazarlo?
export-replace = Reemplazar

view-preview = Vista previa
preview-base = Base:
preview-mannequin = Maniquí
preview-current = Skin actual
preview-loaded = Skin cargada
preview-load-skin = Cargar skin…
preview-cape = Capa de la espalda
preview-cape-color = Color de la capa
preview-load-cape = Cargar capa…
preview-remove-cape = Quitar capa cargada
preview-notice = Vista aproximada: no es exactamente como se verá en el juego. Lo que cargues aquí no se guarda.
error-cape = La capa debe medir 64×32 (o un múltiplo, como 128×64) o 22×17.

settings-general = General
settings-editor = Editor
settings-3d = Vista 3D
settings-export = Exportación
settings-advanced = Avanzado
settings-language = Idioma
lang-system = Idioma del sistema
# Los nombres de idioma van siempre en su propio idioma.
lang-es = Español
lang-en = English
lang-pt = Português
lang-ca = Català
settings-start-view = Vista al iniciar
start-view-last = La última usada
settings-open-last = Abrir la última skin al iniciar
settings-reduce-motion = Reducir animaciones
settings-autosave = Autoguardado
settings-autosave-hint = Solo guarda proyectos que ya tienen archivo.
unit-minutes = { " " }min
settings-start-tool = Herramienta inicial
settings-background = Fondo
settings-background-custom = Color propio
settings-lighting = Iluminación
settings-rotate-speed = Velocidad de rotación
settings-shadow = Sombra en el suelo
settings-capture-note = La captura de imagen siempre sale con fondo transparente y sin sombra.
settings-export-dir = Carpeta predeterminada
settings-export-dir-none = Ninguna
settings-choose = Elegir…
settings-export-name = Nombre del archivo
settings-export-name-hint = Usa {"{"}name{"}"} para el nombre del proyecto y {"{"}model{"}"} para Steve o Alex.
settings-confirm-overwrite = Confirmar antes de reemplazar
settings-export-note = «Exportar rápido» guarda directo en la carpeta predeterminada con este nombre. Sin carpeta, pregunta dónde guardar.
settings-reset = Restablecer ajustes
settings-reset-question = ¿Volver todos los ajustes a sus valores por defecto? La paleta de colores se conserva.
settings-view-log = Ver registro
settings-data-path = Rutas de datos y de ajustes:
log-title = Registro
log-refresh = Actualizar
log-empty = El registro está vacío.

tab-close = Cerrar pestaña
tab-next = Pestaña siguiente
tab-prev = Pestaña anterior

view-library = Biblioteca
library-search = Buscar por nombre o etiqueta…
library-favorites = ★ Solo favoritas
library-new = Nueva skin
library-import = Importar PNG…
library-refresh = Actualizar
library-no-match = Ninguna skin coincide con la búsqueda.
library-no-dir = No se encontró una carpeta para la biblioteca. Elige una en Ajustes → Biblioteca.
status-library-imported =
    { $count ->
        [one] Se importó 1 skin.
       *[other] Se importaron { $count } skins.
    }
card-open = Abrir
card-edit = Editar tarjeta…
card-duplicate = Duplicar
card-delete = Eliminar…
card-favorite = Marcar o quitar de favoritas
card-more = Más opciones
card-title = Editar tarjeta
card-name = Nombre
card-name-hint = Vacío: se usa el nombre del archivo
card-tags = Etiquetas
card-tags-hint = Separadas por comas
card-favorite-check = Favorita
card-background = Fondo
card-add-background = Agregar fondo…
card-save = Guardar
card-delete-question = ¿Eliminar «{ $name }»? Se borra el archivo de la carpeta y no se puede deshacer.
card-delete-confirm = Eliminar
bg-slate = Pizarra
bg-sky = Cielo
bg-night = Noche
bg-meadow = Pradera
bg-sunset = Atardecer
bg-cave = Cueva
settings-library = Biblioteca
settings-library-dir = Carpeta
settings-library-default = Usar la predeterminada
settings-card-size = Tamaño de tarjetas
settings-clear-cache = Limpiar caché de miniaturas
status-cache-cleared = Se borró la caché de miniaturas.

file-new-base = Nueva desde plantilla base
edit-apply-template = Aplicar plantilla
template-color-note = Camiseta, pantalón y pelo usan el color principal.
template-shirt = Camiseta
template-pants = Pantalón
template-hair = Pelo
template-armor = Armadura

anim-play = ▶ Animar
anim-mirror = Espejo de la pose
anim-target = Pose hacia la que va y vuelve
anim-speed = Velocidad
joint-title = Ajustar pose
joint-hint = Haz clic en una parte en la vista 3D para ajustar su articulación.
joint-reset = Enderezar
joint-axes = X: adelante y atrás. Y: girar. Z: hacia los lados.

edit-image-front = Imagen al frente…
import-images = Imágenes
import-title = Imagen al frente
import-note = Arrastra sobre la imagen para recortar. Se pinta sobre el frente de cabeza, torso, brazos y piernas; puedes deshacerlo.
import-fit-contain = Ajustar
import-fit-cover = Rellenar
import-whole = Usar la imagen entera
error-image = No se pudo abrir la imagen. Se aceptan PNG y JPEG.
about-version = Deepslate Studio { $version } · Licencia GPLv3

home-title = Empieza una skin nueva
home-subtitle = Tu biblioteca todavía está vacía. Elige cómo empezar:
home-drop-hint = También puedes arrastrar archivos PNG a esta ventana.
start-empty-steve = Vacía · Steve
start-steve-desc = Lienzo transparente, con brazos de 4 píxeles.
start-empty-alex = Vacía · Alex
start-alex-desc = Lienzo transparente, con brazos de 3 píxeles.
start-base = Plantilla base
start-base-desc = Piel y cara listas para vestir con las demás plantillas.
start-open = Abrir archivo…
start-open-desc = Una skin o un proyecto de cualquier carpeta.
start-import = Importar PNG…
start-import-desc = Copia tus skins a la biblioteca para verlas como tarjetas.
drop-open = Suelta para abrir
drop-library = Suelta para agregar a la biblioteca
error-drop-library = En la biblioteca solo entran skins PNG de 64×64 o 64×32. Para usar otra imagen, suéltala en el editor.
tool-lighten = Aclarar
tool-darken = Oscurecer
action-toggle-overlay = Mostrar u ocultar la capa exterior
view-front = Ver de frente
view-back = Ver de espalda
view-side = Ver de lado
view-focus-part = Enfocar la parte seleccionada
menu-view = Ver
view-split = Vista dividida (3D y 2D)
brush-size = Pincel
brush-hint = Tamaño del pincel para lápiz, borrador, aclarar y oscurecer
line-hint = Shift + clic: línea recta desde el último punto pintado
edit-replace-color = Reemplazar color…
replace-title = Reemplazar color
replace-from = Color a cambiar
replace-to = Nuevo color
replace-scope-all = Toda la skin
replace-scope-part = Solo { $part }
replace-apply = Reemplazar
edit-vary = Variar tono
edit-vary-hint = Agrega pequeñas diferencias de brillo a la selección, a la parte seleccionada o a toda la skin
settings-ui-scale = Escala de la interfaz
render-title = Exportar render
render-size = Tamaño
render-transparent = Fondo transparente
render-export = Exportar…
