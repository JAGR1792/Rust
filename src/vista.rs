// vista.rs
use crate::modelo::*;
use ggez::{graphics, Context, GameResult};
use std::sync::Mutex;
use std::sync::Once;

// Estructuras para el caché de meshes
struct MeshCache {
    lineas_h: Vec<graphics::Mesh>,
    lineas_v: Vec<graphics::Mesh>,
    bases_semaforos: graphics::Mesh,
    vehiculos: [graphics::Mesh; 3], // Automóvil, Camioneta, Camión
}

impl MeshCache {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let mut lineas_h = Vec::with_capacity(30);
        let mut lineas_v = Vec::with_capacity(30);

        for i in 0..30 {
            lineas_h.push(graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                graphics::Rect::new(i as f32 * 20.0, 325.0, 10.0, 4.0),
                COLOR_LINEA_CENTRAL,
            )?);

            lineas_v.push(graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                graphics::Rect::new(325.0, i as f32 * 20.0, 4.0, 10.0),
                COLOR_LINEA_CENTRAL,
            )?);
        }

        let bases_semaforos = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(0.0, 0.0, 10.0, 30.0),
            graphics::Color::new(0.3, 0.3, 0.3, 1.0),
        )?;

        let vehiculos = [
            // Automóvil
            graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                graphics::Rect::new(0.0, 0.0, 30.0, 15.0),
                graphics::Color::WHITE, // El color se aplica en tiempo de dibujo
            )?,
            // Camioneta
            graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                graphics::Rect::new(0.0, 0.0, 35.0, 18.0),
                graphics::Color::WHITE,
            )?,
            // Camión
            graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                graphics::Rect::new(0.0, 0.0, 45.0, 20.0),
                graphics::Color::WHITE,
            )?,
        ];

        Ok(Self {
            lineas_h,
            lineas_v,
            bases_semaforos,
            vehiculos,
        })
    }
}

// Patrón singleton thread-safe para nuestro caché
struct MeshCacheSingleton {
    cache: Mutex<Option<MeshCache>>,
    init: Once,
}

// Instancia global segura
static MESH_CACHE_SINGLETON: MeshCacheSingleton = MeshCacheSingleton {
    cache: Mutex::new(None),
    init: Once::new(),
};

pub fn inicializar_cache(ctx: &mut Context) -> GameResult {
    MESH_CACHE_SINGLETON.init.call_once(|| {
        let cache = MeshCache::new(ctx).unwrap(); // Manejo básico de errores
        *MESH_CACHE_SINGLETON.cache.lock().unwrap() = Some(cache);
    });
    Ok(())
}

fn get_cache<'a>() -> Option<std::sync::MutexGuard<'a, Option<MeshCache>>> {
    match MESH_CACHE_SINGLETON.cache.lock() {
        Ok(guard) => Some(guard),
        Err(_) => None, // Error de mutex envenenado
    }
}

pub fn dibujar_carreteras(canvas: &mut graphics::Canvas, ctx: &mut Context) -> GameResult {
    // Asfalto
    canvas.draw(&graphics::Quad, graphics::DrawParam::new()
        .dest_rect(VIA_HORIZONTAL)
        .color(COLOR_ASFALTO));

    canvas.draw(&graphics::Quad, graphics::DrawParam::new()
        .dest_rect(VIA_VERTICAL)
        .color(COLOR_ASFALTO));

    // Inicializar caché si es necesario
    MESH_CACHE_SINGLETON.init.call_once(|| {
        if let Ok(cache) = MeshCache::new(ctx) {
            *MESH_CACHE_SINGLETON.cache.lock().unwrap() = Some(cache);
        }
    });

    // Dibujar líneas centrales desde el caché
    if let Some(ref guard) = get_cache() {
        if let Some(ref cache) = **guard {
            // Dibujar líneas usando el caché
            for linea in &cache.lineas_h {
                canvas.draw(linea, graphics::DrawParam::default());
            }

            for linea in &cache.lineas_v {
                canvas.draw(linea, graphics::DrawParam::default());
            }
        }
    }

    Ok(())
}

pub fn dibujar_semaforo(canvas: &mut graphics::Canvas, ctx: &mut Context, semaforo: &Semaforo) -> GameResult {
    let color = match semaforo.estado {
        EstadoSemaforo::Verde => graphics::Color::GREEN,
        EstadoSemaforo::Amarillo => graphics::Color::YELLOW,
        EstadoSemaforo::Rojo => graphics::Color::RED,
    };

    // Usar base desde caché
    if let Some(ref guard) = get_cache() {
        if let Some(ref cache) = **guard {
            canvas.draw(&cache.bases_semaforos, graphics::DrawParam::new()
                .dest([semaforo.posicion[0] - 5.0, semaforo.posicion[1] - 15.0]));
        }
    } else {
        // Fallback si no hay caché
        let base = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(0.0, 0.0, 10.0, 30.0),
            graphics::Color::new(0.3, 0.3, 0.3, 1.0),
        )?;
        canvas.draw(&base, graphics::DrawParam::new()
            .dest([semaforo.posicion[0] - 5.0, semaforo.posicion[1] - 15.0]));
    }

    // La luz del semáforo siempre se renderiza dinámicamente por su cambio de color
    let luz = graphics::Mesh::new_circle(
        ctx,
        graphics::DrawMode::fill(),
        [semaforo.posicion[0], semaforo.posicion[1]],
        8.0,
        0.1,
        color,
    )?;
    canvas.draw(&luz, graphics::DrawParam::default());

    Ok(())
}

pub fn dibujar_vehiculo(canvas: &mut graphics::Canvas, ctx: &mut Context, carro: &Carro) -> GameResult {
    let mesh_idx = match carro.tipo {
        TipoVehiculo::Automovil => 0,
        TipoVehiculo::Camioneta => 1,
        TipoVehiculo::Camion => 2,
    };

    let rotacion = match carro.direccion {
        "este" => 0.0,
        "norte" => std::f32::consts::FRAC_PI_2,
        _ => 0.0,
    };

    // Usar el mesh del vehículo desde caché
    if let Some(ref guard) = get_cache() {
        if let Some(ref cache) = **guard {
            canvas.draw(&cache.vehiculos[mesh_idx], graphics::DrawParam::new()
                .dest(carro.posicion)
                .rotation(rotacion)
                .color(carro.color));
            return Ok(());
        }
    }

    // Fallback si no hay caché
    let (ancho, alto) = match carro.tipo {
        TipoVehiculo::Automovil => (30.0, 15.0),
        TipoVehiculo::Camioneta => (35.0, 18.0),
        TipoVehiculo::Camion => (45.0, 20.0),
    };

    let cuerpo = graphics::Mesh::new_rectangle(
        ctx,
        graphics::DrawMode::fill(),
        graphics::Rect::new(0.0, 0.0, ancho, alto),
        carro.color,
    )?;

    canvas.draw(&cuerpo, graphics::DrawParam::new()
        .dest(carro.posicion)
        .rotation(rotacion));

    Ok(())
}

pub fn dibujar_ui(
    canvas: &mut graphics::Canvas,
    num_vehiculos: usize,
    direccion_activa: &str,
    fps: usize
) -> GameResult {
    let texto = graphics::Text::new(format!(
        "Vehículos: {}\nDirección activa: {}\nFPS: {}",
        num_vehiculos,
        direccion_activa,
        fps
    ));

    canvas.draw(&texto, graphics::DrawParam::new()
        .dest([10.0, 10.0])
        .color(graphics::Color::WHITE));

    Ok(())
}