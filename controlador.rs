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

            // Actualizar estado en compartido
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

pub fn iniciar_generador_carros(emisor: mpsc::Sender<Carro>, compartido: EstadoCompartido) {
    thread::spawn(move || {
        let mut rng = rand::thread_rng();
        loop {
            thread::sleep(Duration::from_secs(INTERVALO_APARICION));

            if rng.gen_bool(0.8) {
                let idx = rng.gen_range(0..2);
                let (direccion, pos) = PUNTOS_APARICION[idx];
                let es_loco = rng.gen_bool(0.1);

                let espacio_suficiente = {
                    let carros = compartido.carros.lock().unwrap();
                    let distancia_minima = 60.0;

                    !carros.iter().any(|carro| {
                        if carro.direccion != direccion {
                            return false;
                        }

                        match direccion {
                            "este" => carro.posicion[0] < 100.0 &&
                                (carro.posicion[0] - pos[0]).abs() < distancia_minima,
                            "norte" => carro.posicion[1] > 500.0 &&
                                (carro.posicion[1] - pos[1]).abs() < distancia_minima,
                            _ => false
                        }
                    })
                };

                if !espacio_suficiente {
                    continue;
                }

                let tipo_vehiculo = match rng.gen_range(0..3) {
                    0 => TipoVehiculo::Automovil,
                    1 => TipoVehiculo::Camioneta,
                    _ => TipoVehiculo::Camion,
                };

                let color = match tipo_vehiculo {
                    TipoVehiculo::Automovil => graphics::Color::from_rgb(
                        rng.gen_range(100..255),
                        rng.gen_range(100..255),
                        rng.gen_range(100..255)
                    ),
                    TipoVehiculo::Camioneta => graphics::Color::from_rgb(
                        rng.gen_range(50..150),
                        rng.gen_range(50..150),
                        rng.gen_range(50..150)
                    ),
                    TipoVehiculo::Camion => graphics::Color::from_rgb(
                        rng.gen_range(0..100),
                        rng.gen_range(0..100),
                        rng.gen_range(0..100)
                    ),
                    _ => graphics::Color::WHITE,
                };

                if emisor.send(Carro {
                    posicion: pos,
                    direccion,
                    color,
                    velocidad: VELOCIDAD_VEHICULO as f32,
                    tipo: tipo_vehiculo,
                    loco: es_loco,
                }).is_err() {
                    break;
                }
            }
        }
    });
}

fn crear_vehiculo_emergencia() -> Carro {
    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..PUNTOS_APARICION_EMERGENCIA.len());
    let (direccion, pos) = PUNTOS_APARICION_EMERGENCIA[idx];

    let tipo = if rng.gen_bool(0.5) {
        TipoVehiculo::Ambulancia
    } else {
        TipoVehiculo::Policia
    };

    Carro {
        posicion: pos,
        direccion,
        color: match tipo {
            TipoVehiculo::Ambulancia => graphics::Color::from_rgb(255, 255, 255),
            TipoVehiculo::Policia => graphics::Color::from_rgb(0, 0, 255),
            _ => graphics::Color::from_rgb(255, 255, 255),
        },
        velocidad: VELOCIDAD_VEHICULO_EMERGENCIA as f32,
        tipo,
        loco: false,
    }
}

pub fn iniciar_motor_fisica(compartido: EstadoCompartido, emisor_emergencia: mpsc::Sender<Carro>) {
    thread::spawn(move || {
        let mut ultimo_update = Instant::now();
        let mut tiempo_accidente: Option<Instant> = None;
        let mut vehiculos_emergencia: Vec<usize> = Vec::new();

        loop {
            let ahora = Instant::now();
            let delta_ms = ahora.duration_since(ultimo_update).as_millis() as f32;
            let fps_interval = 1000.0 / FPS_SIMULACION as f32;

            if delta_ms < fps_interval {
                thread::sleep(Duration::from_millis((fps_interval - delta_ms) as u64));
                continue;
            }

            ultimo_update = ahora;
            let factor_movimiento = delta_ms / 1000.0;
            let mut removidos = Vec::new();
            let mut accidentes = Vec::new();

            // Manejo de accidentes
            let en_accidente = tiempo_accidente.map_or(false, |t| {
                ahora.duration_since(t).as_secs() < DURACION_ACCIDENTE
            });

            if en_accidente {
                let mut carros = compartido.carros.lock().unwrap();
                let mut nuevos_emergencia = Vec::new();

                // Mover vehículos de emergencia
                for &idx in &vehiculos_emergencia {
                    if idx < carros.len() {
                        let carro = &mut carros[idx];
                        let dx = POSICION_ACCIDENTE[0] - carro.posicion[0];
                        let dy = POSICION_ACCIDENTE[1] - carro.posicion[1];
                        let distancia = (dx * dx + dy * dy).sqrt();

                        if distancia > 10.0 {
                            carro.posicion[0] += (dx / distancia) * carro.velocidad * factor_movimiento;
                            carro.posicion[1] += (dy / distancia) * carro.velocidad * factor_movimiento;
                            nuevos_emergencia.push(idx);
                        } else {
                            removidos.push(idx);
                        }
                    }
                }

                // Eliminar vehículos que llegaron
                removidos.sort_by(|a, b| b.cmp(a));
                for idx in removidos {
                    if idx < carros.len() {
                        carros.swap_remove(idx);
                    }
                }

                vehiculos_emergencia = nuevos_emergencia;
            } else {
                vehiculos_emergencia.clear();
            }

            // Detección de colisiones
            let carros_en_interseccion = {
                let carros = compartido.carros.lock().unwrap();
                carros.iter().enumerate()
                    .filter_map(|(idx, carro)| {
                        if carro.posicion[0] >= 300.0 && carro.posicion[0] <= 350.0 &&
                            carro.posicion[1] >= 300.0 && carro.posicion[1] <= 350.0 {
                            Some((idx, carro.direccion.clone(), carro.loco))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            };

            for (i, dir_i, loco_i) in &carros_en_interseccion {
                if *loco_i {
                    for (j, dir_j, _) in &carros_en_interseccion {
                        if i != j && dir_i != dir_j {
                            accidentes.push(*i);
                            accidentes.push(*j);
                        }
                    }
                }
            }

            // Manejo de nuevo accidente
            if !accidentes.is_empty() && tiempo_accidente.is_none() {
                tiempo_accidente = Some(ahora);

                // Poner semáforos en amarillo
                {
                    let mut semaforos = compartido.semaforos.lock().unwrap();
                    for semaforo in semaforos.iter_mut() {
                        semaforo.estado = EstadoSemaforo::Amarillo;
                    }
                }

                // Incrementar contador
                {
                    let mut contador = compartido.contador_accidentes.lock().unwrap();
                    *contador += 1;
                }

                // Generar vehículos de emergencia
                for _ in 0..2 {
                    let _ = emisor_emergencia.send(crear_vehiculo_emergencia());
                }

                // Eliminar vehículos accidentados
                {
                    let mut carros = compartido.carros.lock().unwrap();
                    accidentes.sort_by(|a, b| b.cmp(a));
                    accidentes.dedup();

                    for idx in accidentes {
                        if idx < carros.len() {
                            carros.swap_remove(idx);
                        }
                    }

                    // Registrar vehículos de emergencia
                    vehiculos_emergencia = carros.iter().enumerate()
                        .filter(|(_, c)| matches!(c.tipo, TipoVehiculo::Ambulancia | TipoVehiculo::Policia))
                        .map(|(i, _)| i)
                        .collect();
                }
            }

            // Movimiento normal
            if !en_accidente {
                // Restaurar semáforos después del accidente
                if tiempo_accidente.map_or(false, |t| {
                    ahora.duration_since(t).as_secs() >= DURACION_ACCIDENTE
                }) {
                    tiempo_accidente = None;
                    let direccion_activa = compartido.direccion_activa.lock().unwrap().clone();
                    actualizar_semaforos(&compartido, &direccion_activa, EstadoSemaforo::Verde);
                }

                // Procesamiento de movimiento normal
                {
                    let mut carros = compartido.carros.lock().unwrap();
                    let semaforos = compartido.semaforos.lock().unwrap();

                    // Primera pasada: identificar vehículos a eliminar
                    let mut indices_a_eliminar = Vec::new();
                    for (i, carro) in carros.iter().enumerate() {
                        if carro.posicion[0] > 650.0 || carro.posicion[1] < -50.0 {
                            indices_a_eliminar.push(i);
                        }
                    }

                    // Segunda pasada: eliminar vehículos (de atrás hacia adelante)
                    indices_a_eliminar.sort_by(|a, b| b.cmp(a));
                    for idx in indices_a_eliminar {
                        if idx < carros.len() {
                            carros.swap_remove(idx);
                        }
                    }

                    // Ordenar vehículos
                    carros.sort_by(|a, b| {
                        a.direccion.cmp(&b.direccion)
                            .then_with(|| match a.direccion {
                                "este" => b.posicion[0].partial_cmp(&a.posicion[0]).unwrap(),
                                "norte" => a.posicion[1].partial_cmp(&b.posicion[1]).unwrap(),
                                _ => std::cmp::Ordering::Equal
                            })
                    });

                    // Mover vehículos
                    for carro in carros.iter_mut() {
                        let semaforo = semaforos.iter()
                            .find(|s| s.direccion == carro.direccion)
                            .unwrap();

                        let puede_avanzar = match semaforo.estado {
                            EstadoSemaforo::Verde => true,
                            EstadoSemaforo::Amarillo => match carro.direccion {
                                "este" => carro.posicion[0] > POSICION_SEMAFORO_VERTICAL,
                                "norte" => carro.posicion[1] < POSICION_SEMAFORO_HORIZONTAL,
                                _ => false
                            },
                            EstadoSemaforo::Rojo => carro.loco,
                        };

                        if puede_avanzar {
                            match carro.direccion {
                                "este" => carro.posicion[0] += carro.velocidad * factor_movimiento,
                                "norte" => carro.posicion[1] -= carro.velocidad * factor_movimiento,
                                _ => {}
                            }
                        }
                    }
                }
            }

            // Actualizar timestamp
            {
                let mut ultima = compartido.ultima_actualizacion.lock().unwrap();
                *ultima = ahora;
            }
        }
    });
}