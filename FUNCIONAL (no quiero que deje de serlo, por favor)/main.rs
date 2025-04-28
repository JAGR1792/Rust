// main.rs
mod modelo;
mod controlador;
mod vista;

use ggez::{conf, event, graphics, Context, GameResult, audio};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Instant;
use std::path::Path;
use ggez::audio::SoundSource;

struct EstadoPrincipal {
    compartido: modelo::EstadoCompartido,
    receptor_carros: mpsc::Receiver<modelo::Carro>,
    receptor_emergencia: mpsc::Receiver<modelo::Carro>,
    ultimo_update: Instant,
    fps_contador: usize,
    ultima_medicion_fps: Instant,
    fps_actual: usize,
    musica: audio::Source,
    sirena: audio::Source, 
    emergencia_activa: bool,
}

impl EstadoPrincipal {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        // Canal para vehículos normales
        let (emisor_carros, receptor_carros) = mpsc::channel();

        // Canal separado para vehículos de emergencia
        let (emisor_emergencia, receptor_emergencia) = mpsc::channel();

        let semaforos = modelo::SEMAFOROS.iter().map(|(pos, dir)| modelo::Semaforo {
            posicion: *pos,
            estado: if *dir == "este" {
                modelo::EstadoSemaforo::Verde
            } else {
                modelo::EstadoSemaforo::Rojo
            },
            direccion: dir.to_string(),
        }).collect();

        let compartido = modelo::EstadoCompartido {
            carros: Arc::new(Mutex::new(Vec::with_capacity(100))),
            semaforos: Arc::new(Mutex::new(semaforos)),
            direccion_activa: Arc::new(Mutex::new("este".to_string())),
            ultima_actualizacion: Arc::new(Mutex::new(Instant::now())),
            contador_accidentes: Arc::new(Mutex::new(0)),
        };

        // Iniciar sistemas de control
        controlador::iniciar_semaforos(compartido.clone());
        controlador::iniciar_generador_carros(emisor_carros, compartido.clone());
        controlador::iniciar_motor_fisica(compartido.clone(), emisor_emergencia);

        // Cargar y reproducir música (solución definitiva para ggez 0.9.3)
        let mut musica = audio::Source::new(ctx, "/L'amore_dice_ciao.ogg")?;
        // Configurar para repetir
        musica.set_repeat(true);
        // Reproducir en el canal de música 
        musica.play(ctx).expect("TODO: panic message");
        
        let mut sirena = audio::Source::new(ctx, "/sirena.ogg")?; // 🔥
        sirena.set_repeat(true);
        
        Ok(Self {
            compartido,
            receptor_carros,
            receptor_emergencia,
            ultimo_update: Instant::now(),
            fps_contador: 0,
            ultima_medicion_fps: Instant::now(),
            fps_actual: 0,
            musica,  
            sirena,
            emergencia_activa: false,
        })
    }
}

impl event::EventHandler<ggez::GameError> for EstadoPrincipal {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // Limitar actualizaciones a 60 por segundo para ahorrar CPU
        let ahora = Instant::now();
        let delta = ahora.duration_since(self.ultimo_update);

        if delta.as_millis() >= 16 { // ~60 FPS
            // Procesar nuevos vehículos normales
            let mut nuevos_carros = Vec::new();
            while let Ok(carro) = self.receptor_carros.try_recv() {
                nuevos_carros.push(carro);
            }

            // Procesar nuevos vehículos de emergencia
            let mut emergencias = Vec::new();
            while let Ok(carro) = self.receptor_emergencia.try_recv() {
                emergencias.push(carro);
            }
            // Actualizar solo si hay nuevos vehículos
            if !nuevos_carros.is_empty() || !emergencias.is_empty() {
                let mut carros = self.compartido.carros.lock().unwrap();
                carros.extend(nuevos_carros);
                carros.extend(emergencias);
            }

            {
                let carros = self.compartido.carros.lock().unwrap();
                let hay_emergencias = carros.iter().any(|c| matches!(c.tipo, modelo::TipoVehiculo::Ambulancia | modelo::TipoVehiculo::Policia));

                if hay_emergencias && !self.emergencia_activa {
                    self.sirena.play(ctx)?;
                    self.emergencia_activa = true;
                } else if !hay_emergencias && self.emergencia_activa {
                    self.sirena.stop(ctx)?;
                    self.emergencia_activa = false;
                }
            }

            self.ultimo_update = ahora;
        }

        // Cálculo de FPS
        self.fps_contador += 1;
        if ahora.duration_since(self.ultima_medicion_fps).as_secs() >= 1 {
            self.fps_actual = self.fps_contador;
            self.fps_contador = 0;
            self.ultima_medicion_fps = ahora;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::new(0.5, 0.7, 0.9, 1.0));

        // Inicializar el caché en el primer frame
        vista::inicializar_cache(ctx)?;

        // Dibujar capas en orden
        vista::dibujar_fondo(&mut canvas, ctx)?;
        vista::dibujar_elementos_decorativos(&mut canvas, ctx)?;
        vista::dibujar_carreteras(&mut canvas, ctx)?;

        // Dibujar semáforos
        let semaforos = {
            let lock = self.compartido.semaforos.lock().unwrap();
            lock.clone()
        };

        for semaforo in &semaforos {
            vista::dibujar_semaforo(&mut canvas, ctx, semaforo)?;
        }

        // Dibujar vehículos
        let carros = {
            let lock = self.compartido.carros.lock().unwrap();
            lock.clone()
        };

        // Dibujar vehículos normales primero
        for carro in carros.iter().filter(|c| !matches!(c.tipo, modelo::TipoVehiculo::Ambulancia | modelo::TipoVehiculo::Policia)) {
            vista::dibujar_vehiculo(&mut canvas, ctx, carro)?;
        }

        // Dibujar vehículos de emergencia encima (para que sean más visibles)
        for carro in carros.iter().filter(|c| matches!(c.tipo, modelo::TipoVehiculo::Ambulancia | modelo::TipoVehiculo::Policia)) {
            vista::dibujar_vehiculo(&mut canvas, ctx, carro)?;
        }

        // Obtener datos para la UI
        let direccion_activa = {
            let lock = self.compartido.direccion_activa.lock().unwrap();
            lock.clone()
        };

        let num_accidentes = {
            let lock = self.compartido.contador_accidentes.lock().unwrap();
            *lock
        };

        // Dibujar UI
        vista::dibujar_ui(
            &mut canvas,
            ctx,
            carros.len(),
            &direccion_activa,
            self.fps_actual,
            num_accidentes
        )?;

        canvas.finish(ctx)?;
        Ok(())
    }
}

fn main() -> GameResult {
    // Configurar la ruta de recursos
    let resource_dir = if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        Path::new(&manifest_dir).join("resources")
    } else {
        Path::new("./resources").to_path_buf()
    };

    let (mut ctx, event_loop) = ggez::ContextBuilder::new("simulacion-trafico", "rust")
        .window_setup(conf::WindowSetup::default().title("Simulación de Tráfico"))
        .window_mode(conf::WindowMode::default().dimensions(600.0, 600.0))
        .add_resource_path(resource_dir)
        .build()?;

    let estado = EstadoPrincipal::new(&mut ctx)?;
    event::run(ctx, event_loop, estado)
}
