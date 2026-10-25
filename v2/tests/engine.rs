use serde::Deserialize;
use tezbar_asciimath_to_latex_converter::{convert, wrapped, MAX_INPUT_BYTES};
#[derive(Deserialize)]
struct Row {
    input: String,
    #[serde(rename = "outputUnits")]
    output_units: Vec<u16>,
}
#[test]
fn executable_locked_library_corpus() {
    let rows: Vec<Row> = serde_json::from_str(include_str!("../fixtures/oracle.json")).unwrap();
    let (mut valid, mut invalid) = (0, 0);
    for row in rows {
        match String::from_utf16(&row.output_units) {
            Ok(expected) => {
                assert_eq!(convert(&row.input), Ok(expected), "{:?}", row.input);
                valid += 1;
            }
            Err(_) => {
                assert_eq!(
                    convert(&row.input),
                    Err("Library output contains an unpaired UTF-16 surrogate"),
                    "{:?}",
                    row.input
                );
                invalid += 1;
            }
        }
    }
    assert!(valid > 3600 && invalid > 250);
}
#[test]
fn wrapping_empty_and_huge_separation() {
    assert_eq!(wrapped("", "single"), "$$");
    assert_eq!(wrapped("", "double"), "$$$$");
    assert_eq!(wrapped("", "none"), "");
    assert_eq!(wrapped("{x}", "double"), "$${x}$$");
    assert_eq!(wrapped("{x}", "unknown"), "{x}");
}
#[test]
fn budgets_reject_instead_of_truncate_and_parser_recovers() {
    assert!(convert(&"x".repeat(MAX_INPUT_BYTES + 1)).is_err());
    assert!(convert(&format!("{}x{}", "(".repeat(200), ")".repeat(200))).is_err());
    assert!(convert(&"x+".repeat(3000)).is_err());
    assert_eq!(convert("x+1"), Ok("{x}+{1}".into()));
    assert_eq!(convert("(x_1^)"), Ok("Error parsing expression".into()));
}
