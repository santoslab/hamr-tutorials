// #Sireum -- required marker indicating that this file is a Slang file (needed to trigger Slang parsing).

package TempControlMixedUI.TempControlSystem

import org.sireum._                     // all Slang files must start with this import
import TempControlMixedUI._             // make other packages within TempControlMixedUI visible
import TempControlMixedUI.TempSensor._  // make definitions within TempSensor package visible
import TempControlMixedUI.CoolingFan._  // make definitions within CoolingFan package visible



// This file will not be overwritten so is safe to edit
object TempControl_i_tcproc_tempControl {

  //===============================================
  //  C o m p o n e n t    L o c a l    S t a t e
  //===============================================

  // -- S e t  P o i n t
  //     ...holds the current set point values (both low and high set points).
  //     This will be the most recent set points received over the setPoint in event data port
  //     (from the operator interface) or the initial (default) set points if no
  //     setpoints have been received yet.
  //
  // Note: Slang requires all declared variables to be initialized, so initialize to initial
  //    value even though the Initialize Entry Point should also do this.
  var setPoint: SetPoint_i = Defs.initialSetPoint

  //=================================================
  //  I n i t i a l i z e    E n t r y    P o i n t
  //=================================================
  def initialise(api: TempControl_i_Initialization_Api): Unit = {
    api.logInfo("Initialize Entry Point")

    // The Initialize Entry Point must initialize all component local state and all output data ports.

    // initialize component local state
    setPoint = Defs.initialSetPoint

    // initialize output data ports
    //  (no output data ports to initialize)
  }

  //=================================================
  //  C o m p u t e    E n t r y    P o i n t
  //
  //  Event handlers for sporadic AADL thread component
  //=================================================

  //------------------------------------------------
  //  f a n A c k   event data handler
  //
  // Handler for event data arriving on fanAck in event data port
  //------------------------------------------------
  def handle_fanAck(api: TempControl_i_Operational_Api, value: CoolingFan.FanAck.Type): Unit = {
    // log to indicate that that a fanAck event was received
    // on the fanAck in event data port
    api.logInfo(s"received fanAck $value")
    if (value == CoolingFan.FanAck.Error) {
      // In a more complete implementation, we would implement some sort
      // of mitigation or recovery action at this point.
      // For now, we just log that fan is telling us that it did not
      // respond as expected to the last sent command.
      api.logError("Actuation failed!")
    } else {
      // Log actuation succeeded
      api.logInfo("Actuation worked.")
    }
  }

  //------------------------------------------------
  //  s e t P o i n t   event data handler
  //
  // Handler for event data arriving on setPoint event data port
  //------------------------------------------------
  def handle_setPoint(api: TempControl_i_Operational_Api, value: TempControlSystem.SetPoint_i): Unit = {
    // log to indicate that that a setPoint event was received/handled
    // on the setPoint in event data port
    api.logInfo(s"received setPoint $value")
    // assign the setPoint record (containing both low and high set points)
    // to a component local variable "setPoint" that holds the current set point values
    setPoint = value
  }

  //------------------------------------------------
  //  t e m p  C h a n g e d    event handler
  //
  // Event handler for event arriving on tempChanged in event port
  //------------------------------------------------
  def handle_tempChanged(api: TempControl_i_Operational_Api): Unit = {
    // log to indicate that that a tempChanged event was received/handled
    api.logInfo(s"received tempChanged")

    // get current temp from currentTemp in data port
    val temp: Temperature_i = api.get_currentTemp().get // type decl for temp is optional
    // convert current temp to Fahrenheit
    val tempInF = Util.toFahrenheit(temp)
    // convert stored setpoint values to Fahrenheit
    val setPointLowInF = Util.toFahrenheit(setPoint.low)
    val setPointHighInF = Util.toFahrenheit(setPoint.high)
    // compute command to send to fan
    // (use Option type to capture the fact that we may not need to send a command
    //  if temperature is in setpoint range)
    val cmdOpt: Option[FanCmd.Type] =
    if (tempInF.degrees > setPointHighInF.degrees)
    // if current temp exceeds high set point,
    // produce a command (Some) that turns cooling fan on
      Some(FanCmd.On)
    else if (tempInF.degrees < setPointLowInF.degrees)
    // if current temp is below low set point,
    // produce a command (Some) that turns cooling fan off
      Some(FanCmd.Off)
    // if current temp is between low and high set point (inclusive),
    // don't produce a command (None)
    else None[FanCmd.Type]()
    cmdOpt match {
      // if a command was produced, send it and log it
      case Some(cmd) =>
        // put on/off command on fanCmd output event data port
        api.put_fanCmd(cmd)
        api.logInfo(s"Sent fan command: ${if (cmd == FanCmd.On) "on" else "off"}")
      case _ =>
        api.logInfo(s"Temperature ok: ${tempInF.degrees} F")
    }
  }

  //=================================================
  //  AADL entry points below are unused
  //=================================================
  def activate(api: TempControl_i_Operational_Api): Unit = { }

  def deactivate(api: TempControl_i_Operational_Api): Unit = { }

  def finalise(api: TempControl_i_Operational_Api): Unit = { }

  def recover(api: TempControl_i_Operational_Api): Unit = { }
}
