//@ HAMR: --platform Microkit --output-dir ../../hamr

/*============================================================
    SysPropStructSplit   S y s t e m

    Top-level architecture for the struct-split pipeline.  Each
    software component (thread) is wrapped in its own process
    (one thread per process, as required for Microkit), the
    processes are connected into a pipeline, and all processes are
    allocated to a single processor.

    Dataflow:

      gen --outstruct--> split --xfield--> incx --> clampx --+
                               \-yfield--> decy --outyfield--+
                                                              v
        consume <--outstruct-- merge <----- inxfield/inyfield-+
  ============================================================*/

package SysPropStructSplit {

  private import HAMR::*;
  private import SysPropStructSplit_Software::*;

  //===============================================================================
  //
  //  S y s t e m
  //
  //===============================================================================
  part def StructSplit_System :> System {
    //-- P r o c e s s e s  (partitions) --
    part gen      : Gen_Process;
    part splitter : Split_Process;
    part incx     : IncX_Process;
    part clampx   : ClampX_Process;
    part decy     : DecY_Process;
    part merger   : Merge_Process;
    part consume  : Consume_Process;
    part struct_split_processor : Platform::Processor_1;

    //-- C o n n e c t i o n s  (information flows between partitions) --
    connection c1 : PortConnection connect gen.outstruct     to splitter.instruct;
    connection c2 : PortConnection connect splitter.xfield   to incx.inxfield;
    connection c3 : PortConnection connect splitter.yfield   to decy.inyfield;
    connection c4 : PortConnection connect incx.outxfield    to clampx.inxfield;
    connection c5 : PortConnection connect clampx.outxfield  to merger.inxfield;
    connection c6 : PortConnection connect decy.outyfield    to merger.inyfield;
    connection c7 : PortConnection connect merger.outstruct  to consume.instruct;

    //-- A l l o c a t i o n s  of software/partitions to hardware --
    allocation pb0 : Deployment_Properties::Actual_Processor_Binding
      allocate gen to struct_split_processor;
    allocation pb1 : Deployment_Properties::Actual_Processor_Binding
      allocate splitter to struct_split_processor;
    allocation pb2 : Deployment_Properties::Actual_Processor_Binding
      allocate incx to struct_split_processor;
    allocation pb3 : Deployment_Properties::Actual_Processor_Binding
      allocate clampx to struct_split_processor;
    allocation pb4 : Deployment_Properties::Actual_Processor_Binding
      allocate decy to struct_split_processor;
    allocation pb5 : Deployment_Properties::Actual_Processor_Binding
      allocate merger to struct_split_processor;
    allocation pb6 : Deployment_Properties::Actual_Processor_Binding
      allocate consume to struct_split_processor;

    //===============================================================================
    //
    //  S y s t e m - L e v e l   P r o p e r t i e s   (GUMBO composition)
    //
    //  MergeStruct intentionally carries NO integration constraints, so the range of
    //  its output struct can only be recovered from upstream component guarantees at
    //  the system level.  The composition below declares an abstract schedule of the
    //  pipeline plus short aliases for the components and the ports whose values the
    //  properties reason about, and then states system-wide properties as assertions
    //  attached to points in that schedule.
    //===============================================================================
    language "GUMBO" /*{

      functions

        // Repeated range predicates, factored as GUMBO functions (spec-pattern reuse).
        def inRange100(v: Base_Types::Integer_32): Base_Types::Boolean :=
          (-100 [i32] <= v) & (v <= 100 [i32]);

        def inConsumeRange(s: SysPropStructSplit_Data_Model::StructXY): Base_Types::Boolean :=
          (-200 [i32] <= s.x) & (s.x <= 200 [i32]) &
          (-200 [i32] <= s.y) & (s.y <= 200 [i32]);

        def inRange200(v: Base_Types::Integer_32): Base_Types::Boolean :=
          (-200 [i32] <= v) & (v <= 200 [i32]);

        // The end-to-end functional result, named as a spec function: the struct
        // arriving at the consumer equals (g.x + 1 saturated at 100, g.y - 1).
        // ClampX's saturation only bites at g.x == 100, so the x field is stated
        // as a case split on g.x.
        def mergedReflectsGen(m: SysPropStructSplit_Data_Model::StructXY,
                              g: SysPropStructSplit_Data_Model::StructXY): Base_Types::Boolean :=
          ((g.x < 100 [i32]) implies (m.x == g.x + 1 [i32])) &
          ((g.x == 100 [i32]) implies (m.x == 100 [i32])) &
          (m.y == g.y - 1 [i32]);

      composition nominal {

        // Short alias  =  <process-instance>.<thread-instance>
        components
          gen         = gen.gen;
          split_stage = splitter.splitter;
          incx        = incx.incx;
          clampx      = clampx.clampx;
          decy        = decy.decy;
          merge_stage = merger.merger;
          consume     = consume.consume;

        // Named handles for the port values the properties refer to.
        ports
          gen_out = gen.outstruct;
          x0      = split_stage.xfield;
          y0      = split_stage.yfield;
          x1      = incx.outxfield;
          x2      = clampx.outxfield;
          y1      = decy.outyfield;
          merged  = merge_stage.outstruct;

        // Named handles for the component GUMBO state variables the properties refer to.
        state
          consume_last_x = consume.last_x;

        // Abstract schedule: after the splitter, the x branch (incx then clampx --
        // a multi-element branch) and the decy branch run independently and rejoin
        // (branches_joined) before the merger.
        schema {
          gen;
          split_stage;
          split {
            sequence { incx; clampx },
            sequence { decy }
          };
          label branches_joined;
          merge_stage;
          consume
        }

        // Smoke test: a minimal single-step property discharged directly by
        // GenStruct's integration guarantee -- exercises the whole toolchain path.
        property Gen_Range_Sanity
          "toolchain smoke test: the generator output lies in [-100, 100]" {
          after gen "GenStruct integration guarantee":
            inRange100(gen_out.x) & inRange100(gen_out.y);
        }

        //-----------------------------------------------------------------------------
        //  Property 1 -- recover the merger's output range from upstream contracts and
        //  show it satisfies ConsumeStruct's defensive [-200, 200] assume.
        //
        //  The system proof is modular per schema place: to use a component's contract
        //  the proof must discharge that component's input `assume` from the assertion
        //  carried into its `before` place, and every `after`/join assertion must be
        //  re-established to carry forward.  Pipeline_Ranges factors this shared range
        //  plumbing so the concrete properties only state their end conclusion.
        //-----------------------------------------------------------------------------

        abstract property Pipeline_Ranges
          "shared range plumbing: carry [-100,100] from gen through split into each
          |branch (discharging IncX's and DecY's [-1000,1000] compute assumes), then
          |carry the transformed ranges across the branch join to the merger" {
          after gen "GenStruct guarantees both fields in [-100, 100]":
            inRange100(gen_out.x) & inRange100(gen_out.y);
          after split_stage "SplitStruct forwards each field, preserving [-100, 100]":
            inRange100(x0) & inRange100(y0);
          before incx "branch entry: x still in [-100, 100] -- discharges IncX's
            |[-1000, 1000] overflow-guard compute assume":
            inRange100(x0);
          after incx "IncX's compute guarantee (x1 == x0 + 1) plus the carried
            |[-100, 100] range yield x1 in [-99, 101]; this place IS `before clampx`
            |(consecutive branch elements share a place), and ClampX -- whose
            |compute_cases are total -- needs no assume discharged here":
            (-99 [i32] <= x1) & (x1 <= 101 [i32]);
          after clampx "ClampX saturates back into [-100, 100]":
            inRange100(x2);
          before decy "branch entry: y still in [-100, 100] -- discharges DecY's
            |[-1000, 1000] underflow-guard compute assume":
            inRange100(y0);
          after decy "DecY's compute guarantee (y1 == y0 - 1) plus the carried
            |[-100, 100] range yield y1 in [-101, 99]":
            (-101 [i32] <= y1) & (y1 <= 99 [i32]);
          at branches_joined "join: both transformed ranges are available to the merger":
            inRange100(x2) &
            (-101 [i32] <= y1) & (y1 <= 99 [i32]);
        }

        property Merged_In_Consume_Range :> Pipeline_Ranges
          "the merger carries no integration constraint; its output range is recovered
          |here from the upstream guarantees and shown to satisfy ConsumeStruct's
          |defensive [-200, 200] assume" {
          before consume "ConsumeStruct's precondition, established at its dispatch
            |from MergeStruct's guarantees merged.x == x2 and merged.y == y1
            |(this place IS `after merge_stage` -- consecutive schema elements
            |share a place)":
            inConsumeRange(merged);
        }

        //-----------------------------------------------------------------------------
        //  Property 2 -- end-to-end functional result: the consumer ultimately sees
        //  (gen.x + 1 saturated at 100, gen.y - 1); ClampX's saturation only bites
        //  at gen.x == 100, so the x result is stated as a case split on gen.x.
        //  Pipeline_Functional EXTENDS Pipeline_Ranges: the range plumbing is still
        //  needed to discharge each component's input assume (and to select ClampX's
        //  compute case), and the value-equality carries are conjoined onto the same
        //  schema places.
        //-----------------------------------------------------------------------------

        abstract property Pipeline_Functional :> Pipeline_Ranges
          "adds value-equality carries on top of the inherited range plumbing" {
          after split_stage "SplitStruct forwards the struct fields unchanged":
            (x0 == gen_out.x) & (y0 == gen_out.y);
          before incx "branch entry: x still equals the generated x":
            x0 == gen_out.x;
          after incx "IncX adds one":
            x1 == gen_out.x + 1 [i32];
          after clampx "ClampX's compute cases yield the saturated value: the clamp
            |only bites when gen.x == 100 (i.e., x1 == 101)":
            ((gen_out.x < 100 [i32]) implies (x2 == gen_out.x + 1 [i32])) &
            ((gen_out.x == 100 [i32]) implies (x2 == 100 [i32]));
          before decy "branch entry: y still equals the generated y":
            y0 == gen_out.y;
          after decy "DecY subtracts one":
            y1 == gen_out.y - 1 [i32];
          at branches_joined "join: the transformed values are available to the merger":
            ((gen_out.x < 100 [i32]) implies (x2 == gen_out.x + 1 [i32])) &
            ((gen_out.x == 100 [i32]) implies (x2 == 100 [i32])) &
            (y1 == gen_out.y - 1 [i32]);
        }

        property End_To_End_Functional :> Pipeline_Functional
          "end-to-end: the merged struct equals (gen.x + 1 saturated at 100, gen.y - 1)" {
          before consume "MergeStruct reassembles the transformed fields; the named
            |spec function states the end-to-end result at the end of the chain --
            |the struct arriving at the consumer reflects the generated struct":
            mergedReflectsGen(merged, gen_out);
        }

        //-----------------------------------------------------------------------------
        //  Property 3 -- a loop invariant over ConsumeStruct's `last_x` state variable:
        //  in every hyperperiod, the value it latched lies in [-200, 200].
        //
        //  Structure: `at START` holds on frame entry (established initially by
        //  ConsumeStruct's initialize guarantee `last_x == 0`); when consume fires,
        //  its compute guarantee `last_x == instruct.x` plus the carried
        //  inConsumeRange(merged) fact re-establish the invariant; `at END` closes
        //  the frame, and the Post-Pre VC (END |- START) extends it to every cycle.
        //
        //  The concrete property uses MULTIPLE inheritance: Pipeline_Ranges supplies
        //  the range chain that establishes inConsumeRange(merged) at consume's
        //  dispatch, and Consume_State_Base supplies the state-variable plumbing.
        //-----------------------------------------------------------------------------

        abstract property Consume_State_Base
          "loop-invariant plumbing for ConsumeStruct's last_x state variable: hold
          |the invariant at frame entry, carry the merged range to consume's dispatch,
          |and re-establish the invariant when consume fires" {
          at START "frame entry: established initially by consume's initialize guarantee":
            inRange200(consume_last_x);
          before consume "the merged range fact at consume's dispatch; discharges
            |consume's [-200, 200] compute assume":
            inConsumeRange(merged);
          after consume "consume's guarantee last_x == instruct.x re-establishes the invariant":
            inRange200(consume_last_x);
        }

        property Consume_Range_Loop_Invariant :> Pipeline_Ranges, Consume_State_Base
          "every cycle, ConsumeStruct's latched last_x lies in [-200, 200]; the
          |Post-Pre VC (END |- START) makes the invariant hold across hyperperiod
          |boundaries" {
          at END "loop closure: carried from `after consume` to the frame boundary":
            inRange200(consume_last_x);
        }
      }
    }*/
  }

  //===============================================================================
  //
  //  G e n   Process
  //
  //===============================================================================
  part def Gen_Process :> Process {
    //-- I n t e r f a c e  --
    port outstruct : DataPort { out :>> type : SysPropStructSplit_Data_Model::StructXY; }

    //-- P r o p e r t i e s  --
    attribute :>> Domain = SysPropStructSplit_Properties::Domain_Gen;

    //-- I n t e r n a l s --
    part gen : GenStruct;
    connection co : PortConnection connect gen.outstruct to outstruct;
  }

  //===============================================================================
  //
  //  S p l i t   Process
  //
  //===============================================================================
  part def Split_Process :> Process {
    //-- I n t e r f a c e  --
    port instruct : DataPort { in :>> type : SysPropStructSplit_Data_Model::StructXY; }
    port xfield   : DataPort { out :>> type : Base_Types::Integer_32; }
    port yfield   : DataPort { out :>> type : Base_Types::Integer_32; }

    //-- P r o p e r t i e s  --
    attribute :>> Domain = SysPropStructSplit_Properties::Domain_Split;

    //-- I n t e r n a l s --
    part splitter : SplitStruct;
    connection ci : PortConnection connect instruct to splitter.instruct;
    connection cx : PortConnection connect splitter.xfield to xfield;
    connection cy : PortConnection connect splitter.yfield to yfield;
  }

  //===============================================================================
  //
  //  I n c X   Process
  //
  //===============================================================================
  part def IncX_Process :> Process {
    //-- I n t e r f a c e  --
    port inxfield  : DataPort { in :>> type : Base_Types::Integer_32; }
    port outxfield : DataPort { out :>> type : Base_Types::Integer_32; }

    //-- P r o p e r t i e s  --
    attribute :>> Domain = SysPropStructSplit_Properties::Domain_IncX;

    //-- I n t e r n a l s --
    part incx : IncX;
    connection ci : PortConnection connect inxfield to incx.inxfield;
    connection co : PortConnection connect incx.outxfield to outxfield;
  }

  //===============================================================================
  //
  //  C l a m p X   Process
  //
  //===============================================================================
  part def ClampX_Process :> Process {
    //-- I n t e r f a c e  --
    port inxfield  : DataPort { in :>> type : Base_Types::Integer_32; }
    port outxfield : DataPort { out :>> type : Base_Types::Integer_32; }

    //-- P r o p e r t i e s  --
    attribute :>> Domain = SysPropStructSplit_Properties::Domain_ClampX;

    //-- I n t e r n a l s --
    part clampx : ClampX;
    connection ci : PortConnection connect inxfield to clampx.inxfield;
    connection co : PortConnection connect clampx.outxfield to outxfield;
  }

  //===============================================================================
  //
  //  D e c Y   Process
  //
  //===============================================================================
  part def DecY_Process :> Process {
    //-- I n t e r f a c e  --
    port inyfield  : DataPort { in :>> type : Base_Types::Integer_32; }
    port outyfield : DataPort { out :>> type : Base_Types::Integer_32; }

    //-- P r o p e r t i e s  --
    attribute :>> Domain = SysPropStructSplit_Properties::Domain_DecY;

    //-- I n t e r n a l s --
    part decy : DecY;
    connection ci : PortConnection connect inyfield to decy.inyfield;
    connection co : PortConnection connect decy.outyfield to outyfield;
  }

  //===============================================================================
  //
  //  M e r g e   Process
  //
  //===============================================================================
  part def Merge_Process :> Process {
    //-- I n t e r f a c e  --
    port inxfield : DataPort { in :>> type : Base_Types::Integer_32; }
    port inyfield : DataPort { in :>> type : Base_Types::Integer_32; }
    port outstruct : DataPort { out :>> type : SysPropStructSplit_Data_Model::StructXY; }

    //-- P r o p e r t i e s  --
    attribute :>> Domain = SysPropStructSplit_Properties::Domain_Merge;

    //-- I n t e r n a l s --
    part merger : MergeStruct;
    connection cix : PortConnection connect inxfield to merger.inxfield;
    connection ciy : PortConnection connect inyfield to merger.inyfield;
    connection co  : PortConnection connect merger.outstruct to outstruct;
  }

  //===============================================================================
  //
  //  C o n s u m e   Process
  //
  //===============================================================================
  part def Consume_Process :> Process {
    //-- I n t e r f a c e  --
    port instruct : DataPort { in :>> type : SysPropStructSplit_Data_Model::StructXY; }

    //-- P r o p e r t i e s  --
    attribute :>> Domain = SysPropStructSplit_Properties::Domain_Consume;

    //-- I n t e r n a l s --
    part consume : ConsumeStruct;
    connection ci : PortConnection connect instruct to consume.instruct;
  }
}
