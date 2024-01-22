// #Sireum

package ProdConsExample.Producer

import org.sireum._
import ProdConsExample._
import org.sireum.S32._

// This file will not be overwritten so is safe to edit
object ProducerThr_i_prod_producerThr {

  //======================================================
  // C o m p o n e n t    L o c a l    V a r i a b l e s
  //======================================================
  var value: Base_Types.Integer_32 = s32"0" // representation of Slang literal 0 for the Slang `S32` type
  var section: Base_Types.Integer_32 = s32"0"

  //======================================================
  // I n i t i a l i z e    E n t r y   P o i n t
  //======================================================
  def initialise(api: ProducerThr_i_Initialization_Api): Unit = {
    api.logInfo("Producer Initialize Entry Point")

    // initialize component local state
    value = s32"0"    // start value at 0
    section = s32"0"  // start section at 0

    // initialize output data ports
    //  (no output data ports to initialize)
  }

  //======================================================
  // C o m p u t e    E n t r y   P o i n t
  //======================================================
  def timeTriggered(api: ProducerThr_i_Operational_Api): Unit = {
    // construct message "record"
    val m: ProdCons.Message_i =  ProdCons.Message_i(value,section)
    // put the message on the `outMessage` port
    api.put_outMessage(m)

    api.logInfo("Producer Compute Entry Point puts message on outMessage port")

    // increment value
    value = value + s32"1"
    if (value == s32"20") {
      // rollover to 0
      value = s32"0"
      // if value rolls over, increment section
      section = section + s32"1"
    }
  }

  //======================================================
  // F i n a l i z e    E n t r y   P o i n t
  //======================================================
  def finalise(api: ProducerThr_i_Operational_Api): Unit = { }

  // other AADL entry points not currently used
  def activate(api: ProducerThr_i_Operational_Api): Unit = { }

  def deactivate(api: ProducerThr_i_Operational_Api): Unit = { }

  def recover(api: ProducerThr_i_Operational_Api): Unit = { }
}
