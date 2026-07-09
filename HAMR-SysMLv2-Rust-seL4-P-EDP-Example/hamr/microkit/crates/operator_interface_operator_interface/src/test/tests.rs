// This file will not be overwritten if HAMR codegen is rerun

//============================================================================
//  O p e r a t o r   I n t e r f a c e  -- Unit / GUMBOX / Property-Based Tests
//
//  The operator interface has no input ports; the tests check its OUTPUT
//  behavior on the desired_temp event data port:
//   - REQ_OP_1: every emitted set point message is well-formed
//     (lower <= upper -- the LDT_LE_UDT integration guarantee, checked
//      explicitly in the manual tests and automatically by GUMBOX)
//   - REQ_OP_2/3: the emitted set points stay within their simulated ranges
//   - REQ_OP_4: a message is emitted only when the (simulated) operator
//     changes the set points -- every 6th activation, not on every dispatch
//
//  The long-run test also serves as the regression test for the trajectory
//  bug fixed relative to the original all-DataPort variant (the original
//  assigned 1 to upper_desired_temp instead of to its trajectory, which
//  would eventually emit an ill-formed set point message).
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

  fn set_points_well_formed(sp: Set_Points) -> bool {
    sp.lower.degrees <= sp.upper.degrees          // REQ_OP_1
      && 97 <= sp.lower.degrees && sp.lower.degrees <= 99    // REQ_OP_2
      && 99 <= sp.upper.degrees && sp.upper.degrees <= 102   // REQ_OP_3
  }

  // Clear the desired_temp output channel (plays the role of the HAMR
  // infrastructure's end-of-dispatch output release in multi-dispatch tests).
  fn clear_desired_temp() {
    *crate::bridge::extern_c_api::OUT_desired_temp.lock().unwrap_or_else(|e| e.into_inner()) = None;
  }

  //-------------------------------------------
  //  Initialize Entry Point tests
  //-------------------------------------------

  #[test]
  #[serial]
  fn test_initialization() {
    crate::operator_interface_operator_interface_initialize();

    // EDP variant: no set point message is emitted during initialization
    assert!(test_apis::get_desired_temp().is_none());
  }

  //-------------------------------------------
  //  Compute Entry Point tests
  //-------------------------------------------

  #[test]
  #[serial]
  fn test_REQ_OP_4_send_cadence_first_update() {
    crate::operator_interface_operator_interface_initialize();

    // activations 1..5: the operator has not changed the set points --
    // no message is emitted
    for i in 1..=5 {
      clear_desired_temp();
      crate::operator_interface_operator_interface_timeTriggered();
      assert!(test_apis::get_desired_temp().is_none(),
        "activation {}: no message expected while holding", i);
    }

    // activation 6: the simulated operator updates the set points --
    // a (well-formed) message is emitted: lower 98 -> 97, upper 101 -> 102
    clear_desired_temp();
    crate::operator_interface_operator_interface_timeTriggered();
    let msg = test_apis::get_desired_temp();
    assert!(msg == Some(Set_Points { lower: Temp { degrees: 97 },
                                     upper: Temp { degrees: 102 } }),
      "activation 6: expected set points [97, 102], got {:?}", msg);
  }

  #[test]
  #[serial]
  fn test_REQ_OP_1_2_3_long_run() {
    // run for several update cycles: messages are emitted exactly every 6th
    // activation, and every message is well-formed and in range
    // (this is also the regression test for the fixed trajectory bug)
    crate::operator_interface_operator_interface_initialize();

    let mut messages = 0;
    for i in 1..=60 {
      clear_desired_temp();
      crate::operator_interface_operator_interface_timeTriggered();
      match test_apis::get_desired_temp() {
        Some(sp) => {
          assert!(i % 6 == 0, "activation {}: unexpected message (REQ_OP_4)", i);
          assert!(set_points_well_formed(sp),
            "activation {}: ill-formed set points [{}, {}]",
            i, sp.lower.degrees, sp.upper.degrees);
          messages += 1;
        }
        None => {
          assert!(i % 6 != 0, "activation {}: expected a message (REQ_OP_4)", i);
        }
      }
    }
    assert!(messages == 10, "expected 10 update messages in 60 activations");
  }
}

mod GUMBOX_manual_tests {
  // Manual GUMBOX (contract-based) tests: the component has no input ports,
  // so the harness functions take no test vector; the GUMBOX oracle checks
  // the LDT_LE_UDT integration guarantee on the emitted message (vacuously
  // on non-send dispatches, substantively on send dispatches).
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
