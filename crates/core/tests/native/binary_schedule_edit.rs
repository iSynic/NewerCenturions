use crate::prelude::*;

use centurions_core::story_schedule::edit;

#[test]
fn scheduled_record_edits_match_original_complete_table_and_wrapping_day() {
    let fixture = crate::support::fixture("schedule-edit.json");
    for case in fixture.array("cases") {
        if case.n::<bool>("boundary_only") {
            continue;
        }
        let mut schedule = case.bytes("before");
        let extra = case.bytes("extra");
        edit(&mut schedule, &extra, case.n::<i16>("day")).unwrap();
        assert_eq!(schedule, case.bytes("after"), "{case}");
    }
}

#[test]
fn unsigned_native_seek_boundaries_reject_without_changing_the_table() {
    let fixture = crate::support::fixture("schedule-edit.json");
    for case in fixture.array("cases") {
        if !case.n::<bool>("boundary_only") {
            continue;
        }
        let original = case.bytes("before");
        let mut schedule = original.clone();
        let extra = case.bytes("extra");
        let unsigned_offset = u32::from(word(&extra, 0).unwrap() as u16) * 40;
        assert_eq!(case["file_cursor_after"], unsigned_offset);
        assert!(edit(&mut schedule, &extra, case.n::<i16>("day"),).is_err());
        assert_eq!(schedule, original);
    }
}
