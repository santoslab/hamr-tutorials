/*============================================================
    P l a t f o r m

    Define computational resources and other attributes
    associated with a platform.  Independent of any particular
    system.
  ============================================================*/

package Platform {
  private import HAMR::*;

  //===============================================================================
  //
  //  P r o c e s s o r  1
  //
  //===============================================================================
  part def Processor_1 :> Processor {
    attribute :>> Frame_Period = 1 [s];
    attribute :>> Clock_Period = 1 [s];
  }
}
