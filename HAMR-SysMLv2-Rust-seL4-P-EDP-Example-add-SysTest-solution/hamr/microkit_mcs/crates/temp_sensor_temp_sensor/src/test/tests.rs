// This file will not be overwritten if HAMR codegen is rerun

//============================================================================
//  T e m p   S e n s o r  -- Unit / GUMBOX / Property-Based Tests
//
//  The temp sensor has no input ports, so its test vectors are trivial;
//  what the tests check is the component's OUTPUT behavior:
//   - REQ_TS_1: every reported temperature lies in [96, 103]
//     (this is the temp_range integration guarantee, checked explicitly in
//      the manual tests and automatically by the GUMBOX oracles)
//   - REQ_TS_2: a temp_changed event is raised
//     exactly when the reported value differs from the previous one
//============================================================================

mod tests {
  // NOTE: need to run tests sequentially to prevent race conditions
  //       on the app and the testing apis which are static
  use serial_test::serial;

  use crate::test::util::*;
  use data::*;
  use data::Isolette_Data_Model::*;

  //-------------------------------------------
  //  Helpers
  //-------------------------------------------

  fn temp_in_range(t: Temp) -> bool {
    96 <= t.degrees && t.degrees <= 103
  }

  // Clear the temp_changed output channel.  In the deployed system the HAMR
  // infrastructure releases output port state at the end of each dispatch; in
  // unit tests that invoke timeTriggered multiple times after one initialize,
  // this helper plays that role so each dispatch's output can be observed
  // independently.
  fn clear_temp_changed() {
    *crate::bridge::extern_c_api::OUT_temp_changed.lock().unwrap_or_else(|e| e.into_inner()) = None;
  }

  //-------------------------------------------
  //  Initialize Entry Point tests
  //-------------------------------------------

  #[test]
  #[serial]
  fn test_initialization() {
    crate::temp_sensor_temp_sensor_initialize();

    // the output data port must be initialized -- to the range's lower bound
    let t = test_apis::get_current_temp();
    assert!(t == Temp { degrees: 96 });

    // no change has been announced yet
    assert!(test_apis::get_temp_changed().is_none());
  }

  //-------------------------------------------
  //  Compute Entry Point tests
  //-------------------------------------------

  #[test]
  #[serial]
  fn test_compute_REQ_TS_1_and_REQ_TS_2_long_run() {
    // run the simulation for several major-frame cycles and check, on every
    // dispatch, that:
    //  - the reported temperature is in [96, 103]           (REQ_TS_1)
    //  - a temp_changed event is present exactly when the reported value
    //    differs from the previously reported value          (REQ_TS_2)
    crate::temp_sensor_temp_sensor_initialize();
    let mut prev = test_apis::get_current_temp();

    let mut changes = 0;
    let mut holds = 0;
    for i in 0..40 {
      clear_temp_changed();
      crate::temp_sensor_temp_sensor_timeTriggered();

      let reported = test_apis::get_current_temp();
      let event = test_apis::get_temp_changed();

      assert!(temp_in_range(reported), "dispatch {}: {} out of range", i, reported.degrees);

      if reported == prev {
        assert!(event.is_none(),
          "dispatch {}: no value change but temp_changed event present", i);
        holds += 1;
      } else {
        assert!(event.is_some(),
          "dispatch {}: value changed but event absent", i);
        changes += 1;
      }
      prev = reported;
    }

    // the simulation holds the value for 2 dispatches, then changes it
    assert!(changes > 0, "simulation never changed the temperature");
    assert!(holds > 0, "simulation never held the temperature");
  }

  #[test]
  #[serial]
  fn test_compute_hold_then_change_cadence() {
    // with activations_between_changes = 2, the first two dispatches hold the
    // initialized value (96) and the third reports 97 with an event
    crate::temp_sensor_temp_sensor_initialize();

    for expected_hold in 0..2 {
      clear_temp_changed();
      crate::temp_sensor_temp_sensor_timeTriggered();
      assert!(test_apis::get_current_temp() == Temp { degrees: 96 },
        "dispatch {} should hold 96", expected_hold);
      assert!(test_apis::get_temp_changed().is_none());
    }

    clear_temp_changed();
    crate::temp_sensor_temp_sensor_timeTriggered();
    assert!(test_apis::get_current_temp() == Temp { degrees: 97 });
    assert!(test_apis::get_temp_changed().is_some());
  }
}

mod GUMBOX_manual_tests {
  // Manual GUMBOX (contract-based) tests: the component has no input ports,
  // so the harness functions take no test vector; the GUMBOX oracles check
  // the integration guarantee (temp_range) on the
  // component's outputs.
  use serial_test::serial;

  use crate::test::util::*;

  #[test]
  #[serial]
  fn test_GUMBOX_initialize() {
    let result = cb_apis::testInitializeCB();
    assert!(matches!(result, cb_apis::HarnessResult::Passed));
  }

  #[test]
  #[serial]
  fn test_GUMBOX_compute() {
    let result = cb_apis::testComputeCB();
    assert!(matches!(result, cb_apis::HarnessResult::Passed));
  }
}

mod GUMBOX_tests {
  // Automated property-based GUMBOX tests.  With no input ports there is
  // nothing to randomize; the value of the macros here is repeated execution
  // with automatic contract checking on each dispatch.
  use serial_test::serial;
  use proptest::prelude::*;

  use crate::test::util::*;
  use crate::testInitializeCB_macro;
  use crate::testComputeCB_macro;

  // number of valid (i.e., non-rejected) test cases that must be executed for the compute method.
  const numValidComputeTestCases: u32 = 100;

  // how many total test cases (valid + rejected) that may be attempted.
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
