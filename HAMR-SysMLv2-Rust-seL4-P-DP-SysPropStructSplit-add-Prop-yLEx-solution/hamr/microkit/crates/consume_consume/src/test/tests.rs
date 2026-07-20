// This file will not be overwritten if HAMR codegen is rerun

mod tests {
  // NOTE: need to run tests sequentially to prevent race conditions
  //       on the app and the testing apis which are static
  use serial_test::serial;

  use crate::test::util::*;
  use data::*;

  // ---- Manual unit tests ----

  #[test]
  #[serial]
  fn test_initialization() {
    crate::consume_consume_initialize();
  }

  #[test]
  #[serial]
  fn test_compute_consumes_without_panicking() {
    // Consume is a sink: it reads and logs the struct and produces no output.
    crate::consume_consume_initialize();
    test_apis::put_instruct(SysPropStructSplit_Data_Model::StructXY { x: 10, y: 20 });
    crate::consume_consume_timeTriggered();
  }

  // ---- Manual GUMBOX (contract-based) test ----

  #[test]
  #[serial]
  fn test_GUMBOX_boundary_sweep() {
    // instruct_range assumption is [-200, 200].
    for v in [-200, -1, 0, 1, 200] {
      let input = SysPropStructSplit_Data_Model::StructXY { x: v, y: -v };
      let result = cb_apis::testComputeCB(input);
      assert!(matches!(result, cb_apis::HarnessResult::Passed),
        "GUMBOX failed for v={}", v);
    }
  }

  #[test]
  #[serial]
  fn test_GUMBOX_rejects_out_of_range() {
    let input = SysPropStructSplit_Data_Model::StructXY { x: 201, y: 0 };
    let result = cb_apis::testComputeCB(input);
    assert!(matches!(result, cb_apis::HarnessResult::RejectedPrecondition));
  }
}

mod GUMBOX_tests {
  use serial_test::serial;
  use proptest::prelude::*;

  use crate::test::util::*;
  use crate::testInitializeCB_macro;
  use crate::testComputeCB_macro;
    use crate::testComputeCBwGSV_macro;

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
      (-200i32..=200i32),
      (-200i32..=200i32))
  }

  testComputeCBwGSV_macro! {
    prop_testComputeCBwGSV_macro, // test name
    config: ProptestConfig { // proptest configuration, built by overriding fields from default config
      cases: numValidComputeTestCases,
      max_global_rejects: numValidComputeTestCases * computeRejectRatio,
      verbose: verbosity,
      ..ProptestConfig::default()
    },
    // strategies for generating each component input
    // TODO: full-range default; if last_x is constrained by a GUMBO assume clause then
    //       narrow this strategy (e.g. generators::i32_strategy_cust(loi32..=hii32))
    //       to avoid exhausting the proptest rejection budget
    In_last_x: generators::i32_strategy_default(),
    // field range(s) derived from GUMBO assume clause(s) constraining instruct
    api_instruct: generators::SysPropStructSplit_Data_Model_StructXY_strategy_cust(
      (-200i32..=200i32),
      (-200i32..=200i32))
  }
}
