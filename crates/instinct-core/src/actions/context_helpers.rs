//! Helpers de contexto: bounds, salud, noop, value_to_text, creature_context, action_result.
use crate::actions::consts::DIAG_LINE;
use crate::actions::{ActionResult, ReadContext};
use crate::diagnostic::Diagnostic;
use crate::event::{Event, Pos};
use crate::value::Value;
use crate::world::CreatureId;

pub(super) fn checked_sub_health(
    current: i64,
    amount: i64,
    action: &str,
    pos: Pos,
) -> Result<i64, Diagnostic> {
    current.checked_sub(amount).ok_or_else(|| {
        Diagnostic::new(
            action,
            DIAG_LINE,
            format!("overflow reducing {action} health at ({},{})", pos.x, pos.y),
        )
    })
}

pub(super) fn is_out_of_bounds(ctx: &dyn ReadContext, x: i64, y: i64) -> bool {
    x < 0 || y < 0 || x >= ctx.columns() as i64 || y >= ctx.rows() as i64
}

pub(super) fn noop_result() -> ActionResult {
    action_result(0, Vec::new())
}

pub(super) fn value_to_text(value: &Value) -> String {
    match value {
        Value::Int(n) => n.to_string(),
        Value::Text(s) => s.clone(),
    }
}

pub(super) fn creature_context(
    ctx: &dyn ReadContext,
    creature_id: CreatureId,
) -> Result<(Pos, String, u64), Diagnostic> {
    let pos = ctx.creature_position(creature_id)?;
    let species = ctx.creature_species(creature_id)?;
    let tick = ctx.tick();
    Ok((pos, species, tick))
}

pub(super) fn action_result(sleep_turns: u32, events: Vec<Event>) -> ActionResult {
    ActionResult {
        sleep_turns,
        events,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::ActionResult;
    use crate::event::{Event, Pos};

    #[test]
    fn noop_result_returns_empty() {
        let result = noop_result();
        assert_eq!(result.sleep_turns, 0);
        assert!(result.events.is_empty());
    }

    #[test]
    fn action_result_constructs() {
        let result = action_result(3, vec![]);
        assert_eq!(
            result,
            ActionResult {
                sleep_turns: 3,
                events: vec![]
            }
        );
    }

    #[test]
    fn action_result_with_events() {
        let result = action_result(
            0,
            vec![Event::Said {
                tick: 5,
                species: "wolf".to_string(),
                pos: Pos { x: 1, y: 2 },
                text: "hi".to_string(),
            }],
        );
        assert_eq!(result.sleep_turns, 0);
        assert_eq!(result.events.len(), 1);
    }

    #[test]
    fn checked_sub_health_ok() {
        let pos = Pos { x: 3, y: 4 };
        assert_eq!(checked_sub_health(10, 3, "test", pos), Ok(7));
    }

    #[test]
    fn checked_sub_health_no_overflow() {
        let pos = Pos { x: 0, y: 0 };
        assert_eq!(checked_sub_health(5, 3, "test", pos), Ok(2));
    }

    #[test]
    fn checked_sub_health_overflow() {
        let pos = Pos { x: 9, y: 9 };
        let err = checked_sub_health(i64::MIN, 1, "reproduce", pos).unwrap_err();
        assert!(err
            .message
            .contains("overflow reducing reproduce health at (9,9)"));
    }
}
