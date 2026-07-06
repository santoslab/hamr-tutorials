// This file will not be overwritten if HAMR codegen is rerun

mod tests {
  // NOTE: need to run tests sequentially to prevent race conditions
  //       on the app and the testing apis which are static
  use serial_test::serial;

  use crate::test::util::*;

  fn run_merge(x: i32, y: i32) -> (i32, i32) {
    crate::merger_merger_initialize();
    test_apis::put_inxfield(x);
    test_apis::put_inyfield(y);
    crate::merger_merger_timeTriggered();
    let o = test_apis::get_outstruct();
    (o.x, o.y)
  }

  // ---- Manual unit tests ----

  #[test]
  #[serial]
  fn test_initialization() {
    crate::merger_merger_initialize();
    let o = test_apis::get_outstruct();
    assert_eq!(o.x, 0);
    assert_eq!(o.y, 0);
  }

  #[test]
  #[serial]
  fn test_compute_reassembles() {
    assert_eq!(run_merge(3, 9), (3, 9));
    assert_eq!(run_merge(-50, 50), (-50, 50));
  }

  #[test]
  #[serial]
  fn test_compute_extremes() {
    // Merge carries no integration constraints, so it accepts the full i32 range.
    assert_eq!(run_merge(i32::MIN, i32::MAX), (i32::MIN, i32::MAX));
  }

  // ---- Manual GUMBOX (contract-based) test ----

  #[test]
  #[serial]
  fn test_GUMBOX_samples() {
    for (x, y) in [(0, 0), (3, 9), (-101, 101), (i32::MIN, i32::MAX)] {
      let result = cb_apis::testComputeCB(x, y);
      assert!(matches!(result, cb_apis::HarnessResult::Passed),
        "GUMBOX failed for inxfield={}, inyfield={}", x, y);
    }
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

  // Merge has no input assumptions, so generate across the full i32 range.
  testComputeCB_macro! {
    prop_testComputeCB_macro,
    config: ProptestConfig {
      cases: numValidComputeTestCases,
      max_global_rejects: numValidComputeTestCases * computeRejectRatio,
      verbose: verbosity,
      ..ProptestConfig::default()
    },
    api_inxfield: any::<i32>(),
    api_inyfield: any::<i32>()
  }
}
