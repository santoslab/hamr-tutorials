// This file will not be overwritten if HAMR codegen is rerun

//============================================================================
//  H e a t   S o u r c e  -- Unit / GUMBOX / Property-Based Tests
//
//  The heat source consumes command messages from the heat_control event
//  data port and latches them in the GUMBO state variable heater_state.
//  The tests check:
//   - REQ_HS_1: the heater is initially Off
//   - REQ_HS_2/3: a received On/Off command is latched
//   - REQ_HS_4: with no command message, the heater state is unchanged
//
//  In the original all-DataPort variant this component had NO GUMBO
//  contracts (so no GUMBOX tests were possible); the EDP variant's
//  latch/hold contracts make all three test styles available.
//============================================================================

mod tests {
  // NOTE: need to run tests sequentially to prevent race conditions
  //       on the app and the testing apis which are static
  use serial_test::serial;

  use crate::test::util::*;
  use data::*;
  use data::Isolette_Data_Model::*;

  //-------------------------------------------
  //  Initialize Entry Point tests
  //-------------------------------------------

  #[test]
  #[serial]
  fn test_initialization_REQ_HS_1() {
    crate::heat_source_heat_source_initialize();
    assert!(test_apis::get_heater_state() == On_Off::Off,
      "REQ_HS_1: the heater must be initially Off");
  }

  //-------------------------------------------
  //  Compute Entry Point tests
  //-------------------------------------------

  // run one dispatch with the given (possibly absent) command message and
  // return the resulting heater state
  fn run_heat_source(cmd: Option<On_Off>) -> On_Off {
    test_apis::put_heat_control(cmd);
    crate::heat_source_heat_source_timeTriggered();
    test_apis::get_heater_state()
  }

  #[test]
  #[serial]
  fn test_REQ_HS_2_on_command_latched() {
    crate::heat_source_heat_source_initialize();
    assert!(run_heat_source(Some(On_Off::Onn)) == On_Off::Onn);
  }

  #[test]
  #[serial]
  fn test_REQ_HS_3_off_command_latched() {
    crate::heat_source_heat_source_initialize();
    test_apis::put_heater_state(On_Off::Onn);   // preset: heater on
    assert!(run_heat_source(Some(On_Off::Off)) == On_Off::Off);
  }

  #[test]
  #[serial]
  fn test_REQ_HS_4_no_command_holds_state() {
    crate::heat_source_heat_source_initialize();
    for pre in [On_Off::Off, On_Off::Onn] {
      test_apis::put_heater_state(pre);
      assert!(run_heat_source(None) == pre,
        "REQ_HS_4: heater state must be unchanged without a command");
    }
  }

  #[test]
  #[serial]
  fn test_command_sequence_with_send_on_change_gaps() {
    // the message pattern the send-on-change Thermostat actually produces:
    // commands arrive only on change, with quiet dispatches in between
    crate::heat_source_heat_source_initialize();

    let script: [(Option<On_Off>, On_Off); 6] = [
      (Some(On_Off::Onn), On_Off::Onn),  // turn on
      (None,              On_Off::Onn),  // quiet -> stays on
      (None,              On_Off::Onn),  // quiet -> stays on
      (Some(On_Off::Off), On_Off::Off),  // turn off
      (None,              On_Off::Off),  // quiet -> stays off
      (Some(On_Off::Onn), On_Off::Onn),  // turn on again
    ];
    for (i, (cmd, expected)) in script.iter().enumerate() {
      assert!(run_heat_source(*cmd) == *expected, "step {} failed", i);
    }
  }
}

mod GUMBOX_manual_tests {
  // Manual GUMBOX (contract-based) tests: the developer supplies the input
  // test vector; the HAMR-generated GUMBOX oracles check the latch/hold
  // contract automatically.  testComputeCBwGSV additionally sets the
  // pre-state value of the heater_state GUMBO state variable.
  use serial_test::serial;

  use crate::test::util::*;
  use data::*;
  use data::Isolette_Data_Model::*;

  #[test]
  #[serial]
  fn test_GUMBOX_initialize() {
    let result = cb_apis::testInitializeCB();
    assert!(matches!(result, cb_apis::HarnessResult::Passed));
  }

  #[test]
  #[serial]
  fn test_GUMBOX_all_state_input_combinations() {
    // (In_heater_state, api_heat_control): all pre-state x input combinations
    for pre in [On_Off::Off, On_Off::Onn] {
      for cmd in [None, Some(On_Off::Off), Some(On_Off::Onn)] {
        let result = cb_apis::testComputeCBwGSV(pre, cmd);
        assert!(matches!(result, cb_apis::HarnessResult::Passed),
          "failed for pre-state {:?}, command {:?}", pre, cmd);
      }
    }
  }
}

mod GUMBOX_tests {
  // Automated property-based GUMBOX tests: random command messages (and
  // random heater pre-states for the wGSV variant); the component has no
  // preconditions, so the default generator strategies suffice.
  use serial_test::serial;
  use proptest::prelude::*;

  use crate::test::util::*;
  use crate::testInitializeCB_macro;
  use crate::testComputeCB_macro;
  use crate::testComputeCBwGSV_macro;

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
    config: ProptestConfig {
      cases: numValidComputeTestCases,
      max_global_rejects: numValidComputeTestCases * computeRejectRatio,
      verbose: verbosity,
      ..ProptestConfig::default()
    },
    // strategies for generating each component input
    api_heat_control: generators::option_strategy_default(generators::Isolette_Data_Model_On_Off_strategy_default())
  }

  testComputeCBwGSV_macro! {
    prop_testComputeCBwGSV_macro, // test name
    config: ProptestConfig {
      cases: numValidComputeTestCases,
      max_global_rejects: numValidComputeTestCases * computeRejectRatio,
      verbose: verbosity,
      ..ProptestConfig::default()
    },
    // strategies for generating each component input
    In_heater_state: generators::Isolette_Data_Model_On_Off_strategy_default(),
    api_heat_control: generators::option_strategy_default(generators::Isolette_Data_Model_On_Off_strategy_default())
  }
}
