# HAMR Exercise: System Testing the Struct Split Example (Part 2 - Property-Based Tests and Contract Checking)

**Purpose**: In Part 1 you wrote tests that inject a struct into the running Struct Split system and check what each component produces.  In this part you will go beyond hand-picked inputs and hand-written expectations:
 - drive the pipeline with **randomly generated** structs, the run-time counterpart of the `End_To_End_Functional` property you proved earlier,
 - **seed a bug** and see the contract checks that run at every dispatch report it -- in tests that never mention it,
 - reproduce a failing random case from its seed, and see what the tests catch with the checks switched off,
 - write **robustness tests** that break contracts on purpose: one that a component's assumption rejects, and one that only the system-level assertions can detect.

## Prerequisites and Resources

You should have completed [Part 1](01-Ex-SysPropStructSplit-add-SysTest-write.md) of this exercise; this part continues in the same project folder, `HAMR-SysMLv2-Rust-seL4-P-DP-SysPropStructSplit-add-SysTest`.

Sections 7 and 8 of the documentation chapter [HAMR System Testing on seL4](https://hamr.sireum.org/hamr-doc/hamr-system-testing/) are the background for this part (property-based system tests, and contract checking during tests).  The [Automated GUMBOX Property-Based Testing](https://hamr.sireum.org/hamr-doc/gumbox-property-based-testing/) chapter introduces PropTest at the component level.

A completed solution for both parts is in the folder `HAMR-SysMLv2-Rust-seL4-P-DP-SysPropStructSplit-add-SysTest-solution` in the [HAMR tutorials repository](https://github.com/santoslab/hamr-tutorials/tree/main/HAMR-SysMLv2-Rust-seL4-P-DP-SysPropStructSplit-add-SysTest-solution).

## Activity 1 - Write a Property-Based System Test

In Part 1 you chose three structs by hand.  The `End_To_End_Functional` property you verified in the system-property exercises makes a claim about *every* struct GenStruct can produce: the struct reaching ConsumeStruct is `(x + 1 saturated at 100, y - 1)`.  A property-based test checks that claim on many random structs, against the real code.

* **Task:** In `tests.rs`, add two `use` lines and a helper that states the expected result:

```rust
use crate::system_tests::harness::run_property;
use crate::system_tests::observe;

/// What the pipeline should deliver for a generated struct `g`: x
/// incremented and saturated at 100 by ClampX, y decremented.
fn expected_merge(g: StructXY) -> StructXY {
  sxy(if g.x < 100 { g.x + 1 } else { 100 }, g.y - 1)
}
```

(`observe` is not used until Activity 5; add it now so all the imports are in place.)

* **Task:** Add a new suite after `pipeline`:

```rust
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
```

`run_property(cases, make_strategy, property)` runs `cases` random cases of `property`, each drawn from the strategy -- here, pairs of integers in GenStruct's range -- and returns whether every case passed.  Each case is a complete scenario on the running system: inject, run through ConsumeStruct, check.  Three things to notice:

- **The strategy is a closure** (`structs`), not a strategy value.  The test controller has no operating system; its heap is set up when `run_property` first runs, and building a strategy can allocate.
- **Inside the case, assert with `proptest::prop_assert_eq!`**, which reports a failure to PropTest so it can shrink the input; the whole run's result goes to `sys_assert!`.
- **Each case sets what it depends on** (here, `inject_generated` sets the struct), because the cases share the running system, just as tests do.

* **Task:** Run the new suite:

```
bin/run-tests.cmd properties::
```

You should see:

```
TEST | BEGIN properties::end_to_end_holds_for_random_structs
TEST | INFO  32 generated cases passed (PROPTEST_SEED=6589198df6250524eb3fe83c8dd16cf9145741abc53ad010386232760106e61c)
TEST | PASS  properties::end_to_end_holds_for_random_structs
```

with a different seed.  There is no source of randomness on the target, so the cases are determined by `PROPTEST_SEED`, which the driver picks afresh for every run -- each run explores new cases -- and prints.

## Activity 2 - Seed a Bug

So far you have focused on the assertions in your tests.  As Part 1 mentioned, the test controller also checks something else at **every dispatch of every thread**: the executable form of each component's GUMBO contract (the same GUMBOX predicates the component tests use), and every assertion of the `nominal` composition's properties as the schedule passes its place.  A violation of either during a test fails that test, whether or not the test asserted anything about it.  (A broken initialize guarantee, checked at boot, fails the whole run instead, with `init=failed` on the `DONE` line.)  Until now nothing has violated them, so they have been silent.

* **Task:** Seed a bug in IncX: in `hamr/microkit_mcs/crates/incx_incx/src/component/incx_incx_app.rs`, change the line that sends the result, `api.put_outxfield(v + 1i32);`, to add 2 instead of 1.  Rerun every test:

```
bin/run-tests.cmd
```

Every one of your six tests now fails.  Look first at the two smoke tests, which assert nothing about Struct Split at all:

```
TEST | BEGIN smoke::schedule_advances_by_hyperperiod
TEST | VIOLATION CEP_Post incx_incx hp=0 slot=2
TEST | INFO  incx_incx pre: PreState_incx_incx { api_inxfield: 1 }
TEST | INFO  incx_incx post: PostState_incx_incx { api_outxfield: 3 }
TEST | VIOLATION SysAssert nominal/End_To_End_Functional at after incx hp=0 slot=2
TEST | VIOLATION SysAssert nominal/End_To_End_Functional at after clampx hp=0 slot=4
TEST | VIOLATION SysAssert nominal/End_To_End_Functional at at branches_joined hp=0 slot=4
TEST | VIOLATION SysAssert nominal/End_To_End_Functional at before consume hp=0 slot=5
TEST | FAIL  contract violation (5 in this test)
```

* **Task:** Read these lines and answer:
  - Which contract does the `CEP_Post incx_incx` line report, and which GUMBO clause in `SysPropStructSplit_Software.sysml` is it?  The two `INFO` lines show the values it was checked on.  (CEP_Post is a thread's *compute entry point post-condition*: its compute guarantees, together with the integration guarantees on its output ports.)
  - The `SysAssert` lines come from the **system layer**: `End_To_End_Functional` first breaks `after incx`.  Find that place in the composition's `Pipeline_Functional` property.  Why does the property keep breaking at every later place of the frame?
  - Where do `hp=` and `slot=` say the violations happened?  Compare with the schedule in Part 1, Activity 3.

* **Task:** Now look at `x_saturates_at_the_upper_bound`:

```
TEST | BEGIN pipeline::x_saturates_at_the_upper_bound
TEST | VIOLATION CEP_Post incx_incx hp=3 slot=2
TEST | INFO  incx_incx pre: PreState_incx_incx { api_inxfield: 100 }
TEST | INFO  incx_incx post: PostState_incx_incx { api_outxfield: 102 }
TEST | VIOLATION SysAssert nominal/Merged_In_Consume_Range at after incx hp=3 slot=2
TEST | VIOLATION SysAssert nominal/End_To_End_Functional at after incx hp=3 slot=2
TEST | VIOLATION SysAssert nominal/Consume_Range_Loop_Invariant at after incx hp=3 slot=2
TEST | FAIL  src/system_tests/tests.rs:<line> inspect::get_incx_incx_outxfield() == Some(101)
TEST | INFO  also failed: contract violation (4 in this test)
```

Here two more properties break, `Merged_In_Consume_Range` and `Consume_Range_Loop_Invariant`.  Look at what they assert `after incx` (they inherit it from the abstract property `Pipeline_Ranges`).  Why does 102 break them when the smoke test's 3 did not?  Notice also how the failure is reported: the test's **first** failure is its own assertion, on the `FAIL` line; the contract violations follow as `also failed`.

* **Task:** Find the property test's output.  PropTest **shrinks** a failing case to a minimal one before reporting it:

```
TEST | INFO  assertion failed: `(left == right)`
TEST | INFO    left: `Some(StructXY { x: 2, y: -1 })`,
TEST | INFO   right: `Some(StructXY { x: 1, y: -1 })` at src/system_tests/tests.rs:<line>
TEST | INFO  minimal failing input: (0, 0)
TEST | INFO  reproduce with PROPTEST_SEED=48063e1dc61e91ed7af04f8470fec8b807cc55fcf9ad7fde3cc47437b38e2a9d
```

`(0, 0)` is the simplest struct that shows the bug.  (Shrinking reruns cases on the live system, so this test reports many contract violations.  Those violations fail the test, but they do not decide which case is the failing one: a case fails, and is shrunk, only through its own `prop_assert_eq!`s.)

(If you run the solution's eight tests with this bug, one still passes: the robustness test you will write in Activity 5.  SplitStruct rejects its out-of-range struct, so the struct is not forwarded, and the test's body ends right after SplitStruct's slot: IncX is not dispatched while the test is running.  The runner dispatches it afterwards, when moving back to slot 0 for the next test, but violations then belong to no test and are not reported.)

## Activity 3 - Reproduce a Random Failure

A random failure is only useful if you can get it back.

* **Task:** Copy the seed from the `reproduce with` line of your run, and rerun just the property test with it:

```
PROPTEST_SEED=<the seed> bin/run-tests.cmd properties::
```

The same cases run, and the same minimal failing input is reported.  The seed is built into the test image, so changing it only relinks the image.

## Activity 4 - What the Contracts Contribute

How much of what you just saw came from your own assertions, and how much from the contract checks?  Both layers can be switched off for a whole run.

* **Task:** With the bug still in place, run every test with both layers off:

```
GUMBO_CHECKS=off SYSVERIF_CHECKS=off bin/run-tests.cmd
```

* **Task:** Compare with Activity 2:
  - The **smoke tests now pass**.  With no checks running, nothing they assert can see the bug.
  - The **pipeline tests and the property test still fail**: their own assertions catch the bug, on the same `FAIL` lines -- without the `also failed: contract violation` lines.
  - If you have already done Activities 5-6 (or are running the solution), the **robustness tests fail**, and they fail differently: they depend on the checks.  The first reports `expected violation CepPre(splitter_splitter) cannot be reported: the GUMBO checks are off` -- and, with no check to reject the dispatch, SplitStruct really does forward the out-of-range 150.

The contract checks turn every test into a test of every contract, for whatever the test drives the system through.  Your own assertions say what the *scenario* should produce; the checks say that no component, and no system property, went wrong along the way.

* **Task:** Restore IncX (`v + 1i32`), and rerun everything to confirm the suite passes again.

## Activity 5 - A Robustness Test: Breaking an Assumption on Purpose

SplitStruct's contract has an integration **assumption** on its input: both fields lie in `[-100, 100]`.  GenStruct's guarantee normally ensures that.  A robustness test asks what happens when the assumption is broken anyway.

When a thread's assumptions (its CEP_Pre) do not hold at a dispatch, the test controller treats that as the sender's fault: it reports the violation and **rejects the dispatch** -- the thread's slot passes without running it, so the out-of-contract values never reach its code.

* **Task:** Add a `robustness` suite with this test:

```rust
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
  }
```

`observe::expect(..)` declares the violation the test causes on purpose: a matching violation is reported as `INFO` instead of failing the test, and the test **fails if the violation does not happen**.  Every *other* contract is still checked.  `observe::Thread::splitter_splitter` is a generated name, so a misspelled thread is a compile error.

* **Task:** Run `bin/run-tests.cmd robustness::`.  You should see:

```
TEST | BEGIN robustness::out_of_range_struct_is_rejected_by_splitter
TEST | INFO  expected violation: CEP_Pre splitter_splitter hp=37 slot=1
TEST | INFO  channel 3 not dispatched at hp=37 slot=1: its inputs did not meet its CEP_Pre; system assertions suspended until the next frame
TEST | PASS  robustness::out_of_range_struct_is_rejected_by_splitter
```

(with different `hp` numbers).  Because SplitStruct produced nothing in that frame, the system layer suspends its assertions until the next hyperperiod, and says so.

* **Task:** Remove the `observe::expect(..)` line and rerun.  The test now fails:

```
TEST | BEGIN robustness::out_of_range_struct_is_rejected_by_splitter
TEST | VIOLATION CEP_Pre splitter_splitter hp=0 slot=1
TEST | INFO  splitter_splitter pre: PreState_splitter_splitter { api_instruct: StructXY { x: 150, y: 0 } }
TEST | INFO  channel 3 not dispatched at hp=0 slot=1: its inputs did not meet its CEP_Pre; system assertions suspended until the next frame
TEST | FAIL  contract violation (1 in this test)
```

The same violation, reported as a failure because nothing declared it.  Put the line back.

## Activity 6 - A Fault Only the System Layer Can See

The bug in Activity 2 broke a component's own contract, so both layers saw it.  Some faults break no component contract at all.  Suppose the y value is corrupted on its way into MergeStruct -- a faulty channel delivers it undecremented.  MergeStruct copies what it receives, so it meets its contract; every value stays within every range; no thread is at fault.  But the struct reaching ConsumeStruct is no longer `(x + 1, y - 1)` of the generated one, and `End_To_End_Functional` says it must be.

You could declare that violation with `observe::expect(observe::Expect::SysAssert(..))`.  But a `SysAssert` expectation names a *property*, not one of its assertion points: it would accept a violation of `End_To_End_Functional` *anywhere* -- including one caused by an unrelated bug, like Activity 2's, which breaks the same property `after incx`.  When the place matters, take the violations instead with `observe::take()`, and assert on them yourself.  Violations taken this way no longer fail the test.

* **Task:** Add this test to the `robustness` suite:

```rust
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
```

The test injects a struct as usual, then parks again just before MergeStruct and overwrites its inputs with the corrupted values -- `11` is the correct x (10 + 1), `20` is y undecremented.

* **Task:** Run `bin/run-tests.cmd robustness::`:

```
TEST | BEGIN robustness::corrupted_merge_input_breaks_end_to_end
TEST | INFO  system assertions resumed at hp=38
TEST | VIOLATION SysAssert nominal/End_To_End_Functional at before consume hp=38 slot=5
TEST | PASS  robustness::corrupted_merge_input_breaks_end_to_end
```

(The first `INFO` line is the previous test's rejected dispatch ending.)  The violation is still printed, but the test took it and checked it is exactly the one expected.  Notice what is *not* reported: no `CEP_Post` for any thread.  This fault is invisible to every component contract; the end-to-end property is what catches it.

* **Task:** Seed Activity 2's bug again (IncX adds 2) and run `bin/run-tests.cmd robustness::`.  The test now fails on `violations.count() == 1`: the taken violations include IncX's `CEP_Post` and four `End_To_End_Functional` violations, from `after incx` on.  A test written with `observe::expect` would have accepted those `End_To_End_Functional` violations as the one it expected.  Restore IncX.

## Activity 7 - Run Everything and Commit

* **Task:** Run the whole suite with no filter, and check that all eight tests pass:

```
TEST | DONE  matched=8 passed=8 failed=0 init=ok
OK: 8 of 8 tests passed
```

* **Task:** Commit/push your changes with a commit message such as "StructSplit add-SysTest - property and robustness tests".

**Conclusion**: You have checked the end-to-end property you proved earlier against the real code on many random inputs, reproduced a random failure from its seed, and seen the two layers of contract checking at work: a bug in one component broke its own contract and the system properties built on it, and showed up even in tests that never mention it; a fault in transit broke no component contract and only the system layer saw it.  The system proof says the pipeline is correct *if* every component meets its contract; the system tests check, on the platform, that the components do -- and that the composed system delivers what the proof promises.
