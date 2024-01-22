// #Sireum -- required marker indicating that this file is a Slang file (needed to trigger Slang parsing).

package TempControlMixedUI.TempSensor

import org.sireum._
import TempControlMixedUI._

// This file will not be overwritten so is safe to edit
object TempSensor_i_tcproc_tempSensor {

  //=================================================
  //  I n i t i a l i z e    E n t r y    P o i n t
  //=================================================
  def initialise(api: TempSensor_i_Initialization_Api): Unit = {
    // The Initialize Entry Point must initialize all component local state and all output data ports.

    // initialize component local state
    //   (no component local state to initialize)

    // initialize output data ports
    api.put_currentTemp(TempControlSystem.Defs.initialTemp)
  }

  //=================================================
  //  C o m p u t e    E n t r y    P o i n t
  //=================================================
  def timeTriggered(api: TempSensor_i_Operational_Api): Unit = {
    // read temperature from temperature sensor,
    // via interface realized via Slang Extension "TempSensorNative"
    val temp = TempSensorDevice.currentTempGet()

    // set the out data port currentTemp to hold the read temperature
    api.put_currentTemp(temp)
    // put an event on tempChanged out event port to
    // notify subscribers (e.g., tempControl thermostat) that the
    // temperature has changed
    api.put_tempChanged()

    // log the current temperature (after conversion to Fahrenheit)
    val degree = TempControlSystem.Util.toFahrenheit(temp).degrees
    api.logInfo(s"Sensed temperature: $degree F")
  }

  //=================================================
  //  AADL entry points below are unused
  //=================================================
  def activate(api: TempSensor_i_Operational_Api): Unit = { }

  def deactivate(api: TempSensor_i_Operational_Api): Unit = { }

  def finalise(api: TempSensor_i_Operational_Api): Unit = { }

  def recover(api: TempSensor_i_Operational_Api): Unit = { }
}

//=================================================
//
//  Slang extension used to interface with non-Slang
//  code to retreive a temperature value.
//
//  In this case, the extension is abstracting a call
//  to underlying communication/hardware infrastructure
//  to retrieve a sensor value.
//
//  Early in development we may choose to simulate the
//  sensor.  Later we may construct a extension implementation that
//  reads from actual hardware.
//=================================================

@ext("TempSensorDevice_Ext_Sim") object TempSensorDevice {
  def currentTempGet(): Temperature_i = $
}
