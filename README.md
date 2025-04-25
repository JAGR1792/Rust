# 🦀 Proyecto de Hilos en Rust  

Este proyecto muestra cómo utilizar **hilos (threads)** en Rust para realizar operaciones concurrentes de manera eficiente (Principalmente la eficiencia). 
- Se crea una simulacion de un cruce semaforico, donde cada vehículo, y los semafóros, son un hilo diferente.
  
## 📌 Características  
- Creación y gestión de múltiples hilos.  
- Sincronización entre hilos utilizando **Mutex** y **Arc**.  
- Implementación de comunicación entre hilos.  

## 🚀 Requisitos  
- Antes de comenzar, asegúrate de tener instalado Rust en tu sistema. Puedes instalarlo desde [rustup.rs](https://rustup.rs/).  
- Descargue el IDE de RUSTROVER, Facilitara la instalacion del codigo (si instala RustRover se puede saltar el paso de instalar Rustup, pues viene incluido). 

## 📂 Instalación  
Sigue estos pasos para configurar el proyecto correctamente:  

1. **Descargar el código fuente:**  
   - Descarga la carpeta `src` y coloca los archivos `.rs` dentro del directorio donde se genera el `main.rs`.  
   - Abre el archivo `Cargo.toml` y copia las **dependencias** necesarias para el proyecto.  

2. **Ejecutar comandos en la terminal:**  
   Ejecuta los siguientes comandos en este orden para asegurar una instalación limpia y actualizada:  
   ```bash
   cargo clean
   cargo update
   cargo build
3. **Si tienen algun problema aqui esta la BIBLIA DE RUST**
   -[LA_BIBLIA](https://doc.rust-lang.org/error_codes/error-index.html)
