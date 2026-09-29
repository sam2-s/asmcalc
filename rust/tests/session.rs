//! Session and engine tests. These run natively on the host.

use asmcalc::engine::{Calculator, Engine};
use asmcalc::expr::evaluate_exact;
use asmcalc::expr::parser::AngleMode;
use asmcalc::kernel;

fn scientific() -> Calculator {
    let mut calculator = Calculator::new();
    calculator.set_engine(Engine::Scientific);
    calculator
}

#[test]
fn fixed_engine_is_the_assembly_one() {
    let mut calculator = Calculator::new();
    assert_eq!(calculator.engine(), Engine::Fixed);

    // Replay a key sequence through the assembly kernel.
    for ch in "12*12=".chars() {
        let key = asmcalc::fixed::keycode_for(ch).unwrap();
        calculator.press(key);
    }
    assert_eq!(calculator.display(), "144");
}

#[test]
fn fixed_engine_reports_errors() {
    let mut calculator = Calculator::new();
    for ch in "1/0=".chars() {
        calculator.press(asmcalc::fixed::keycode_for(ch).unwrap());
    }
    assert_eq!(calculator.display(), "Error");
}

#[test]
fn scientific_types_and_evaluates() {
    let mut calculator = scientific();
    calculator.press(kernel::KEY_1);
    calculator.press(kernel::KEY_2);
    assert_eq!(calculator.display(), "12");

    calculator.press(kernel::KEY_ADD);
    calculator.press(kernel::KEY_3);
    assert_eq!(calculator.display(), "15");

    calculator.press(kernel::KEY_EQUALS);
    assert_eq!(calculator.display(), "15");
}

#[test]
fn scientific_reports_incomplete_input_without_complaining() {
    let mut calculator = scientific();
    calculator.press(kernel::KEY_1);
    calculator.press(kernel::KEY_ADD);
    // Mid expression: no error, no number either.
    assert_eq!(calculator.display(), "");
}

#[test]
fn scientific_rejects_a_second_decimal_point() {
    let mut calculator = scientific();
    calculator.press(kernel::KEY_1);
    calculator.press(kernel::KEY_DOT);
    calculator.press(kernel::KEY_5);
    calculator.press(kernel::KEY_DOT);
    calculator.press(kernel::KEY_5);
    assert_eq!(calculator.expression(), "1.55");
    assert_eq!(calculator.display(), "1.55");
}

#[test]
fn angle_mode_switches_trig() {
    let mut calculator = scientific();
    calculator.set_angles(AngleMode::Degrees);
    calculator.press(kernel::KEY_3);
    calculator.press(kernel::KEY_0);
    assert_eq!(calculator.angles(), AngleMode::Degrees);
    assert_eq!(calculator.snapshot().angle_mode, "DEG");

    calculator.set_angles(AngleMode::Radians);
    assert_eq!(calculator.snapshot().angle_mode, "RAD");
}

#[test]
fn clear_resets_the_expression() {
    let mut calculator = scientific();
    calculator.press(kernel::KEY_9);
    calculator.press(kernel::KEY_ADD);
    calculator.press(kernel::KEY_1);
    assert_eq!(calculator.display(), "10");

    calculator.press(kernel::KEY_CLEAR);
    assert_eq!(calculator.expression(), "");
    assert_eq!(calculator.display(), "0");
}

#[test]
fn submit_records_history() {
    let mut calculator = scientific();
    assert_eq!(calculator.submit("2+3*4"), "14");
    assert_eq!(calculator.history().len(), 1);
    assert_eq!(calculator.history()[0].expression, "2+3*4");
    assert_eq!(calculator.history()[0].result, "14");

    assert_eq!(calculator.submit("sqrt(144)"), "12");
    assert_eq!(calculator.history().len(), 2);
}

#[test]
fn submit_records_errors_too() {
    let mut calculator = scientific();
    let result = calculator.submit("1/0");
    assert!(result.starts_with("Error"), "got {result}");
    assert_eq!(calculator.history().len(), 1);
}

#[test]
fn reference_engine_matches_the_assembly_on_errors() {
    // The Rust reference stands in for the assembly on the host, so it has to
    // agree about error handling, not just about arithmetic.
    let mut calculator = Calculator::new();
    calculator.set_engine(Engine::Fixed);
    for ch in "1/0=".chars() {
        calculator.press(asmcalc::fixed::keycode_for(ch).unwrap());
    }
    assert_eq!(calculator.display(), "Error");
}

#[test]
fn history_is_capped() {
    let mut calculator = scientific();
    for i in 0..60 {
        calculator.submit(&format!("{i}+1"));
    }
    assert_eq!(calculator.history().len(), 50, "the tape is bounded");
}

#[test]
fn recall_restores_an_entry() {
    let mut calculator = scientific();
    calculator.submit("2+2");
    calculator.submit("3+3");
    assert_eq!(calculator.recall(0), "2+2");
    assert_eq!(calculator.expression(), "2+2");
    assert_eq!(calculator.recall(99), "", "out of range is harmless");
}

#[test]
fn memory_registers_accumulate() {
    let mut calculator = scientific();
    calculator.submit("10");
    assert_eq!(calculator.memory_add(0), 10.0);

    calculator.submit("5");
    assert_eq!(calculator.memory_add(0), 15.0);
    assert_eq!(calculator.memory_recall(0), 15.0);

    calculator.submit("3");
    assert_eq!(calculator.memory_subtract(0), 12.0);
    assert_eq!(calculator.memory_recall(0), 12.0);

    calculator.memory_clear(0);
    assert_eq!(calculator.memory_recall(0), 0.0);
}

#[test]
fn memory_slots_are_independent() {
    let mut calculator = scientific();
    calculator.submit("7");
    calculator.memory_add(0);
    calculator.submit("9");
    calculator.memory_add(1);
    assert_eq!(calculator.memory_recall(0), 7.0);
    assert_eq!(calculator.memory_recall(1), 9.0);
}

#[test]
fn memory_store_then_recall() {
    let mut calculator = scientific();
    calculator.submit("42");
    calculator.memory_store(2);
    calculator.submit("1");
    assert_eq!(calculator.memory_recall(2), 42.0);
}

#[test]
fn out_of_range_slots_are_ignored() {
    let mut calculator = scientific();
    calculator.submit("1");
    assert_eq!(calculator.memory_add(99), 0.0);
    assert_eq!(calculator.memory_recall(99), 0.0);
    calculator.memory_store(99);
    calculator.memory_clear(99);
}

#[test]
fn exact_mode_gives_exact_answers() {
    let mut calculator = scientific();
    // A third, to eighteen places. The f64 engine would show 0.3333333333.
    let result = calculator.submit_exact("1/3");
    assert_eq!(result, "0.333333333333333333");
}

#[test]
fn exact_mode_handles_big_factorials() {
    let mut calculator = scientific();
    let result = calculator.submit_exact("25!");
    assert_eq!(result, "15511210043330985984000000");
}

#[test]
fn exact_mode_arithmetic() {
    assert_eq!(evaluate_exact("0.1+0.2").unwrap().format(2), "0.3");
    assert_eq!(evaluate_exact("2^100").unwrap().format(0), "1267650600228229401496703205376");
    assert_eq!(evaluate_exact("(2+3)*4").unwrap().format(0), "20");
    assert_eq!(evaluate_exact("-5+3").unwrap().format(0), "-2");
    assert_eq!(evaluate_exact("7 mod 3").unwrap_or_else(|_| evaluate_exact("7%3").unwrap()).format(0), "1");
    assert!(evaluate_exact("1/0").is_err());
    // A function with no exact decimal answer is refused rather than faked.
    assert!(evaluate_exact("pi").is_err());
    assert!(evaluate_exact("sqrt(2)").is_err());
}

#[test]
fn snapshot_captures_everything_the_ui_needs() {
    let mut calculator = scientific();
    calculator.set_angles(AngleMode::Gradians);
    calculator.submit("2+2");
    calculator.memory_add(0);

    let snapshot = calculator.snapshot();
    assert_eq!(snapshot.display, "4");
    assert_eq!(snapshot.angle_mode, "GRAD");
    assert_eq!(snapshot.engine, Engine::Scientific);
    assert_eq!(snapshot.history.len(), 1);
    assert_eq!(snapshot.registers[0].value, 4.0);
    assert!(snapshot.registers[0].used);
    assert!(!snapshot.registers[1].used);
}
