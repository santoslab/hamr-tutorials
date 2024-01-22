package TempControlMixedUI.TempControlSystem

import org.sireum._
import TempControlMixedUI.TempSensor.{TempUnit, Temperature_i}
import TempControlMixedUI.CoolingFan.FanCmd

// This file will not be overwritten so is safe to edit
class TempControl_i_tcproc_tempControl_Test extends TempControl_i_tcproc_tempControl_ScalaTest {

  //========================================================
  // Auto-generated Example Tests
  //========================================================
  test("Example Unit Test for Initialise Entry Point"){
    // Initialise Entry Point doesn't read input port values, so just proceed with
    // launching the entry point code
    testInitialise()
    // use get_XXX methods and check_concrete_output() from test/util/../YYY_TestApi
    // retrieve values from output ports and check against expected results
  }

  //========================================================
  // Developer-written Tests
  //========================================================
  test("Example Unit Test for Compute Entry Point") {
    // use put_XXX methods from test/util/../YYY_TestApi to seed input ports with values
    testCompute()
    // use get_XXX methods and check_concrete_output() from test/util/../YYY_TestApi
    // retrieve values from output ports and check against expected results
  }

  def setPointandCurrentTempInteractions(low: Float, high: Float, current: Float): Unit = {
    //-----------------
    // Interaction 1: cause a set point to be stored in TempControl component's local state
    //-----------------

    // create set point structure
    val low_setPoint = Temperature_i(low, TempUnit.Fahrenheit)
    val high_setPoint = Temperature_i(high, TempUnit.Fahrenheit)
    val setPoint = SetPoint_i(low_setPoint, high_setPoint)
    // put setPoint value on input event data port
    put_setPoint(setPoint)
    // execute compute entry point to process updated setPoint
    // ..this will cause the setPoint values to be stored in the
    //   component local state
    testCompute()

    //-----------------
    // Interaction 2: send a temperature value
    //-----------------

    // create current temp value
    val currentTemp = Temperature_i(current, TempUnit.Fahrenheit)
    // put currentTemp value on input data port
    put_currentTemp(currentTemp)
    // put notification of new temp value on input event port
    put_tempChanged()
    // execute compute entry point to process temperature
    testCompute()
  }

  test("High after set point") {
    // Test outputs of TempControl when temperature received lies
    // above the bounds of the set point region:
    //  ...in such a case, an "On" command should be sent to the Fan
    //
    // This scenario is realized using two interactions with the TempControl component.
    //  1. process the set point  (no output to observe)
    //  2. process a current temperature value
    //      (check the output to see that an on command is sent to the fan)
    //

    setPointandCurrentTempInteractions(60.0F, 75.0F, 80.0F)

    //-----------------
    // Check Results
    //-----------------
    val optFanCmd = get_fanCmd()
    // first assert that the fanCmd is not empty (i.e., a command was sent)
    assert(!get_fanCmd().isEmpty)
    // get the command
    val fanCmd = optFanCmd.get
    // assert that then fan cmd is to turn on
    assert(fanCmd == FanCmd.On)
  }

}
