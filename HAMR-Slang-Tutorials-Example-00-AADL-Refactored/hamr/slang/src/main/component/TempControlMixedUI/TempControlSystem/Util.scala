// #Sireum     -- required marker indicating that this file is a Slang file (needed to trigger Slang parsing).

package TempControlMixedUI.TempControlSystem

import org.sireum._
import TempControlMixedUI.TempSensor._


//==========================================================
//  U t i l
//
//  "Library" routines, etc. not associated with a particular
//  system component.
//
//  Note: The inclusion of of Util object is not strictly necessary
//  for HAMR.   It simply reflects a code organization choice
//  made by the authors of this example.
//
//==========================================================
object Util {


  @pure def toFahrenheit(value: Temperature_i) : Temperature_i = {
    if(value.unit == TempUnit.Fahrenheit) {
      return value
    } else if (value.unit == TempUnit.Celsius) {
      return Temperature_i(value.degrees * 9f / 5f + 32f, TempUnit.Fahrenheit)
    } else {
      return Temperature_i(value.degrees * 9f / 5f - 459.67f, TempUnit.Fahrenheit)
    }
  }
}
