//! Control de tiempo de simulación: play, pausa, paso y velocidades 2/10/30 tps.
//! Módulo puro y headless: sin eframe/egui, testable sin ventana.

/// Velocidad de simulación en ticks por segundo (TPS). Decisión 5: exactamente
/// 2 / 10 / 30 ticks por segundo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Speed {
    #[default]
    T2,
    T10,
    T30,
}

impl Speed {
    /// Ticks por segundo para esta velocidad.
    pub const fn tps(self) -> u64 {
        match self {
            Speed::T2 => 2,
            Speed::T10 => 10,
            Speed::T30 => 30,
        }
    }

    /// Milisegundos por tick para esta velocidad (entero, redondeado hacia
    /// abajo). Usado por `advance` para decidir cuándo disparar el siguiente
    /// tick.
    pub const fn ms_per_tick(self) -> u64 {
        match self {
            Speed::T2 => 500,
            Speed::T10 => 100,
            Speed::T30 => 33,
        }
    }
}

/// Estado de la máquina de tiempo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RunState {
    #[default]
    Idle,
    Running(Speed),
    Paused(Speed),
}

/// Control de tiempo de simulación. Lógica pura y headless: no depende de
/// eframe/egui, totalmente testeable sin ventana.
pub struct TimeControl {
    pub state: RunState,
    pub tick: u64,
    pub accumulator_ms: u64,
    pub speed: Speed,
}

impl Default for TimeControl {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeControl {
    /// Crea un control de tiempo en estado `Idle`, tick 0, accumulator 0,
    /// velocidad T2.
    pub fn new() -> Self {
        TimeControl {
            state: RunState::Idle,
            tick: 0,
            accumulator_ms: 0,
            speed: Speed::default(),
        }
    }

    /// Pasa a `Running(speed)`. Desde `Idle` o `Paused` arranca con la velocidad
    /// actual (`self.speed`); desde `Running` no-op.
    pub fn play(&mut self) {
        match self.state {
            RunState::Idle => {
                self.state = RunState::Running(self.speed);
            }
            RunState::Paused(_) => {
                self.state = RunState::Running(self.speed);
            }
            RunState::Running(_) => {}
        }
    }

    /// Pasa a `Paused(speed)` solo desde `Running`. `Idle`/`Paused` no-op.
    pub fn pause(&mut self) {
        if let RunState::Running(s) = self.state {
            self.state = RunState::Paused(s);
        }
    }

    /// Avanza exactamente 1 tick desde cualquier estado, sin tocar el
    /// accumulator. Devuelve 1.
    pub fn step_once(&mut self) -> u64 {
        self.tick = self.tick.saturating_add(1);
        1
    }

    /// Cambia la velocidad. Si está `Running`, la velocidad del estado se
    /// actualiza a la nueva (el siguiente tick usará el nuevo intervalo).
    pub fn set_speed(&mut self, speed: Speed) {
        self.speed = speed;
        if let RunState::Running(_) = self.state {
            self.state = RunState::Running(speed);
        }
        if let RunState::Paused(_) = self.state {
            self.state = RunState::Paused(speed);
        }
    }

    /// Acumula `elapsed_ms` y, mientras se esté `Running` y el accumulator
    /// alcance el intervalo de la velocidad, consume intervalos e incrementa
    /// el tick. Devuelve el número de ticks ejecutados en esta llamada.
    ///
    /// Cuando no está `Running` no acumula ni consume. El residuo se conserva
    /// en `accumulator_ms`. Usa división (no bucle por tick) para evitar
    /// desbordar con deltas grandes.
    pub fn advance(&mut self, elapsed_ms: u64) -> u64 {
        if !matches!(self.state, RunState::Running(_)) {
            return 0;
        }

        self.accumulator_ms = self.accumulator_ms.saturating_add(elapsed_ms);
        let interval = self.speed.ms_per_tick();
        let n_ticks = self.accumulator_ms / interval;
        self.accumulator_ms %= interval;
        self.tick = self.tick.saturating_add(n_ticks);
        n_ticks
    }

    /// Devuelve el tick actual.
    pub fn tick_count(&self) -> u64 {
        self.tick
    }
}
