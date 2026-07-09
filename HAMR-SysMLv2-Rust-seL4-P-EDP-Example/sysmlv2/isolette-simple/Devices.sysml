/*============================================================
    D e v i c e s

    Defines the software components (threads), intuitively 
    corresponding to components that encapsulate drivers 
    interacting with lower-level hardware components of the 
    system.

    In a full development context, where we want to emphasize
    managed collections of reusable resources, each physical 
    device might be accompanied by both software (as illustrated here)
    and hardware oriented modeling elements (not shown here).
    We might have a separate package for each physical device
    or families of physical devices.

  ============================================================*/

package Devices {
  private import HAMR::*;

  //===============================================================================
  // 
  //  T e m p S e n s o r   Thread 
  // 
  //===============================================================================
  part def TempSensor :> Thread {
    //-- I n t e r f a c e  --
    port current_temp : DataPort { out :>> type : Isolette_Data_Model::Temp; }
    // EDP variant: an event data message announcing that the sensed temperature
    // value changed (the payload carries the new value).  Downstream components
    // (e.g., the Thermostat) use the event as a trigger to run their control
    // logic instead of re-sampling on every dispatch.
    // NOTE: conceptually this could be a pure (payload-less) EventPort -- and the
    // Microkit/Rust backend supports pure event ports in general -- but the current
    // codegen version does not yet support a component that has BOTH integration
    // constraints and a pure event port, so the event carries the new Temp value.
    port temp_changed : EventDataPort { out :>> type : Isolette_Data_Model::Temp; }

    //-- P r o p e r t i e s  --
    attribute :>> Period = 1000 [ms];
    attribute :>> Dispatch_Protocol = Supported_Dispatch_Protocols::Periodic;
    attribute :>> Implementation_Language = Implementation_Languages::Rust;

    //-- B e h a v i o r    C o n s t r a i n t s --
    language "GUMBO" /*{
       integration
          guarantee temp_range:
              96 [i32] <= current_temp.degrees & current_temp.degrees <= 103 [i32];
          // -- integration constraints on an event data port constrain the message
          //    payload whenever a message is present: any announced temperature
          //    change carries a value in the same sensed range
          guarantee temp_changed_range:
              96 [i32] <= temp_changed.degrees & temp_changed.degrees <= 103 [i32];
    }*/
  }

  //===============================================================================
  // 
  //  H e a t S o u r c e   Thread 
  // 
  //===============================================================================
  part def HeatSource :> Thread {
    //-- I n t e r f a c e  --
    // EDP variant: commands arrive as event data messages (sent by the Thermostat
    // only when the commanded state changes) instead of a continuously sampled
    // data port.  Because event data port inputs do not persist between dispatches,
    // the most recently received command is latched in the GUMBO state variable
    // `heater_state` below.
    port heat_control : EventDataPort { in :>> type : Isolette_Data_Model::On_Off; }

    //-- P r o p e r t i e s  --
    attribute :>> Period = 1000 [ms];
    attribute :>> Dispatch_Protocol = Supported_Dispatch_Protocols::Periodic;
    attribute :>> Implementation_Language = Implementation_Languages::Rust;

    //-- B e h a v i o r    C o n s t r a i n t s --
    language "GUMBO" /*{
      state
        heater_state: Isolette_Data_Model::On_Off;

      initialize
        guarantee REQ_THERM_1_HS "The heater shall be Off initially.
                                 |This preserves the system-level intent of REQ_THERM_1:
                                 |event data ports need no initialization message, so
                                 |the heat source itself guarantees the initial Off state.":
          heater_state == Isolette_Data_Model::On_Off.Off;

      compute
        guarantee HS_latch_cmd "A received command is latched into the heater state.":
          HasEvent(heat_control) implies heater_state == heat_control;
        guarantee HS_hold_cmd "If no command arrives, the heater state is unchanged.":
          (not HasEvent(heat_control)) implies heater_state == In(heater_state);
    }*/
  }
}
