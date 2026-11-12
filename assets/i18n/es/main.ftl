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
view3d-hint = Arrastra para girar, usa la rueda para acercar y haz clic en una parte para seleccionarla.
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
