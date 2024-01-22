// #Sireum     -- required marker indicating that this file is a Slang file (needed to trigger Slang parsing).

package TempControlMixedUI.TempControlSystem

import org.sireum._
import TempControlMixedUI.TempSensor._

//==========================================================
//  D e f s
//
//  Defines values and predicates that are derived from
//  requirements and other external stakeholder artifacts
//
//==========================================================
object Defs {

  //==================================
  // S y s t e m    C o n s t a n t s
  //==================================
  val alarmThreshold: F32 = 1.0f

  //==================================
  // I n i t i a l    V a l u e s
  //==================================
  val initialTemp: Temperature_i = Temperature_i(85f, TempUnit.Fahrenheit)

  val initialSetPoint: SetPoint_i = SetPoint_i(
    Temperature_i(55f, TempUnit.Fahrenheit),
    Temperature_i(100f, TempUnit.Fahrenheit))
}
