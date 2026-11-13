use xrtak_core::casevac::*;

#[test]
fn test_precedence_doctrine_codes() {
    assert_eq!(PatientPrecedence::Urgent.code(), "A - URGENT");
    assert_eq!(PatientPrecedence::UrgentSurgical.code(), "B - URGENT-SURG");
    assert_eq!(PatientPrecedence::Priority.code(), "C - PRIORITY");
    assert_eq!(PatientPrecedence::Routine.code(), "D - ROUTINE");
    assert_eq!(PatientPrecedence::Convenience.code(), "E - CONVENIENCE");
}

fn sample_report(precedence: PatientPrecedence) -> MedevacReport {
    MedevacReport {
        line1_location: "GRID 12S TK 1234 5678".to_string(),
        line2_freq_callsign: "45.00 MHz / DUSTOFF".to_string(),
        line3_precedence: precedence,
        line4_special_equipment: "Hoist".to_string(),
        line5_litter_count: 2,
        line5_ambulatory_count: 1,
        line6_security: "N".to_string(),
        line7_hlz_marking: "Smoke".to_string(),
        line8_patient_nationality: "Friendly".to_string(),
        line9_nbc_or_terrain: "None".to_string(),
        lat: 34.0,
        lon: -117.0,
        east_m: 0.0,
        north_m: 0.0,
        up_m: 0.0,
    }
}

#[test]
fn test_nine_line_order_and_content() {
    let rep = sample_report(PatientPrecedence::Urgent);
    let text = format_nato_nine_line_text(&rep);
    let lines: Vec<&str> = text.lines().collect();

    assert_eq!(lines[0], "--- NATO 9-LINE MEDEVAC REQUEST ---");
    for i in 1..=9 {
        let prefix = format!("LINE {i}");
        assert!(
            lines[i].starts_with(&prefix),
            "Expected line {} to start with {}, got {}",
            i,
            prefix,
            lines[i]
        );
    }
}
