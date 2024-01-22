// #Sireum

package ProdConsExample.Consumer

import org.sireum._
import art._
import ProdConsExample._

@sig trait ConsumerThr_i_Api {
  def id: Art.BridgeId
  def inMessage_Id : Art.PortId


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

@datatype class ConsumerThr_i_Initialization_Api (
  val id: Art.BridgeId,
  val inMessage_Id : Art.PortId) extends ConsumerThr_i_Api

@datatype class ConsumerThr_i_Operational_Api (
  val id: Art.BridgeId,
  val inMessage_Id : Art.PortId) extends ConsumerThr_i_Api {

  // Logika spec var representing port state for incoming event data port
  @spec var inMessage: Option[ProdCons.Message_i] = $

  def get_inMessage() : Option[ProdCons.Message_i] = {
    Contract(
      Ensures(
        Res == inMessage
      )
    )
    val value : Option[ProdCons.Message_i] = Art.getValue(inMessage_Id) match {
      case Some(ProdCons.Message_i_Payload(v)) => Some(v)
      case Some(v) =>
        Art.logError(id, s"Unexpected payload on port inMessage.  Expecting 'ProdCons.Message_i_Payload' but received ${v}")
        None[ProdCons.Message_i]()
      case _ => None[ProdCons.Message_i]()
    }
    return value
  }
}
