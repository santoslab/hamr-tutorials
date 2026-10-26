/*============================================================
    SysPropStructSplit   S o f t w a r e

    Defines the software components (threads) of the struct-split
    pipeline:

      GenStruct --outstruct--> SplitStruct --xfield--> IncX --> ClampX --+
                                           \-yfield--> DecY --outyfield--+
                                                                          \
        MergeStruct <----------------------------------------------------+
            |
            +--outstruct--> ConsumeStruct

    All components are periodic Rust threads that communicate over
    DATA ports.  Each thread carries a GUMBO *component* contract
    (integration constraints on its ports and/or compute guarantees
    describing its functional behavior).

    Note: only ordinary GUMBO component contracts appear below.
    The system-level properties that compose them are declared in
    the `composition nominal` block of SysPropStructSplit.sysml.
  ============================================================*/

package SysPropStructSplit_Software {
  private import HAMR::*;

  //===============================================================================
  //
  //  S t y l e s
  //
  //  Common thread style reused by all components in the pipeline.
  //===============================================================================
  part def Periodic_Rust_Thread :> Thread {
    attribute :>> Period = 1000 [ms];
    attribute :>> Dispatch_Protocol = Supported_Dispatch_Protocols::Periodic;
    attribute :>> Implementation_Language = Implementation_Languages::Rust;
  }

  //===============================================================================
  //
  //  G e n S t r u c t   Thread
  //
  //  Source component.  Has no inputs.  Produces a StructXY on its
  //  output data port `outstruct`, with both fields in [-100, 100].
  //  (The implementation cycles deterministically through that range.)
  //===============================================================================
  part def GenStruct :> Periodic_Rust_Thread {
    //-- I n t e r f a c e  --
    port outstruct : DataPort { out :>> type : SysPropStructSplit_Data_Model::StructXY; }

    //-- B e h a v i o r    C o n s t r a i n t s --
    language "GUMBO" /*{
      integration
        guarantee outstruct_range "Both fields of the generated struct lie in [-100, 100].":
          (-100 [i32] <= outstruct.x) & (outstruct.x <= 100 [i32]) &
          (-100 [i32] <= outstruct.y) & (outstruct.y <= 100 [i32]);

      initialize
        guarantee init_outstruct "The output data port must be initialized.":
          (outstruct.x == 0 [i32]) & (outstruct.y == 0 [i32]);
    }*/
  }

  //===============================================================================
  //
  //  S p l i t S t r u c t   Thread
  //
  //  Receives a StructXY and forwards its two fields onto separate
  //  Integer_32 output data ports.
  //===============================================================================
  part def SplitStruct :> Periodic_Rust_Thread {
    //-- I n t e r f a c e  --
    port instruct : DataPort { in :>> type : SysPropStructSplit_Data_Model::StructXY; }
    port xfield   : DataPort { out :>> type : Base_Types::Integer_32; }
    port yfield   : DataPort { out :>> type : Base_Types::Integer_32; }

    //-- B e h a v i o r    C o n s t r a i n t s --
    language "GUMBO" /*{
      integration
        assume instruct_range "Incoming struct fields lie in [-100, 100].":
          (-100 [i32] <= instruct.x) & (instruct.x <= 100 [i32]) &
          (-100 [i32] <= instruct.y) & (instruct.y <= 100 [i32]);
        guarantee xfield_range "xfield lies in [-100, 100].":
          (-100 [i32] <= xfield) & (xfield <= 100 [i32]);
        guarantee yfield_range "yfield lies in [-100, 100].":
          (-100 [i32] <= yfield) & (yfield <= 100 [i32]);

      compute
        guarantee split_x "xfield equals the x field of the incoming struct.":
          xfield == instruct.x;
        guarantee split_y "yfield equals the y field of the incoming struct.":
          yfield == instruct.y;
    }*/
  }

  //===============================================================================
  //
  //  I n c X   Thread
  //
  //  Increments its scalar input by 1.
  //
  //  Contract design: this component is written to be REUSABLE -- it carries no
  //  integration constraints tying it to this system's [-100, 100] value range.
  //  Its only assumption is a compute assume that guards against overflow of the
  //  increment (GUMBO cannot currently reference the i32 maximum, so a
  //  conservative [-1000, 1000] envelope is used -- deliberately distinct from
  //  the [-100, 100] range of this system context).
  //===============================================================================
  part def IncX :> Periodic_Rust_Thread {
    //-- I n t e r f a c e  --
    port inxfield  : DataPort { in :>> type : Base_Types::Integer_32; }
    port outxfield : DataPort { out :>> type : Base_Types::Integer_32; }

    //-- B e h a v i o r    C o n s t r a i n t s --
    language "GUMBO" /*{
      compute
        assume inxfield_bounded "Overflow guard only: the input lies in a
          |conservative [-1000, 1000] envelope so the increment cannot overflow.":
          (-1000 [i32] <= inxfield) & (inxfield <= 1000 [i32]);
        guarantee incx "Output is the input incremented by one.":
          outxfield == inxfield + 1 [i32];
    }*/
  }

  //===============================================================================
  //
  //  C l a m p X   Thread
  //
  //  Saturating clamp: forwards its scalar input unchanged when it lies in
  //  [-100, 100] and saturates it to the nearest bound otherwise.  Sits after
  //  IncX in the x branch, restoring the pipeline's nominal [-100, 100] range.
  //  Its compute contract is written as GUMBO compute_cases (one case per
  //  region of the input domain).
  //
  //  Contract design: the component carries NO integration constraints -- the
  //  compute_cases are total (they cover every possible i32 input), the clamp
  //  performs no arithmetic that could overflow, and its output range is fully
  //  determined by the cases themselves.  It needs no assumptions about its
  //  environment and is therefore maximally reusable.
  //===============================================================================
  part def ClampX :> Periodic_Rust_Thread {
    //-- I n t e r f a c e  --
    port inxfield  : DataPort { in :>> type : Base_Types::Integer_32; }
    port outxfield : DataPort { out :>> type : Base_Types::Integer_32; }

    //-- B e h a v i o r    C o n s t r a i n t s --
    language "GUMBO" /*{
      compute
        compute_cases
          case In_Range "values already in [-100, 100] pass through unchanged":
            assume (-100 [i32] <= inxfield) & (inxfield <= 100 [i32]);
            guarantee outxfield == inxfield;
          case Above "values above 100 saturate to 100":
            assume inxfield > 100 [i32];
            guarantee outxfield == 100 [i32];
          case Below "values below -100 saturate to -100":
            assume inxfield < -100 [i32];
            guarantee outxfield == -100 [i32];
    }*/
  }

  //===============================================================================
  //
  //  D e c Y   Thread
  //
  //  Decrements its scalar input by 1.
  //
  //  Contract design: like IncX, this component is written to be REUSABLE --
  //  no integration constraints; its only assumption is a compute assume that
  //  guards against underflow of the decrement, using the same conservative
  //  [-1000, 1000] envelope.
  //===============================================================================
  part def DecY :> Periodic_Rust_Thread {
    //-- I n t e r f a c e  --
    port inyfield  : DataPort { in :>> type : Base_Types::Integer_32; }
    port outyfield : DataPort { out :>> type : Base_Types::Integer_32; }

    //-- B e h a v i o r    C o n s t r a i n t s --
    language "GUMBO" /*{
      compute
        assume inyfield_bounded "Underflow guard only: the input lies in a
          |conservative [-1000, 1000] envelope so the decrement cannot underflow.":
          (-1000 [i32] <= inyfield) & (inyfield <= 1000 [i32]);
        guarantee decy "Output is the input decremented by one.":
          outyfield == inyfield - 1 [i32];
    }*/
  }

  //===============================================================================
  //
  //  M e r g e S t r u c t   Thread
  //
  //  Re-assembles a StructXY from its two scalar inputs.
  //
  //  NOTE: this component intentionally carries NO integration
  //  constraints -- it makes no assumption about the ranges of its
  //  inputs and offers no range guarantee on its output.  Its only
  //  contract is the functional compute guarantee below.  Recovering
  //  the output range is the motivating use case for the system-level
  //  property feature (see the composition block in SysPropStructSplit.sysml).
  //===============================================================================
  part def MergeStruct :> Periodic_Rust_Thread {
    //-- I n t e r f a c e  --
    port inxfield : DataPort { in :>> type : Base_Types::Integer_32; }
    port inyfield : DataPort { in :>> type : Base_Types::Integer_32; }
    port outstruct : DataPort { out :>> type : SysPropStructSplit_Data_Model::StructXY; }

    //-- B e h a v i o r    C o n s t r a i n t s --
    language "GUMBO" /*{
      compute
        guarantee merge_x "Output struct's x field equals the inxfield input.":
          outstruct.x == inxfield;
        guarantee merge_y "Output struct's y field equals the inyfield input.":
          outstruct.y == inyfield;
    }*/
  }

  //===============================================================================
  //
  //  C o n s u m e S t r u c t   Thread
  //
  //  Sink component.  Receives a StructXY and logs it; has no output ports.
  //  Carries a GUMBO state variable `last_x` that records the x field of the
  //  most recently consumed struct -- persistent component state that the
  //  system-level loop-invariant property (see the composition block in
  //  SysPropStructSplit.sysml) reasons about across hyperperiod boundaries.
  //===============================================================================
  part def ConsumeStruct :> Periodic_Rust_Thread {
    //-- I n t e r f a c e  --
    port instruct : DataPort { in :>> type : SysPropStructSplit_Data_Model::StructXY; }

    //-- B e h a v i o r    C o n s t r a i n t s --
    language "GUMBO" /*{
      state
        last_x: Base_Types::Integer_32;

      initialize
        guarantee init_last_x "The state variable starts at zero.":
          last_x == 0 [i32];

      compute
        assume instruct_range "Incoming struct fields lie in [-200, 200].":
          (-200 [i32] <= instruct.x) & (instruct.x <= 200 [i32]) &
          (-200 [i32] <= instruct.y) & (instruct.y <= 200 [i32]);
        guarantee track_x "last_x records the x field of the most recently consumed struct.":
          last_x == instruct.x;
    }*/
  }
}
