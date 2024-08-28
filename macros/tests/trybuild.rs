// These tests are currently only run on nightly.
//
// Ideally they'd be run on stable, but the foreign_mod.rs test uses a nightly feature.
#[rustversion::attr(not(nightly), ignore = "trybuild tests are only run on nightly")]
#[test]
fn trybuild() {
    trybuild::TestCases::new().compile_fail("tests/trybuild/*.rs");
}
