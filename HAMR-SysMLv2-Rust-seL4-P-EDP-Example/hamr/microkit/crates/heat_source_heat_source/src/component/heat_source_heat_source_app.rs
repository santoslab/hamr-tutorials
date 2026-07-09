// This file will not be overwritten if HAMR codegen is rerun

//============================================================================
//  H e a t   S o u r c e   -- Thread Component
//
//  This Thread component implements a (simulated) heater for the air
//  in the Isolette.  It takes as input command messages indicating the
//  desired state of the heater (on or off).
//
//  There is no software application level output for this component.
//  The component encapsulates a hardware driver that interacts with a
//  simulated hardware interface.  The (very simple!) simulation just holds
//  the current state of the heater.
//
//  EDP variant: heat_control is now an event data port.  The Thermostat
//  sends a command message only when the commanded state CHANGES
//  (send-on-change), so on most dispatches no message is present and the
//  heater simply remains in its current state.  The most recently received
//  command is latched in the GUMBO state variable `heater_state`
//  (in the original all-DataPort variant this was a hand-written,
//  non-GUMBO simulation variable).
//============================================================================

use data::*;
use data::Isolette_Data_Model::*;
use crate::bridge::heat_source_heat_source_api::*;
use vstd::prelude::*;

verus! {

  pub struct heat_source_heat_source {
    // BEGIN MARKER STATE VARS
    pub heater_state: Isolette_Data_Model::On_Off,
    // END MARKER STATE VARS
  }

  impl heat_source_heat_source {
    pub fn new() -> Self
    {
      Self {
        // BEGIN MARKER STATE VAR INIT
        heater_state: Isolette_Data_Model::On_Off::default(),
        // END MARKER STATE VAR INIT
      }
    }

    pub fn initialize<API: heat_source_heat_source_Put_Api> (
      &mut self,
      api: &mut heat_source_heat_source_Application_Api<API>)
      ensures
        // BEGIN MARKER INITIALIZATION ENSURES
        // guarantee REQ_THERM_1_HS
        //   The heater shall be Off initially.
        //   This preserves the system-level intent of REQ_THERM_1:
        //   event data ports need no initialization message, so
        //   the heat source itself guarantees the initial Off state.
        self.heater_state == Isolette_Data_Model::On_Off::Off,
        // END MARKER INITIALIZATION ENSURES
    {
      log_info("initialize entrypoint invoked");

      // REQ_HS_1 / REQ_THERM_1_HS: the heater is initially Off
      self.heater_state = On_Off::Off;
    }

    pub fn timeTriggered<API: heat_source_heat_source_Full_Api> (
      &mut self,
      api: &mut heat_source_heat_source_Application_Api<API>)
      requires
        // PLACEHOLDER MARKER TIME TRIGGERED REQUIRES
      ensures
        // BEGIN MARKER TIME TRIGGERED ENSURES
        // guarantee HS_latch_cmd
        //   A received command is latched into the heater state.
        api.heat_control.is_some() ==>
          (self.heater_state == api.heat_control.unwrap()),
        // guarantee HS_hold_cmd
        //   If no command arrives, the heater state is unchanged.
        !(api.heat_control.is_some()) ==>
          (self.heater_state == old(self).heater_state),
        // END MARKER TIME TRIGGERED ENSURES
    {
      log_info("compute entrypoint invoked");

      // get the (possibly absent) command message from the input port
      let heat_control: Option<On_Off> = api.get_heat_control();

      match heat_control {
        Some(cmd) => {
          // REQ_HS_2 / REQ_HS_3: latch the received command into the
          // heater state (update the simulation)
          self.heater_state = cmd;
          log_command_received(cmd);
        }
        None => {
          // REQ_HS_4: no command message -- the heater remains in its
          // current state
        }
      }

      // log state of the heater simulation
      log_heat_source_simulation(self.heater_state);
    }

    pub fn notify(
      &mut self,
      channel: microkit_channel)
    {
      // this method is called when the monitor does not handle the passed in channel
      match channel {
        _ => {
          log_warn_channel(channel)
        }
      }
    }
  }

  #[verifier::external_body]
  pub fn log_info(msg: &str)
  {
    log::info!("{0}", msg);
  }

  #[verifier::external_body]
  pub fn log_warn_channel(channel: u32)
  {
    log::warn!("Unexpected channel: {0}", channel);
  }

  #[verifier::external_body]
  pub fn log_command_received(cmd: On_Off)
  {
    log::info!("Command received: {:?}", cmd);
  }

  #[verifier::external_body]
  pub fn log_heat_source_simulation(state: On_Off)
  {
    log::info!("Heater State: {:?}", state);
  }

  // PLACEHOLDER MARKER GUMBO METHODS

}
