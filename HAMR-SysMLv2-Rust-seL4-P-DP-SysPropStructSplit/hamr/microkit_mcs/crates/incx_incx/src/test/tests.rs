// This file will not be overwritten if HAMR codegen is rerun

mod tests {
  // NOTE: need to run tests sequentially to prevent race conditions
  //       on the app and the testing apis which are static
  use serial_test::serial;

  use crate::test::util::*;

  fn run_incx(v: i32) -> i32 {
    crate::incx_incx_initialize();
    test_apis::put_inxfield(v);
    crate::incx_incx_timeTriggered();
    test_apis::get_outxfield()
  }

  // ---- Manual unit tests ----

  #[test]
  #[serial]
  fn test_initialization() {
    crate::incx_incx_initialize();
    assert_eq!(test_apis::get_outxfield(), 0);
  }

  #[test]
  #[serial]
  fn test_compute_increments() {
    assert_eq!(run_incx(5), 6);
    assert_eq!(run_incx(0), 1);
    assert_eq!(run_incx(-1), 0);
  }

  #[test]
  #[serial]
  fn test_compute_boundaries() {
    // The inxfield_bounded compute assume is [-1000, 1000]; the system
    // context only drives [-100, 100], but the component itself accepts
    // the full envelope.
    assert_eq!(run_incx(-1000), -999);
    assert_eq!(run_incx(-100), -99);
    assert_eq!(run_incx(100), 101);
    assert_eq!(run_incx(1000), 1001);
  }

  // ---- Manual GUMBOX (contract-based) test ----

  #[test]
  #[serial]
  fn test_GUMBOX_boundary_sweep() {
    for v in [-1000, -100, -1, 0, 1, 99, 100, 101, 1000] {
      let result = cb_apis::testComputeCB(v);
      assert!(matches!(result, cb_apis::HarnessResult::Passed),
        "GUMBOX failed for inxfield={}", v);
    }
  }

  #[test]
  #[serial]
  fn test_GUMBOX_rejects_out_of_range() {
    // Values outside [-1000, 1000] violate the inxfield_bounded compute
    // assume (the overflow guard) and must be rejected by the precondition.
    let result = cb_apis::testComputeCB(1001);
    assert!(matches!(result, cb_apis::HarnessResult::RejectedPrecondition));
  }
}

mod GUMBOX_tests {
  use serial_test::serial;
  use proptest::prelude::*;

  use crate::test::util::*;
  use crate::testInitializeCB_macro;
  use crate::testComputeCB_macro;

  // number of valid (i.e., non-rejected) test cases that must be executed for the compute method.
  const numValidComputeTestCases: u32 = 100;

  // how many total test cases (valid + rejected) that may be attempted.
  //   0 means all inputs must satisfy the precondition (if present),
  //   5 means at most 5 rejected inputs are allowed per valid test case
  const computeRejectRatio: u32 = 5;

  const verbosity: u32 = 2;

  testInitializeCB_macro! {
    prop_testInitializeCB_macro, // test name
    config: ProptestConfig { // proptest configuration, built by overriding fields from default config
      cases: numValidComputeTestCases,
      max_global_rejects: numValidComputeTestCases * computeRejectRatio,
      verbose: verbosity,
      ..ProptestConfig::default()
    }
  }

  testComputeCB_macro! {
    prop_testComputeCB_macro, // test name
    config: ProptestConfig { // proptest configuration, built by overriding fields from default config
      cases: numValidComputeTestCases,
      max_global_rejects: numValidComputeTestCases * computeRejectRatio,
      verbose: verbosity,
      ..ProptestConfig::default()
    },
    // strategies for generating each component input
    // range derived from GUMBO assume clause(s) constraining inxfield
    api_inxfield: (-1000i32..=1000i32)
  }
}
