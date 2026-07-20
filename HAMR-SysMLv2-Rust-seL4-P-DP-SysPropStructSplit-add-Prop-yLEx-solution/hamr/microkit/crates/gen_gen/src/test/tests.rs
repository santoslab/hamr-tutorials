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
    crate::gen_gen_initialize();
    // The init_outstruct guarantee requires the output port to start at (0, 0).
    let o = test_apis::get_outstruct();
    assert_eq!(o.x, 0);
    assert_eq!(o.y, 0);
  }

  #[test]
  #[serial]
  fn test_compute_in_range_and_increments() {
    // NOTE: the component instance is a persistent static, so the absolute
    // counter value is not predictable across tests.  Instead we assert the
    // outstruct_range guarantee and the relative sawtooth behavior.
    crate::gen_gen_initialize();

    crate::gen_gen_timeTriggered();
    let a = test_apis::get_outstruct();

    crate::gen_gen_timeTriggered();
    let b = test_apis::get_outstruct();

    // Both fields stay within the guaranteed range.
    assert!(-100 <= a.x && a.x <= 100, "x out of range: {}", a.x);
    assert!(-100 <= a.y && a.y <= 100, "y out of range: {}", a.y);
    // Gen produces equal x and y.
    assert_eq!(a.x, a.y);
    // Successive ticks advance the sawtooth by +1, wrapping at the top.
    let expected = if a.x < 100 { a.x + 1 } else { -100 };
    assert_eq!(b.x, expected);
    assert_eq!(b.y, expected);
  }

  // ---- Manual GUMBOX (contract-based) test ----

  #[test]
  #[serial]
  fn test_GUMBOX_compute() {
    // The harness runs the compute entry point and checks the GUMBO
    // output contract (outstruct_range) automatically.
    let result = cb_apis::testComputeCB();
    assert!(matches!(result, cb_apis::HarnessResult::Passed));
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
    }
  }
}
