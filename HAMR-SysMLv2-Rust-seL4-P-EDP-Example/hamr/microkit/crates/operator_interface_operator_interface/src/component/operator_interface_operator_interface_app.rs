// This file will not be overwritten if HAMR codegen is rerun

//============================================================================
//  O p e r a t o r    I n t e r f a c e   -- Thread Component
//
//  This Thread component implements a (simulated) operator interface
//  for the Isolette.
//
//  It uses a simple notion of simulation similar to that of the Temp Sensor
//  thread to move the values of the set points (desired temperature
//  range for the Isolette) up and down.
//
//  EDP variant: the desired_temp port is now an event data port.  A set point
//  message is emitted ONLY when the (simulated) operator actually changes the
//  set points -- every (activations_between_updates + 1)th activation -- not
//  on every dispatch as in the original all-DataPort variant.  Because the
//  output is an event data port, no initialization message is required (the
//  Thermostat's latched set points start at the matching default range).
//
//  Verus notes: the LDT_LE_UDT integration guarantee from the model becomes a
//  `requires` clause on put_desired_temp, discharged by an explicit
//  well-formedness guard before the send.  The +/-1 simulation arithmetic is
//  written with range clamps (see clamp) so Verus can prove it cannot
//  overflow.
//============================================================================

use data::*;
use data::Isolette_Data_Model::*; // Add for easier reference to data types
use crate::bridge::operator_interface_operator_interface_api::*;
use vstd::prelude::*;

verus! {

  //-------------------------------------------
  //  Constants
  //-------------------------------------------

  // --- set point simulation bounds ---
  pub const lower_desired_temp_min: i32 = 97;
  pub const lower_desired_temp_max: i32 = 99;
  pub const upper_desired_temp_min: i32 = 99;
  pub const upper_desired_temp_max: i32 = 102;

  // initial (default) set points -- these match the Thermostat's
  // Default_Lower_Set_Point / Default_Upper_Set_Point GUMBO functions
  pub const initial_lower_desired_temp: i32 = 98;
  pub const initial_upper_desired_temp: i32 = 101;

  // number of activations between (simulated) operator set point updates
  pub const activations_between_updates: i32 = 5;

  //-------------------------------------------
  //  Application State (as a struct)
  //
  //  The application state includes non-GUMBO state variables
  //  that support the "simulation" of the operator interface.
  //
  //  There is no GUMBO declared application state for this component.
  //-------------------------------------------
  pub struct operator_interface_operator_interface {
    // PLACEHOLDER MARKER STATE VARS

    // The variables below are used for operator input (set points) simulation.
    // This also illustrates non-GUMBO declared state variables for a component
    pub lower_desired_temp: i32,
    pub upper_desired_temp: i32,
    pub lower_desired_temp_trajectory: i32,
    pub upper_desired_temp_trajectory: i32,
    pub activations_until_update: i32,
  }

  impl operator_interface_operator_interface {
    //-------------------------------------------
    //  Application Component Constructor
    //-------------------------------------------
    pub fn new() -> Self
    {
      Self {
        // PLACEHOLDER MARKER STATE VAR INIT

        // initialization of non-GUMBO declared state variables
        // ...simulated operator input
        lower_desired_temp: initial_lower_desired_temp,
        upper_desired_temp: initial_upper_desired_temp,
        lower_desired_temp_trajectory: -1, // value will either be +1 or -1
        upper_desired_temp_trajectory: 1, // value will either be +1 or -1
        activations_until_update: activations_between_updates,
      }
    }

    //-------------------------------------------
    //  Initialize Entry Point
    //-------------------------------------------
    pub fn initialize<API: operator_interface_operator_interface_Put_Api> (
      &mut self,
      api: &mut operator_interface_operator_interface_Application_Api<API>)
      ensures
        // PLACEHOLDER MARKER INITIALIZATION ENSURES
    {
      log_info("initialize entrypoint invoked");

      // initialize operator input simulation
      self.lower_desired_temp = initial_lower_desired_temp;
      self.upper_desired_temp = initial_upper_desired_temp;
      self.lower_desired_temp_trajectory = -1;
      self.upper_desired_temp_trajectory = 1;
      self.activations_until_update = activations_between_updates;

      // EDP variant: no message is sent during initialization -- event data
      // ports need no initialization, and the Thermostat's latched set points
      // start at the same default range [98, 101].
    }

    //-------------------------------------------
    //  Compute Entry Point
    //-------------------------------------------
    pub fn timeTriggered<API: operator_interface_operator_interface_Full_Api> (
      &mut self,
      api: &mut operator_interface_operator_interface_Application_Api<API>)
      requires
        // PLACEHOLDER MARKER TIME TRIGGERED REQUIRES
      ensures
        // PLACEHOLDER MARKER TIME TRIGGERED ENSURES
    {
      log_info("compute entrypoint invoked");

      // --------- Process operator's configuration of set points ------------

      // set point simulation
      if self.activations_until_update > 0 {
        // Case: Not updating set points yet... hold the current values and
        // send NO set point message this dispatch.
        self.activations_until_update = self.activations_until_update - 1;
      } else {
        // Case: Updating set points

        //   ..update lower_desired_temp simulation according to trajectory.
        //   The previous value is clamped into the simulation range first so
        //   that Verus can discharge the overflow obligations.
        let prev_lower: i32 = clamp(self.lower_desired_temp, lower_desired_temp_min, lower_desired_temp_max);
        let next_lower: i32;
        if self.lower_desired_temp_trajectory >= 0 {
          if prev_lower < lower_desired_temp_max { next_lower = prev_lower + 1; } else { next_lower = lower_desired_temp_max; }
        } else {
          if prev_lower > lower_desired_temp_min { next_lower = prev_lower - 1; } else { next_lower = lower_desired_temp_min; }
        }

        //   ..reverse the simulation trajectory when the value reaches bounds
        if next_lower >= lower_desired_temp_max {
          self.lower_desired_temp_trajectory = -1;
        } else if next_lower <= lower_desired_temp_min {
          self.lower_desired_temp_trajectory = 1;
        }
        self.lower_desired_temp = next_lower;

        //   ..update upper_desired_temp simulation according to trajectory
        let prev_upper: i32 = clamp(self.upper_desired_temp, upper_desired_temp_min, upper_desired_temp_max);
        let next_upper: i32;
        if self.upper_desired_temp_trajectory >= 0 {
          if prev_upper < upper_desired_temp_max { next_upper = prev_upper + 1; } else { next_upper = upper_desired_temp_max; }
        } else {
          if prev_upper > upper_desired_temp_min { next_upper = prev_upper - 1; } else { next_upper = upper_desired_temp_min; }
        }

        //   ..reverse the simulation trajectory when the value reaches bounds
        //   (NOTE: the original all-DataPort variant had a bug here -- it
        //    assigned 1 to upper_desired_temp instead of to the trajectory)
        if next_upper >= upper_desired_temp_max {
          self.upper_desired_temp_trajectory = -1;
        } else if next_upper <= upper_desired_temp_min {
          self.upper_desired_temp_trajectory = 1;
        }
        self.upper_desired_temp = next_upper;

        // reset activation count
        self.activations_until_update = activations_between_updates;

        // build set points struct and SEND it -- the operator just changed
        // the set points, so this dispatch emits a set point message
        let set_points =
          Set_Points { lower: Temp { degrees: next_lower },
                       upper: Temp { degrees: next_upper } };

        // well-formedness guard: discharges the LDT_LE_UDT integration
        // guarantee (`requires` on put_desired_temp).  By construction
        // next_lower <= 99 <= next_upper, so the guard always holds at runtime.
        if set_points.lower.degrees <= set_points.upper.degrees {
          api.put_desired_temp(set_points);
        }
      }

      log_set_point_simulation(
        self.lower_desired_temp,
        self.upper_desired_temp,
        self.lower_desired_temp_trajectory,
        self.upper_desired_temp_trajectory,
        self.activations_until_update,
      );
    }

    //-------------------------------------------
    //  seL4 / Microkit Error Handling
    //-------------------------------------------
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

  // Clamp a value into [lo, hi].  The ensures clauses give Verus what it needs
  // to discharge the overflow obligations on the +/-1 simulation updates; at
  // runtime the simulation never actually leaves its range, so the second
  // ensures clause guarantees the clamp is the identity on simulated values.
  pub fn clamp(v: i32, lo: i32, hi: i32) -> (res: i32)
    requires
      lo <= hi,
    ensures
      lo <= res && res <= hi,
      (lo <= v && v <= hi) ==> res == v,
  {
    if v > hi {
      hi
    } else if v < lo {
      lo
    } else {
      v
    }
  }

  //-------------------------------------------
  //  Logging Helper Functions
  //-------------------------------------------
  #[verifier::external_body]
  pub fn log_set_point_simulation(
    lower_desired_temp: i32,
    upper_desired_temp: i32,
    lower_desired_temp_trajectory: i32,
    upper_desired_temp_trajectory: i32,
    activations_until_update: i32,
  )
  {
     log::info!("LDT: {}", lower_desired_temp);
     log::info!("UDT: {}", upper_desired_temp);
     log::info!("LDT Trajectory: {}", lower_desired_temp_trajectory);
     log::info!("UDT Trajectory: {}", upper_desired_temp_trajectory);
     log::info!("Activations until update: {}", activations_until_update);
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

  // PLACEHOLDER MARKER GUMBO METHODS

}
