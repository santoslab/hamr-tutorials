// #Sireum

package ProdConsExample.Producer

import org.sireum._
import art._
import ProdConsExample._

@sig trait ProducerThr_i_Api {
  def id: Art.BridgeId
  def outMessage_Id : Art.PortId

  // Logika spec var representing port state for outgoing event data port
  @spec var outMessage: Option[ProdCons.Message_i] = $

  def put_outMessage(value : ProdCons.Message_i) : Unit = {
    Contract(
      Modifies(outMessage),
      Ensures(
        outMessage == Some(value)
      )
    )
    Spec {
      outMessage = Some(value)
    }

    Art.putValue(outMessage_Id, ProdCons.Message_i_Payload(value))
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

@datatype class ProducerThr_i_Initialization_Api (
  val id: Art.BridgeId,
  val outMessage_Id : Art.PortId) extends ProducerThr_i_Api

@datatype class ProducerThr_i_Operational_Api (
  val id: Art.BridgeId,
  val outMessage_Id : Art.PortId) extends ProducerThr_i_Api {

}
