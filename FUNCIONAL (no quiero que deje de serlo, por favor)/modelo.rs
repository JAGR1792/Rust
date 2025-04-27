// modelo.rs
use ggez::graphics;
use std::sync::{Arc, Mutex};
use std::time::Instant;

// Configuración constante
pub const VIA_HORIZONTAL: graphics::Rect = graphics::Rect::new(0.0, 300.0, 600.0, 50.0);
pub const VIA_VERTICAL: graphics::Rect = graphics::Rect::new(300.0, 0.0, 50.0, 600.0);
pub const COLOR_ASFALTO: graphics::Color = graphics::Color::new(0.2, 0.2, 0.2, 1.0);
pub const COLOR_LINEA_CENTRAL: graphics::Color = graphics::Color::new(1.0, 1.0, 0.0, 1.0);
pub const COLOR_FONDO: graphics::Color = graphics::Color::new(0.5, 0.7, 0.9, 1.0); // Cielo azul

pub const POSICION_SEMAFORO_VERTICAL: f32 = 250.0;
pub const POSICION_SEMAFORO_HORIZONTAL: f32 = 360.0;
// Agregar estas constantes en modelo.rs
pub const DURACION_ACCIDENTE: u64 = 10; // segundos que dura el accidente
pub const POSICION_ACCIDENTE: [f32; 2] = [325.0, 325.0]; // centro de la intersección
pub const PROBABILIDAD_EMERGENCIA: f64 = 0.5; // 50% chance de ambulancia o policía

pub const PUNTOS_APARICION: [(&str, [f32; 2]); 2] = [
    ("este", [0.0, 326.0]),     // Carril este
    ("norte", [320.0, 570.0]),  // Carril norte
];
pub const PUNTOS_APARICION_EMERGENCIA: [(&str, [f32; 2]); 2] = [
    ("este", [0.0, 308.0]),     // Carril este
    ("norte", [335.0, 570.0]),  // Carril norte
];
pub const SEMAFOROS: [([f32; 2], &str); 2] = [
    ([270.0, 300.0], "este"),   // Semáforo este
    ([300.0, 370.0], "norte"),  // Semáforo norte
];

// Parámetros de simulación 
pub const VELOCIDAD_VEHICULO: i32 = 40;
pub const VELOCIDAD_VEHICULO_EMERGENCIA: i32 = 60;
pub const INTERVALO_APARICION: u64 = 3;
pub const DURACION_VERDE: u64 = 10;
pub const DURACION_AMARILLO: u64 = 2;
pub const FPS_SIMULACION: u64 = 120;

// Estado compartido
#[derive(Clone)]
pub struct EstadoCompartido {
    pub carros: Arc<Mutex<Vec<Carro>>>,
    pub semaforos: Arc<Mutex<Vec<Semaforo>>>,
    pub direccion_activa: Arc<Mutex<String>>,
    pub ultima_actualizacion: Arc<Mutex<Instant>>,
    pub contador_accidentes: Arc<Mutex<usize>>,
}

#[derive(Clone, Copy)]
pub struct Carro {
    pub posicion: [f32; 2],
    pub direccion: &'static str,
    pub color: graphics::Color,
    pub velocidad: f32,
    pub tipo: TipoVehiculo,
    pub loco: bool,
}

#[derive(Clone)]
pub struct Semaforo {
    pub posicion: [f32; 2],
    pub estado: EstadoSemaforo,
    pub direccion: String,
}

#[derive(Clone, Copy, PartialEq)]
pub enum EstadoSemaforo {
    Rojo,
    Amarillo,
    Verde,
}

#[derive(Clone, Copy, PartialEq)]
pub enum TipoVehiculo {
    Automovil,
    Camioneta,
    Camion,
    Ambulancia,
    Policia,
}
