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

pub fn iniciar_generador_carros(emisor: mpsc::Sender<Carro>, compartido: EstadoCompartido) {
    thread::spawn(move || {
        let mut rng = rand::rng();
        loop {
            thread::sleep(Duration::from_secs(INTERVALO_APARICION));

            // Añadir aleatoriedad para evitar ráfagas de vehículos
            if rng.random_bool(0.8) { // 80% de probabilidad de generar
                let idx = rng.random_range(0..2);
                let (direccion, pos) = PUNTOS_APARICION[idx];
                let es_loco = rng.random_bool(0.1);

                // Verificar si hay espacio suficiente para un nuevo vehículo
                let espacio_suficiente = {
                    let carros = compartido.carros.lock().unwrap();
                    let distancia_minima = 60.0; // Distancia mínima entre vehículos

                    !carros.iter().any(|carro| {
                        // Solo verificar vehículos en la misma dirección
                        if carro.direccion != direccion {
                            return false;
                        }

                        // Calcular distancia según la dirección
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
                    continue; // Esperar al siguiente ciclo
                }

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
                    // Canal cerrado, terminar hilo
                    break;
                }
            }
        }
    });
}

fn crear_vehiculo_emergencia() -> Carro {
    let mut rng = rand::rng();
    let idx = rng.random_range(0..PUNTOS_APARICION_EMERGENCIA.len());
    let (direccion, pos) = PUNTOS_APARICION_EMERGENCIA[idx];

    let tipo = if rng.random_bool(0.5) {
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
            // Control de velocidad de actualización para reducir uso de CPU
            let ahora = Instant::now();
            let delta_ms = ahora.duration_since(ultimo_update).as_millis() as f32;
            let fps_interval = 1000.0 / FPS_SIMULACION as f32;

            if delta_ms < fps_interval {
                let espera = (fps_interval - delta_ms) as u64;
                thread::sleep(Duration::from_millis(espera));
                continue;
            }

            ultimo_update = ahora;
            let factor_movimiento = delta_ms / 1000.0;
            let mut removidos = Vec::new();
            let mut accidentes = Vec::new();

            // Manejo de accidentes activos
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
            {
                let mut carros = compartido.carros.lock().unwrap();
                let mut accidentes_temp = Vec::new();

                // Definir radios de colisión según tipo de vehículo
                for i in 0..carros.len() {
                    let radio_i = match carros[i].tipo {
                        TipoVehiculo::Automovil => 15.0,
                        TipoVehiculo::Camioneta => 20.0,
                        TipoVehiculo::Camion => 25.0,
                        _ => 15.0,
                    };

                    for j in (i + 1)..carros.len() {
                        let radio_j = match carros[j].tipo {
                            TipoVehiculo::Automovil => 15.0,
                            TipoVehiculo::Camioneta => 20.0,
                            TipoVehiculo::Camion => 25.0,
                            _ => 15.0,
                        };

                        let dx = carros[i].posicion[0] - carros[j].posicion[0];
                        let dy = carros[i].posicion[1] - carros[j].posicion[1];
                        let distancia = (dx * dx + dy * dy).sqrt();

                        // Verificar colisión y que no sean vehículos de emergencia
                        if distancia < (radio_i + radio_j)
                            && !matches!(carros[i].tipo, TipoVehiculo::Ambulancia | TipoVehiculo::Policia)
                            && !matches!(carros[j].tipo, TipoVehiculo::Ambulancia | TipoVehiculo::Policia)
                        {
                            accidentes_temp.push(i);
                            accidentes_temp.push(j);
                        }
                    }
                }

                // Eliminar duplicados y ordenar
                accidentes_temp.sort();
                accidentes_temp.dedup();
                accidentes = accidentes_temp.clone();
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

            // Movimiento normal de vehículos
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

                    // Primera pasada: identificar vehículos a eliminar (fuera de pantalla)
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

                    // Ordenar vehículos para mejor procesamiento
                    carros.sort_by(|a, b| {
                        match a.direccion.cmp(&b.direccion) {
                            std::cmp::Ordering::Equal => {
                                // Si van en la misma dirección, ordenar por posición
                                match a.direccion {
                                    "este" => b.posicion[0].partial_cmp(&a.posicion[0]).unwrap_or(std::cmp::Ordering::Equal),
                                    "norte" => a.posicion[1].partial_cmp(&b.posicion[1]).unwrap_or(std::cmp::Ordering::Equal),
                                    _ => std::cmp::Ordering::Equal
                                }
                            },
                            other => other
                        }
                    });

                    // Mantener registro de posiciones para evitar colisiones entre vehículos
                    let mut posiciones_este = Vec::new();
                    let mut posiciones_norte = Vec::new();

                    // Mover vehículos
                    for carro in carros.iter_mut() {
                        let semaforo = semaforos.iter()
                            .find(|s| s.direccion == carro.direccion)
                            .unwrap();

                        // Determinar el espacio necesario según el tipo de vehículo
                        let espacio_necesario = match carro.tipo {
                            TipoVehiculo::Automovil => 50.0,
                            TipoVehiculo::Camioneta => 50.0,
                            TipoVehiculo::Camion => 60.0,
                            _ => 50.0, // Para vehículos de emergencia
                        };

                        // Verificar si hay vehículos adelante que bloqueen el paso
                        let hay_obstaculo = match carro.direccion {
                            "este" => posiciones_este.iter().any(|&pos|
                                pos > carro.posicion[0] && pos - carro.posicion[0] < espacio_necesario),
                            "norte" => posiciones_norte.iter().any(|&pos|
                                pos < carro.posicion[1] && carro.posicion[1] - pos < espacio_necesario),
                            _ => false
                        };


                        let ya_paso_semaforo = match carro.direccion {
                            "este" => carro.posicion[0] > POSICION_SEMAFORO_VERTICAL + 30.0, // Ya pasó el semáforo
                            "norte" => carro.posicion[1] < POSICION_SEMAFORO_HORIZONTAL - 30.0, // Ya pasó el semáforo
                            _ => false
                        };

                        let puede_avanzar = if hay_obstaculo {
                            false
                        } else if ya_paso_semaforo {

                            true
                        } else {
                            match semaforo.estado {
                                EstadoSemaforo::Verde => true,
                                EstadoSemaforo::Amarillo => {
                                    // Los vehículos deben avanzar hasta el borde del semáforo con luz amarilla
                                    // y detenerse solo cuando llegan exactamente al semáforo
                                    if carro.loco {
                                        true // Los locos siempre pasan
                                    } else {
                                        match carro.direccion {
                                            "este" => {
                                                // Avanzar hasta llegar al semáforo
                                                carro.posicion[0] < POSICION_SEMAFORO_VERTICAL
                                            },
                                            "norte" => {
                                                // Avanzar hasta llegar al semáforo
                                                carro.posicion[1] > POSICION_SEMAFORO_HORIZONTAL
                                            },
                                            _ => false
                                        }
                                    }
                                },
                                EstadoSemaforo::Rojo => {
                                    // CORRECCIÓN: Con luz roja, los vehículos pueden acercarse al semáforo sin pasarlo
                                    if carro.loco {
                                        true
                                    } else {
                                        match carro.direccion {
                                            "este" => carro.posicion[0] < POSICION_SEMAFORO_VERTICAL - 10.0,
                                            "norte" => carro.posicion[1] > POSICION_SEMAFORO_HORIZONTAL + 10.0,
                                            _ => false
                                        }
                                    }
                                }
                            }
                        };

                        if puede_avanzar {
                            match carro.direccion {
                                "este" => carro.posicion[0] += carro.velocidad * factor_movimiento,
                                "norte" => carro.posicion[1] -= carro.velocidad * factor_movimiento,
                                _ => {}
                            }
                        }

                        // Registrar la posición para el siguiente vehículo
                        match carro.direccion {
                            "este" => posiciones_este.push(carro.posicion[0]),
                            "norte" => posiciones_norte.push(carro.posicion[1]),
                            _ => {}
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

pub fn debe_detenerse(carro: &Carro, semaforos: &[Semaforo]) -> bool {
    let semaforo_relevante = semaforos.iter()
        .find(|s| s.direccion == carro.direccion);

    // Nueva definición de límites de intersección
    let (limite_entrada, limite_salida) = match carro.direccion {
        "este" => (POSICION_SEMAFORO_VERTICAL + 30.0, POSICION_SEMAFORO_VERTICAL - 30.0),
        "norte" => (POSICION_SEMAFORO_HORIZONTAL - 30.0, POSICION_SEMAFORO_HORIZONTAL + 30.0),
        _ => (0.0, 0.0)
    };

    // Verificar si ya está dentro de la intersección
    let en_interseccion = match carro.direccion {
        "este" => carro.posicion[0] > limite_entrada && carro.posicion[0] < limite_salida,
        "norte" => carro.posicion[1] < limite_entrada && carro.posicion[1] > limite_salida,
        _ => false
    };

    // Si está dentro de la intersección, nunca debe detenerse
    if en_interseccion {
        return false;
    }

    if let Some(semaforo) = semaforo_relevante {
        match semaforo.estado {
            EstadoSemaforo::Rojo => {
                // Con luz roja, los locos o con baja probabilidad otros vehículos pasarán
                if carro.loco {
                    false // Los vehículos locos siempre pasan
                } else {
                    // Algunos vehículos normales a veces también se pasan el rojo
                    let mut rng = rand::rng();
                    if rng.random_bool(0.05) {
                        false // No se detiene (se pasa el rojo)
                    } else {
                        // Se detiene justo en el borde del semáforo
                        match carro.direccion {
                            "este" => carro.posicion[0] >= POSICION_SEMAFORO_VERTICAL - 10.0,
                            "norte" => carro.posicion[1] <= POSICION_SEMAFORO_HORIZONTAL + 10.0,
                            _ => true
                        }
                    }
                }
            },
            EstadoSemaforo::Amarillo => {
                // Con luz amarilla, los vehículos deben continuar hasta el borde del semáforo
                if carro.loco {
                    false // Los locos siempre pasan
                } else {
                    // Otros vehículos deberían detenerse SOLO SI han llegado al semáforo
                    // pero no antes
                    match carro.direccion {
                        "este" => {
                            // Detener solo cuando llega exactamente al borde del semáforo
                            let distancia_al_semaforo = POSICION_SEMAFORO_VERTICAL - carro.posicion[0];
                            distancia_al_semaforo <= 0.0 && distancia_al_semaforo > -5.0
                        },
                        "norte" => {
                            // Detener solo cuando llega exactamente al borde del semáforo
                            let distancia_al_semaforo = carro.posicion[1] - POSICION_SEMAFORO_HORIZONTAL;
                            distancia_al_semaforo <= 0.0 && distancia_al_semaforo > -5.0
                        },
                        _ => true
                    }
                }
            },
            EstadoSemaforo::Verde => false // Con luz verde, nunca debe detenerse
        }
    } else {
        false
    }
}

pub fn actualizar_vehiculos(carros: &mut Vec<Carro>, semaforos: &[Semaforo], delta_tiempo: f32) {
    for carro in carros.iter_mut() {
        if debe_detenerse(carro, semaforos) {
            continue;
        }

        match carro.direccion {
            "este" => carro.posicion[0] += carro.velocidad * delta_tiempo,
            "norte" => carro.posicion[1] -= carro.velocidad * delta_tiempo,
            _ => {} // Para otras direcciones
        }
    }
}