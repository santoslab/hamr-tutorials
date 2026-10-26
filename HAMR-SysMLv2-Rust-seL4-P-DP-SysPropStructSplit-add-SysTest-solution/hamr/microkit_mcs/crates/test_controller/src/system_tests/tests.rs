// This file will not be overwritten if HAMR codegen is rerun

//! System tests for the Struct Split pipeline, run on seL4 under the test
//! scheduler.
//!
//! The schedule is gen, splitter, incx, decy, clampx, merger, consume, then
//! a pad.  Every test starts at slot 0 of a fresh hyperperiod (the runner
//! normalizes the schedule position), so `run_to_thread(splitter)` parks
//! after GenStruct has run and before anything has read its output.
//! Injecting there replaces the struct GenStruct sent, for every downstream
//! thread and for the system assertions of the `nominal` composition, which
//! judge the injected struct as the frame's `gen_out`.
//!
//! The controller checks every dispatch against the GUMBO component
//! contracts and the composition's system assertions, so each test also
//! fails on any violation it did not declare with `observe::expect`.

use crate::system_tests::api;
use crate::system_tests::harness::run_property;
use crate::system_tests::inspect;
use crate::system_tests::inspect::channels;
use crate::system_tests::observe;
use crate::{system_tests, sys_assert, sys_assert_eq};
use data::SysPropStructSplit_Data_Model::StructXY;

fn sxy(x: i32, y: i32) -> StructXY {
  StructXY { x, y }
}

/// Injects `g` as the struct GenStruct sent this hyperperiod.
fn inject_generated(g: StructXY) {
  let _ = api::run_to_thread(channels::splitter_splitter_MON);
  inspect::set_splitter_splitter(
    inspect::splitter_splitter_PreState { instruct: g });
}

/// Runs to the end of ConsumeStruct's dispatch.
fn run_through_consume() {
  let _ = api::run_to_thread(channels::consume_consume_MON);
  let _ = api::sstep(1);
}

/// What the pipeline should deliver for a generated struct `g`: x
/// incremented and saturated at 100 by ClampX, y decremented.
fn expected_merge(g: StructXY) -> StructXY {
  sxy(if g.x < 100 { g.x + 1 } else { 100 }, g.y - 1)
}

fn merged() -> Option<StructXY> {
  inspect::get_merger_merger_outstruct()
}

fn last_x() -> Option<i32> {
  inspect::get_consume_consume_sv_last_x()
}

system_tests! {
  suite smoke {
    fn schedule_advances_by_hyperperiod() {
      let before = api::info_state();
      let after = api::hstep(1);
      sys_assert!(after.flags & api::FLAG_BAD_COMMAND == 0);
      sys_assert_eq!(after.hyperperiod_num, before.hyperperiod_num + 1);
    }

    fn stepping_one_slot_at_a_time_wraps() {
      let n = api::schedule().num_timeslices;
      let before = api::info_state();
      let mut i = 0;
      while i < n {
        let _ = api::sstep(1);
        i += 1;
      }
      let after = api::info_state();
      sys_assert_eq!(after.current_timeslice, before.current_timeslice);
      sys_assert_eq!(after.hyperperiod_num, before.hyperperiod_num + 1);
    }
  }

  suite pipeline {
    // Follows one struct through every stage, checking each thread's output
    // as soon as it has run.
    fn struct_flows_through_each_stage() {
      inject_generated(sxy(37, -12));

      let _ = api::sstep(1); // splitter
      sys_assert_eq!(inspect::get_splitter_splitter_xfield(), Some(37));
      sys_assert_eq!(inspect::get_splitter_splitter_yfield(), Some(-12));

      let _ = api::sstep(1); // incx
      sys_assert_eq!(inspect::get_incx_incx_outxfield(), Some(38));

      let _ = api::sstep(1); // decy
      sys_assert_eq!(inspect::get_decy_decy_outyfield(), Some(-13));

      let _ = api::sstep(1); // clampx: 38 is in range, so it passes through
      sys_assert_eq!(inspect::get_clampx_clampx_outxfield(), Some(38));

      let _ = api::sstep(1); // merger
      sys_assert_eq!(merged(), Some(sxy(38, -13)));

      let _ = api::sstep(1); // consume latches the x field it received
      sys_assert_eq!(last_x(), Some(38));
    }

    // At the top of GenStruct's range IncX produces 101, which ClampX
    // saturates back to 100: the only case where the x field arrives
    // unchanged.
    fn x_saturates_at_the_upper_bound() {
      inject_generated(sxy(100, 100));
      run_through_consume();
      sys_assert_eq!(inspect::get_incx_incx_outxfield(), Some(101));
      sys_assert_eq!(merged(), Some(sxy(100, 99)));
      sys_assert_eq!(last_x(), Some(100));
    }

    // At the bottom of the range nothing saturates; y leaves [-100, 100] but
    // stays within ConsumeStruct's [-200, 200] assumption.
    fn lower_bound_passes_through() {
      inject_generated(sxy(-100, -100));
      run_through_consume();
      sys_assert_eq!(merged(), Some(sxy(-99, -101)));
      sys_assert_eq!(last_x(), Some(-99));
    }
  }

  suite properties {
    // The dynamic counterpart of the End_To_End_Functional system property:
    // for random generated structs, the struct ConsumeStruct receives is
    // (x + 1 saturated at 100, y - 1).  Each case drives the real system for
    // one hyperperiod; the case's own assertions decide pass or fail, and
    // the system assertions are checked on every case as well.
    fn end_to_end_holds_for_random_structs() {
      let structs = || (-100i32..=100, -100i32..=100);
      sys_assert!(run_property(32, structs, |(x, y)| {
        let g = sxy(x, y);
        inject_generated(g);
        run_through_consume();
        proptest::prop_assert_eq!(merged(), Some(expected_merge(g)));
        proptest::prop_assert_eq!(last_x(), Some(expected_merge(g).x));
        Ok(())
      }));
    }
  }

  suite robustness {
    // SplitStruct assumes its input lies in [-100, 100] (an integration
    // assume).  A struct outside it fails SplitStruct's CEP_Pre, so the
    // controller rejects the dispatch: SplitStruct does not run, and the
    // out-of-range x never enters the pipeline.  The test declares the
    // violation it causes.
    fn out_of_range_struct_is_rejected_by_splitter() {
      observe::expect(observe::Expect::CepPre(
        observe::Thread::splitter_splitter));
      inject_generated(sxy(150, 0));
      let _ = api::sstep(1); // splitter's slot passes; it does not run
      sys_assert!(inspect::get_splitter_splitter_xfield() != Some(150));
    }

    // Corrupts y on its way into MergeStruct, as a faulty channel might: it
    // arrives undecremented.  Every thread still meets its own contract --
    // MergeStruct copies what it receives, and the value is within every
    // range -- so only the system layer can tell: End_To_End_Functional's
    // assertion before consume no longer relates the merged struct to the
    // generated one.
    //
    // observe::expect would accept any End_To_End_Functional violation, at
    // any of the property's assertion points.  This test takes the
    // violations instead and checks there is exactly the one it caused, so
    // a fault elsewhere in the pipeline (an IncX that adds 2, say) still
    // fails it.
    fn corrupted_merge_input_breaks_end_to_end() {
      inject_generated(sxy(10, 20));
      let _ = api::run_to_thread(channels::merger_merger_MON);
      inspect::set_merger_merger(
        inspect::merger_merger_PreState { inxfield: 11, inyfield: 20 });
      run_through_consume();
      sys_assert_eq!(merged(), Some(sxy(11, 20)));

      let violations = observe::take();
      sys_assert_eq!(violations.count(), 1);
      sys_assert!(violations.iter().all(|v|
        v.kind == observe::Kind::SysAssert &&
        v.property == observe::sys::nominal::End_To_End_Functional &&
        v.point == "before consume"));
    }
  }
}
