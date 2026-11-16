//! Parsing of action arguments.
use super::consts::{ARG_DAMAGE, ARG_DX, ARG_DY, DIAG_LINE};
use crate::diagnostic::Diagnostic;
use crate::value::Value;

pub(super) fn parse_int_arg(
    value: &Value,
    action: &str,
    arg_name: &str,
) -> Result<i64, Diagnostic> {
    match value {
        Value::Int(n) => Ok(*n),
        Value::Text(_) => Err(Diagnostic::new(
            action,
            DIAG_LINE,
            format!("argument '{arg_name}' must be an integer, got text"),
        )),
    }
}

pub(super) fn parse_dir_arg(
    value: &Value,
    action: &str,
    arg_name: &str,
) -> Result<i64, Diagnostic> {
    let n = parse_int_arg(value, action, arg_name)?;
    if !((-1..=1).contains(&n)) {
        return Err(Diagnostic::new(
            action,
            DIAG_LINE,
            format!("argument '{arg_name}' must be -1, 0, or 1, got {n}"),
        ));
    }
    Ok(n)
}

pub(super) fn parse_dir_args(args: &[Value], action: &str) -> Result<(i64, i64), Diagnostic> {
    let dx = parse_dir_arg(&args[0], action, ARG_DX)?;
    let dy = parse_dir_arg(&args[1], action, ARG_DY)?;
    Ok((dx, dy))
}

pub(super) fn parse_strike_args(
    action: &str,
    args: &[Value],
) -> Result<(i64, i64, i64), Diagnostic> {
    let dx = parse_dir_arg(&args[0], action, ARG_DX)?;
    let dy = parse_dir_arg(&args[1], action, ARG_DY)?;
    let damage = parse_int_arg(&args[2], action, ARG_DAMAGE)?;
    Ok((dx, dy, damage))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::Value;

    #[test]
    fn parse_dir_args_ok() {
        let args = [Value::Int(1), Value::Int(0)];
        assert_eq!(parse_dir_args(&args, "attack"), Ok((1, 0)));
    }

    #[test]
    fn parse_dir_args_bad_dx() {
        let args = [Value::Int(2), Value::Int(0)];
        let err = parse_dir_args(&args, "consume").unwrap_err();
        assert!(err.message.contains("dx"));
    }

    #[test]
    fn parse_dir_args_bad_dy() {
        let args = [Value::Int(0), Value::Int(-2)];
        let err = parse_dir_args(&args, "move").unwrap_err();
        assert!(err.message.contains("dy"));
    }

    #[test]
    fn parse_dir_args_text_dx() {
        let args = [Value::Text("x".to_string()), Value::Int(0)];
        let err = parse_dir_args(&args, "attack").unwrap_err();
        assert!(err.message.contains("dx"));
    }

    #[test]
    fn parse_dir_args_text_dy() {
        let args = [Value::Int(0), Value::Text("y".to_string())];
        let err = parse_dir_args(&args, "consume").unwrap_err();
        assert!(err.message.contains("dy"));
    }

    #[test]
    fn parse_strike_args_ok() {
        let args = [Value::Int(1), Value::Int(0), Value::Int(10)];
        assert_eq!(parse_strike_args("attack", &args), Ok((1, 0, 10)));
    }

    #[test]
    fn parse_strike_args_bad_dx() {
        let args = [Value::Int(2), Value::Int(0), Value::Int(10)];
        let err = parse_strike_args("consume", &args).unwrap_err();
        assert!(err.message.contains("dx"));
    }

    #[test]
    fn parse_strike_args_text_damage() {
        let args = [
            Value::Int(1),
            Value::Int(0),
            Value::Text("tres".to_string()),
        ];
        let err = parse_strike_args("attack", &args).unwrap_err();
        assert!(err.message.contains("damage"));
    }
}
