// This file will not be overwritten if HAMR codegen is rerun

mod tests {
  // NOTE: need to run tests sequentially to prevent race conditions
  //       on the app and the testing apis which are static
  use serial_test::serial;

  use crate::test::util::*;
  use data::*;

  fn run_split(x: i32, y: i32) -> (i32, i32) {
    crate::splitter_splitter_initialize();
    test_apis::put_instruct(SysPropStructSplit_Data_Model::StructXY { x, y });
    crate::splitter_splitter_timeTriggered();
    (test_apis::get_xfield(), test_apis::get_yfield())
  }

  // ---- Manual unit tests ----

  #[test]
  #[serial]
  fn test_initialization() {
    crate::splitter_splitter_initialize();
    // Output data ports are initialized to 0.
    assert_eq!(test_apis::get_xfield(), 0);
    assert_eq!(test_apis::get_yfield(), 0);
  }

  #[test]
  #[serial]
  fn test_compute_forwards_fields() {
    let (x, y) = run_split(7, -3);
    assert_eq!(x, 7);
    assert_eq!(y, -3);
  }

  #[test]
  #[serial]
  fn test_compute_boundaries() {
    assert_eq!(run_split(-100, 100), (-100, 100));
    assert_eq!(run_split(100, -100), (100, -100));
  }

  // ---- Manual GUMBOX (contract-based) test ----

  #[test]
  #[serial]
  fn test_GUMBOX_boundary_sweep() {
    let values = [-100, -1, 0, 1, 100];
    for &x in &values {
      for &y in &values {
        let input = SysPropStructSplit_Data_Model::StructXY { x, y };
        let result = cb_apis::testComputeCB(input);
        assert!(matches!(result, cb_apis::HarnessResult::Passed),
          "GUMBOX failed for x={}, y={}", x, y);
      }
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
    // field range(s) derived from GUMBO assume clause(s) constraining instruct
    api_instruct: generators::SysPropStructSplit_Data_Model_StructXY_strategy_cust(
      (-100i32..=100i32),
      (-100i32..=100i32))
  }
}
