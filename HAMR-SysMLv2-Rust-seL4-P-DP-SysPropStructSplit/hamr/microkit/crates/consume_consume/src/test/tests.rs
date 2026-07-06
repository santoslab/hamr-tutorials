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

  // Generate the incoming struct within the [-200, 200] assumption.
  testComputeCB_macro! {
    prop_testComputeCB_macro,
    config: ProptestConfig {
      cases: numValidComputeTestCases,
      max_global_rejects: numValidComputeTestCases * computeRejectRatio,
      verbose: verbosity,
      ..ProptestConfig::default()
    },
    api_instruct: generators::SysPropStructSplit_Data_Model_StructXY_strategy_cust(
      -200i32..=200i32,
      -200i32..=200i32
    )
  }
}
