# Proyecto 2 - Raytracer

Proyecto para el curso de gráficas por computadora que implementa un raytracer en Rust capaz de renderizar un diorama del Nether de Minecraft. \
**Video**: https://youtu.be/lVsxrPhxKyI

## Características

### Renderizado
- **Ray tracing completo** con soporte para reflexión, refracción y transparencia
- **Multithreading** usando todos los núcleos del CPU disponibles
- **Skybox texturizado** para ambientación del Nether
- **Materiales emisivos** (Shroomlight emite luz)

### Materiales Disponibles
- `obsidian` - Bloques oscuros y ligeramente reflectivos
- `shroomlight` - Fuente de luz naranja brillante (emisiva)
- `crimson_nylium` - Bloque de tierra del Nether
- `crimson_stem` - Tronco/madera
- `nether_wart_block` - Bloque orgánico rojo oscuro
- `portal` - Efecto de portal con textura (semi-transparente)

### Cámara
- **Movimiento orbital** alrededor del centro de la escena
- **Zoom** hacia/desde el punto focal
- **Límite de ángulo** para evitar gimbal lock

## Instalación

1. Clonar el repositorio:
```bash
git clone https://github.com/vicperezch/graphics-project2.git
cd raytracer
```

2. Compilar y ejecutar:
```bash
cargo run
```

## Configuración de Escenas

El proyecto utiliza un archivo `scene.txt` en la raíz con el siguiente formato:

```txt
# Formato: x y z tamaño material
0.0 0.0 0.0 1.0 crimson_nylium
1.0 0.0 0.0 1.0 obsidian
2.0 1.0 0.0 0.5 shroomlight
```
