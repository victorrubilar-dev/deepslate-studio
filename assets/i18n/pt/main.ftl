# Português (Brasil).

menu-file = Arquivo
menu-edit = Editar
menu-settings = Configurações
file-new = Novo
file-open = Abrir…
file-save = Salvar
file-save-as = Salvar como…
file-export = Exportar PNG…
edit-undo = Desfazer
edit-redo = Refazer
edit-copy-selection = Copiar seleção para o carimbo
edit-erase-selection = Apagar seleção
edit-deselect = Remover seleção
edit-copy-parts = Copiar partes…
edit-paste-parts = Colar partes…

tools = Ferramentas
tool-pencil = Lápis
tool-eraser = Borracha
tool-fill = Balde de tinta
tool-picker = Conta-gotas
tool-gradient = Degradê
tool-stamp = Carimbo
tool-select = Seleção
stamp-empty = O carimbo está vazio. Selecione uma área e use “Copiar seleção para o carimbo”.
mirror = Espelho:
mirror-off = Não
mirror-all = Tudo
mirror-part = Parte selecionada
mirror-pick-part = Escolha uma parte na lista abaixo ou clique nela na visão 3D.
action-toggle-mirror = Ativar ou desativar espelho
action-swap-colors = Trocar cores
action-toggle-view = Alternar entre 2D e 3D
color = Cor
color-primary = Cor principal
color-secondary = Cor secundária (fim do degradê)
color-hex = Código hex: #RRGGBB ou #RRGGBBAA
color-recent = Usadas recentemente
palette = Paleta
palette-name = Nome
palette-add = Adicionar cor atual
palette-remove = Remover da paleta
palette-import = Importar de imagem ou skin…
palette-hint = Clique com o botão direito em uma cor para renomeá-la ou removê-la.
status-palette-imported =
    { $count ->
        [one] 1 cor adicionada à paleta.
       *[other] { $count } cores adicionadas à paleta.
    }
model-current = Modelo: { $model }
model-convert = Converter para { $model }…

layers = Visibilidade
layer-base = Base
layer-overlay = Camada externa
part-head = Cabeça
part-body = Tronco
part-right-arm = Braço direito
part-left-arm = Braço esquerdo
part-right-leg = Perna direita
part-left-leg = Perna esquerda
part-label-overlay = { $part } · ext.
part-select-hint = Clique para selecionar a parte (serve para o espelho e para copiar partes).

untitled = Sem título
status-opened = Aberto: { $file }
status-imported = Importada: { $file }. Use “Salvar como…” para criar o projeto.
status-saved = Salvo: { $file }
status-exported = Exportado: { $file }
status-error = Erro: { $detail }

error-not-png = O arquivo não é um PNG válido.
error-size = A skin mede { $w }×{ $h }. Só são aceitos 64×64 e 64×32.
error-newer = Este projeto é de uma versão mais nova do app (formato { $version }). Atualize o app para abri-lo.
error-corrupt = Os dados do projeto estão danificados.
error-io = Não foi possível acessar o arquivo: { $detail }

discard-question = Há alterações não salvas. O que você quer fazer?
discard-save = Salvar
discard-discard = Descartar alterações
cancel = Cancelar
ok = OK

convert-title = Converter para { $model }
convert-note = Os braços mudam de largura. Ao passar para Alex, uma coluna por face é removida; ao passar para Steve, uma é duplicada. Você pode desfazer.
convert-before = Antes
convert-after = Depois
convert-apply = Aplicar

export-translucent =
    { $count ->
        [one] 1 pixel da camada base não é opaco.
       *[other] { $count } pixels da camada base não são opacos.
    } O jogo vai mostrá-los opacos.
export-anyway = Exportar mesmo assim

settings-title = Configurações
settings-theme = Tema
theme-system = Sistema
theme-dark = Escuro
theme-light = Claro
settings-pixel-size = Tamanho do pixel
settings-grid = Mostrar grade
settings-labels = Mostrar nomes das partes
settings-undo = Níveis de desfazer
settings-shortcuts = Atalhos
shortcut-press = Pressione uma combinação… (Esc cancela)
shortcut-none = Sem atalho
shortcut-clear = Remover
shortcut-reset = Restaurar atalhos
shortcut-clash = Essa combinação já é usada por “{ $action }”. Remova-a lá primeiro.
key-ctrl = Ctrl
key-shift = Shift
key-alt = Alt

view-2d = 2D
view-3d = 3D
view-reset = Restaurar visão
view3d-unavailable = A visão 3D não está disponível neste computador.
view3d-hint = Clique esquerdo: pinta com lápis, borracha, balde ou conta-gotas, e seleciona a parte. Clique direito e arraste: gira. Roda: aproxima ou afasta.
capture = Capturar imagem…
status-captured = Captura salva: { $file }
error-capture = Não foi possível obter a imagem da visão 3D.

pose = Pose
pose-custom = Personalizada
pose-standing = Em pé
pose-walking = Andando
pose-waving = Acenando
pose-arm-up = Braço erguido
pose-crouching = Agachado
pose-running = Correndo
pose-combat = Combate
pose-neutral = Neutra

parts-copy-title = Copiar partes
parts-copy-note = Marque as partes que quer copiar. Você pode abrir outra skin e colá-las lá.
parts-copy = Copiar
parts-paste-title = Colar partes
parts-paste-note = Marque as partes que entram. Elas substituem o que houver nessas partes; você pode desfazer.
parts-paste = Colar
status-parts-copied = Partes copiadas.
status-stamp-copied = Seleção copiada para o carimbo.

file-quick-export = Exportação rápida
export-overwrite = “{ $file }” já existe. Quer substituí-lo?
export-replace = Substituir

view-preview = Pré-visualização
preview-base = Base:
preview-mannequin = Manequim
preview-current = Skin atual
preview-loaded = Skin carregada
preview-load-skin = Carregar skin…
preview-cape = Capa
preview-cape-color = Cor da capa
preview-load-cape = Carregar capa…
preview-remove-cape = Remover capa carregada
preview-notice = Visão aproximada: não é exatamente como vai aparecer no jogo. O que você carregar aqui não é salvo.
error-cape = A capa deve medir 64×32 (ou um múltiplo, como 128×64) ou 22×17.

settings-general = Geral
settings-editor = Editor
settings-3d = Visão 3D
settings-export = Exportação
settings-advanced = Avançado
settings-language = Idioma
lang-system = Idioma do sistema
lang-es = Español
lang-en = English
lang-pt = Português
lang-ca = Català
settings-start-view = Visão ao iniciar
start-view-last = A última usada
settings-open-last = Abrir a última skin ao iniciar
settings-reduce-motion = Reduzir animações
settings-autosave = Salvamento automático
settings-autosave-hint = Só salva projetos que já têm arquivo.
unit-minutes = { " " }min
settings-start-tool = Ferramenta inicial
settings-background = Fundo
settings-background-custom = Cor personalizada
settings-lighting = Iluminação
settings-rotate-speed = Velocidade de rotação
settings-shadow = Sombra no chão
settings-capture-note = A captura de imagem sempre sai com fundo transparente e sem sombra.
settings-export-dir = Pasta padrão
settings-export-dir-none = Nenhuma
settings-choose = Escolher…
settings-export-name = Nome do arquivo
settings-export-name-hint = Use {"{"}name{"}"} para o nome do projeto e {"{"}model{"}"} para Steve ou Alex.
settings-confirm-overwrite = Confirmar antes de substituir
settings-export-note = A “Exportação rápida” salva direto na pasta padrão com este nome. Sem pasta, pergunta onde salvar.
settings-reset = Restaurar configurações
settings-reset-question = Voltar todas as configurações aos valores padrão? A paleta de cores é mantida.
settings-view-log = Ver registro
settings-data-path = Caminhos de dados e configurações:
log-title = Registro
log-refresh = Atualizar
log-empty = O registro está vazio.

tab-close = Fechar aba
tab-next = Próxima aba
tab-prev = Aba anterior

view-library = Biblioteca
library-search = Buscar por nome ou etiqueta…
library-favorites = ★ Só favoritas
library-new = Nova skin
library-import = Importar PNG…
library-refresh = Atualizar
library-no-match = Nenhuma skin corresponde à busca.
library-no-dir = Não foi encontrada uma pasta para a biblioteca. Escolha uma em Configurações → Biblioteca.
status-library-imported =
    { $count ->
        [one] 1 skin importada.
       *[other] { $count } skins importadas.
    }
card-open = Abrir
card-edit = Editar cartão…
card-duplicate = Duplicar
card-delete = Excluir…
card-favorite = Marcar ou desmarcar como favorita
card-more = Mais opções
card-title = Editar cartão
card-name = Nome
card-name-hint = Vazio: usa o nome do arquivo
card-tags = Etiquetas
card-tags-hint = Separadas por vírgulas
card-favorite-check = Favorita
card-background = Fundo
card-add-background = Adicionar fundo…
card-save = Salvar
card-delete-question = Excluir “{ $name }”? O arquivo é apagado da pasta e não dá para desfazer.
card-delete-confirm = Excluir
bg-slate = Ardósia
bg-sky = Céu
bg-night = Noite
bg-meadow = Campo
bg-sunset = Pôr do sol
bg-cave = Caverna
settings-library = Biblioteca
settings-library-dir = Pasta
settings-library-default = Usar a padrão
settings-card-size = Tamanho dos cartões
settings-clear-cache = Limpar cache de miniaturas
status-cache-cleared = Cache de miniaturas limpo.

file-new-base = Nova a partir do modelo base
edit-apply-template = Aplicar modelo
template-color-note = Camiseta, calça e cabelo usam a cor principal.
template-shirt = Camiseta
template-pants = Calça
template-hair = Cabelo
template-armor = Armadura

anim-play = ▶ Animar
anim-mirror = Espelho da pose
anim-target = Pose para a qual vai e volta
anim-speed = Velocidade
joint-title = Ajustar pose
joint-hint = Clique em uma parte na visão 3D para ajustar a articulação.
joint-reset = Endireitar
joint-axes = X: para frente e para trás. Y: girar. Z: para os lados.

edit-image-front = Imagem na frente…
import-images = Imagens
import-title = Imagem na frente
import-note = Arraste sobre a imagem para recortar. Ela é pintada na frente da cabeça, tronco, braços e pernas; você pode desfazer.
import-fit-contain = Ajustar
import-fit-cover = Preencher
import-whole = Usar a imagem inteira
error-image = Não foi possível abrir a imagem. São aceitos PNG e JPEG.
about-version = Deepslate Studio { $version } · Licença GPLv3

home-title = Comece uma skin nova
home-subtitle = Sua biblioteca ainda está vazia. Escolha como começar:
home-drop-hint = Você também pode arrastar arquivos PNG para esta janela.
start-empty-steve = Vazia · Steve
start-steve-desc = Tela transparente, com braços de 4 pixels.
start-empty-alex = Vazia · Alex
start-alex-desc = Tela transparente, com braços de 3 pixels.
start-base = Modelo base
start-base-desc = Pele e rosto prontos para vestir com os outros modelos.
start-open = Abrir arquivo…
start-open-desc = Uma skin ou projeto de qualquer pasta.
start-import = Importar PNG…
start-import-desc = Copie suas skins para a biblioteca para vê-las como cartões.
drop-open = Solte para abrir
drop-library = Solte para adicionar à biblioteca
error-drop-library = Na biblioteca só entram skins PNG de 64×64 ou 64×32. Para usar outra imagem, solte-a no editor.
