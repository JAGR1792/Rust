use ggez::{conf, event, graphics, Context, GameResult};
use rand::Rng;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

// #############################################################################
// CONFIGURACIÓN
// #############################################################################
const VIA_HORIZONTAL: graphics::Rect = graphics::Rect::new(0.0, 250.0, 600.0, 100.0);
const VIA_VERTICAL: graphics::Rect = graphics::Rect::new(250.0, 0.0, 100.0, 600.0);
const COLOR_ASFALTO: graphics::Color = graphics::Color::new(0.2, 0.2, 0.2, 1.0);
const COLOR_LINEA_CENTRAL: graphics::Color = graphics::Color::new(1.0, 1.0, 0.0, 1.0);
const COLOR_FONDO: graphics::Color = graphics::Color::new(0.3, 0.7, 0.3, 1.0);

const PUNTOS_APARICION: [(&str, [f32; 2]); 4] = [
    ("este", [-50.0, 280.0]),
    ("oeste", [650.0, 320.0]),
    ("norte", [270.0, 650.0]),
    ("sur", [300.0, -50.0]),
];

const SEMAFOROS: [([f32; 2], &str); 4] = [
    ([285.0, 100.0], "norte"),
    ([285.0, 470.0], "sur"),
    ([470.0, 285.0], "este"),
    ([100.0, 285.0], "oeste"),
];

const VELOCIDAD_VEHICULO: f32 = 2.0;
const INTERVALO_APARICION: u64 = 3;
const INTERVALO_CAMBIO_SEMAFORO: u64 = 10;
const FPS_SIMULACION: u64 = 60;

// #############################################################################
// ESTRUCTURAS DE DATOS
// #############################################################################
#[derive(Clone)]
struct EstadoCompartido {
    carros: Arc<Mutex<Vec<Carro>>>,
    semaforos: Arc<Mutex<Vec<Semaforo>>>,
    direccion_activa: Arc<Mutex<String>>,
    ultimo_cambio: Arc<Mutex<Instant>>,
}

#[derive(Clone, Copy)]
struct Carro {
    posicion: [f32; 2],
    direccion: &'static str,
    color: graphics::Color,
    velocidad: f32,
    tipo: TipoVehiculo,
}

#[derive(Clone, Copy, PartialEq)]
enum TipoVehiculo {
    Automovil,
    Camioneta,
    Camion,
}

#[derive(Clone)]
struct Semaforo {
    posicion: [f32; 2],
    estado: EstadoSemaforo,
    direccion: String,
}

#[derive(Clone, Copy, PartialEq)]
enum EstadoSemaforo {
    Rojo,
    Verde,
    Amarillo,
}

// #############################################################################
// HILOS
// #############################################################################
fn controlador_semaforos(compartido: EstadoCompartido) {
    thread::spawn(move || {
        let mut timer = Instant::now();
        loop {
            thread::sleep(Duration::from_secs(1));

            let tiempo_transcurrido = timer.elapsed().as_secs();
            if tiempo_transcurrido >= INTERVALO_CAMBIO_SEMAFORO {
                let mut dir_activa = compartido.direccion_activa.lock().unwrap();
                let mut ultimo_cambio = compartido.ultimo_cambio.lock().unwrap();

                *dir_activa = if *dir_activa == "este-oeste" {
                    "norte-sur".to_string()
                } else {
                    "este-oeste".to_string()
                };

                *ultimo_cambio = Instant::now();
                timer = Instant::now();

                let mut luces = compartido.semaforos.lock().unwrap();
                for luz in luces.iter_mut() {
                    luz.estado = if dir_activa.contains(&luz.direccion) {
                        EstadoSemaforo::Verde
                    } else {
                        EstadoSemaforo::Rojo
                    };
                }
            }
        }
    });
}

fn generador_carros(compartido: EstadoCompartido, emisor: mpsc::Sender<Carro>) {
    thread::spawn(move || {
        let mut rng = rand::thread_rng();
        loop {
            thread::sleep(Duration::from_secs(INTERVALO_APARICION));

            let (direccion, pos) = PUNTOS_APARICION[rng.gen_range(0..4)];
            let x = pos[0];
            let y = pos[1];

            // Verificar semáforo
            let semaforos = compartido.semaforos.lock().unwrap();
            let semaforo = semaforos.iter()
                .find(|s| s.direccion == direccion)
                .unwrap();

            if semaforo.estado != EstadoSemaforo::Verde {
                continue;
            }
            drop(semaforos);

            // Verificar proximidad
            let carros = compartido.carros.lock().unwrap();
            let muy_cerca = carros.iter().any(|c| {
                let dx = c.posicion[0] - x;
                let dy = c.posicion[1] - y;
                (dx * dx + dy * dy).sqrt() < 50.0
            });
            drop(carros);

            if muy_cerca {
                continue;
            }

            let tipo_vehiculo = match rng.gen_range(0..3) {
                0 => TipoVehiculo::Automovil,
                1 => TipoVehiculo::Camioneta,
                _ => TipoVehiculo::Camion,
            };

            let color = match tipo_vehiculo {
                TipoVehiculo::Automovil => match rng.gen_range(0..4) {
                    0 => graphics::Color::new(0.9, 0.1, 0.1, 1.0),
                    1 => graphics::Color::new(0.1, 0.1, 0.8, 1.0),
                    2 => graphics::Color::new(0.9, 0.9, 0.9, 1.0),
                    _ => graphics::Color::new(0.7, 0.7, 0.7, 1.0),
                },
                TipoVehiculo::Camioneta => match rng.gen_range(0..3) {
                    0 => graphics::Color::new(0.2, 0.5, 0.2, 1.0),
                    1 => graphics::Color::new(0.6, 0.4, 0.2, 1.0),
                    _ => graphics::Color::new(0.1, 0.1, 0.1, 1.0),
                },
                TipoVehiculo::Camion => match rng.gen_range(0..3) {
                    0 => graphics::Color::new(0.1, 0.2, 0.4, 1.0),
                    1 => graphics::Color::new(0.9, 0.6, 0.2, 1.0),
                    _ => graphics::Color::new(0.9, 0.9, 0.9, 1.0),
                },
            };

            let nuevo_carro = Carro {
                posicion: [x, y],
                direccion,
                color,
                velocidad: VELOCIDAD_VEHICULO,
                tipo: tipo_vehiculo,
            };

            emisor.send(nuevo_carro).unwrap();
        }
    });
}

fn motor_movimiento(compartido: EstadoCompartido) {
    thread::spawn(move || {
        loop {
            let dir_activa = compartido.direccion_activa.lock().unwrap().clone();
            let mut carros = compartido.carros.lock().unwrap();

            for carro in carros.iter_mut() {
                let semaforos = compartido.semaforos.lock().unwrap();
                let semaforo = semaforos.iter()
                    .find(|s| s.direccion == carro.direccion)
                    .unwrap();

                if semaforo.estado == EstadoSemaforo::Verde {
                    match carro.direccion {
                        "este" => carro.posicion[0] += carro.velocidad,
                        "oeste" => carro.posicion[0] -= carro.velocidad,
                        "norte" => carro.posicion[1] -= carro.velocidad,
                        "sur" => carro.posicion[1] += carro.velocidad,
                        _ => {}
                    }
                }
            }

            carros.retain(|c|
                c.posicion[0] > -100.0 &&
                    c.posicion[0] < 700.0 &&
                    c.posicion[1] > -100.0 &&
                    c.posicion[1] < 700.0
            );

            thread::sleep(Duration::from_millis(1000 / FPS_SIMULACION));
        }
    });
}

// #############################################################################
// LÓGICA PRINCIPAL
// #############################################################################
struct EstadoPrincipal {
    compartido: EstadoCompartido,
    receptor: mpsc::Receiver<Carro>,
}

impl EstadoPrincipal {
    fn nuevo() -> GameResult<Self> {
        let (emisor, receptor) = mpsc::channel();

        let compartido = EstadoCompartido {
            carros: Arc::new(Mutex::new(Vec::new())),
            semaforos: Arc::new(Mutex::new(
                SEMAFOROS.iter().map(|(pos, dir)| Semaforo {
                    posicion: *pos,
                    estado: if *dir == "este" || *dir == "oeste" {
                        EstadoSemaforo::Verde
                    } else {
                        EstadoSemaforo::Rojo
                    },
                    direccion: dir.to_string(),
                }).collect()
            )),
            direccion_activa: Arc::new(Mutex::new("este-oeste".to_string())),
            ultimo_cambio: Arc::new(Mutex::new(Instant::now())),
        };

        controlador_semaforos(compartido.clone());
        generador_carros(compartido.clone(), emisor);
        motor_movimiento(compartido.clone());

        Ok(EstadoPrincipal { compartido, receptor })
    }
}

impl event::EventHandler<ggez::GameError> for EstadoPrincipal {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        while let Ok(nuevo_carro) = self.receptor.try_recv() {
            self.compartido.carros.lock().unwrap().push(nuevo_carro);
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, COLOR_FONDO);
        dibujar_carreteras(&mut canvas, ctx)?;

        let semaforos = self.compartido.semaforos.lock().unwrap();
        for semaforo in semaforos.iter() {
            dibujar_semaforo(&mut canvas, ctx, semaforo)?;
        }

        let carros = self.compartido.carros.lock().unwrap();
        for carro in carros.iter() {
            dibujar_vehiculo(&mut canvas, ctx, carro)?;
        }

        let dir_activa = self.compartido.direccion_activa.lock().unwrap().clone();
        let ultimo_cambio = self.compartido.ultimo_cambio.lock().unwrap();
        let tiempo_restante = INTERVALO_CAMBIO_SEMAFORO - Instant::now().duration_since(*ultimo_cambio).as_secs();

        dibujar_info(
            &mut canvas,
            ctx,
            &dir_activa,
            carros.len(),
            tiempo_restante
        )?;

        canvas.finish(ctx)?;
        Ok(())
    }
}

// #############################################################################
// FUNCIONES DE DIBUJO
// #############################################################################
fn dibujar_carreteras(canvas: &mut graphics::Canvas, ctx: &mut Context) -> GameResult {
    canvas.draw(&graphics::Quad, graphics::DrawParam::new()
        .dest_rect(VIA_HORIZONTAL)
        .color(COLOR_ASFALTO));

    canvas.draw(&graphics::Quad, graphics::DrawParam::new()
        .dest_rect(VIA_VERTICAL)
        .color(COLOR_ASFALTO));

    for i in 0..30 {
        let linea_h = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(i as f32 * 20.0, 300.0 - 2.0, 10.0, 4.0),
            COLOR_LINEA_CENTRAL,
        )?;
        canvas.draw(&linea_h, graphics::DrawParam::default());
    }

    for i in 0..30 {
        let linea_v = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            graphics::Rect::new(300.0 - 2.0, i as f32 * 20.0, 4.0, 10.0),
            COLOR_LINEA_CENTRAL,
        )?;
        canvas.draw(&linea_v, graphics::DrawParam::default());
    }

    Ok(())
}

fn dibujar_semaforo(canvas: &mut graphics::Canvas, ctx: &mut Context, semaforo: &Semaforo) -> GameResult {
    let base = graphics::Mesh::new_rectangle(
        ctx,
        graphics::DrawMode::fill(),
        graphics::Rect::new(semaforo.posicion[0] - 10.0, semaforo.posicion[1] - 25.0, 20.0, 30.0),
        graphics::Color::new(0.3, 0.3, 0.3, 1.0),
    )?;
    canvas.draw(&base, graphics::DrawParam::default());

    let color = match semaforo.estado {
        EstadoSemaforo::Rojo => graphics::Color::RED,
        EstadoSemaforo::Verde => graphics::Color::GREEN,
        EstadoSemaforo::Amarillo => graphics::Color::YELLOW,
    };

    let circulo = graphics::Mesh::new_circle(
        ctx,
        graphics::DrawMode::fill(),
        semaforo.posicion,
        10.0,
        0.1,
        color,
    )?;
    canvas.draw(&circulo, graphics::DrawParam::default());

    let borde = graphics::Mesh::new_circle(
        ctx,
        graphics::DrawMode::stroke(1.5),
        semaforo.posicion,
        10.0,
        0.1,
        graphics::Color::BLACK,
    )?;
    canvas.draw(&borde, graphics::DrawParam::default());

    Ok(())
}

fn dibujar_vehiculo(canvas: &mut graphics::Canvas, ctx: &mut Context, carro: &Carro) -> GameResult {
    let (ancho, alto) = match carro.tipo {
        TipoVehiculo::Automovil => (30.0, 20.0),
        TipoVehiculo::Camioneta => (35.0, 25.0),
        TipoVehiculo::Camion => (45.0, 30.0),
    };

    let rotacion = match carro.direccion {
        "este" => 0.0,
        "oeste" => std::f32::consts::PI,
        "norte" => std::f32::consts::PI / 2.0,
        "sur" => -std::f32::consts::PI / 2.0,
        _ => 0.0,
    };

    // Cuerpo principal
    let cuerpo = graphics::Mesh::new_rectangle(
        ctx,
        graphics::DrawMode::fill(),
        graphics::Rect::new(0.0, 0.0, ancho, alto),
        carro.color,
    )?;
    canvas.draw(&cuerpo, graphics::DrawParam::new()
        .dest(carro.posicion)
        .rotation(rotacion));

    // Ruedas
    let radio_rueda = match carro.tipo {
        TipoVehiculo::Automovil => 3.0,
        TipoVehiculo::Camioneta => 4.0,
        TipoVehiculo::Camion => 5.0,
    };

    match carro.direccion {
        "este" | "oeste" => {
            let y_rueda = carro.posicion[1] + alto / 2.0 - radio_rueda;
            for offset in &[0.25, 0.75] {
                let pos_rueda = [
                    carro.posicion[0] + ancho * offset,
                    y_rueda
                ];
                let rueda = graphics::Mesh::new_circle(
                    ctx,
                    graphics::DrawMode::fill(),
                    pos_rueda,
                    radio_rueda,
                    0.1,
                    graphics::Color::BLACK,
                )?;
                canvas.draw(&rueda, graphics::DrawParam::default());
            }
        }
        "norte" | "sur" => {
            let x_rueda = carro.posicion[0] + ancho / 2.0 - radio_rueda;
            for offset in &[0.25, 0.75] {
                let pos_rueda = [
                    x_rueda,
                    carro.posicion[1] + alto * offset
                ];
                let rueda = graphics::Mesh::new_circle(
                    ctx,
                    graphics::DrawMode::fill(),
                    pos_rueda,
                    radio_rueda,
                    0.1,
                    graphics::Color::BLACK,
                )?;
                canvas.draw(&rueda, graphics::DrawParam::default());
            }
        }
        _ => {}
    }

    Ok(())
}

fn dibujar_info(
    canvas: &mut graphics::Canvas,
    ctx: &mut Context,
    dir_activa: &str,
    num_vehiculos: usize,
    tiempo_restante: u64,
) -> GameResult {
    let panel = graphics::Mesh::new_rectangle(
        ctx,
        graphics::DrawMode::fill(),
        graphics::Rect::new(400.0, 10.0, 190.0, 100.0),
        graphics::Color::new(0.2, 0.2, 0.2, 0.7),
    )?;
    canvas.draw(&panel, graphics::DrawParam::default());

    let texto_fase = format!("Fase: {}", if dir_activa == "este-oeste" {
        "Este-Oeste"
    } else {
        "Norte-Sur"
    });

    let texto_tiempo = format!("Cambio en: {}s", tiempo_restante);
    let texto_vehiculos = format!("Vehículos: {}", num_vehiculos);

    let textos = [
        (texto_fase, [410.0, 20.0]),
        (texto_tiempo, [410.0, 50.0]),
        (texto_vehiculos, [410.0, 80.0]),
    ];

    for (texto, pos) in textos {
        canvas.draw(
            graphics::Text::new(texto),
            graphics::DrawParam::new()
                .dest(pos)
                .color(graphics::Color::WHITE)
        );
    }

    canvas.draw(
        graphics::Text::new("Simulación de Tráfico"),
        graphics::DrawParam::new()
            .dest([10.0, 10.0])
            .color(graphics::Color::WHITE)
            .scale([1.5, 1.5])
    );

    Ok(())
}

// #############################################################################
// INICIO
// #############################################################################
fn main() -> GameResult {
    let (mut ctx, event_loop) = ggez::ContextBuilder::new("simulacion_trafico", "Autor")
        .window_setup(conf::WindowSetup::default().title("Simulación de Tráfico Inteligente"))
        .window_mode(conf::WindowMode::default().dimensions(600.0, 600.0))
        .build()?;

    let estado = EstadoPrincipal::nuevo()?;
    event::run(ctx, event_loop, estado)
}