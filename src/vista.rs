// vista.rs
use crate::modelo::*;
use ggez::{graphics, Context, GameResult};
use std::sync::Mutex;
use std::sync::Once;
use rand::Rng;

// Estructuras para elementos decorativos
struct Arbol {
    posicion: [f32; 2],
    escala: f32,
    variacion: usize, // Para diferentes tipos de árboles
}

struct Edificio {
    posicion: [f32; 2],
    ancho: f32,
    alto: f32,
    color: graphics::Color,
    ventanas: bool,
}

// Estructuras para el caché de meshes
struct MeshCache {
    lineas_h: Vec<graphics::Mesh>,
    lineas_v: Vec<graphics::Mesh>,
    bases_semaforos: graphics::Mesh,
    vehiculos: [graphics::Mesh; 3], // Carro, Camioneta, Camión
    arboles: Vec<graphics::Mesh>,   // Arboles, redondos puntiagudos
    edificios: Vec<Edificio>,       // Lista de edificios
    nubes: Vec<graphics::Mesh>,     // Nubecitas
    banco: graphics::Mesh,          // Banco de parque
    fuente: graphics::Mesh,         // Fuente de agua
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

        // Crear meshes para árboles
        let mut arboles = Vec::new();

        // Árbol tipo 1 (Pino)
        let tronco = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(-3.0, 0.0, 6.0, 15.0),
            graphics::Color::new(0.6, 0.3, 0.0, 1.0),
        )?;
        arboles.push(tronco);

        // Copa pino (triángulo)
        let copa_pino = graphics::Mesh::new_polygon(
            ctx,
            graphics::DrawMode::fill(),
            &[
                [0.0, -25.0],
                [-12.0, 0.0],
                [12.0, 0.0],
            ],
            graphics::Color::new(0.0, 0.5, 0.0, 1.0),
        )?;
        arboles.push(copa_pino);

        // Árbol tipo 2 (Redondo)
        let tronco2 = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(-2.5, 0.0, 5.0, 12.0),
            graphics::Color::new(0.5, 0.25, 0.0, 1.0),
        )?;
        arboles.push(tronco2);

        // Copa redonda
        let copa_redonda = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            [0.0, -12.0],
            10.0,
            0.1,
            graphics::Color::new(0.1, 0.6, 0.1, 1.0),
        )?;
        arboles.push(copa_redonda);

        // Generar edificios
        let mut rng = rand::rng();
        let mut edificios = Vec::new();

        // Cuadrante superior izquierdo
        for _ in 0..6 {
            edificios.push(Edificio {
                posicion: [rng.random_range(20.0..250.0), rng.random_range(20.0..270.0)],
                ancho: rng.random_range(30.0..80.0),
                alto: rng.random_range(40.0..100.0),
                color: graphics::Color::new(
                    rng.random_range(0.4..0.8),
                    rng.random_range(0.4..0.8),
                    rng.random_range(0.4..0.9),
                    1.0,
                ),
                ventanas: true,
            });
        }

        // Cuadrante superior derecho
        for _ in 0..6 {
            edificios.push(Edificio {
                posicion: [rng.random_range(370.0..550.0), rng.random_range(20.0..270.0)],
                ancho: rng.random_range(30.0..80.0),
                alto: rng.random_range(40.0..100.0),
                color: graphics::Color::new(
                    rng.random_range(0.4..0.8),
                    rng.random_range(0.4..0.8),
                    rng.random_range(0.4..0.9),
                    1.0,
                ),
                ventanas: true,
            });
        }

        // Cuadrante inferior izquierdo
        for _ in 0..5 {
            edificios.push(Edificio {
                posicion: [rng.random_range(20.0..250.0), rng.random_range(370.0..550.0)],
                ancho: rng.random_range(30.0..80.0),
                alto: rng.random_range(40.0..100.0),
                color: graphics::Color::new(
                    rng.random_range(0.4..0.8),
                    rng.random_range(0.4..0.8),
                    rng.random_range(0.4..0.9),
                    1.0,
                ),
                ventanas: true,
            });
        }

        // Cuadrante inferior derecho
        for _ in 0..5 {
            edificios.push(Edificio {
                posicion: [rng.random_range(370.0..550.0), rng.random_range(370.0..550.0)],
                ancho: rng.random_range(30.0..80.0),
                alto: rng.random_range(40.0..100.0),
                color: graphics::Color::new(
                    rng.random_range(0.4..0.8),
                    rng.random_range(0.4..0.8),
                    rng.random_range(0.4..0.9),
                    1.0,
                ),
                ventanas: true,
            });
        }



        // Banco de parque
        let banco = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(0.0, 0.0, 30.0, 10.0),
            graphics::Color::new(0.6, 0.4, 0.2, 1.0),
        )?;

        // Fuente de agua
        let fuente = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            [0.0, 0.0],
            15.0,
            0.1,
            graphics::Color::new(0.0, 0.6, 0.9, 1.0),
        )?;

        Ok(Self {
            lineas_h,
            lineas_v,
            bases_semaforos,
            vehiculos,
            arboles,
            edificios,
            nubes: vec![],
            banco,
            fuente,
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

// vista.rs
pub fn dibujar_fondo(canvas: &mut graphics::Canvas, ctx: &mut Context) -> GameResult {
    // Cielo
    let cielo = graphics::Mesh::new_rectangle(
        ctx,
        graphics::DrawMode::fill(),
        graphics::Rect::new(0.0, 0.0, 600.0, 600.0),
        graphics::Color::new(0.5, 0.7, 0.9, 1.0),
    )?;
    canvas.draw(&cielo, graphics::DrawParam::default());

    // Pasto estático (4 rectángulos grandes alrededor de las carreteras)
    let color_pasto = graphics::Color::new(0.2, 0.5, 0.2, 1.0);

    // Zona superior (sobre la carretera horizontal)
    let pasto_superior = graphics::Mesh::new_rectangle(
        ctx,
        graphics::DrawMode::fill(),
        graphics::Rect::new(0.0, 0.0, 600.0, 295.0),
        color_pasto,
    )?;
    canvas.draw(&pasto_superior, graphics::DrawParam::default());

    // Zona inferior (debajo de la carretera horizontal)
    let pasto_inferior = graphics::Mesh::new_rectangle(
        ctx,
        graphics::DrawMode::fill(),
        graphics::Rect::new(0.0, 355.0, 600.0, 245.0),
        color_pasto,
    )?;
    canvas.draw(&pasto_inferior, graphics::DrawParam::default());

    // Zona izquierda (entre carreteras)
    let pasto_izquierdo = graphics::Mesh::new_rectangle(
        ctx,
        graphics::DrawMode::fill(),
        graphics::Rect::new(0.0, 295.0, 295.0, 60.0),
        color_pasto,
    )?;
    canvas.draw(&pasto_izquierdo, graphics::DrawParam::default());

    // Zona derecha (entre carreteras)
    let pasto_derecho = graphics::Mesh::new_rectangle(
        ctx,
        graphics::DrawMode::fill(),
        graphics::Rect::new(355.0, 295.0, 245.0, 60.0),
        color_pasto,
    )?;
    canvas.draw(&pasto_derecho, graphics::DrawParam::default());

    // Dibujar nubes (código existente)
    if let Some(ref guard) = get_cache() {
        if let Some(ref cache) = **guard {
            let posiciones_nubes = [
                [50.0, 50.0],
                [300.0, 80.0],
                [500.0, 60.0],
            ];

            for (i, nube) in cache.nubes.iter().enumerate() {
                if i < posiciones_nubes.len() {
                    canvas.draw(nube, graphics::DrawParam::new()
                        .dest(posiciones_nubes[i])
                        .scale([1.5, 1.0]));
                }
            }
        }
    }

    Ok(())
}

pub fn dibujar_elementos_decorativos(canvas: &mut graphics::Canvas, ctx: &mut Context) -> GameResult {
    if let Some(ref guard) = get_cache() {
        if let Some(ref cache) = **guard {
            // Dibujar edificios
            for edificio in &cache.edificios {
                let cuerpo = graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::fill(),
                    graphics::Rect::new(0.0, 0.0, edificio.ancho, edificio.alto),
                    edificio.color,
                )?;
                canvas.draw(&cuerpo, graphics::DrawParam::new()
                    .dest(edificio.posicion));

                // Añadir ventanas si corresponde
                if edificio.ventanas {
                    let filas = (edificio.alto / 15.0).floor() as i32;
                    let columnas = (edificio.ancho / 15.0).floor() as i32;

                    for i in 0..filas {
                        for j in 0..columnas {
                            let ventana = graphics::Mesh::new_rectangle(
                                ctx,
                                graphics::DrawMode::fill(),
                                graphics::Rect::new(0.0, 0.0, 8.0, 8.0),
                                graphics::Color::new(0.9, 0.9, 0.7, 1.0),
                            )?;

                            canvas.draw(&ventana, graphics::DrawParam::new()
                                .dest([
                                    edificio.posicion[0] + 5.0 + j as f32 * 15.0,
                                    edificio.posicion[1] + 5.0 + i as f32 * 15.0
                                ]));
                        }
                    }
                }
            }


            let posiciones_arboles = [
                // Cuadrante superior izquierdo
                ([50.0, 50.0], 1.2, 0),
                ([120.0, 80.0], 1.0, 1),
                ([200.0, 40.0], 1.3, 0),
                ([80.0, 150.0], 1.1, 1),
                ([150.0, 200.0], 1.0, 0),
                // Cuadrante superior derecho
                ([400.0, 50.0], 1.2, 1),
                ([470.0, 80.0], 1.0, 0),
                ([500.0, 150.0], 1.3, 1),
                ([400.0, 200.0], 1.1, 0),
                // Cuadrante inferior izquierdo
                ([80.0, 400.0], 1.2, 0),
                ([150.0, 450.0], 1.0, 1),
                ([200.0, 500.0], 1.3, 0),
                ([50.0, 530.0], 1.1, 1),
                // Cuadrante inferior derecho
                ([400.0, 400.0], 1.2, 1),
                ([470.0, 450.0], 1.0, 0),
                ([520.0, 500.0], 1.3, 1),
                ([450.0, 530.0], 1.1, 0),
            ];

            for ([x, y], escala, tipo) in posiciones_arboles.iter() {
                // Dibujar tronco
                canvas.draw(&cache.arboles[tipo * 2], graphics::DrawParam::new()
                    .dest([*x, *y])
                    .scale([*escala, *escala]));

                // Dibujar la cabeza del arbol o su copa
                canvas.draw(&cache.arboles[tipo * 2 + 1], graphics::DrawParam::new()
                    .dest([*x, *y])
                    .scale([*escala, *escala]));
            }

            //  fuente de agua en cuadrante número 3 (de un plano cartesiano)
            canvas.draw(&cache.fuente, graphics::DrawParam::new()
                .dest([120.0, 380.0]));

            // Añadir bordes de fuente
            let borde_fuente = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::stroke(2.0),
                [120.0, 380.0],
                17.0,
                0.1,
                graphics::Color::new(0.7, 0.7, 0.7, 1.0),
            )?;
            canvas.draw(&borde_fuente, graphics::DrawParam::default());

            // Añadir bancos de parque
            let posiciones_bancos = [
                ([100.0, 420.0], 0.0),
                ([140.0, 420.0], 0.0),
                ([120.0, 340.0], std::f32::consts::PI / 2.0),
            ];

            for ([x, y], rotacion) in posiciones_bancos.iter() {
                canvas.draw(&cache.banco, graphics::DrawParam::new()
                    .dest([*x, *y])
                    .rotation(*rotacion));
            }
        }
    }

    Ok(())
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

    // Añadir aceras en los bordes de las carreteras
    let aceras = [
        // Aceras horizontales (arriba y abajo de la vía)
        graphics::Rect::new(0.0, 290.0, 600.0, 10.0),
        graphics::Rect::new(0.0, 350.0, 600.0, 10.0),
        // Aceras verticales (izquierda y derecha de la vía)
        graphics::Rect::new(290.0, 0.0, 10.0, 600.0),
        graphics::Rect::new(350.0, 0.0, 10.0, 600.0),
    ];

    for acera in &aceras {
        let mesh = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            *acera,
            graphics::Color::new(0.8, 0.8, 0.8, 1.0),
        )?;
        canvas.draw(&mesh, graphics::DrawParam::default());
    }

    // Añadir paso de peatones
    let posiciones_paso = [
        // Paso horizontal (a la izquierda de la intersección)
        ([270.0, 310.0], false),
        // Paso vertical (debajo de la intersección)
        ([310.0, 350.0], true),
    ];

    for ([x, y], es_vertical) in &posiciones_paso {
        for i in 0..5 {
            let offset = i as f32 * 8.0;
            let paso = if *es_vertical {
                graphics::Rect::new(*x + offset, *y, 4.0, 20.0)
            } else {
                graphics::Rect::new(*x, *y + offset, 20.0, 4.0)
            };

            let mesh = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                paso,
                graphics::Color::WHITE,
            )?;
            canvas.draw(&mesh, graphics::DrawParam::default());
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

    // Añadir un borde para mejor visibilidad
    let borde = graphics::Mesh::new_circle(
        ctx,
        graphics::DrawMode::stroke(1.0),
        [semaforo.posicion[0], semaforo.posicion[1]],
        8.0,
        0.1,
        graphics::Color::BLACK,
    )?;
    canvas.draw(&borde, graphics::DrawParam::default());

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

    if carro.loco {
        match carro.tipo {
            TipoVehiculo::Automovil => 0,
            TipoVehiculo::Camioneta => 1,
            TipoVehiculo::Camion => 2,
        };

        // Dimensiones según tipo de vehículo
        let (ancho, alto) = match carro.tipo {
            TipoVehiculo::Automovil => (34.0, 19.0),
            TipoVehiculo::Camioneta => (39.0, 22.0),
            TipoVehiculo::Camion => (49.0, 24.0),
        };

        // Halo rojo para indicar vehículo loco
        let halo = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::stroke(2.0),
            graphics::Rect::new(-2.0, -2.0, ancho, alto),
            graphics::Color::RED,
        )?;

        canvas.draw(&halo, graphics::DrawParam::new()
            .dest(carro.posicion)
            .rotation(rotacion));
    }

    // Usar el mesh del vehículo desde caché
    if let Some(ref guard) = get_cache() {
        if let Some(ref cache) = **guard {
            canvas.draw(&cache.vehiculos[mesh_idx], graphics::DrawParam::new()
                .dest(carro.posicion)
                .rotation(rotacion)
                .color(carro.color));

            // Añadir detalles al vehículo
            match carro.tipo {
                TipoVehiculo::Automovil => {
                    // Ventanas para automóvil
                    let pos_x = carro.posicion[0];
                    let pos_y = carro.posicion[1];

                    // Ajustar posición según rotación
                    let (ventana_x, ventana_y, ventana_ancho, ventana_alto) = if carro.direccion == "este" {
                        (pos_x + 10.0, pos_y + 3.0, 12.0, 9.0)
                    } else {
                        (pos_x + 3.0, pos_y + 8.0, 9.0, 12.0)
                    };

                    let ventana = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::fill(),
                        graphics::Rect::new(0.0, 0.0, ventana_ancho, ventana_alto),
                        graphics::Color::new(0.7, 0.8, 0.9, 1.0),
                    )?;

                    canvas.draw(&ventana, graphics::DrawParam::new()
                        .dest([ventana_x, ventana_y])
                        .rotation(rotacion));
                },
                TipoVehiculo::Camioneta | TipoVehiculo::Camion => {
                    // Ventanas para camionetas y camiones
                    let pos_x = carro.posicion[0];
                    let pos_y = carro.posicion[1];

                    // Ajustar posición según rotación
                    let (ventana_x, ventana_y, ventana_ancho, ventana_alto) = if carro.direccion == "este" {
                        let ancho = if carro.tipo == TipoVehiculo::Camioneta { 10.0 } else { 12.0 };
                        (pos_x + 5.0, pos_y + 3.0, ancho, 12.0)
                    } else {
                        let ancho = if carro.tipo == TipoVehiculo::Camioneta { 12.0 } else { 12.0 };
                        (pos_x + 3.0, pos_y + 5.0, ancho, 10.0)
                    };

                    let ventana = graphics::Mesh::new_rectangle(
                        ctx,
                        graphics::DrawMode::fill(),
                        graphics::Rect::new(0.0, 0.0, ventana_ancho, ventana_alto),
                        graphics::Color::new(0.7, 0.8, 0.9, 1.0),
                    )?;

                    canvas.draw(&ventana, graphics::DrawParam::new()
                        .dest([ventana_x, ventana_y])
                        .rotation(rotacion));
                },
            }

            return Ok(());
        }
    }


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
    ctx: &mut Context,       // Añadir este parámetro
    num_vehiculos: usize,
    direccion_activa: &str,
    fps: usize,
    num_accidentes: usize
) -> GameResult {
    // Panel para UI
    let panel = graphics::Mesh::new_rectangle(
        ctx,
        graphics::DrawMode::fill(),
        graphics::Rect::new(5.0, 5.0, 200.0, 100.0),
        graphics::Color::new(0.0, 0.0, 0.0, 0.6),
    )?;
    canvas.draw(&panel, graphics::DrawParam::default());

    let texto = graphics::Text::new(format!(
        "Vehículos: {}\nDirección activa: {}\nFPS: {}\nAccidentes: {}",  // <-- Añadir accidentes
        num_vehiculos,
        direccion_activa,
        fps,
        num_accidentes
    ));

    canvas.draw(&texto, graphics::DrawParam::new()
        .dest([15.0, 15.0])
        .color(graphics::Color::WHITE));

    if num_accidentes > 0 {
        let panel_alerta = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(400.0, 5.0, 195.0, 30.0),
            graphics::Color::new(0.8, 0.0, 0.0, 0.8),  // Rojo semi-transparente
        )?;
        canvas.draw(&panel_alerta, graphics::DrawParam::default());

        let texto_alerta = graphics::Text::new("¡Cuidado! Conductores locos");
        canvas.draw(&texto_alerta, graphics::DrawParam::new()
            .dest([410.0, 10.0])
            .color(graphics::Color::WHITE));
    }


    Ok(())
}