//! Tipos que el núcleo entrega a cualquier interfaz (CLI, panel, móvil).

use serde::Serialize;

/// Nivel de seguridad de un comando. Lo decide siempre `safety::classify`,
/// nunca quien propone la acción.
#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    /// Solo lectura: se puede ejecutar y mostrar dentro del chat.
    Safe,
    /// Cambia el sistema: se explica y se pide confirmación.
    Confirm,
    /// Puede borrar datos o dejar el equipo inservible: advertencia aparte.
    Danger,
}

#[derive(Serialize, Clone, Debug)]
pub struct Action {
    pub id: u64,
    pub label: String,
    pub command: String,
    pub level: Level,
    pub description: String,
    pub danger_note: String,
}

/// Una pieza de un comando con su explicación.
#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Step {
    pub token: String,
    pub doc: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct Stat {
    pub label: String,
    pub pct: f32,
    pub value: String,
    /// good | warn | danger
    pub severity: &'static str,
}

/// El comando real que hay detrás de una respuesta, para que el usuario lo
/// aprenda en vez de depender de Archito.
#[derive(Serialize, Clone, Debug)]
pub struct Learn {
    pub command: String,
    pub note: String,
}

#[derive(Serialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Ok,
    Warn,
    Error,
    Bye,
}

#[derive(Serialize, Clone, Debug)]
pub struct Response {
    pub text: String,
    pub code: String,
    pub steps: Vec<Step>,
    pub actions: Vec<Action>,
    pub chips: Vec<String>,
    pub stats: Vec<Stat>,
    pub learn: Option<Learn>,
    pub status: Status,
    /// Orden para la interfaz: "camino", "bitacora" o vacío.
    pub view: &'static str,
}

impl Response {
    pub fn new(text: impl Into<String>) -> Self {
        Response {
            text: text.into(),
            code: String::new(),
            steps: Vec::new(),
            actions: Vec::new(),
            chips: Vec::new(),
            stats: Vec::new(),
            learn: None,
            status: Status::Ok,
            view: "",
        }
    }

    pub fn code(mut self, code: impl Into<String>) -> Self {
        self.code = code.into();
        self
    }

    pub fn chips(mut self, chips: &[&str]) -> Self {
        self.chips = chips.iter().map(|c| c.to_string()).collect();
        self
    }

    pub fn status(mut self, status: Status) -> Self {
        self.status = status;
        self
    }

    pub fn learn(mut self, command: &str, note: &str) -> Self {
        self.learn = Some(Learn { command: command.into(), note: note.into() });
        self
    }

    pub fn stat(mut self, label: &str, pct: f32, value: String, severity: &'static str) -> Self {
        self.stats.push(Stat { label: label.into(), pct, value, severity });
        self
    }

    pub fn steps(mut self, steps: Vec<Step>) -> Self {
        self.steps = steps;
        self
    }

    pub fn view(mut self, view: &'static str) -> Self {
        self.view = view;
        self
    }
}

/// Resultado de ejecutar una acción confirmada.
#[derive(Serialize, Clone, Debug)]
pub struct ExecResult {
    pub ok: bool,
    /// inline | terminal
    pub mode: &'static str,
    pub output: String,
}
