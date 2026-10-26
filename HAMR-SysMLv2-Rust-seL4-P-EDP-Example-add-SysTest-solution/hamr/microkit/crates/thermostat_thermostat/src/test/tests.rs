// This file will not be overwritten if HAMR codegen is rerun

//============================================================================
//  T h e r m o s t a t  -- Unit / GUMBOX / Property-Based Tests
//
//  Test styles illustrated (see the HAMR testing guide):
//   1. Manual unit tests -- explicit inputs, explicit expected-result checks.
//      Adapted from the original all-DataPort variant's REQ_THERM_2/3/4 tests:
//      each test now supplies a triggering event (temp_changed and/or
//      desired_temp) and the heat_control output is an Option (send-on-change).
//   2. Manual GUMBOX tests -- explicit inputs, expected results checked
//      automatically against the GUMBO contract (compute_CEP_Pre/Post).
//   3. Automated property-based GUMBOX tests -- random inputs from custom
//      PropTest strategies, checked against the GUMBO contract.
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

  fn sp(lower: i32, upper: i32) -> Set_Points {
    Set_Points { lower: Temp { degrees: lower }, upper: Temp { degrees: upper } }
  }

  // Clear the heat_control output channel.  In the deployed system the HAMR
  // infrastructure releases output port state at the end of each dispatch; in
  // unit tests that invoke timeTriggered multiple times after one initialize,
  // this helper plays that role so each dispatch's output can be observed
  // independently.
  fn clear_heat_control() {
    *crate::bridge::extern_c_api::OUT_heat_control.lock().unwrap_or_else(|e| e.into_inner()) = None;
  }

  // Run one dispatch of the thermostat: initialize, preset the lastCmd
  // pre-state, set the input ports, invoke the compute entry point, and
  // return the (possibly absent) heat_control output message.
  fn run_thermostat(
    in_lastCmd: On_Off,
    current_temp: i32,
    temp_changed: bool,
    desired_temp: Option<Set_Points>) -> Option<On_Off>
  {
    crate::thermostat_thermostat_initialize();
    test_apis::put_lastCmd(in_lastCmd);
    test_apis::put_current_temp(Temp { degrees: current_temp });
    test_apis::put_temp_changed(if temp_changed { Some(0u8) } else { None });
    test_apis::put_desired_temp(desired_temp);
    crate::thermostat_thermostat_timeTriggered();
    test_apis::get_heat_control()
  }

  //-------------------------------------------
  //  Initialize Entry Point tests
  //-------------------------------------------

  #[test]
  #[serial]
  fn test_initialization() {
    crate::thermostat_thermostat_initialize();

    // REQ_THERM_1 (adapted): the commanded heat state is initially Off
    assert!(test_apis::get_lastCmd() == On_Off::Off);

    // REQ_THERM_LATCH: the latched set points start at the default range
    assert!(test_apis::get_currentSetPoints() == sp(98, 101));

    // no command message is emitted during initialization
    // (event data ports need no initialization)
    assert!(test_apis::get_heat_control().is_none());
  }

  //-------------------------------------------
  //  Compute Entry Point tests
  //-------------------------------------------

  #[test]
  #[serial]
  fn test_REQ_THERM_2_low_temp_turns_heat_on() {
    // temp 96 < default lower set point 98, triggered by a temp_changed event
    let output = run_thermostat(On_Off::Off, 96, true, None);
    assert!(output == Some(On_Off::Onn), "expected an On command message");
    assert!(test_apis::get_lastCmd() == On_Off::Onn);
  }

  #[test]
  #[serial]
  fn test_REQ_THERM_3_high_temp_turns_heat_off() {
    // temp 103 > default upper set point 101; heater currently commanded On,
    // so an Off command message is sent
    let output = run_thermostat(On_Off::Onn, 103, true, None);
    assert!(output == Some(On_Off::Off), "expected an Off command message");
    assert!(test_apis::get_lastCmd() == On_Off::Off);
  }

  #[test]
  #[serial]
  fn test_REQ_THERM_3_heater_already_off_no_send() {
    // temp 103 > upper, but the commanded state is already Off: the control
    // law selects Off and the send-on-change policy suppresses the message
    let output = run_thermostat(On_Off::Off, 103, true, None);
    assert!(output.is_none(), "no message expected when the command is unchanged");
    assert!(test_apis::get_lastCmd() == On_Off::Off);
  }

  #[test]
  #[serial]
  fn test_REQ_THERM_4_in_range_no_change_no_send() {
    // temp 100 within the default range [98, 101]: the commanded state is
    // unchanged and no message is sent -- for both possible pre-states
    for pre in [On_Off::Off, On_Off::Onn] {
      let output = run_thermostat(pre, 100, true, None);
      assert!(output.is_none(), "no message expected for in-range temperature");
      assert!(test_apis::get_lastCmd() == pre, "commanded state must be unchanged");
    }
  }

  #[test]
  #[serial]
  fn test_REQ_THERM_TRIGGER_no_events_no_action() {
    // no triggering event: the control logic does not run, even though the
    // current temperature is outside the desired range
    let output = run_thermostat(On_Off::Off, 96, false, None);
    assert!(output.is_none(), "no trigger => no message");
    assert!(test_apis::get_lastCmd() == On_Off::Off, "no trigger => command unchanged");
    assert!(test_apis::get_currentSetPoints() == sp(98, 101), "no message => latch unchanged");
  }

  #[test]
  #[serial]
  fn test_REQ_THERM_LATCH_set_point_message_latched_and_triggers() {
    // a set point message alone is a trigger: with the new range [99, 100]
    // and temp 98 < 99 the heat is commanded On, and the message is latched
    let output = run_thermostat(On_Off::Off, 98, false, Some(sp(99, 100)));
    assert!(test_apis::get_currentSetPoints() == sp(99, 100), "set points should be latched");
    assert!(output == Some(On_Off::Onn));
    assert!(test_apis::get_lastCmd() == On_Off::Onn);
  }

  #[test]
  #[serial]
  fn test_REQ_THERM_SOC_send_on_change_two_dispatches() {
    // dispatch 1: temperature change to 96 -> On command message sent
    crate::thermostat_thermostat_initialize();
    test_apis::put_current_temp(Temp { degrees: 96 });
    test_apis::put_temp_changed(Some(0u8));
    test_apis::put_desired_temp(None);
    crate::thermostat_thermostat_timeTriggered();
    assert!(test_apis::get_heat_control() == Some(On_Off::Onn));

    // dispatch 2: identical conditions -> command unchanged -> NO message
    clear_heat_control();
    crate::thermostat_thermostat_timeTriggered();
    assert!(test_apis::get_heat_control().is_none(),
      "an unchanged command must not be re-sent (send-on-change)");
    assert!(test_apis::get_lastCmd() == On_Off::Onn);
  }
}

mod GUMBOX_manual_tests {
  // Manual GUMBOX (contract-based) tests: the developer supplies the input
  // test vector; the expected-result checking is performed automatically by
  // the HAMR-generated GUMBOX oracles (executable versions of the GUMBO
  // contract).  testComputeCBwGSV additionally lets the test set the
  // pre-state values of the GUMBO state variables.
  use serial_test::serial;

  use crate::test::util::*;
  use data::*;
  use data::Isolette_Data_Model::*;

  fn sp(lower: i32, upper: i32) -> Set_Points {
    Set_Points { lower: Temp { degrees: lower }, upper: Temp { degrees: upper } }
  }

  #[test]
  #[serial]
  fn test_GUMBOX_initialize() {
    let result = cb_apis::testInitializeCB();
    assert!(matches!(result, cb_apis::HarnessResult::Passed));
  }

  #[test]
  #[serial]
  fn test_GUMBOX_REQ_THERM_2_low_temp() {
    // (In_currentSetPoints, In_lastCmd, api_temp_changed, api_desired_temp, api_current_temp)
    let result = cb_apis::testComputeCBwGSV(
      sp(98, 101), On_Off::Off, Some(0u8), None, Temp { degrees: 96 });
    assert!(matches!(result, cb_apis::HarnessResult::Passed));
  }

  #[test]
  #[serial]
  fn test_GUMBOX_REQ_THERM_3_high_temp_both_prestates() {
    for pre in [On_Off::Off, On_Off::Onn] {
      let result = cb_apis::testComputeCBwGSV(
        sp(98, 101), pre, Some(0u8), None, Temp { degrees: 103 });
      assert!(matches!(result, cb_apis::HarnessResult::Passed),
        "failed for pre-state {:?}", pre);
    }
  }

  #[test]
  #[serial]
  fn test_GUMBOX_REQ_THERM_4_in_range_both_prestates() {
    for pre in [On_Off::Off, On_Off::Onn] {
      let result = cb_apis::testComputeCBwGSV(
        sp(98, 101), pre, Some(0u8), None, Temp { degrees: 100 });
      assert!(matches!(result, cb_apis::HarnessResult::Passed),
        "failed for pre-state {:?}", pre);
    }
  }

  #[test]
  #[serial]
  fn test_GUMBOX_no_trigger() {
    let result = cb_apis::testComputeCBwGSV(
      sp(98, 101), On_Off::Onn, None, None, Temp { degrees: 96 });
    assert!(matches!(result, cb_apis::HarnessResult::Passed));
  }

  #[test]
  #[serial]
  fn test_GUMBOX_set_point_latching() {
    // new set points arrive (and act as the trigger)
    let result = cb_apis::testComputeCBwGSV(
      sp(98, 101), On_Off::Off, None, Some(sp(99, 100)), Temp { degrees: 98 });
    assert!(matches!(result, cb_apis::HarnessResult::Passed));
  }

  #[test]
  #[serial]
  fn test_GUMBOX_boundary_temp_sweep() {
    // sweep the sensed temperature across the boundaries of the default
    // latched range [98, 101] (and the sensor range [95, 104]), for both
    // command pre-states
    for pre in [On_Off::Off, On_Off::Onn] {
      for t in [95, 96, 97, 98, 99, 100, 101, 102, 103, 104] {
        let result = cb_apis::testComputeCBwGSV(
          sp(98, 101), pre, Some(0u8), None, Temp { degrees: t });
        assert!(matches!(result, cb_apis::HarnessResult::Passed),
          "failed for pre-state {:?}, temp {}", pre, t);
      }
    }
  }

  #[test]
  #[serial]
  fn test_GUMBOX_rejected_ill_formed_latched_set_points() {
    // the pre-state violates the INV_CSP compute assume (lower > upper),
    // so the harness rejects the vector instead of running the entry point
    let result = cb_apis::testComputeCBwGSV(
      sp(101, 98), On_Off::Off, Some(0u8), None, Temp { degrees: 96 });
    assert!(matches!(result, cb_apis::HarnessResult::RejectedPrecondition));
  }

  #[test]
  #[serial]
  fn test_GUMBOX_rejected_out_of_range_temp() {
    // the sensed temperature violates the ASSM_CT_Range integration assume
    let result = cb_apis::testComputeCB(None, None, Temp { degrees: 200 });
    assert!(matches!(result, cb_apis::HarnessResult::RejectedPrecondition));
  }

  #[test]
  #[serial]
  fn test_GUMBOX_rejected_ill_formed_set_point_message() {
    // the incoming set point message violates the ASSM_LDT_LE_UDT
    // integration assume (lower > upper)
    let result = cb_apis::testComputeCB(None, Some(sp(102, 97)), Temp { degrees: 100 });
    assert!(matches!(result, cb_apis::HarnessResult::RejectedPrecondition));
  }
}

mod GUMBOX_tests {
  // Automated property-based GUMBOX tests: PropTest generates random input
  // vectors from the strategies below; the GUMBOX oracles check the GUMBO
  // contract on every vector.  Vectors that violate the preconditions
  // (integration assumes, INV_CSP) are rejected -- the custom strategies
  // below are constrained so that rejections are rare by construction.
  use serial_test::serial;
  use proptest::prelude::*;

  use crate::test::util::*;
  use crate::testInitializeCB_macro;
  use crate::testComputeCB_macro;
  use crate::testComputeCBwGSV_macro;
  use data::*;
  use data::Isolette_Data_Model::*;

  // number of valid (i.e., non-rejected) test cases that must be executed for the compute method.
  const numValidComputeTestCases: u32 = 100;

  // how many total test cases (valid + rejected) that may be attempted.
  //   0 means all inputs must satisfy the precondition (if present),
  //   5 means at most 5 rejected inputs are allowed per valid test case
  const computeRejectRatio: u32 = 5;

  const verbosity: u32 = 2;

  //-------------------------------------------
  //  Custom strategies
  //-------------------------------------------

  // Correlated strategy for WELL-FORMED set points (lower <= upper): the
  // generated Set_Points_strategy_cust takes independent strategies for the
  // two fields and so cannot enforce the cross-field constraint -- generating
  // (lower, delta) pairs instead makes precondition rejections impossible.
  fn well_formed_set_points_strategy() -> impl Strategy<Value = Set_Points> {
    (95i32..=104i32, 0i32..=9i32).prop_map(|(lo, d)| Set_Points {
      lower: Temp { degrees: lo },
      upper: Temp { degrees: if lo + d > 104 { 104 } else { lo + d } },
    })
  }

  // Sensed temperatures within the ASSM_CT_Range integration assume [95, 104]
  fn sensed_temp_strategy() -> impl Strategy<Value = Temp> {
    generators::Isolette_Data_Model_Temp_strategy_cust(95i32..=104i32)
  }

  //-------------------------------------------
  //  Initialize entry point
  //-------------------------------------------

  testInitializeCB_macro! {
    prop_testInitializeCB_macro, // test name
    config: ProptestConfig { // proptest configuration, built by overriding fields from default config
      cases: numValidComputeTestCases,
      max_global_rejects: numValidComputeTestCases * computeRejectRatio,
      verbose: verbosity,
      ..ProptestConfig::default()
    }
  }

  //-------------------------------------------
  //  Compute entry point -- port inputs only
  //  (the pre-state is the just-initialized component:
  //   latched set points = default [98, 101], lastCmd = Off)
  //-------------------------------------------

  testComputeCB_macro! {
    prop_testComputeCB_macro, // test name
    config: ProptestConfig {
      cases: numValidComputeTestCases,
      max_global_rejects: numValidComputeTestCases * computeRejectRatio,
      verbose: verbosity,
      ..ProptestConfig::default()
    },
    // strategies for generating each component input
    api_temp_changed: generators::option_strategy_default(Just(0u8)),
    api_desired_temp: generators::option_strategy_bias(3, well_formed_set_points_strategy()),
    api_current_temp: sensed_temp_strategy()
  }

  //-------------------------------------------
  //  Compute entry point -- port inputs AND GUMBO state variables
  //-------------------------------------------

  testComputeCBwGSV_macro! {
    prop_testComputeCBwGSV_macro, // test name
    config: ProptestConfig {
      cases: numValidComputeTestCases,
      max_global_rejects: numValidComputeTestCases * computeRejectRatio,
      verbose: verbosity,
      ..ProptestConfig::default()
    },
    // strategies for generating each component input
    In_currentSetPoints: well_formed_set_points_strategy(),
    In_lastCmd: generators::Isolette_Data_Model_On_Off_strategy_default(),
    api_temp_changed: generators::option_strategy_default(Just(0u8)),
    api_desired_temp: generators::option_strategy_bias(3, well_formed_set_points_strategy()),
    api_current_temp: sensed_temp_strategy()
  }

  //-------------------------------------------
  //  Boundary-biased variant: hammer the control-law thresholds around the
  //  default latched range [98, 101]
  //-------------------------------------------

  testComputeCBwGSV_macro! {
    prop_testComputeCBwGSV_boundary_biased, // test name
    config: ProptestConfig {
      cases: numValidComputeTestCases,
      max_global_rejects: numValidComputeTestCases * computeRejectRatio,
      verbose: verbosity,
      ..ProptestConfig::default()
    },
    In_currentSetPoints: Just(Set_Points { lower: Temp { degrees: 98 }, upper: Temp { degrees: 101 } }),
    In_lastCmd: generators::Isolette_Data_Model_On_Off_strategy_default(),
    api_temp_changed: generators::option_strategy_bias(3, Just(0u8)),
    api_desired_temp: Just(None::<Set_Points>),
    api_current_temp: prop_oneof![
      Just(Temp { degrees: 97 }),   // just below lower
      Just(Temp { degrees: 98 }),   // at lower
      Just(Temp { degrees: 100 }),  // in range
      Just(Temp { degrees: 101 }),  // at upper
      Just(Temp { degrees: 102 })   // just above upper
    ]
  }

  //-------------------------------------------
  //  No-event-biased variant: hammer the no-trigger frame condition
  //  (noTriggerNoChange / noSendNoChange)
  //-------------------------------------------

  testComputeCBwGSV_macro! {
    prop_testComputeCBwGSV_no_event_biased, // test name
    config: ProptestConfig {
      cases: numValidComputeTestCases,
      max_global_rejects: numValidComputeTestCases * computeRejectRatio,
      verbose: verbosity,
      ..ProptestConfig::default()
    },
    In_currentSetPoints: well_formed_set_points_strategy(),
    In_lastCmd: generators::Isolette_Data_Model_On_Off_strategy_default(),
    api_temp_changed: Just(None::<u8>),
    api_desired_temp: Just(None::<Set_Points>),
    api_current_temp: sensed_temp_strategy()
  }
}
