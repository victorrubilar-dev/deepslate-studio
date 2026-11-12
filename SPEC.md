# SPEC: Editor de skins de Minecraft

Nombre: Deepslate Studio.
Estado: borrador v1. Las decisiones marcadas como "cerrada" no se cambian sin conversar.

## 1. Visión

Editor de skins de escritorio, rápido y sin cuenta ni publicidad. Funciona en Linux y Windows, se entrega en cuatro idiomas y tiene una identidad visual propia.

## 2. Decisiones

| Tema | Decisión | Estado |
|---|---|---|
| Lenguaje | Rust | cerrada |
| Interfaz | egui (eframe) | cerrada |
| Vista 3D | wgpu | cerrada |
| Formato de skin | 64×64 estándar. 64×32 se importa y convierte | cerrada |
| Plataformas | Linux (principal) y Windows | cerrada |
| Licencia | GPLv3 | cerrada |
| Idiomas | español (base), inglés, portugués, catalán | cerrada |
| Fondos de tarjeta | predefinidos (estilo propio) y personalizados | cerrada |
| Nombre | Deepslate Studio (pendiente revisar disponibilidad y marca) | elegida, por confirmar |

## 3. Modelo de skin

- Textura de 64×64 píxeles, PNG con transparencia.
- Partes: cabeza, torso, brazo derecho, brazo izquierdo, pierna derecha, pierna izquierda.
- Cada parte tiene capa base y capa overlay.
- Modelo **Steve**: brazos de 4 px de ancho. Modelo **Alex**: brazos de 3 px.
- Conversión Steve ↔ Alex: reacomoda las columnas de los brazos. Muestra vista previa antes de aplicar.
- Conversión 64×32 → 64×64: copia la pierna y el brazo derecho hacia el lado izquierdo (espejados) y deja los overlays de extremidades vacíos.

## 4. Funciones por módulo

### 4.1 Editor (hitos 1 y 2)
- Vista 2D (textura desplegada, edición píxel a píxel).
- Vista 3D (modelo, selección de partes con clic).
- Botón para cambiar entre 2D y 3D. Recuerda la última elección.
- Herramientas: lápiz, borrador, bote, cuentagotas, degradado, sello, selección rectangular.
- Espejo global y por parte.
- Visibilidad por parte y por capa (base, overlay o ambas).
- Cuadrícula y etiquetas de partes.
- Deshacer y rehacer, con pila de comandos.

### 4.2 Colores (hito 2)
- Selector con código hex.
- Historial de colores usados.
- Paleta propia con nombres.
- Importar paleta desde imagen o desde otra skin.

### 4.3 Partes (hito 2)
- Copiar partes marcadas desde una skin abierta.
- Pegar eligiendo qué partes entran.

### 4.4 Poses (hito 2)
- Poses predefinidas: de pie, caminando, saludando, brazo arriba, agachado, corriendo, combate, neutra.
- Rotación arrastrando el personaje.
- La pose se guarda con la skin.
- Ajuste fino por articulación: hito 4.

### 4.5 Plantillas (hito 3)
- Base, camiseta, pantalón, pelo, armadura.

### 4.6 Biblioteca en tarjetas (hito 3)
- Tarjeta con: nombre editable, fondo temático, personaje en pose, etiqueta de modelo (Steve/Alex), fecha de última edición, favorito.
- Acciones: abrir, favorito, menú de más opciones.
- Fondos predefinidos (estilo propio, no copiados) y fondos personalizados subidos por el usuario.
- Miniaturas en caché para que la biblioteca cargue rápido.
- Búsqueda por nombre y etiqueta.

### 4.7 Pestañas (hito 3)
- Varias skins abiertas a la vez.
- Guardado independiente por pestaña.

### 4.8 Importar y exportar (hito 1, ampliado en hito 2)
- Importar y exportar PNG.
- Validación de formato antes de exportar.
- Importar 64×32 con conversión automática.
- Captura de la vista 3D (hito 2).

### 4.9 Ajustes (hito 1 básico, completo en hito 2)
- **General:** idioma, tema (claro, oscuro, sistema), vista por defecto (2D o 3D), abrir la última skin al iniciar.
- **Editor:** tamaño de píxel, cuadrícula visible, niveles de deshacer, autoguardado y su intervalo, herramienta inicial.
- **Atajos:** editables, con detección de choques.
- **Vista 3D:** fondo, iluminación, velocidad de rotación, sombras.
- **Biblioteca:** carpeta de datos, tamaño de tarjetas, limpiar caché.
- **Exportación:** carpeta predeterminada, formato del nombre de archivo, confirmar sobrescritura.
- **Avanzado:** restablecer ajustes, ver registro, ruta de datos.
- Formato: archivo legible (RON o TOML), respaldable.

### 4.10 Idiomas (hito 1 en español, hito 3 en los demás)
- Español, inglés, portugués, catalán.
- Textos en archivos de traducción separados, nunca en el código.
- Detecta el idioma del sistema. Si falta una traducción, usa español.
- Cambio desde Ajustes, sin reiniciar.
- Sin traducir: nombres propios (Steve, Alex) y el nombre de la app.

### 4.11 Estilo visual (hito 3)
- Tarjetas con bordes tipo bloque y sombras marcadas.
- Tipografía pixel para títulos, fuente limpia para el resto.
- Acento propio, distinto de Nova Skin.
- Modo oscuro por defecto.
- Sin logos, texturas oficiales ni el nombre "Minecraft" en la interfaz.

### 4.12 Previsualizador (hito 2, con base simple en hito 1)
- Maniquí neutro por defecto como base de prueba. El usuario puede cargar otra skin como base.
- Capa de la espalda: color liso seleccionable, blanco por defecto. Mostrar u ocultar.
- Cargar una capa PNG de 64×32 que el usuario consiga por su cuenta. La app no la guarda ni la incluye en el repositorio ni en los instaladores.
- Vista 3D con el mismo ángulo y estilo de render que la vista del editor.
- Aviso en la interfaz: es una aproximación, no el resultado exacto del juego.
- La app no aplica capas oficiales de las cuentas de Minecraft.

### 4.13 Vista 3D avanzada (hito 4)
- Importar imagen y convertirla a skin, con recorte y ajuste.
- Animaciones simples entre poses.
- Pulido general.

## 5. Hitos

**Hito 1: núcleo**
- Modelo de skin, capas, Steve y Alex, conversión 64×32.
- Editor 2D con lápiz, borrador, bote, cuentagotas.
- Overlay y visibilidad de capas.
- Deshacer y rehacer.
- Guardar proyecto y exportar PNG.
- Ajustes básicos. Idioma español.
- Compila en Linux. Pruebas en `core/` y `project/`.

**Hito 2: 3D y edición avanzada**
- Vista 3D con botón 2D/3D.
- Selección de partes y espejo por parte.
- Copiar y pegar partes.
- Paleta, historial de colores, atajos.
- Poses predefinidas.
- Ajustes completos.
- Compila en Windows también (CI con GitHub Actions).

**Hito 3: biblioteca e idiomas**
- Pestañas.
- Biblioteca en tarjetas con fondos y poses.
- Plantillas.
- Traducciones en inglés, portugués y catalán.
- Estilo visual.

**Hito 4: extras**
- Importar imagen.
- Ajuste fino de poses y animaciones.
- Pulido.

## 6. Criterios de calidad

- Cada hito compila y pasa las pruebas en Linux y Windows.
- `core/` y `project/` tienen pruebas unitarias.
- Ningún texto visible queda fuera de `assets/i18n/`.
- Ningún cambio en el formato de skin o de proyecto sin pruebas de compatibilidad.

## 7. Fuera de alcance (por ahora)

- Cuentas de usuario y sincronización en la nube.
- Subir skins a servicios de Mojang o de terceros.
- Edición de otros formatos de Minecraft (Bedrock, recursos, mods).
