// #Sireum

package TempControlMixedUI.TempControlSystem

import org.sireum._
import art._
import TempControlMixedUI._

@sig trait TempControl_i_Api {
  def id: Art.BridgeId
  def currentTemp_Id : Art.PortId
  def fanAck_Id : Art.PortId
  def setPoint_Id : Art.PortId
  def fanCmd_Id : Art.PortId
  def tempChanged_Id : Art.PortId

  // Logika spec var representing port state for outgoing event data port
  @spec var fanCmd: Option[CoolingFan.FanCmd.Type] = $

  def put_fanCmd(value : CoolingFan.FanCmd.Type) : Unit = {
    Contract(
      Modifies(fanCmd),
      Ensures(
        fanCmd == Some(value)
      )
    )
    Spec {
      fanCmd = Some(value)
    }

    Art.putValue(fanCmd_Id, CoolingFan.FanCmd_Payload(value))
  }

  def logInfo(msg: String): Unit = {
    Art.logInfo(id, msg)
  }

  def logDebug(msg: String): Unit = {
    Art.logDebug(id, msg)
  }

  def logError(msg: String): Unit = {
    Art.logError(id, msg)
  }
}

@datatype class TempControl_i_Initialization_Api (
  val id: Art.BridgeId,
  val currentTemp_Id : Art.PortId,
  val fanAck_Id : Art.PortId,
  val setPoint_Id : Art.PortId,
  val fanCmd_Id : Art.PortId,
  val tempChanged_Id : Art.PortId) extends TempControl_i_Api

@datatype class TempControl_i_Operational_Api (
  val id: Art.BridgeId,
  val currentTemp_Id : Art.PortId,
  val fanAck_Id : Art.PortId,
  val setPoint_Id : Art.PortId,
  val fanCmd_Id : Art.PortId,
  val tempChanged_Id : Art.PortId) extends TempControl_i_Api {

  // Logika spec var representing port state for incoming data port
  @spec var currentTemp: TempSensor.Temperature_i = $

  def get_currentTemp() : Option[TempSensor.Temperature_i] = {
    Contract(
      Ensures(
        Res == Some(currentTemp)
      )
    )
    val value : Option[TempSensor.Temperature_i] = Art.getValue(currentTemp_Id) match {
      case Some(TempSensor.Temperature_i_Payload(v)) => Some(v)
      case Some(v) =>
        Art.logError(id, s"Unexpected payload on port currentTemp.  Expecting 'TempSensor.Temperature_i_Payload' but received ${v}")
        None[TempSensor.Temperature_i]()
      case _ => None[TempSensor.Temperature_i]()
    }
    return value
  }

  // Logika spec var representing port state for incoming event data port
  @spec var fanAck: Option[CoolingFan.FanAck.Type] = $

  def get_fanAck() : Option[CoolingFan.FanAck.Type] = {
    Contract(
      Ensures(
        Res == fanAck
      )
    )
    val value : Option[CoolingFan.FanAck.Type] = Art.getValue(fanAck_Id) match {
      case Some(CoolingFan.FanAck_Payload(v)) => Some(v)
      case Some(v) =>
        Art.logError(id, s"Unexpected payload on port fanAck.  Expecting 'CoolingFan.FanAck_Payload' but received ${v}")
        None[CoolingFan.FanAck.Type]()
      case _ => None[CoolingFan.FanAck.Type]()
    }
    return value
  }

  // Logika spec var representing port state for incoming event data port
  @spec var setPoint: Option[TempControlSystem.SetPoint_i] = $

  def get_setPoint() : Option[TempControlSystem.SetPoint_i] = {
    Contract(
      Ensures(
        Res == setPoint
      )
    )
    val value : Option[TempControlSystem.SetPoint_i] = Art.getValue(setPoint_Id) match {
      case Some(TempControlSystem.SetPoint_i_Payload(v)) => Some(v)
      case Some(v) =>
        Art.logError(id, s"Unexpected payload on port setPoint.  Expecting 'TempControlSystem.SetPoint_i_Payload' but received ${v}")
        None[TempControlSystem.SetPoint_i]()
      case _ => None[TempControlSystem.SetPoint_i]()
    }
    return value
  }

  // Logika spec var representing port state for incoming event port
  @spec var tempChanged: Option[art.Empty] = $

  def get_tempChanged() : Option[art.Empty] = {
    Contract(
      Ensures(
        Res == tempChanged
      )
    )
    val value : Option[art.Empty] = Art.getValue(tempChanged_Id) match {
      case Some(Empty()) => Some(Empty())
      case Some(v) =>
        Art.logError(id, s"Unexpected payload on port tempChanged.  Expecting 'Empty' but received ${v}")
        None[art.Empty]()
      case _ => None[art.Empty]()
    }
    return value
  }
}
