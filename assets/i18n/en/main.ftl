# English.

menu-file = File
menu-edit = Edit
menu-settings = Settings
file-new = New
file-open = Open…
file-save = Save
file-save-as = Save as…
file-export = Export PNG…
edit-undo = Undo
edit-redo = Redo
edit-copy-selection = Copy selection to stamp
edit-erase-selection = Erase selection
edit-deselect = Deselect
edit-copy-parts = Copy parts…
edit-paste-parts = Paste parts…

tools = Tools
tool-pencil = Pencil
tool-eraser = Eraser
tool-fill = Paint bucket
tool-picker = Eyedropper
tool-gradient = Gradient
tool-stamp = Stamp
tool-select = Selection
stamp-empty = The stamp is empty. Select an area and use “Copy selection to stamp”.
mirror = Mirror:
mirror-off = Off
mirror-all = Everything
mirror-part = Selected part
mirror-pick-part = Pick a part in the list below or click it in the 3D view.
action-toggle-mirror = Toggle mirror
action-swap-colors = Swap colors
action-toggle-view = Switch between 2D and 3D
color = Color
color-primary = Primary color
color-secondary = Secondary color (gradient end)
color-hex = Hex code: #RRGGBB or #RRGGBBAA
color-recent = Recently used
palette = Palette
palette-name = Name
palette-add = Add current color
palette-remove = Remove from palette
palette-import = Import from image or skin…
palette-hint = Right-click a color to rename or remove it.
status-palette-imported =
    { $count ->
        [one] Added 1 color to the palette.
       *[other] Added { $count } colors to the palette.
    }
model-current = Model: { $model }
model-convert = Convert to { $model }…

layers = Visibility
layer-base = Base
layer-overlay = Outer layer
part-head = Head
part-body = Body
part-right-arm = Right arm
part-left-arm = Left arm
part-right-leg = Right leg
part-left-leg = Left leg
part-label-overlay = { $part } · outer
part-select-hint = Click to select the part (used by mirror and by copy parts).

untitled = Untitled
status-opened = Opened: { $file }
status-imported = Imported: { $file }. Use “Save as…” to create the project.
status-saved = Saved: { $file }
status-exported = Exported: { $file }
status-error = Error: { $detail }

error-not-png = The file is not a valid PNG.
error-size = The skin is { $w }×{ $h }. Only 64×64 and 64×32 are supported.
error-newer = This project comes from a newer version of the app (format { $version }). Update the app to open it.
error-corrupt = The project data is damaged.
error-io = Could not access the file: { $detail }

discard-question = There are unsaved changes. What do you want to do?
discard-save = Save
discard-discard = Discard changes
cancel = Cancel
ok = OK

convert-title = Convert to { $model }
convert-note = The arms change width. Going to Alex removes one column per face and going to Steve duplicates one. You can undo it.
convert-before = Before
convert-after = After
convert-apply = Apply

export-translucent =
    { $count ->
        [one] 1 pixel of the base layer is not opaque.
       *[other] { $count } pixels of the base layer are not opaque.
    } The game will show them as opaque.
export-anyway = Export anyway

settings-title = Settings
settings-theme = Theme
theme-system = System
theme-dark = Dark
theme-light = Light
settings-pixel-size = Pixel size
settings-grid = Show grid
settings-labels = Show part names
settings-undo = Undo levels
settings-shortcuts = Shortcuts
shortcut-press = Press a key combination… (Esc cancels)
shortcut-none = No shortcut
shortcut-clear = Remove
shortcut-reset = Reset shortcuts
shortcut-clash = That combination is already used by “{ $action }”. Remove it there first.
key-ctrl = Ctrl
key-shift = Shift
key-alt = Alt

view-2d = 2D
view-3d = 3D
view-reset = Reset view
view3d-unavailable = The 3D view is not available on this computer.
view3d-hint = Left click: paint with pencil, eraser, bucket or eyedropper, and select the part. Right-click and drag: rotate. Wheel: zoom.
capture = Capture image…
status-captured = Capture saved: { $file }
error-capture = Could not get the image from the 3D view.

pose = Pose
pose-custom = Custom
pose-standing = Standing
pose-walking = Walking
pose-waving = Waving
pose-arm-up = Arm up
pose-crouching = Crouching
pose-running = Running
pose-combat = Combat
pose-neutral = Neutral

parts-copy-title = Copy parts
parts-copy-note = Check the parts you want to copy. You can open another skin and paste them there.
parts-copy = Copy
parts-paste-title = Paste parts
parts-paste-note = Check the parts to bring in. They replace what those parts have now; you can undo it.
parts-paste = Paste
status-parts-copied = Parts copied.
status-stamp-copied = Selection copied to the stamp.

file-quick-export = Quick export
export-overwrite = “{ $file }” already exists. Do you want to replace it?
export-replace = Replace

view-preview = Preview
preview-base = Base:
preview-mannequin = Mannequin
preview-current = Current skin
preview-loaded = Loaded skin
preview-load-skin = Load skin…
preview-cape = Cape
preview-cape-color = Cape color
preview-load-cape = Load cape…
preview-remove-cape = Remove loaded cape
preview-notice = Approximate view: it is not exactly how it will look in the game. Nothing you load here is saved.
error-cape = The cape must be 64×32 (or a multiple, like 128×64) or 22×17.

settings-general = General
settings-editor = Editor
settings-3d = 3D view
settings-export = Export
settings-advanced = Advanced
settings-language = Language
lang-system = System language
lang-es = Español
lang-en = English
lang-pt = Português
lang-ca = Català
settings-start-view = View on startup
start-view-last = Last used
settings-open-last = Open the last skin on startup
settings-autosave = Autosave
settings-autosave-hint = Only saves projects that already have a file.
unit-minutes = { " " }min
settings-start-tool = Starting tool
settings-background = Background
settings-background-custom = Custom color
settings-lighting = Lighting
settings-rotate-speed = Rotation speed
settings-shadow = Ground shadow
settings-capture-note = Image captures always have a transparent background and no shadow.
settings-export-dir = Default folder
settings-export-dir-none = None
settings-choose = Choose…
settings-export-name = File name
settings-export-name-hint = Use {"{"}name{"}"} for the project name and {"{"}model{"}"} for Steve or Alex.
settings-confirm-overwrite = Confirm before replacing
settings-export-note = “Quick export” saves straight to the default folder with this name. Without a folder, it asks where to save.
settings-reset = Reset settings
settings-reset-question = Reset all settings to their defaults? The color palette is kept.
settings-view-log = View log
settings-data-path = Data and settings paths:
log-title = Log
log-refresh = Refresh
log-empty = The log is empty.
