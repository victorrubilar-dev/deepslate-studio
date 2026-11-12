# Català.

menu-file = Fitxer
menu-edit = Edita
menu-settings = Configuració
file-new = Nou
file-open = Obre…
file-save = Desa
file-save-as = Anomena i desa…
file-export = Exporta PNG…
edit-undo = Desfés
edit-redo = Refés
edit-copy-selection = Copia la selecció al segell
edit-erase-selection = Esborra la selecció
edit-deselect = Treu la selecció
edit-copy-parts = Copia parts…
edit-paste-parts = Enganxa parts…

tools = Eines
tool-pencil = Llapis
tool-eraser = Goma
tool-fill = Pot de pintura
tool-picker = Comptagotes
tool-gradient = Degradat
tool-stamp = Segell
tool-select = Selecció
stamp-empty = El segell és buit. Selecciona una àrea i fes servir «Copia la selecció al segell».
mirror = Mirall:
mirror-off = No
mirror-all = Tot
mirror-part = Part seleccionada
mirror-pick-part = Tria una part a la llista de sota o fes-hi clic a la vista 3D.
action-toggle-mirror = Activa o desactiva el mirall
action-swap-colors = Intercanvia els colors
action-toggle-view = Canvia entre 2D i 3D
color = Color
color-primary = Color principal
color-secondary = Color secundari (final del degradat)
color-hex = Codi hex: #RRGGBB o #RRGGBBAA
color-recent = Fets servir fa poc
palette = Paleta
palette-name = Nom
palette-add = Afegeix el color actual
palette-remove = Treu de la paleta
palette-import = Importa des d'una imatge o skin…
palette-hint = Fes clic dret en un color per canviar-li el nom o treure'l.
status-palette-imported =
    { $count ->
        [one] S'ha afegit 1 color a la paleta.
       *[other] S'han afegit { $count } colors a la paleta.
    }
model-current = Model: { $model }
model-convert = Converteix a { $model }…

layers = Visibilitat
layer-base = Base
layer-overlay = Capa exterior
part-head = Cap
part-body = Tors
part-right-arm = Braç dret
part-left-arm = Braç esquerre
part-right-leg = Cama dreta
part-left-leg = Cama esquerra
part-label-overlay = { $part } · ext.
part-select-hint = Fes clic per seleccionar la part (serveix per al mirall i per copiar parts).

untitled = Sense títol
status-opened = Obert: { $file }
status-imported = Importada: { $file }. Fes servir «Anomena i desa…» per crear el projecte.
status-saved = Desat: { $file }
status-exported = Exportat: { $file }
status-error = Error: { $detail }

error-not-png = El fitxer no és un PNG vàlid.
error-size = La skin fa { $w }×{ $h }. Només s'accepten 64×64 i 64×32.
error-newer = Aquest projecte és d'una versió més nova de l'aplicació (format { $version }). Actualitza l'aplicació per obrir-lo.
error-corrupt = Les dades del projecte estan malmeses.
error-io = No s'ha pogut accedir al fitxer: { $detail }

discard-question = Hi ha canvis sense desar. Què vols fer?
discard-save = Desa
discard-discard = Descarta els canvis
cancel = Cancel·la
ok = D'acord

convert-title = Converteix a { $model }
convert-note = Els braços canvien d'amplada. En passar a Alex es treu una columna per cara i en passar a Steve se'n duplica una. Ho pots desfer.
convert-before = Abans
convert-after = Després
convert-apply = Aplica

export-translucent =
    { $count ->
        [one] Hi ha 1 píxel de la capa base que no és opac.
       *[other] Hi ha { $count } píxels de la capa base que no són opacs.
    } El joc els mostrarà opacs.
export-anyway = Exporta igualment

settings-title = Configuració
settings-theme = Tema
theme-system = Sistema
theme-dark = Fosc
theme-light = Clar
settings-pixel-size = Mida del píxel
settings-grid = Mostra la quadrícula
settings-labels = Mostra els noms de les parts
settings-undo = Nivells de desfer
settings-shortcuts = Dreceres
shortcut-press = Prem una combinació… (Esc cancel·la)
shortcut-none = Sense drecera
shortcut-clear = Treu
shortcut-reset = Restableix les dreceres
shortcut-clash = Aquesta combinació ja la fa servir «{ $action }». Treu-la d'allà primer.
key-ctrl = Ctrl
key-shift = Maj
key-alt = Alt

view-2d = 2D
view-3d = 3D
view-reset = Restableix la vista
view3d-unavailable = La vista 3D no està disponible en aquest ordinador.
view3d-hint = Clic esquerre: pinta amb llapis, goma, pot o comptagotes, i selecciona la part. Arrossega fora del personatge: el mou. Clic dret i arrossega: gira. Roda: apropa o allunya.
capture = Captura la imatge…
status-captured = Captura desada: { $file }
error-capture = No s'ha pogut obtenir la imatge de la vista 3D.

pose = Postura
pose-custom = Personalitzada
pose-standing = Dret
pose-walking = Caminant
pose-waving = Saludant
pose-arm-up = Braç enlaire
pose-crouching = Ajupit
pose-running = Corrent
pose-combat = Combat
pose-neutral = Neutra

parts-copy-title = Copia parts
parts-copy-note = Marca les parts que vols copiar. Pots obrir una altra skin i enganxar-les-hi.
parts-copy = Copia
parts-paste-title = Enganxa parts
parts-paste-note = Marca les parts que entren. Substitueixen el que hi hagi en aquestes parts; ho pots desfer.
parts-paste = Enganxa
status-parts-copied = Parts copiades.
status-stamp-copied = Selecció copiada al segell.

file-quick-export = Exportació ràpida
export-overwrite = Ja existeix «{ $file }». Vols substituir-lo?
export-replace = Substitueix

view-preview = Previsualització
preview-base = Base:
preview-mannequin = Maniquí
preview-current = Skin actual
preview-loaded = Skin carregada
preview-load-skin = Carrega una skin…
preview-cape = Capa
preview-cape-color = Color de la capa
preview-load-cape = Carrega una capa…
preview-remove-cape = Treu la capa carregada
preview-notice = Vista aproximada: no és exactament com es veurà al joc. El que carreguis aquí no es desa.
error-cape = La capa ha de fer 64×32 (o un múltiple, com 128×64) o 22×17.

settings-general = General
settings-editor = Editor
settings-3d = Vista 3D
settings-export = Exportació
settings-advanced = Avançat
settings-language = Idioma
lang-system = Idioma del sistema
lang-es = Español
lang-en = English
lang-pt = Português
lang-ca = Català
settings-start-view = Vista en iniciar
start-view-last = L'última feta servir
settings-open-last = Obre l'última skin en iniciar
settings-reduce-motion = Redueix les animacions
settings-autosave = Desament automàtic
settings-autosave-hint = Només desa projectes que ja tenen fitxer.
unit-minutes = { " " }min
settings-start-tool = Eina inicial
settings-background = Fons
settings-background-custom = Color propi
settings-lighting = Il·luminació
settings-rotate-speed = Velocitat de rotació
settings-shadow = Ombra a terra
settings-capture-note = La captura d'imatge sempre surt amb fons transparent i sense ombra.
settings-export-dir = Carpeta predeterminada
settings-export-dir-none = Cap
settings-choose = Tria…
settings-export-name = Nom del fitxer
settings-export-name-hint = Fes servir {"{"}name{"}"} per al nom del projecte i {"{"}model{"}"} per a Steve o Alex.
settings-confirm-overwrite = Confirma abans de substituir
settings-export-note = «Exportació ràpida» desa directament a la carpeta predeterminada amb aquest nom. Sense carpeta, pregunta on desar.
settings-reset = Restableix la configuració
settings-reset-question = Vols tornar tota la configuració als valors per defecte? La paleta de colors es conserva.
settings-view-log = Mostra el registre
settings-data-path = Camins de dades i de configuració:
log-title = Registre
log-refresh = Actualitza
log-empty = El registre és buit.

tab-close = Tanca la pestanya
tab-next = Pestanya següent
tab-prev = Pestanya anterior

view-library = Biblioteca
library-search = Cerca per nom o etiqueta…
library-favorites = ★ Només preferides
library-new = Skin nova
library-import = Importa PNG…
library-refresh = Actualitza
library-no-match = Cap skin coincideix amb la cerca.
library-no-dir = No s'ha trobat cap carpeta per a la biblioteca. Tria'n una a Configuració → Biblioteca.
status-library-imported =
    { $count ->
        [one] S'ha importat 1 skin.
       *[other] S'han importat { $count } skins.
    }
card-open = Obre
card-edit = Edita la targeta…
card-duplicate = Duplica
card-delete = Elimina…
card-favorite = Marca o desmarca com a preferida
card-more = Més opcions
card-title = Edita la targeta
card-name = Nom
card-name-hint = Buit: es fa servir el nom del fitxer
card-tags = Etiquetes
card-tags-hint = Separades per comes
card-favorite-check = Preferida
card-background = Fons
card-add-background = Afegeix un fons…
card-save = Desa
card-delete-question = Vols eliminar «{ $name }»? El fitxer s'esborra de la carpeta i no es pot desfer.
card-delete-confirm = Elimina
bg-slate = Pissarra
bg-sky = Cel
bg-night = Nit
bg-meadow = Prat
bg-sunset = Posta de sol
bg-cave = Cova
settings-library = Biblioteca
settings-library-dir = Carpeta
settings-library-default = Fes servir la predeterminada
settings-card-size = Mida de les targetes
settings-clear-cache = Neteja la memòria cau de miniatures
status-cache-cleared = S'ha netejat la memòria cau de miniatures.

file-new-base = Nova a partir de la plantilla base
edit-apply-template = Aplica una plantilla
template-color-note = La samarreta, els pantalons i els cabells fan servir el color principal.
template-shirt = Samarreta
template-pants = Pantalons
template-hair = Cabells
template-armor = Armadura

anim-play = ▶ Anima
anim-mirror = Mirall de la postura
anim-target = Postura cap a la qual va i torna
anim-speed = Velocitat
joint-title = Ajusta la postura
joint-hint = Fes clic en una part a la vista 3D per ajustar-ne l'articulació.
joint-reset = Redreça
joint-axes = X: endavant i enrere. Y: girar. Z: cap als costats.

edit-image-front = Imatge al davant…
import-images = Imatges
import-title = Imatge al davant
import-note = Arrossega sobre la imatge per retallar-la. Es pinta al davant del cap, el tors, els braços i les cames; ho pots desfer.
import-fit-contain = Ajusta
import-fit-cover = Omple
import-whole = Fes servir la imatge sencera
error-image = No s'ha pogut obrir la imatge. S'accepten PNG i JPEG.
about-version = Deepslate Studio { $version } · Llicència GPLv3

home-title = Comença una skin nova
home-subtitle = La teva biblioteca encara és buida. Tria com començar:
home-drop-hint = També pots arrossegar fitxers PNG a aquesta finestra.
start-empty-steve = Buida · Steve
start-steve-desc = Llenç transparent, amb braços de 4 píxels.
start-empty-alex = Buida · Alex
start-alex-desc = Llenç transparent, amb braços de 3 píxels.
start-base = Plantilla base
start-base-desc = Pell i cara a punt per vestir amb les altres plantilles.
start-open = Obre un fitxer…
start-open-desc = Una skin o un projecte de qualsevol carpeta.
start-import = Importa PNG…
start-import-desc = Copia les teves skins a la biblioteca per veure-les com a targetes.
drop-open = Deixa anar per obrir
drop-library = Deixa anar per afegir a la biblioteca
error-drop-library = A la biblioteca només hi entren skins PNG de 64×64 o 64×32. Per fer servir una altra imatge, deixa-la anar a l'editor.
tool-lighten = Aclareix
tool-darken = Enfosqueix
action-toggle-overlay = Mostra o amaga la capa exterior
view-front = Vista de davant
view-back = Vista d'esquena
view-side = Vista de costat
view-focus-part = Enfoca la part seleccionada
menu-view = Visualització
view-split = Vista dividida (3D i 2D)
brush-size = Pinzell
brush-hint = Mida del pinzell per al llapis, la goma, aclarir i enfosquir
line-hint = Maj + clic: línia recta des de l'últim punt pintat
edit-replace-color = Substitueix el color…
replace-title = Substitueix el color
replace-from = Color a canviar
replace-to = Color nou
replace-scope-all = Tota la skin
replace-scope-part = Només { $part }
replace-apply = Substitueix
edit-vary = Varia el to
edit-vary-hint = Afegeix petites diferències de brillantor a la selecció, a la part seleccionada o a tota la skin
settings-ui-scale = Escala de la interfície
render-title = Exporta el render
render-size = Mida
render-transparent = Fons transparent
render-export = Exporta…
