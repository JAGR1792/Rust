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

// En controlador.rs, función iniciar_generador_carros
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
                            "este" => {
                                carro.posicion[0] < 100.0 &&
                                    (carro.posicion[0] - pos[0]).abs() < distancia_minima
                            },
                            "norte" => {
                                carro.posicion[1] > 500.0 &&
                                    (carro.posicion[1] - pos[1]).abs() < distancia_minima
                            },
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
                };

                // Variar ligeramente la velocidad para evitar agrupaciones

                let velocidad_ajustada = VELOCIDAD_VEHICULO;

                if let Err(_) = emisor.send(Carro {
                    posicion: pos,
                    direccion,
                    color,
                    velocidad: velocidad_ajustada as f32,
                    tipo: tipo_vehiculo,
                    loco: es_loco,
                }){
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
            let mut accidentes = Vec::new();

            // Recolectar información de intersección antes de mover los vehículos
            let carros_en_interseccion = {
                let carros = compartido.carros.lock().unwrap();
                // Recopilar los índices y direcciones de los vehículos en la intersección
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

            // Detectar colisiones basadas en la información recopilada
            for (i, dir_i, loco_i) in &carros_en_interseccion {
                if *loco_i { // Solo los vehículos locos pueden causar accidentes
                    for (j, dir_j, _) in &carros_en_interseccion {
                        if i != j && dir_i != dir_j {
                            // Colisión detectada
                            accidentes.push(*i);
                            accidentes.push(*j);
                        }
                    }
                }
            }

            // Scope para minimizar tiempo de lock - procesamiento principal
            {
                let mut carros = compartido.carros.lock().unwrap();
                let semaforos = compartido.semaforos.lock().unwrap();

                // Ordenar los carros por posición según dirección
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

                // Mantener un registro de los vehículos y sus posiciones para detectar colisiones
                let mut posiciones_este = Vec::new();
                let mut posiciones_norte = Vec::new();

                // Procesar cada vehículo
                for (i, carro) in carros.iter_mut().enumerate() {
                    let semaforo = semaforos.iter()
                        .find(|s| s.direccion == carro.direccion)
                        .unwrap();

                    // Determinar el espacio necesario según el tipo de vehículo
                    let espacio_necesario = match carro.tipo {
                        TipoVehiculo::Automovil => 50.0,
                        TipoVehiculo::Camioneta => 50.0,
                        TipoVehiculo::Camion => 60.0,
                    };

                    // Verificar si hay vehículos adelante que bloqueen el paso
                    let hay_obstaculo = match carro.direccion {
                        "este" => posiciones_este.iter().any(|&pos|
                            pos > carro.posicion[0] && pos - carro.posicion[0] < espacio_necesario),
                        "norte" => posiciones_norte.iter().any(|&pos|
                            pos < carro.posicion[1] && carro.posicion[1] - pos < espacio_necesario),
                        _ => false
                    };

                    let puede_avanzar = if hay_obstaculo {
                        false
                    } else {
                        match semaforo.estado {
                            EstadoSemaforo::Verde => true,
                            EstadoSemaforo::Amarillo | EstadoSemaforo::Rojo => {
                                if carro.loco {
                                    true
                                } else {
                                    match carro.direccion {
                                        "este" => carro.posicion[0] > POSICION_SEMAFORO_VERTICAL ||
                                            carro.posicion[0] < POSICION_SEMAFORO_VERTICAL - 20.0,
                                        "norte" => carro.posicion[1] < POSICION_SEMAFORO_HORIZONTAL ||
                                            carro.posicion[1] > POSICION_SEMAFORO_HORIZONTAL + 20.0,
                                        "oeste" => carro.posicion[0] < POSICION_SEMAFORO_VERTICAL,
                                        "sur" => carro.posicion[1] > POSICION_SEMAFORO_HORIZONTAL,
                                        _ => false
                                    }
                                }
                            }
                        }
                    };

                    if puede_avanzar {
                        // Movimiento ajustado por tiempo para mantener velocidad constante
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

            // Gestionar accidentes - ahora con su propio scope y lock de carros
            if !accidentes.is_empty() {
                let mut contador = compartido.contador_accidentes.lock().unwrap();
                *contador += 1;

                // Lock carros de nuevo para eliminar los vehículos accidentados
                {
                    let mut carros = compartido.carros.lock().unwrap();
                    // Eliminar vehículos accidentados
                    accidentes.sort_by(|a, b| b.cmp(a));
                    accidentes.dedup(); // Eliminar duplicados
                    for idx in accidentes {
                        if idx < carros.len() {
                            carros.swap_remove(idx);
                        }
                    }
                }
            }
        }
    });
}