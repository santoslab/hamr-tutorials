// This file will not be overwritten if HAMR codegen is rerun

//============================================================================
//  T h e r m o s t a t  -- Thread Component
//
//  This Thread component implements control logic that
//  turns a heat source on/off based on the values of the following inputs...
//    - current temperature (sampled data port)
//    - temp_changed event announcing that the sensed value changed
//    - set points provided by an operator as event data messages
//        (these specify the lower and upper bounds (inclusive) for the
//        desired temperature range)
//
//  EDP variant behavior (contrast with the original all-DataPort variant):
//    - because event data port inputs do not persist between dispatches, the
//      most recently received set points are LATCHED in the GUMBO state
//      variable `currentSetPoints` (initialized to the default range [98,101])
//    - the control logic runs only when a TRIGGERING event is present: a
//      temp_changed event or a new set point message.  Without a trigger the
//      commanded state (and the latched set points) are unchanged
//    - the heat_control output is SEND-ON-CHANGE: a command message is
//      emitted exactly when the commanded state changes (the `lastCmd`
//      GUMBO state variable tracks the commanded state between dispatches)
//============================================================================

use data::*;
use data::Isolette_Data_Model::*; // Add for easier reference of data types
use crate::bridge::thermostat_thermostat_api::*;
use vstd::prelude::*;

verus! {

  pub struct thermostat_thermostat {
    // BEGIN MARKER STATE VARS
    pub currentSetPoints: Isolette_Data_Model::Set_Points,
    pub lastCmd: Isolette_Data_Model::On_Off,
    // END MARKER STATE VARS
  }

  impl thermostat_thermostat {
    pub fn new() -> Self
    {
      Self {
        // BEGIN MARKER STATE VAR INIT
        currentSetPoints: Isolette_Data_Model::Set_Points::default(),
        lastCmd: Isolette_Data_Model::On_Off::default(),
        // END MARKER STATE VAR INIT
      }
    }

    pub fn initialize<API: thermostat_thermostat_Put_Api> (
      &mut self,
      api: &mut thermostat_thermostat_Application_Api<API>)
      ensures
        // BEGIN MARKER INITIALIZATION ENSURES
        // guarantee initCurrentSetPoints
        //   The latched set points start at the default range.
        (self.currentSetPoints.lower.degrees == Default_Lower_Set_Point()) &&
          (self.currentSetPoints.upper.degrees == Default_Upper_Set_Point()),
        // guarantee REQ_THERM_1
        //   The commanded heat state shall be Off initially.
        //   Adapted from the original REQ_THERM_1: no message is
        //   sent during initialization (event data ports need no
        //   initialization), and the Heat Source independently
        //   initializes its heater state to Off.
        self.lastCmd == Isolette_Data_Model::On_Off::Off,
        // END MARKER INITIALIZATION ENSURES
    {
      log_info("initialize entrypoint invoked");

      // the latched set points start at the default desired range
      // (literals match the model-level GUMBO functions
      //  Default_Lower_Set_Point() / Default_Upper_Set_Point())
      self.currentSetPoints =
        Set_Points { lower: Temp { degrees: 98 },
                     upper: Temp { degrees: 101 } };

      // REQ_THERM_1 (adapted): the commanded heat state is initially Off.
      // NO message is placed on the heat_control event data port -- event
      // data ports need no initialization, and the Heat Source independently
      // initializes its heater state to Off.
      self.lastCmd = On_Off::Off;
    }

    pub fn timeTriggered<API: thermostat_thermostat_Full_Api> (
      &mut self,
      api: &mut thermostat_thermostat_Application_Api<API>)
      requires
        // BEGIN MARKER TIME TRIGGERED REQUIRES
        // assume AADL_Requirement
        //   All outgoing event ports must be empty
        old(api).heat_control.is_none(),
        // assume INV_CSP
        //   The latched set points are well-formed.
        (old(self).currentSetPoints).lower.degrees <= (old(self).currentSetPoints).upper.degrees,
        // END MARKER TIME TRIGGERED REQUIRES
      ensures
        // BEGIN MARKER TIME TRIGGERED ENSURES
        // guarantee latchSetPointsOnEvent
        //   A newly received set point message is latched.
        api.desired_temp.is_some() ==>
          (self.currentSetPoints.lower.degrees == api.desired_temp.unwrap().lower.degrees) &&
            (self.currentSetPoints.upper.degrees == api.desired_temp.unwrap().upper.degrees),
        // guarantee latchSetPointsNoEvent
        //   Otherwise the latched set points are unchanged.
        !(api.desired_temp.is_some()) ==>
          (self.currentSetPoints == old(self).currentSetPoints),
        // guarantee invCSPMaintained
        //   Well-formedness of the latched set points is
        //   re-established for the next dispatch.
        self.currentSetPoints.lower.degrees <= self.currentSetPoints.upper.degrees,
        // guarantee noTriggerNoChange
        //   Without a triggering event (temperature change or
        //   new set points) the commanded state is unchanged.
        !(api.temp_changed.is_some() || api.desired_temp.is_some()) ==>
          (self.lastCmd == old(self).lastCmd),
        // guarantee REQ_THERM_2
        //   If triggered and the Current Temperature is less than
        //   the Lower Desired Temperature, the commanded heat state
        //   shall be set to On.
        (api.temp_changed.is_some() || api.desired_temp.is_some()) &&
          (api.current_temp.degrees < self.currentSetPoints.lower.degrees) ==>
          (self.lastCmd == Isolette_Data_Model::On_Off::Onn),
        // guarantee REQ_THERM_3
        //   If triggered and the Current Temperature is greater than
        //   the Upper Desired Temperature, the commanded heat state
        //   shall be set to Off.
        (api.temp_changed.is_some() || api.desired_temp.is_some()) &&
          (api.current_temp.degrees > self.currentSetPoints.upper.degrees) ==>
          (self.lastCmd == Isolette_Data_Model::On_Off::Off),
        // guarantee REQ_THERM_4
        //   If triggered and the Current Temperature is greater than
        //   or equal to the Lower Desired Temperature and less than
        //   or equal to the Upper Desired Temperature, the commanded
        //   heat state shall not be changed.
        (api.temp_changed.is_some() || api.desired_temp.is_some()) &&
          (api.current_temp.degrees >= self.currentSetPoints.lower.degrees) &&
          (api.current_temp.degrees <= self.currentSetPoints.upper.degrees) ==>
          (self.lastCmd == old(self).lastCmd),
        // guarantee mustSendOnChange
        //   A command message is sent exactly when the commanded
        //   state changes, and it carries the new state.
        (self.lastCmd != old(self).lastCmd) ==>
          api.heat_control.is_some() &&
            (api.heat_control.unwrap() == self.lastCmd),
        // guarantee noSendNoChange
        //   No message is sent when the commanded state is unchanged.
        (self.lastCmd == old(self).lastCmd) ==>
          api.heat_control.is_none(),
        // END MARKER TIME TRIGGERED ENSURES
    {
      log_info("compute entrypoint invoked");

      // -------------- Get values of input ports ------------------
      let temp_changed: bool = api.get_temp_changed();
      let desired_temp: Option<Set_Points> = api.get_desired_temp();
      let current_temp: Temp = api.get_current_temp();

      // a triggering event: the sensed temperature changed, or new set
      // points arrived
      let triggered: bool = temp_changed || desired_temp.is_some();

      // -------- Latch newly received set points (if any) ----------
      match desired_temp {
        Some(sp) => {
          // the payload is well-formed (lower <= upper) by the
          // ASSM_LDT_LE_UDT integration assume on the port
          self.currentSetPoints = sp;
          log_set_points_latched(sp);
        }
        None => {
          // no new set points -- keep the latched values
        }
      }

      //================ compute / control logic ===========================
      // ...runs only when a triggering event is present (REQ_THERM_TRIGGER)
      if triggered {
        // current command defaults to value of last command (REQ-THERM-4)
        let mut currentCmd: On_Off = self.lastCmd;

        if current_temp.degrees > self.currentSetPoints.upper.degrees {
          // REQ-THERM-3
          currentCmd = On_Off::Off;
        } else if current_temp.degrees < self.currentSetPoints.lower.degrees {
          // REQ-THERM-2
          currentCmd = On_Off::Onn;
        }

        // -------------- Set values of output ports ------------------
        // SEND-ON-CHANGE: emit a command message exactly when the commanded
        // state changes (REQ_THERM_SOC)
        if !on_off_eq(currentCmd, self.lastCmd) {
          api.put_heat_control(currentCmd);
          self.lastCmd = currentCmd;
          log_command_sent(currentCmd);
        }
      }
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

  //-------------------------------------------
  //  Helper Functions
  //-------------------------------------------

  // Executable equality test on On_Off command values.
  // NOTE: inside verus! blocks, `==` cannot be used on enums from the `data`
  // crate in EXECUTABLE code (the derived PartialEq is external to Verus);
  // pattern matching is used instead.  In SPEC position (the ensures clause
  // below), `==` denotes Verus structural equality and is fine.
  pub fn on_off_eq(a: On_Off, b: On_Off) -> (res: bool)
    ensures res == (a == b)
  {
    match (a, b) {
      (On_Off::Onn, On_Off::Onn) => true,
      (On_Off::Off, On_Off::Off) => true,
      _ => false,
    }
  }

  //-------------------------------------------
  //  Logging Helper Functions
  //-------------------------------------------
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
  pub fn log_set_points_latched(sp: Set_Points)
  {
    log::info!("Set points latched: [{}, {}]", sp.lower.degrees, sp.upper.degrees);
  }

  #[verifier::external_body]
  pub fn log_command_sent(cmd: On_Off)
  {
    log::info!("Heat control command sent: {:?}", cmd);
  }

  // BEGIN MARKER GUMBO METHODS
  pub open spec fn Temp_Lower_Bound() -> i32
  {
    95i32
  }

  pub open spec fn Temp_Upper_Bound() -> i32
  {
    104i32
  }

  pub open spec fn Default_Lower_Set_Point() -> i32
  {
    98i32
  }

  pub open spec fn Default_Upper_Set_Point() -> i32
  {
    101i32
  }
  // END MARKER GUMBO METHODS

}
