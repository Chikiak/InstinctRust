//! Unit tests for the turn interpreter (interp.rs).
//! Programs are compiled from text in memory and executed turn by turn.

use instinct_core::actions::Catalog;
use instinct_core::compile;
use instinct_core::event::{Event, WorldOutput};
use instinct_core::expr::LiteralInt;
use instinct_core::interp::{execute_turn, CreatureTurnState, PendingAction};
use instinct_core::perception::PerceptionCatalog;
use instinct_core::program::{Line, Program};
use instinct_core::resources::{Map, Terrain};
use instinct_core::sim::Simulation;
use instinct_core::species::Species;
use instinct_core::value::Value;
use instinct_core::world::CreatureState;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Comparte un Vec de eventos entre el puerto de salida y el test mediante
/// Rc<RefCell>, para poder comprobar que el puerto recibió el evento.
struct SharedOutput {
    events: std::rc::Rc<std::cell::RefCell<Vec<Event>>>,
}

impl WorldOutput for SharedOutput {
    fn emit(&mut self, event: &Event) {
        self.events.borrow_mut().push(event.clone());
    }
}

/// Compiles a minimal `.ins` body (header + start:) with the given body lines.
fn compile_body(body: &str) -> (Program, Catalog) {
    let cat = Catalog::default();
    let pc = PerceptionCatalog::default();
    let text =
        format!("creature Test\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\n{body}");
    let result = compile::compile(&text, &cat, &pc, "test.ins");
    assert!(
        result.diagnostics.is_empty(),
        "compile failed: {:?}",
        result.diagnostics
    );
    (result.program, cat)
}

/// A fake evaluation context backed by a shared variable map. Assignments made
/// during a turn are visible to later reads in the same turn.
struct FakeCtx {
    vars: Rc<RefCell<HashMap<String, Value>>>,
}

impl instinct_core::expr::EvalContext for FakeCtx {
    fn resolve(&self, name: &str) -> Option<Value> {
        self.vars.borrow().get(name).cloned()
    }
}

/// Runs `execute_turn` once. Returns the pending action and the new state.
fn run_once(
    program: &Program,
    ctx: &FakeCtx,
    state: &mut CreatureTurnState,
) -> Option<PendingAction> {
    match execute_turn(program, ctx, state) {
        Ok(result) => {
            *state = result.state;
            result.pending
        }
        Err((d, line)) => panic!("runtime error at line {line}: {:?}", d),
    }
}

/// Runs a full turn: loops while `say` actions are returned (they don't
/// consume the turn), stops at the first non-say pending action or when no
/// action is returned. Returns the final non-say pending action (if any).
fn run_turn_full(
    program: &Program,
    ctx: &FakeCtx,
    state: &mut CreatureTurnState,
) -> Option<PendingAction> {
    loop {
        match run_once(program, ctx, state)? {
            pending if pending.action.consumes_turn() => return Some(pending),
            _ => continue, // say: applied by simulation, continue turn
        }
    }
}

// ---------------------------------------------------------------------------
// AC1: labels, goto, if, assign, say without consuming turn; stops at first
// non-say action; resumes at the next line.
// ---------------------------------------------------------------------------

#[test]
fn test_turn_resumes_after_action() {
    // start:(0), n = 5:(1), say(n):(2), move(1,0,1):(3), goto start:(4)
    let (program, _cat) = compile_body("n = 5\nsay(n)\nmove(1, 0, 1)\ngoto start\n");

    let vars = Rc::new(RefCell::new(HashMap::new()));
    let ctx = FakeCtx {
        vars: Rc::clone(&vars),
    };
    let mut state = CreatureTurnState {
        pc: 0,
        state: CreatureState::Awake,
        variables: Rc::clone(&vars),
    };

    // Turn 1: assign n=5, say(n) (no turn consumed), move ends turn.
    let pending = run_turn_full(&program, &ctx, &mut state);
    let pa = pending.expect("expected a pending action");
    assert_eq!(pa.action.name(), "move");
    assert_eq!(pa.args, vec![Value::Int(1), Value::Int(0), Value::Int(1)]);
    // pc left after the move line (index 3 -> 4); next turn resumes at goto (index 4).
    assert_eq!(state.pc, 4);
    assert_eq!(ctx.vars.borrow().get("n"), Some(&Value::Int(5)));

    // Turn 2: goto start (sets pc to 0), then n=5, say(n), move ends turn.
    let pending = run_turn_full(&program, &ctx, &mut state);
    let pa = pending.expect("expected a pending action");
    assert_eq!(pa.action.name(), "move");
    assert_eq!(state.pc, 4);
}

// ---------------------------------------------------------------------------
// AC2: line limit - a loop of label + goto ends at 100 lines.
// ---------------------------------------------------------------------------

#[test]
fn test_line_limit_ends_turn() {
    // start:(0), loop:(1), goto loop:(2)  -> 2 lines per iteration after start
    let (program, _cat) = compile_body("loop:\ngoto loop\n");
    let ctx = FakeCtx {
        vars: Rc::new(RefCell::new(HashMap::new())),
    };
    let mut state = CreatureTurnState {
        pc: 0,
        state: CreatureState::Awake,
        variables: Rc::clone(&ctx.vars),
    };

    let pending = run_turn_full(&program, &ctx, &mut state);
    assert!(
        pending.is_none(),
        "line limit should end the turn with no action"
    );
    // pc left where the limit was hit (index 2, the goto line).
    assert_eq!(state.pc, 2);
}

// ---------------------------------------------------------------------------
// AC3: end of file -> restart at start.
// ---------------------------------------------------------------------------

#[test]
fn test_end_of_program_restarts_at_start() {
    // start:(0), move(1,0,1):(1)  -> end of file after move
    let (program, _cat) = compile_body("move(1, 0, 1)\n");
    let ctx = FakeCtx {
        vars: Rc::new(RefCell::new(HashMap::new())),
    };
    let mut state = CreatureTurnState {
        pc: 0,
        state: CreatureState::Awake,
        variables: Rc::clone(&ctx.vars),
    };

    // Turn 1: move ends the turn. pc left after move (index 1 -> 2, end of file).
    let pending = run_turn_full(&program, &ctx, &mut state);
    let pa = pending.expect("expected a pending action");
    assert_eq!(pa.action.name(), "move");
    assert_eq!(state.pc, 2);

    // Turn 2: end of file -> restart at start, no action.
    let pending = run_turn_full(&program, &ctx, &mut state);
    assert!(pending.is_none());
    assert_eq!(state.pc, 0);

    // Turn 3: move again from start.
    let pending = run_turn_full(&program, &ctx, &mut state);
    let pa = pending.expect("expected a pending action");
    assert_eq!(pa.action.name(), "move");
    assert_eq!(state.pc, 2);
}

// ---------------------------------------------------------------------------
// AC4: wait(3) skips two turns.
// ---------------------------------------------------------------------------

#[test]
fn test_wait_skips_turns() {
    // start:(0), wait(3):(1), goto start:(2)
    let (program, _cat) = compile_body("wait(3)\ngoto start\n");
    let ctx = FakeCtx {
        vars: Rc::new(RefCell::new(HashMap::new())),
    };
    let mut state = CreatureTurnState {
        pc: 0,
        state: CreatureState::Awake,
        variables: Rc::clone(&ctx.vars),
    };

    // Turn 1: wait(3) -> pending action. Sleep state set by applying the action.
    let pending = run_turn_full(&program, &ctx, &mut state);
    let pa = pending.expect("expected a pending action");
    assert_eq!(pa.action.name(), "wait");
    assert_eq!(pa.args, vec![Value::Int(3)]);
    state.state = CreatureState::Sleeping(2); // simulate apply_action_result setting sleep
    assert_eq!(state.state, CreatureState::Sleeping(2));

    // Turn 2: sleep 2 -> decrement to 1, no action.
    let pending = run_once(&program, &ctx, &mut state);
    assert!(pending.is_none());
    assert_eq!(state.state, CreatureState::Sleeping(1));

    // Turn 3: sleep 1 -> decrement to 0, no action.
    let pending = run_once(&program, &ctx, &mut state);
    assert!(pending.is_none());
    assert_eq!(state.state, CreatureState::Awake);

    // Turn 4: awake. pc = 2 (goto start) -> goto sets pc to 0, then wait(3) -> sleep 2.
    let pending = run_turn_full(&program, &ctx, &mut state);
    let pa = pending.expect("expected a pending action");
    assert_eq!(pa.action.name(), "wait");
    state.state = CreatureState::Sleeping(2); // simulate apply_action_result
    assert_eq!(state.state, CreatureState::Sleeping(2));
}

// ---------------------------------------------------------------------------
// AC7: division by zero -> runtime error.
// ---------------------------------------------------------------------------

#[test]
fn test_division_by_zero_is_runtime_error() {
    // start:(6), n = 1 / 0:(7)
    let (program, _cat) = compile_body("n = 1 / 0\n");
    let ctx = FakeCtx {
        vars: Rc::new(RefCell::new(HashMap::new())),
    };
    let mut state = CreatureTurnState {
        pc: 0,
        state: CreatureState::Awake,
        variables: Rc::clone(&ctx.vars),
    };

    let result = execute_turn(&program, &ctx, &mut state);
    assert!(result.is_err());
    let (_d, line) = result.unwrap_err();
    assert_eq!(line, 7); // `n = 1 / 0` is on line 7
}

// ---------------------------------------------------------------------------
// Variable read before assign -> runtime error.
// ---------------------------------------------------------------------------

#[test]
fn test_unassigned_variable_is_runtime_error() {
    let (program, _cat) = compile_body("n = m + 1\n");
    let ctx = FakeCtx {
        vars: Rc::new(RefCell::new(HashMap::new())),
    };
    let mut state = CreatureTurnState {
        pc: 0,
        state: CreatureState::Awake,
        variables: Rc::clone(&ctx.vars),
    };

    let result = execute_turn(&program, &ctx, &mut state);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// AC5: execute_turn con un programa sin etiqueta `start` devuelve error.
// ---------------------------------------------------------------------------

#[test]
fn test_execute_turn_missing_start_errors() {
    // Program construido a mano: una línea con la etiqueta "foo", sin "start".
    let program = Program {
        lines: vec![Line::Label {
            name: "foo".to_string(),
            line: 1,
        }],
        labels: std::collections::BTreeMap::from([("foo".to_string(), 0)]),
    };
    let ctx = FakeCtx {
        vars: Rc::new(RefCell::new(HashMap::new())),
    };
    let mut state = CreatureTurnState {
        pc: 0,
        state: CreatureState::Awake,
        variables: Rc::new(RefCell::new(HashMap::new())),
    };

    let result = execute_turn(&program, &ctx, &mut state);
    assert!(result.is_err(), "execute_turn debe fallar sin start");
    let (d, _line) = result.unwrap_err();
    assert!(
        d.message.contains("start"),
        "error nombra la etiqueta que falta: {d:?}"
    );
}

// ---------------------------------------------------------------------------
// AC1: el límite cuenta líneas ejecutadas, no intentos; pc queda válido.
// ---------------------------------------------------------------------------

#[test]
fn test_line_limit_executes_100_and_pc_valid() {
    // 152 líneas: label start + `n = 0` + 150 sumas.
    // El turno debe parar en la línea 100 (índice 100), válida y no ejecutada.
    let mut body = String::from("n = 0\n");
    for i in 0..150 {
        body.push_str(&format!("n = n + {i}\n"));
    }
    let (program, _cat) = compile_body(&body);
    assert_eq!(program.lines.len(), 152);

    let vars = Rc::new(RefCell::new(HashMap::new()));
    let ctx = FakeCtx {
        vars: Rc::clone(&vars),
    };
    let mut state = CreatureTurnState {
        pc: 0,
        state: CreatureState::Awake,
        variables: Rc::clone(&vars),
    };

    let pending = run_turn_full(&program, &ctx, &mut state);
    assert!(pending.is_none(), "límite alcanzado sin acción pendiente");
    // Ejecutadas exactamente 100 líneas: label start, `n = 0` y 98 sumas
    // (i = 0..97), con lo que n = 0 + 0 + ... + 97 = 4753.
    assert_eq!(state.pc, 100, "pc apunta a la primera línea no ejecutada");
    assert_eq!(
        ctx.vars.borrow().get("n"),
        Some(&Value::Int(4753)),
        "solo se ejecutaron 98 sumas"
    );
}

// ---------------------------------------------------------------------------
// AC2: salto a destino fuera de rango -> error de ejecución, sin pánico.
// ---------------------------------------------------------------------------

#[test]
fn test_goto_out_of_range_errors() {
    // `loop` está en la tabla de etiquetas pero apunta al índice 5 de un
    // programa de dos líneas: solo alcanzable si el programa se muta después
    // de compilar. El salto debe fallar nombrando la etiqueta.
    let program = Program {
        lines: vec![
            Line::Label {
                name: "start".to_string(),
                line: 1,
            },
            Line::Goto {
                label: "loop".to_string(),
                line: 2,
            },
        ],
        labels: std::collections::BTreeMap::from([
            ("start".to_string(), 0),
            ("loop".to_string(), 5),
        ]),
    };
    let vars = Rc::new(RefCell::new(HashMap::new()));
    let ctx = FakeCtx {
        vars: Rc::clone(&vars),
    };
    let mut state = CreatureTurnState {
        pc: 0,
        state: CreatureState::Awake,
        variables: Rc::clone(&vars),
    };

    let result = execute_turn(&program, &ctx, &mut state);
    assert!(
        result.is_err(),
        "goto fuera de rango debe ser error de ejecución"
    );
    let (d, line) = result.unwrap_err();
    assert_eq!(line, 2);
    assert!(
        d.message.contains("loop"),
        "el error nombra la etiqueta: {d:?}"
    );
    // pc nunca se corrompió: no se indexó fuera de rango.
    assert_eq!(state.pc, 1);
}

#[test]
fn test_if_goto_out_of_range_errors() {
    // `if 1 == 1 goto far` con `far` fuera de rango: error al saltar.
    let program = Program {
        lines: vec![
            Line::Label {
                name: "start".to_string(),
                line: 1,
            },
            Line::IfGoto {
                expr: Box::new(LiteralInt { value: 1 }) as Box<dyn instinct_core::expr::Expr>,
                label: "far".to_string(),
                line: 2,
            },
        ],
        labels: std::collections::BTreeMap::from([
            ("start".to_string(), 0),
            ("far".to_string(), 9),
        ]),
    };
    let vars = Rc::new(RefCell::new(HashMap::new()));
    let ctx = FakeCtx {
        vars: Rc::clone(&vars),
    };
    let mut state = CreatureTurnState {
        pc: 0,
        state: CreatureState::Awake,
        variables: Rc::clone(&vars),
    };

    let result = execute_turn(&program, &ctx, &mut state);
    assert!(
        result.is_err(),
        "if goto fuera de rango debe ser error de ejecución"
    );
    let (d, line) = result.unwrap_err();
    assert_eq!(line, 2);
    assert!(
        d.message.contains("far"),
        "el error nombra la etiqueta: {d:?}"
    );
    assert_eq!(state.pc, 1);
}

// ---------------------------------------------------------------------------
// I5: IfGoto verdadero, falso y con error de evaluación.
// ---------------------------------------------------------------------------

#[test]
fn test_if_goto_true_jumps() {
    // `if a == 1 goto done` salta sobre `n = 2`; fin de archivo reinicia.
    let (program, _cat) = compile_body("a = 1\nif a == 1 goto done\nn = 2\ndone:\nn = 3\n");
    let vars = Rc::new(RefCell::new(HashMap::new()));
    let ctx = FakeCtx {
        vars: Rc::clone(&vars),
    };
    let mut state = CreatureTurnState {
        pc: 0,
        state: CreatureState::Awake,
        variables: Rc::clone(&vars),
    };

    let pending = run_turn_full(&program, &ctx, &mut state);
    assert!(pending.is_none(), "fin de programa sin acción");
    assert_eq!(state.pc, 0, "reinicio en start tras el fin de archivo");
    assert_eq!(
        ctx.vars.borrow().get("a"),
        Some(&Value::Int(1)),
        "la asignación antes del salto se ejecutó"
    );
    assert_eq!(
        ctx.vars.borrow().get("n"),
        Some(&Value::Int(3)),
        "se saltó la línea de en medio y se ejecutó la de después del label"
    );
}

#[test]
fn test_if_goto_false_falls_through() {
    // `if 1 == 2 goto done` no salta; pc pasa a la siguiente línea.
    let (program, _cat) = compile_body("if 1 == 2 goto done\nn = 5\ndone:\nwait(1)\n");
    let vars = Rc::new(RefCell::new(HashMap::new()));
    let ctx = FakeCtx {
        vars: Rc::clone(&vars),
    };
    let mut state = CreatureTurnState {
        pc: 0,
        state: CreatureState::Awake,
        variables: Rc::clone(&vars),
    };

    let pending = run_turn_full(&program, &ctx, &mut state);
    let pa = pending.expect("se espera una acción pendiente");
    assert_eq!(pa.action.name(), "wait");
    assert_eq!(pa.args, vec![Value::Int(1)]);
    assert_eq!(state.pc, 5, "pc avanza línea a línea sin saltar");
    assert_eq!(
        ctx.vars.borrow().get("n"),
        Some(&Value::Int(5)),
        "la línea tras el if se ejecutó"
    );
}

#[test]
fn test_if_goto_eval_error_is_runtime_error() {
    // La condición con error de evaluación propaga el error, sin pánico.
    let (program, _cat) = compile_body("if 1 / 0 goto lab\nlab:\n");
    let vars = Rc::new(RefCell::new(HashMap::new()));
    let ctx = FakeCtx {
        vars: Rc::clone(&vars),
    };
    let mut state = CreatureTurnState {
        pc: 0,
        state: CreatureState::Awake,
        variables: Rc::clone(&vars),
    };

    let result = execute_turn(&program, &ctx, &mut state);
    assert!(result.is_err(), "if goto con error debe fallar");
}

// ---------------------------------------------------------------------------
// I5: goto a etiqueta desconocida, error en tiempo de ejecución.
// ---------------------------------------------------------------------------

#[test]
fn test_goto_unknown_label_is_runtime_error() {
    // Programa construido a mano: `goto missing` sin `missing` en la tabla.
    let program = Program {
        lines: vec![
            Line::Label {
                name: "start".to_string(),
                line: 1,
            },
            Line::Goto {
                label: "missing".to_string(),
                line: 2,
            },
        ],
        labels: std::collections::BTreeMap::from([("start".to_string(), 0)]),
    };
    let vars = Rc::new(RefCell::new(HashMap::new()));
    let ctx = FakeCtx {
        vars: Rc::clone(&vars),
    };
    let mut state = CreatureTurnState {
        pc: 0,
        state: CreatureState::Awake,
        variables: Rc::clone(&vars),
    };

    let result = execute_turn(&program, &ctx, &mut state);
    assert!(result.is_err(), "goto a etiqueta desconocida debe fallar");
    let (d, line) = result.unwrap_err();
    assert_eq!(line, 2);
    assert!(
        d.message.contains("missing"),
        "el error nombra la etiqueta: {d:?}"
    );
}

// ---------------------------------------------------------------------------
// I5: error evaluando argumentos de una acción.
// ---------------------------------------------------------------------------

#[test]
fn test_action_argument_eval_error_is_runtime_error() {
    // `say(1 / 0)`: el intérprete evalúa los argumentos antes de devolver la
    // acción pendiente, y el error de argumento es error de ejecución.
    let (program, _cat) = compile_body("say(1 / 0)\n");
    let vars = Rc::new(RefCell::new(HashMap::new()));
    let ctx = FakeCtx {
        vars: Rc::clone(&vars),
    };
    let mut state = CreatureTurnState {
        pc: 0,
        state: CreatureState::Awake,
        variables: Rc::clone(&vars),
    };

    let result = execute_turn(&program, &ctx, &mut state);
    assert!(
        result.is_err(),
        "error evaluando argumentos debe ser error de ejecución"
    );
}

// ---------------------------------------------------------------------------
// I5: varias acciones say en un mismo turno (say no consume el turno).
// ---------------------------------------------------------------------------

#[test]
fn test_multiple_says_in_one_turn() {
    // Tres say seguidas y luego fin de archivo. Cada `execute_turn` entrega
    // una acción pendiente (say no consume el turno), como hace el simulador.
    let (program, _cat) = compile_body("say(1)\nsay(2)\nsay(3)\n");
    let vars = Rc::new(RefCell::new(HashMap::new()));
    let ctx = FakeCtx {
        vars: Rc::clone(&vars),
    };
    let mut state = CreatureTurnState {
        pc: 0,
        state: CreatureState::Awake,
        variables: Rc::clone(&vars),
    };

    let pending = run_once(&program, &ctx, &mut state).expect("say(1) pendiente");
    assert_eq!(pending.action.name(), "say");
    assert_eq!(pending.args, vec![Value::Int(1)]);
    assert_eq!(state.pc, 2);

    let pending = run_once(&program, &ctx, &mut state).expect("say(2) pendiente");
    assert_eq!(pending.action.name(), "say");
    assert_eq!(pending.args, vec![Value::Int(2)]);
    assert_eq!(state.pc, 3);

    let pending = run_once(&program, &ctx, &mut state).expect("say(3) pendiente");
    assert_eq!(pending.action.name(), "say");
    assert_eq!(pending.args, vec![Value::Int(3)]);
    assert_eq!(state.pc, 4);

    // Fin de archivo: reinicio en start, sin acción.
    let pending = run_once(&program, &ctx, &mut state);
    assert!(pending.is_none());
    assert_eq!(state.pc, 0);
}

// ---------------------------------------------------------------------------
// AC4: extremos aritméticos -> errores de ejecución, nunca pánico.
// ---------------------------------------------------------------------------

#[test]
fn test_arithmetic_extremes_are_runtime_errors() {
    let vars = Rc::new(RefCell::new(HashMap::new()));
    let ctx = FakeCtx {
        vars: Rc::clone(&vars),
    };

    // Desbordamiento: i64::MAX + 1.
    let (program, _cat) = compile_body("n = 9223372036854775807 + 1\n");
    let mut state = CreatureTurnState {
        pc: 0,
        state: CreatureState::Awake,
        variables: Rc::clone(&vars),
    };
    let (d, _line) = execute_turn(&program, &ctx, &mut state).unwrap_err();
    assert!(
        d.message.contains("overflow"),
        "desbordamiento de suma: {:?}",
        d.message
    );

    // Resto por cero.
    let (program, _cat) = compile_body("n = 7 % 0\n");
    let mut state = CreatureTurnState {
        pc: 0,
        state: CreatureState::Awake,
        variables: Rc::clone(&vars),
    };
    let (d, _line) = execute_turn(&program, &ctx, &mut state).unwrap_err();
    assert!(
        d.message.contains("modulo by zero"),
        "resto por cero: {:?}",
        d.message
    );

    // Negación de i64::MIN: m = i64::MIN (sin desbordar al construirlo),
    // luego -m desborda.
    let (program, _cat) = compile_body("m = -9223372036854775807 - 1\nn = -m\n");
    let mut state = CreatureTurnState {
        pc: 0,
        state: CreatureState::Awake,
        variables: Rc::clone(&vars),
    };
    let (d, line) = execute_turn(&program, &ctx, &mut state).unwrap_err();
    assert_eq!(line, 8, "el error está en la segunda línea del cuerpo");
    assert!(
        d.message.contains("unary minus"),
        "negación de i64::MIN: {:?}",
        d.message
    );
}

// ---------------------------------------------------------------------------
// AC3: I6 - error de ejecución del intérprete produce RuntimeError en el bus
// ---------------------------------------------------------------------------

fn species_with_div_zero_program() -> Species {
    let cat = Catalog::default();
    let pc = PerceptionCatalog::default();
    let text =
        "creature DivZero\nfaction any\nhealth 10\nvision 3\nlifespan 100\nstart:\nn = 1 / 0\n";
    let result = compile::compile(text, &cat, &pc, "test.ins");
    assert!(
        result.diagnostics.is_empty(),
        "compile failed: {:?}",
        result.diagnostics
    );
    let header = result.header.expect("header");
    Species::with_program(
        header.species_name,
        header.faction,
        header.health,
        header.vision,
        header.lifespan,
        Rc::new(result.program),
    )
}

#[test]
fn test_interpreter_emits_runtime_error_to_bus() {
    use instinct_core::world::World;

    let map = Map {
        name: "test".to_string(),
        rows: vec![vec!['.'; 5], vec!['.'; 5]],
    };
    let terrains = vec![Terrain {
        name: "Grass".to_string(),
        character: '.',
        resource_max: 100,
        regen: 1,
        reserve: 100,
    }];
    let objects = vec![];

    let captured = Rc::new(RefCell::new(Vec::<Event>::new()));
    let captured_clone = Rc::clone(&captured);

    let mut world = World::from_map(&map, &terrains, &objects);
    world.set_output(Box::new(SharedOutput {
        events: captured_clone,
    }));

    let species = species_with_div_zero_program();
    let id = world.place_creature(species, 0, 0).expect("place creature");

    // Verifica que la criatura está viva antes de crear la simulación.
    assert!(world.creatures.iter().any(|c| c.id == id));

    let mut sim = Simulation::new(world);
    sim.set_rng_seed(42);

    // El primer tick ejecuta `n = 1 / 0`, lo que mata a la criatura con
    // RuntimeError en el bus.
    let _events = sim.step_tick();

    let captured_events = captured.borrow().clone();
    assert!(
        captured_events
            .iter()
            .any(|e| matches!(e, Event::RuntimeError { .. })),
        "se esperaba un RuntimeError en el bus, se obtuvieron: {captured_events:?}"
    );

    // El RuntimeError debe llevar tick, especie, posición, mensaje y línea.
    let runtime_errors: Vec<&Event> = captured_events
        .iter()
        .filter(|e| matches!(e, Event::RuntimeError { .. }))
        .collect();
    assert!(!runtime_errors.is_empty());
    match &runtime_errors[0] {
        Event::RuntimeError {
            tick,
            species,
            pos,
            message,
            line,
        } => {
            assert_eq!(*tick, 0, "tick del RuntimeError");
            assert_eq!(species, "DivZero", "especie del RuntimeError");
            assert_eq!(pos.x, 0, "posición x del RuntimeError");
            assert_eq!(pos.y, 0, "posición y del RuntimeError");
            assert!(!message.is_empty(), "mensaje del RuntimeError no vacío");
            assert_eq!(*line, 7, "línea del error: n = 1 / 0 está en la línea 7");
        }
        _ => unreachable!(),
    };
    // La criatura debe estar muerta.
    assert!(sim.world.creatures.iter().all(|c| c.id != id));
}
