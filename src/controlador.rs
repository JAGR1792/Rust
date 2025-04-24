// controlador.rs
use crate::modelo::*;
use rand::Rng;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use ggez::graphics;

pub fn iniciar_semaforos(compartido: EstadoCompartido) {
    thread::spawn(move || {
        let mut estado_actual = "este".to_string();
        loop {
            // Fase verde
            actualizar_semaforos(&compartido, &estado_actual, EstadoSemaforo::Verde);
            thread::sleep(Duration::from_secs(DURACION_VERDE));

            // Fase amarillo
            actualizar_semaforos(&compartido, &estado_actual, EstadoSemaforo::Amarillo);
            thread::sleep(Duration::from_secs(DURACION_AMARILLO));

            // Cambiar dirección activa
            estado_actual = if estado_actual == "este" {
                "norte".to_string()
            } else {
                "este".to_string()
            };

            // Actualizar estado en compartido - usar scope para minimizar tiempo de lock
            {
                let mut direccion = compartido.direccion_activa.lock().unwrap();
                *direccion = estado_actual.clone();
            }
        }
    });
}

fn actualizar_semaforos(compartido: &EstadoCompartido, direccion: &str, estado: EstadoSemaforo) {
    let mut semaforos = compartido.semaforos.lock().unwrap();
    for semaforo in semaforos.iter_mut() {
        if semaforo.direccion == direccion {
            semaforo.estado = estado;
        } else {
            semaforo.estado = EstadoSemaforo::Rojo;
        }
    }
}

pub fn iniciar_generador_carros(emisor: mpsc::Sender<Carro>) {
    thread::spawn(move || {
        let mut rng = rand::rng(); // Corregido el error de rand::rng()
        loop {
            thread::sleep(Duration::from_secs(INTERVALO_APARICION));

            // Añadir aleatoriedad para evitar ráfagas de vehículos
            if rng.random_bool(0.8) { // 80% de probabilidad de generar
                let idx = rng.random_range(0..2);
                let (direccion, pos) = PUNTOS_APARICION[idx];

                let tipo_vehiculo = match rng.random_range(0..3) {
                    0 => TipoVehiculo::Automovil,
                    1 => TipoVehiculo::Camioneta,
                    _ => TipoVehiculo::Camion,
                };

                let color = match tipo_vehiculo {
                    TipoVehiculo::Automovil => graphics::Color::from_rgb(
                        rng.random_range(100..255),
                        rng.random_range(100..255),
                        rng.random_range(100..255)
                    ),
                    TipoVehiculo::Camioneta => graphics::Color::from_rgb(
                        rng.random_range(50..150),
                        rng.random_range(50..150),
                        rng.random_range(50..150)
                    ),
                    TipoVehiculo::Camion => graphics::Color::from_rgb(
                        rng.random_range(0..100),
                        rng.random_range(0..100),
                        rng.random_range(0..100)
                    ),
                };

                // Variar ligeramente la velocidad para evitar agrupaciones

                let velocidad_ajustada = VELOCIDAD_VEHICULO;

                if let Err(_) = emisor.send(Carro {
                    posicion: pos,
                    direccion,
                    color,
                    velocidad: velocidad_ajustada as f32,
                    tipo: tipo_vehiculo,
                }) {
                    // Canal cerrado, terminar hilo
                    break;
                }
            }
        }
    });
}

pub fn iniciar_motor_fisica(compartido: EstadoCompartido) {
    thread::spawn(move || {
        let mut ultimo_update = Instant::now();

        loop {
            // Control de velocidad de actualización para reducir uso de CPU
            let ahora = Instant::now();
            let delta_ms = ahora.duration_since(ultimo_update).as_millis() as f32;

            if delta_ms < (1000.0 / FPS_SIMULACION as f32) {
                let espera = (1000.0 / FPS_SIMULACION as f32 - delta_ms) as u64;
                thread::sleep(Duration::from_millis(espera));
                continue;
            }

            ultimo_update = ahora;

            // Factor de movimiento basado en tiempo para movimiento uniforme
            let factor_movimiento = delta_ms / 1000.0;

            // Actualizar estado de vehículos
            let mut removidos = Vec::new();

            // Scope para minimizar tiempo de lock
            {
                let mut carros = compartido.carros.lock().unwrap();
                let semaforos = compartido.semaforos.lock().unwrap();

                // Procesar lotes de vehículos para mejorar rendimiento
                for (i, carro) in carros.iter_mut().enumerate() {
                    let semaforo = semaforos.iter()
                        .find(|s| s.direccion == carro.direccion)
                        .unwrap();

                    let puede_avanzar = match semaforo.estado {
                        EstadoSemaforo::Verde => true,
                        EstadoSemaforo::Amarillo => {
                            // Verificar posición para detenerse en amarillo
                            match carro.direccion {
                                "este" => carro.posicion[0] < 250.0,
                                "norte" => carro.posicion[1] > 320.0,
                                _ => false
                            }
                        }
                        EstadoSemaforo::Rojo => false
                    };

                    if puede_avanzar {
                        // Movimiento ajustado por tiempo para mantener velocidad constante
                        match carro.direccion {
                            "este" => carro.posicion[0] += carro.velocidad * factor_movimiento,
                            "norte" => carro.posicion[1] -= carro.velocidad * factor_movimiento,
                            _ => {}
                        }
                    }

                    // Marcar para eliminación si está fuera de pantalla
                    if carro.posicion[0] > 650.0 || carro.posicion[1] < -50.0 {
                        removidos.push(i);
                    }
                }

                // Eliminar vehículos fuera de pantalla (más eficiente eliminar de atrás hacia adelante)
                removidos.sort_by(|a, b| b.cmp(a));
                for idx in removidos {
                    if idx < carros.len() {
                        carros.swap_remove(idx); // swap_remove es más eficiente que remove
                    }
                }
            }

            // Actualizar timestamp de última actualización
            {
                let mut ultima = compartido.ultima_actualizacion.lock().unwrap();
                *ultima = ahora;
            }
        }
    });
}