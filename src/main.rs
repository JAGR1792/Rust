// main.rs
mod modelo;
mod controlador;
mod vista;

use ggez::{conf, event, graphics, Context, GameResult};
use std::sync::{mpsc, Arc, Mutex};
use std::time::Instant;

struct EstadoPrincipal {
    compartido: modelo::EstadoCompartido,
    receptor: mpsc::Receiver<modelo::Carro>,
    ultimo_update: Instant,
    fps_contador: usize,
    ultima_medicion_fps: Instant,
    fps_actual: usize,
}

impl EstadoPrincipal {
    fn new() -> GameResult<Self> {
        let (emisor, receptor) = mpsc::channel();

        let semaforos = modelo::SEMAFOROS.iter().map(|(pos, dir)| modelo::Semaforo {
            posicion: *pos,
            estado: if *dir == "este" { modelo::EstadoSemaforo::Verde } else { modelo::EstadoSemaforo::Rojo },
            direccion: dir.to_string(),
        }).collect();

        let compartido = modelo::EstadoCompartido {
            carros: Arc::new(Mutex::new(Vec::with_capacity(100))), // Pre-allocate para mejorar rendimiento
            semaforos: Arc::new(Mutex::new(semaforos)),
            direccion_activa: Arc::new(Mutex::new("este".to_string())),
            ultima_actualizacion: Arc::new(Mutex::new(Instant::now())),
        };

        controlador::iniciar_semaforos(compartido.clone());
        controlador::iniciar_generador_carros(emisor);
        controlador::iniciar_motor_fisica(compartido.clone());

        Ok(Self {
            compartido,
            receptor,
            ultimo_update: Instant::now(),
            fps_contador: 0,
            ultima_medicion_fps: Instant::now(),
            fps_actual: 0,
        })
    }
}

impl event::EventHandler<ggez::GameError> for EstadoPrincipal {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        // Limitar actualizaciones a 60 por segundo para ahorrar CPU
        let ahora = Instant::now();
        let delta = ahora.duration_since(self.ultimo_update);

        if delta.as_millis() >= 16 { // ~60 FPS
            // Colectar nuevos vehículos de forma eficiente
            let mut nuevos_carros = Vec::new();
            while let Ok(carro) = self.receptor.try_recv() {
                nuevos_carros.push(carro);
            }

            // Actualizamos solo si hay nuevos vehículos
            if !nuevos_carros.is_empty() {
                let mut carros = self.compartido.carros.lock().unwrap();
                carros.extend(nuevos_carros);
            }

            self.ultimo_update = ahora;
        }

        // Cálculo de FPS para diagnóstico
        self.fps_contador += 1;
        if ahora.duration_since(self.ultima_medicion_fps).as_secs() >= 1 {
            self.fps_actual = self.fps_contador;
            self.fps_contador = 0;
            self.ultima_medicion_fps = ahora;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, modelo::COLOR_FONDO);

        // Inicializar el caché en el primer frame
        vista::inicializar_cache(ctx)?;

        vista::dibujar_carreteras(&mut canvas, ctx)?;

        // Minimizar el tiempo que mantenemos los locks
        let semaforos = {
            let lock = self.compartido.semaforos.lock().unwrap();
            lock.clone()
        };

        for semaforo in &semaforos {
            vista::dibujar_semaforo(&mut canvas, ctx, semaforo)?;
        }

        // Clonamos los carros para liberar el lock rápidamente
        let carros = {
            let lock = self.compartido.carros.lock().unwrap();
            lock.clone()
        };

        for carro in &carros {
            vista::dibujar_vehiculo(&mut canvas, ctx, carro)?;
        }

        let direccion_activa = {
            let lock = self.compartido.direccion_activa.lock().unwrap();
            lock.clone()
        };

        vista::dibujar_ui(&mut canvas, carros.len(), &direccion_activa, self.fps_actual)?;

        canvas.finish(ctx)?;
        Ok(())
    }
}

fn main() -> GameResult {
    let (ctx, event_loop) = ggez::ContextBuilder::new("simulacion-trafico", "rust")
        .window_setup(conf::WindowSetup::default().title("Simulación de Tráfico"))
        .window_mode(conf::WindowMode::default().dimensions(600.0, 600.0))
        .build()?;

    let estado = EstadoPrincipal::new()?;
    event::run(ctx, event_loop, estado)
}