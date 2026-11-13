use xrtak_core::casevac::*;

#[test]
fn test_precedence_doctrine_codes() {
    assert_eq!(PatientPrecedence::Urgent.code(), "A - URGENT");
    assert_eq!(PatientPrecedence::UrgentSurgical.code(), "B - URGENT-SURG");
    assert_eq!(PatientPrecedence::Priority.code(), "C - PRIORITY");
    assert_eq!(PatientPrecedence::Routine.code(), "D - ROUTINE");
    assert_eq!(PatientPrecedence::Convenience.code(), "E - CONVENIENCE");
}
