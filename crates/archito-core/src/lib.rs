//! Núcleo de Archito.
//!
//! No depende de ninguna interfaz: la CLI, el panel de escritorio y un futuro
//! Arch Mobile usan esta misma librería.
//!
//! Flujo: texto del usuario, `Archito::process`, `Response`.
//!
//! Reglas que el núcleo garantiza:
//!   * las sondas (`probe`) solo leen;
//!   * el nivel de riesgo de un comando lo decide `safety::classify`;
//!   * dentro del chat solo se ejecutan lecturas; lo demás va a una terminal
//!     visible y siempre tras una confirmación explícita.

pub mod brain;
pub mod errors;
pub mod exec;
pub mod explain;
pub mod glossary;
pub mod lessons;
pub mod memory;
pub mod model;
pub mod probe;
pub mod safety;
pub mod text;

pub use brain::{Archito, LogEntry};
pub use model::{Action, ExecResult, Learn, Level, Response, Stat, Status, Step};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Hooks de shell, embebidos para que `archito hook` funcione sin el repo.
pub const HOOK_FISH: &str = include_str!("../../../shell/archito.fish");
pub const HOOK_ZSH: &str = include_str!("../../../shell/archito.zsh");
pub const HOOK_BASH: &str = include_str!("../../../shell/archito.bash");
