//@ HAMR: --platform Microkit --output-dir ../../hamr

package Isolette {

  private import HAMR::*;
  private import Isolette_Software::*;

  //===============================================================================
  // 
  //  I s o l e t t e   S y s t e m 
  // 
  //  Top-level architecture definition integrating software and hardware and 
  //  specifying the partitioning and the system.
  //
  //===============================================================================
  part def Isolette_System :> System {
    //-- I n t e r n a l s --
    // ..indicate the processes (partitions) of the system
    part temp_sensor: TempSensor_Process;
    part operator_interface: OperatorInterface_Process;
    part thermostat: Thermostat_Process;
    part heat_source: HeatSource_Process;
    part isolette_processor : Platform::Processor_1;

    // connections (information flows) between partitions
    connection ct : PortConnection connect temp_sensor.current_temp to thermostat.current_temp;
    connection tc : PortConnection connect temp_sensor.temp_changed to thermostat.temp_changed;
    connection hc : PortConnection connect thermostat.heat_control to heat_source.heat_control;
    connection sp : PortConnection connect operator_interface.desired_temp to thermostat.desired_temp;

    //-- A l l o c a t i o n s  of software/partitions to hardware
    allocation pb0: Deployment_Properties::Actual_Processor_Binding 
      allocate temp_sensor to isolette_processor;
    allocation pb1: Deployment_Properties::Actual_Processor_Binding 
      allocate operator_interface to isolette_processor;
    allocation pb2: Deployment_Properties::Actual_Processor_Binding 
      allocate thermostat to isolette_processor;
    allocation pb3: Deployment_Properties::Actual_Processor_Binding
      allocate heat_source to isolette_processor;

    //===============================================================================
    //
    //  S y s t e m - L e v e l   P r o p e r t i e s   (GUMBO composition)
    //
    //  The Thermostat sends a heat command only when its commanded state changes
    //  (send-on-change), and the Heat Source latches each command it receives.
    //  Neither component's contract mentions the other, but together they keep
    //  the heater's state equal to the commanded state in every frame: both start
    //  Off, a change is always sent and latched in the same frame, and no change
    //  means no message and nothing to latch.
    //===============================================================================
    language "GUMBO" /*{
      composition nominal {

        // Short alias  =  <process-instance>.<thread-instance>
        components
          oi     = operator_interface.operator_interface;
          sensor = temp_sensor.temp_sensor;
          therm  = thermostat.thermostat;
          hs     = heat_source.heat_source;

        // Named handles for the port values the properties refer to.
        // The temperature and the change event are the Thermostat's inputs: what it
        // received, which is what its control law reacts to.
        ports
          sensedTemp  = therm.current_temp;    // data port
          tempChanged = therm.temp_changed;    // event port
          heatCmd     = therm.heat_control;    // event data port (output)

        // Named handles for the component GUMBO state variables.
        state
          lastCmd     = therm.lastCmd;
          setPoints   = therm.currentSetPoints;
          heaterState = hs.heater_state;

        // The threads run in this order in every frame.
        schema {
          oi;
          sensor;
          therm;
          hs
        }

        //-----------------------------------------------------------------------------
        //  Property 1 -- a loop invariant: the heater's state equals the Thermostat's
        //  commanded state at the start and end of every frame.  The latched set
        //  points' well-formedness rides along, because the Thermostat assumes it
        //  (INV_CSP) and every property covering the Thermostat must establish it.
        //-----------------------------------------------------------------------------
        abstract property Heater_Agreement_Base
          "loop invariant: the heater's state agrees with the commanded state, and the
          |latched set points are well-formed" {
          at START "established initially by the initialize guarantees (all Off; set
            |points 98..101)":
            (heaterState == lastCmd) &
            (setPoints.lower.degrees <= setPoints.upper.degrees);
          after oi "carried across the Operator Interface (it touches neither)":
            (heaterState == lastCmd) &
            (setPoints.lower.degrees <= setPoints.upper.degrees);
          after sensor "carried across the Temp Sensor; discharges the Thermostat's
            |INV_CSP assume":
            (heaterState == lastCmd) &
            (setPoints.lower.degrees <= setPoints.upper.degrees);
          after therm "send-on-change: either the new command is on its way, or the
            |command did not change and the heater already agrees with it":
            ((HasEvent(heatCmd) and heatCmd == lastCmd) or
             ((not HasEvent(heatCmd)) and heaterState == lastCmd)) &
            (setPoints.lower.degrees <= setPoints.upper.degrees);
          after hs "the Heat Source latched the command, if one arrived":
            (heaterState == lastCmd) &
            (setPoints.lower.degrees <= setPoints.upper.degrees);
          at END "loop closure: END |- START":
            (heaterState == lastCmd) &
            (setPoints.lower.degrees <= setPoints.upper.degrees);
        }

        property Heater_Agreement :> Heater_Agreement_Base
          "every frame, the heater's state equals the Thermostat's commanded state" {
        }

        //-----------------------------------------------------------------------------
        //  Property 2 -- end to end: when the Thermostat receives a temperature change
        //  and the temperature it receives is below the lower set point, the heater is
        //  on by the end of the frame.
        //-----------------------------------------------------------------------------
        property Too_Cold_Heats :> Heater_Agreement_Base
          "a reported temperature below the lower set point turns the heater on in
          |the same frame" {
          after therm "REQ_THERM_2: a reported change with the received temperature below
            |the lower set point makes the Thermostat command heat":
            (HasEvent(tempChanged) and sensedTemp.degrees < setPoints.lower.degrees)
              implies lastCmd == Isolette_Data_Model::On_Off.Onn;
          after hs "the Heat Source has latched it":
            (HasEvent(tempChanged) and sensedTemp.degrees < setPoints.lower.degrees)
              implies heaterState == Isolette_Data_Model::On_Off.Onn;
        }
      }
    }*/
  }

  //===============================================================================
  // 
  //  T e m p S e n s o r   Process  (partition)
  // 
  //===============================================================================
  part def TempSensor_Process :> Process {
     //-- I n t e r f a c e  --
    port current_temp : DataPort { out :>> type : Isolette_Data_Model::Temp; }
    out port temp_changed : EventPort;

    //-- P r o p e r t i e s  --
    attribute :>> Domain = Isolette_Properties::Domain_TS;

    //-- I n t e r n a l s --
    part temp_sensor: Devices::TempSensor;
    connection ct : PortConnection connect temp_sensor.current_temp to current_temp;
    connection tc : PortConnection connect temp_sensor.temp_changed to temp_changed;
  }

  //===============================================================================
  // 
  //  H e a t S o u r c e   Process  (partition)
  // 
  //===============================================================================
  part def HeatSource_Process :> Process {
    //-- I n t e r f a c e  --
    port heat_control : EventDataPort { in :>> type : Isolette_Data_Model::On_Off; }

    //-- P r o p e r t i e s  --
    attribute :>> Domain = Isolette_Properties::Domain_HS;

    //-- I n t e r n a l s --
    part heat_source: Devices::HeatSource; 
    connection hc : PortConnection connect heat_control to heat_source.heat_control;
  }

  //===============================================================================
  // 
  //  O p e r a t o r   I n t e r f a c e    Process  (partition)
  // 
  //===============================================================================
  part def OperatorInterface_Process :> Process {
    //-- I n t e r f a c e  --
    port desired_temp : EventDataPort { out :>> type : Isolette_Data_Model::Set_Points; }

    //-- P r o p e r t i e s  --
    attribute :>> Domain = Isolette_Properties::Domain_OI;

    //-- I n t e r n a l s --
    part operator_interface: OperatorInterface; 
    connection sp : PortConnection connect operator_interface.desired_temp to desired_temp;
  }

  //===============================================================================
  // 
  //  T h e r m o s t a t    Process  (partition)
  // 
  //===============================================================================
  part def Thermostat_Process :> Process {
    //-- I n t e r f a c e  --
    port current_temp : DataPort { in :>> type : Isolette_Data_Model::Temp; }
    in port temp_changed : EventPort;
    port desired_temp : EventDataPort { in :>> type : Isolette_Data_Model::Set_Points; }
    port heat_control : EventDataPort { out :>> type : Isolette_Data_Model::On_Off; }

    attribute :>> Domain = Isolette_Properties::Domain_THERM;

    //-- I n t e r n a l s --
    part thermostat: Thermostat;
    connection ct : PortConnection connect current_temp to thermostat.current_temp;
    connection tc : PortConnection connect temp_changed to thermostat.temp_changed;
    connection hc : PortConnection connect thermostat.heat_control to heat_control;
    connection sp : PortConnection connect desired_temp to thermostat.desired_temp;
  }
}