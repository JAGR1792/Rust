# 🦀 Proyecto de Hilos en Rust  

Este proyecto muestra cómo utilizar **hilos (threads)** en Rust para realizar operaciones concurrentes de manera eficiente.  

## 📌 Características  
- Creación y gestión de múltiples hilos.  
- Sincronización entre hilos utilizando **Mutex** y **Arc**.  
- Implementación de comunicación entre hilos.  

## 🚀 Requisitos  
Antes de comenzar, asegúrate de tener instalado Rust en tu sistema. Puedes instalarlo desde [rustup.rs](https://rustup.rs/).  

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
