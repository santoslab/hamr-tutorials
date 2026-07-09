/*============================================================
    Isolette   S o f t w a r e 

    Defines the software components (threads) used in the 
    Isolette system.  

    Define the software elements in a separate package enables
    us to more easily reuse them and assemble them in different
    ways in different systems as we deploy to different platforms.

  ============================================================*/
 
package Isolette_Software {
  private import HAMR::*;

  //===============================================================================
  // 
  //  S t y l e s  (optional)
  // 
  //  Define common styles/patterns of software to be used.
  //  In a larger development context, these might be managed resources of a 
  //  particular organization's development process.
  //===============================================================================
  part def Periodic_Thread :> Thread {
    attribute :>> Period = 1000 [ms];  // default period for Isolette threads
    attribute :>> Dispatch_Protocol = Supported_Dispatch_Protocols::Periodic;
  }

  part def Periodic_Rust_Thread :> Periodic_Thread {
     attribute :>> Implementation_Language = Implementation_Languages::Rust;
  }

  //===============================================================================
  // 
  //  T h e r m o s t a t   Thread  
  // 
  //===============================================================================
  part def Thermostat :> Periodic_Rust_Thread {
    //-- I n t e r f a c e  --
    //
    // EDP variant of the original (all-DataPort) Thermostat:
    //  - current_temp remains a sampled data port (always has the latest reading)
    //  - temp_changed is an event announcing that the sensed value changed
    //    (its payload carries the new value; the control law below samples the
    //    current_temp data port and uses the event only as a trigger)
    //  - desired_temp is now an event data port: set points arrive as messages
    //    only when the operator changes them, so the most recent message is
    //    latched in the GUMBO state variable `currentSetPoints`
    //  - heat_control is now an event data port with a SEND-ON-CHANGE policy:
    //    a command message is emitted exactly when the commanded state changes
    port current_temp : DataPort { in :>> type : Isolette_Data_Model::Temp; }
    port temp_changed : EventDataPort { in :>> type : Isolette_Data_Model::Temp; }
    port desired_temp : EventDataPort { in :>> type : Isolette_Data_Model::Set_Points; }
    port heat_control : EventDataPort { out :>> type : Isolette_Data_Model::On_Off; }

    //-- B e h a v i o r    C o n s t r a i n t s --
    language "GUMBO" /*{
      state
         currentSetPoints: Isolette_Data_Model::Set_Points;
         lastCmd: Isolette_Data_Model::On_Off;

      functions
      //  -- desired bounds on temperature values in the thermostat
      def Temp_Lower_Bound(): Base_Types::Integer_32 := 95 [i32];
      def Temp_Upper_Bound(): Base_Types::Integer_32 := 104 [s32];
      //  -- default desired temperature range, in force until the first
      //     set point message arrives (chosen to match the Operator
      //     Interface's initial simulated set points)
      def Default_Lower_Set_Point(): Base_Types::Integer_32 := 98 [i32];
      def Default_Upper_Set_Point(): Base_Types::Integer_32 := 101 [i32];

	  //-- == I n t e g r a t i o n     C o n s t r a i n t s
	  integration
  	    assume ASSM_CT_Range:
  	    //-- Literals used in spec below
  	      Temp_Lower_Bound() <= current_temp.degrees & current_temp.degrees <= Temp_Upper_Bound();
        // -- integration constraints on an event data port constrain the message
        //    payload whenever a message is present.  In the original (DataPort)
        //    variant this was a compute-entry-point assume; stating it as an
        //    integration assume lets HAMR discharge it at model-assembly time
        //    against the Operator Interface's matching integration guarantee.
        assume ASSM_LDT_LE_UDT "Incoming set point messages must be well-formed
                               |(lower desired temp at most upper desired temp).":
          desired_temp.lower.degrees <= desired_temp.upper.degrees;

  		//--  == I n i t i a l i z e    E n t r y    P o i n t    Behavior Constraints  ==
        initialize
	      guarantee initCurrentSetPoints "The latched set points start at the default range.":
		    currentSetPoints.lower.degrees == Default_Lower_Set_Point() &
		    currentSetPoints.upper.degrees == Default_Upper_Set_Point();
		  guarantee REQ_THERM_1 "The commanded heat state shall be Off initially.
		                        |Adapted from the original REQ_THERM_1: no message is
		                        |sent during initialization (event data ports need no
		                        |initialization), and the Heat Source independently
		                        |initializes its heater state to Off.":
		    lastCmd == Isolette_Data_Model::On_Off.Off;

		//--  ====== C o m p u t e    E n t r y    P o i n t   Behavior Constraints =====
		  compute
        // -- well-formedness of the LATCHED set points.  Established by the
        //    initialize entry point (defaults 98 <= 101) and re-established by
        //    every dispatch (see invCSPMaintained below) because only payloads
        //    satisfying integration assume ASSM_LDT_LE_UDT are ever latched.
        assume INV_CSP "The latched set points are well-formed.":
          currentSetPoints.lower.degrees <= currentSetPoints.upper.degrees;

        // --- (1) Latching of incoming set point messages ---
        guarantee latchSetPointsOnEvent "A newly received set point message is latched.":
          HasEvent(desired_temp) implies
            (currentSetPoints.lower.degrees == desired_temp.lower.degrees and
             currentSetPoints.upper.degrees == desired_temp.upper.degrees);
        guarantee latchSetPointsNoEvent "Otherwise the latched set points are unchanged.":
          (not HasEvent(desired_temp)) implies currentSetPoints == In(currentSetPoints);
        guarantee invCSPMaintained "Well-formedness of the latched set points is
                                   |re-established for the next dispatch.":
          currentSetPoints.lower.degrees <= currentSetPoints.upper.degrees;

        // --- (2) Frame condition: control logic only runs when triggered ---
        guarantee noTriggerNoChange "Without a triggering event (temperature change or
                                    |new set points) the commanded state is unchanged.":
          (not (HasEvent(temp_changed) or HasEvent(desired_temp)))
            implies lastCmd == In(lastCmd);

        // --- (3) Control law over the latched set points ---
        //     Adapted from the original REQ_THERM_2/3/4 compute cases: the cases
        //     are now guarded by the event triggers, judged against the (freshly)
        //     latched set points, and constrain the commanded state `lastCmd`
        //     rather than the output port (the port only carries changes -- see (4)).
        guarantee REQ_THERM_2 "If triggered and the Current Temperature is less than
                              |the Lower Desired Temperature, the commanded heat state
                              |shall be set to On.":
          ((HasEvent(temp_changed) or HasEvent(desired_temp)) and
            current_temp.degrees < currentSetPoints.lower.degrees)
          implies lastCmd == Isolette_Data_Model::On_Off.Onn;

        guarantee REQ_THERM_3 "If triggered and the Current Temperature is greater than
                              |the Upper Desired Temperature, the commanded heat state
                              |shall be set to Off.":
          ((HasEvent(temp_changed) or HasEvent(desired_temp)) and
            current_temp.degrees > currentSetPoints.upper.degrees)
          implies lastCmd == Isolette_Data_Model::On_Off.Off;

        guarantee REQ_THERM_4 "If triggered and the Current Temperature is greater than
                              |or equal to the Lower Desired Temperature and less than
                              |or equal to the Upper Desired Temperature, the commanded
                              |heat state shall not be changed.":
          ((HasEvent(temp_changed) or HasEvent(desired_temp)) and
            current_temp.degrees >= currentSetPoints.lower.degrees and
            current_temp.degrees <= currentSetPoints.upper.degrees)
          implies lastCmd == In(lastCmd);

        // --- (4) Send-on-change policy for the heat control command ---
        guarantee mustSendOnChange "A command message is sent exactly when the commanded
                                   |state changes, and it carries the new state.":
          (lastCmd != In(lastCmd)) implies MustSend(heat_control, lastCmd);
        guarantee noSendNoChange "No message is sent when the commanded state is unchanged.":
          (lastCmd == In(lastCmd)) implies NoSend(heat_control);
        }*/
    }

    //===============================================================================
    // 
    //  O p e r a t o r   I n t e r f a c e  Thread  
    // 
    //===============================================================================
    part def OperatorInterface :> Periodic_Rust_Thread {
      //-- I n t e r f a c e  --
      // EDP variant: set points are emitted as event data messages, and only when
      // the (simulated) operator actually changes them -- not on every dispatch.
      port desired_temp : EventDataPort { out :>> type : Isolette_Data_Model::Set_Points; }

      //-- B e h a v i o r    C o n s t r a i n t s --
      language "GUMBO" /*{
        integration
          // -- producer side of the handshake with the Thermostat's integration
          //    assume ASSM_LDT_LE_UDT: every emitted set point message is well-formed.
          guarantee LDT_LE_UDT "The lower desired temperature is at most the
                               |upper desired temperature in every emitted message.":
            desired_temp.lower.degrees <= desired_temp.upper.degrees;
      }*/
    }
}