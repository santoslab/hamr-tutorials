// This file will not be overwritten if HAMR codegen is rerun

mod tests {
  // NOTE: need to run tests sequentially to prevent race conditions
  //       on the app and the testing apis which are static
  use serial_test::serial;

  use crate::test::util::*;

  fn run_clampx(v: i32) -> i32 {
    crate::clampx_clampx_initialize();
    test_apis::put_inxfield(v);
    crate::clampx_clampx_timeTriggered();
    test_apis::get_outxfield()
  }

  // ---- Manual unit tests ----

  #[test]
  #[serial]
  fn test_initialization() {
    crate::clampx_clampx_initialize();
    assert_eq!(test_apis::get_outxfield(), 0);
  }

  #[test]
  #[serial]
  fn test_compute_passes_in_range() {
    assert_eq!(run_clampx(0), 0);
    assert_eq!(run_clampx(-99), -99);
    assert_eq!(run_clampx(100), 100);
  }

  #[test]
  #[serial]
  fn test_compute_saturates() {
    // The component carries no assumptions at all: the compute_cases are
    // total, so any i32 input -- including the type extremes -- is valid.
    assert_eq!(run_clampx(101), 100);
    assert_eq!(run_clampx(102), 100);
    assert_eq!(run_clampx(i32::MAX), 100);
    assert_eq!(run_clampx(-101), -100);
    assert_eq!(run_clampx(i32::MIN), -100);
  }

  // ---- Manual GUMBOX (contract-based) tests ----

  #[test]
  #[serial]
  fn test_GUMBOX_boundary_sweep() {
    // No precondition: every i32 input must satisfy the compute_cases.
    for v in [i32::MIN, -102, -101, -100, -99, -1, 0, 1, 99, 100, 101, 102, i32::MAX] {
      let result = cb_apis::testComputeCB(v);
      assert!(matches!(result, cb_apis::HarnessResult::Passed),
        "GUMBOX failed for inxfield={}", v);
    }
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
    api_inxfield: generators::i32_strategy_default()
  }
}
