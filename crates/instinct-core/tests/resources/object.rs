//! Object loading tests for the `resources` module.
use super::common::*;

#[test]
fn test_load_object_example() {
    let content = "object Rock\nchar #\nresource_max 100\n";
    let (result, d) = load_object(content, "rock.ob");
    assert!(d.is_empty(), "unexpected diagnostics: {d:?}");
    let o = result.expect("must load the object");
    assert_eq!(o.name, "Rock");
    assert_eq!(o.character, '#');
    assert_eq!(o.resource_max, 100);
    assert_eq!(o.reserve, 100);
}

#[test]
fn test_char_hash_is_not_a_comment() {
    // A line starting with '#' is a comment, but 'char #' is not.
    let content = "object Rock\nchar #\nresource_max 100\n";
    let (result, d) = load_object(content, "rock.ob");
    assert!(d.is_empty(), "unexpected diagnostics: {d:?}");
    let o = result.expect("must load the object");
    assert_eq!(o.character, '#');
}
