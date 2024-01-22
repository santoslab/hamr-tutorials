// #Sireum

package TempControlMixedUI.TempControlSystem

import org.sireum._
import TempControlMixedUI._

// Do not edit this file as it will be overwritten if HAMR codegen is rerun

object SetPoint_i {
  def example(): TempControlSystem.SetPoint_i = {
    return TempControlSystem.SetPoint_i(
      low = TempSensor.Temperature_i.example(),
      high = TempSensor.Temperature_i.example())
  }
}

@datatype class SetPoint_i(
  val low: TempSensor.Temperature_i,
  val high: TempSensor.Temperature_i) {
}

object SetPoint_i_Payload {
  def example(): SetPoint_i_Payload = {
    return SetPoint_i_Payload(TempControlSystem.SetPoint_i.example())
  }
}

@datatype class SetPoint_i_Payload(value: TempControlSystem.SetPoint_i) extends art.DataContent