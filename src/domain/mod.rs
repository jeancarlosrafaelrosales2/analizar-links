//! domain — Lógica de negocio pura.
//!
//! REGLA INQUEBRANTABLE: Este módulo NUNCA importa axum, sqlx, reqwest, redis.
//! Solo tipos de dominio, value objects, events y port traits (interfaces).

pub mod entities;
pub mod events;
pub mod ports;
pub mod value_objects;
