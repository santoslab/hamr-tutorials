// #Sireum

package ProdConsExample.Consumer

import org.sireum._
import ProdConsExample._

// This file will not be overwritten so is safe to edit
object ConsumerThr_i_cons_consumerThr {

  //======================================================
  // C o m p o n e n t    L o c a l    V a r i a b l e s
  //======================================================
  var numMessagesReceived: Z = 0 // use Slang unbounded integer type "Z"

  //======================================================
  // I n i t i a l i z e    E n t r y   P o i n t
  //======================================================
  def initialise(api: ConsumerThr_i_Initialization_Api): Unit = {
    api.logInfo("Consumer Initialize Entry Point")

    // initialize component local state
    numMessagesReceived = 0 // start message counter at 0

    // initialize output data ports
    // (no output data ports to initialize)
  }

  //======================================================
  // C o m p u t e    E n t r y   P o i n t
  //======================================================
  def handle_inMessage(api: ConsumerThr_i_Operational_Api, value: ProdCons.Message_i): Unit = {
    // access fields of incoming message for logging output
    api.logInfo(s"Consumed inMessage value   ${value.value}")
    api.logInfo(s"Consumed inMessage section ${value.section}")

    // increment message count and log total received so far
    numMessagesReceived = numMessagesReceived + 1
    api.logInfo(s"# messages received by Consumer: ${numMessagesReceived}")
  }

  //======================================================
  // F i n a l i z e    E n t r y   P o i n t
  //======================================================
  def finalise(api: ConsumerThr_i_Operational_Api): Unit = { }

}
