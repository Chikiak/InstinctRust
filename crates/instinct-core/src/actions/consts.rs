//! Action, argument, and diagnostic context names.

pub(super) const ACTION_WAIT: &str = "wait";
pub(super) const ACTION_MOVE: &str = "move";
pub(super) const ACTION_SAY: &str = "say";
pub const ACTION_ATTACK: &str = "attack";
pub(super) const ACTION_CONSUME: &str = "consume";
pub(super) const ACTION_REPRODUCE: &str = "reproduce";
pub(super) const ACTION_ROAR: &str = "roar";

pub(super) const ARG_DX: &str = "dx";
pub(super) const ARG_DY: &str = "dy";
pub(super) const ARG_DAMAGE: &str = "damage";
pub(super) const ARG_HEAL: &str = "heal";
pub(super) const ARG_SPEED: &str = "speed";
pub(super) const ARG_HEALTH: &str = "health";
pub(super) const ARG_N: &str = "n";

pub(super) const DIAG_CATALOG: &str = "catalog";
pub(super) const DIAG_WORLD: &str = "world";

/// Second argument of `Diagnostic::new` (`line: usize`). Diagnostic lines are
/// counted from 1, so this constant represents the implicit line of action
/// diagnostics that do not carry a source location. Avoids the magic literal
/// `1` across the action modules.
pub(super) const DIAG_LINE: usize = 1;
