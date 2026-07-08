// This file will not be overwritten if HAMR codegen is rerun

mod tests {
  // NOTE: need to run tests sequentially to prevent race conditions
  //       on the app and the testing apis which are static
  use serial_test::serial;

  use crate::test::util::*;

  fn run_decy(v: i32) -> i32 {
    crate::decy_decy_initialize();
    test_apis::put_inyfield(v);
    crate::decy_decy_timeTriggered();
    test_apis::get_outyfield()
  }

  // ---- Manual unit tests ----

  #[test]
  #[serial]
  fn test_initialization() {
    crate::decy_decy_initialize();
    assert_eq!(test_apis::get_outyfield(), 0);
  }

  #[test]
  #[serial]
  fn test_compute_decrements() {
    assert_eq!(run_decy(5), 4);
    assert_eq!(run_decy(0), -1);
    assert_eq!(run_decy(1), 0);
  }

  #[test]
  #[serial]
  fn test_compute_boundaries() {
    // The inyfield_bounded compute assume is [-1000, 1000]; the system
    // context only drives [-100, 100], but the component itself accepts
    // the full envelope.
    assert_eq!(run_decy(-1000), -1001);
    assert_eq!(run_decy(-100), -101);
    assert_eq!(run_decy(100), 99);
    assert_eq!(run_decy(1000), 999);
  }

  // ---- Manual GUMBOX (contract-based) test ----

  #[test]
  #[serial]
  fn test_GUMBOX_boundary_sweep() {
    for v in [-1000, -101, -100, -1, 0, 1, 99, 100, 1000] {
      let result = cb_apis::testComputeCB(v);
      assert!(matches!(result, cb_apis::HarnessResult::Passed),
        "GUMBOX failed for inyfield={}", v);
    }
  }

  #[test]
  #[serial]
  fn test_GUMBOX_rejects_out_of_range() {
    // Values outside [-1000, 1000] violate the inyfield_bounded compute
    // assume (the underflow guard) and must be rejected by the precondition.
    let result = cb_apis::testComputeCB(-1001);
    assert!(matches!(result, cb_apis::HarnessResult::RejectedPrecondition));
  }
}

mod GUMBOX_tests {
  use serial_test::serial;
  use proptest::prelude::*;

  use crate::test::util::*;
  use crate::testInitializeCB_macro;
  use crate::testComputeCB_macro;

  const numValidComputeTestCases: u32 = 100;
  const computeRejectRatio: u32 = 5;
  const verbosity: u32 = 0;

  testInitializeCB_macro! {
    prop_testInitializeCB_macro,
    config: ProptestConfig {
      cases: numValidComputeTestCases,
      max_global_rejects: numValidComputeTestCases * computeRejectRatio,
      verbose: verbosity,
      ..ProptestConfig::default()
    }
  }

  // Generate inyfield within its [-1000, 1000] underflow-guard assumption.
  testComputeCB_macro! {
    prop_testComputeCB_macro,
    config: ProptestConfig {
      cases: numValidComputeTestCases,
      max_global_rejects: numValidComputeTestCases * computeRejectRatio,
      verbose: verbosity,
      ..ProptestConfig::default()
    },
    api_inyfield: -1000i32..=1000i32
  }
}
