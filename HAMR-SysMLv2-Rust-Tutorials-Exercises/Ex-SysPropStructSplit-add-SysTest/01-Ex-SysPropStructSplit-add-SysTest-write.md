# HAMR Exercise: System Testing the Struct Split Example (Part 1 - Writing and Running System Tests)

**Purpose**: The purpose of this exercise is to get you familiar with the basic workflow for HAMR *system testing*: tests that run the whole composed system -- the real Rust code of every thread, on seL4 under QEMU, in the production schedule -- and that you drive step by step to inject values, observe results, and check them.

Starting from the Struct Split example you verified in the system-property exercises, you will go through the steps of...
 - finding the generated system-testing infrastructure, and running the generated smoke tests,
 - reading the generated test API for this model,
 - writing tests that inject a struct into the pipeline and check what each component produces,
 - working out expected results from the component contracts, and seeing what a failing test reports,
 - selecting tests with filters, and listing what is runnable.

In Part 2 of this exercise, you will write a property-based system test, see the contract checks that run at every dispatch catch a seeded bug, and write robustness tests that break contracts on purpose.

## Prerequisites and Resources

Before working through this exercise, you should have gone through the following HAMR lectures (or read the equivalent documentation):

* HAMR Overview, HAMR GUMBO Overview
* HAMR GUMBO System-Level Properties (the Struct Split example's `composition` block and its properties)
* [HAMR System Testing on seL4](https://hamr.sireum.org/hamr-doc/hamr-system-testing/) -- the documentation chapter for system testing.  It uses a different running example (the event-data-port Simple Isolette), but the API, the workflow, and the output are the same.  Sections 1-6 and 9 are the background for this part.

System tests run under QEMU, so beyond the tools the earlier exercises needed you need the seL4 Microkit SDK, LionsOS, sdfgen, and `qemu-system-aarch64` (see the "User-land scheduling" and "Build Tools and QEMU" parts of the [Installation](https://hamr.sireum.org/hamr-doc/installation/) chapter).  The test driver does not run on Windows.

## Exercise Overview

The Struct Split example is a seven-component pipeline that manipulates a struct of two `int32` fields `x` and `y`:

- **GenStruct** (`gen`) generates a struct; its integration guarantee promises both fields lie in `[-100, 100]`.
- **SplitStruct** (`splitter`) splits the struct onto two scalar ports `xfield` and `yfield`.
- The **x branch**: **IncX** (`incx`) adds one, then **ClampX** (`clampx`) saturates the result back into `[-100, 100]`.
- The **y branch**: **DecY** (`decy`) subtracts one.
- **MergeStruct** (`merger`) reassembles the struct.
- **ConsumeStruct** (`consume`) receives the final struct and latches its `x` field in the GUMBO state variable `last_x`.

In the system-property exercises you *proved*, from the component contracts alone, that the struct reaching ConsumeStruct is `(x + 1 saturated at 100, y - 1)`.  That proof assumes every component meets its contract; it never looks at the Rust code.  In this exercise you *test* the same pipeline with its real code, running on seL4.

## Setting up the Starting Files for the Project

The starting project can be found in the folder `HAMR-SysMLv2-Rust-seL4-P-DP-SysPropStructSplit` in [this repository](https://github.com/santoslab/hamr-tutorials/tree/main/HAMR-SysMLv2-Rust-seL4-P-DP-SysPropStructSplit). Copy this folder (the *folder* itself, not just the contents) into a personal git repository and rename the folder to `HAMR-SysMLv2-Rust-seL4-P-DP-SysPropStructSplit-add-SysTest`. One reason that it's important to copy the entire folder is that it contains `.gitignore` files that configure the folder for git use (e.g., by ignoring the very large build artifacts associated with Rust builds).

**Do not modify the original `HAMR-SysMLv2-Rust-seL4-P-DP-SysPropStructSplit` folder** -- all work in this exercise happens in your renamed copy.

A completed solution for this exercise (both parts) is available in the folder `HAMR-SysMLv2-Rust-seL4-P-DP-SysPropStructSplit-add-SysTest-solution` in the same repository. Try to work through the exercise without it.

## Activity 1 - Commit the Starting Files to a Git Repository

As in the other HAMR exercises, it is useful to see (via file diffs) the updates *you* make versus the updates HAMR's code generation makes for you.

* **Task:** Commit and push the starting files to your repository with a commit message such as "StructSplit add-SysTest - initial files".

## Activity 2 - Find the System-Testing Build and Run the Smoke Tests

Look in the project's `hamr` folder.  Besides `microkit`, the build you used for the system-property exercises, there is a second Microkit build, `microkit_mcs`:

- `hamr/microkit` uses **domain scheduling**: the seL4 kernel runs the static schedule itself, and nothing a test writes can pause it.
- `hamr/microkit_mcs` uses **MCS user-land scheduling**: the schedule is run by an ordinary scheduler program, so HAMR can generate a *test scheduler* that waits for commands from your tests instead of a timer.  System testing needs this build.  The test scheduler dispatches the threads in the production schedule's order, with a watchdog bounding each dispatch, but it does not reproduce the production timing: it moves on as soon as a thread finishes, and skips the padding time.

Both are generated from the same model, and the component crates contain the same application code, unit tests, and Verus contracts.  (System testing adds a few generated hooks to threads with GUMBO state variables -- here ConsumeStruct -- so that tests can read and inject `last_x`; they are inert outside the test image.)  The `microkit_mcs` tree was generated with system testing switched on:

```
sireum hamr sysml codegen ... --platform Microkit --scheduling UserLand \
  --system-testing --sel4-output-dir ../../hamr/microkit_mcs
```

(If you ever need to regenerate it, run `../bin/run-hamr.cmd` with those options from `sysmlv2/struct-split`; see section 4 of the documentation chapter, which also gives the option to use with Sireum releases older than `--system-testing`.)

* **Task:** Find the files system testing added to `hamr/microkit_mcs`:
  - `crates/test_controller/src/system_tests/` -- the test API for this model (`api.rs`, `inspect.rs`, `observe.rs`, `harness.rs`) and **`tests.rs`**, the file you will edit.  Codegen writes `tests.rs` once and never overwrites it; every other file in the crate is regenerated.
  - `bin/run-tests.cmd` -- the test driver: it builds the test image, boots it under QEMU, and turns the output into a pass/fail exit code.
  - `test_scheduler.mk`, `test_scheduler.meta.py`, `scheduler/src/test_scheduler.scheduler.c` -- the test variant of the system.  The normal image (plain `make`) does not include any of it.

* **Task:** Open `tests.rs`.  Codegen wrote two `smoke` tests into it, which use only the stepping API (`api::`) and so work for any model: one advances the schedule by a hyperperiod, the other steps through it one slot at a time.

* **Task:** From `hamr/microkit_mcs`, run the tests:

```
bin/run-tests.cmd
```

The first run builds the whole test image and takes a few minutes.  The lines that matter all begin with `TEST | `; the rest is the build and the system's own boot and log output.  At the end you should see:

```
TEST | LIST  smoke::schedule_advances_by_hyperperiod
TEST | LIST  smoke::stepping_one_slot_at_a_time_wraps
TEST | BEGIN smoke::schedule_advances_by_hyperperiod
TEST | PASS  smoke::schedule_advances_by_hyperperiod
TEST | BEGIN smoke::stepping_one_slot_at_a_time_wraps
TEST | PASS  smoke::stepping_one_slot_at_a_time_wraps
TEST | DONE  matched=2 passed=2 failed=0 init=ok
OK: 2 of 2 tests passed
```

Passing smoke tests mean the whole path works: the test image builds, every thread initializes on seL4 (`init=ok` means every initialize guarantee held), and the tests can drive the schedule.  They also mean more than that: while they stepped the schedule, every thread dispatched was checked against its GUMBO contract, and the composition's system assertions were checked at each place in the schedule they passed.  You will see those checks at work in Part 2.

## Activity 3 - Read the Generated Test API

* **Task:** Open `crates/test_controller/src/system_tests/inspect.rs` and find:
  - the module `channels`, with one constant per thread (`splitter_splitter_MON`, ...).  These name the threads for `api::run_to_thread`.
  - a getter and a setter for every port and GUMBO state variable, named after the thread that *owns the memory*: `get_incx_incx_outxfield()`, `put_gen_gen_outstruct(..)`, `get_consume_consume_sv_last_x()`.  A getter returns an `Option`: the next value your test has not yet read, or `None` if there is none.  It is not a snapshot -- read the same port twice with nothing written in between and the second read gives `None` -- so read each value once, after the step that produces it.  (Reading never takes the value away from the components, or from the contract checks.)
  - the **pre-state containers**: one struct per thread with every input port and state variable of that thread, and a `set_<thread>` function that writes them all.  Look at `splitter_splitter_PreState` and `consume_consume_PreState`.

* **Task:** Look at the body of `set_splitter_splitter`.  SplitStruct's input `instruct` is written with `put_gen_gen_outstruct`: a connected input lives in its *producer's* output region, so setting it means acting as the producer.  Every consumer of GenStruct's output will receive what you write -- and if GenStruct runs after you write, it overwrites your value.  So the rule for injecting is: **after the producer has run, before the consumer runs.**

* **Task:** Look at the schedule in `meta.py` (`user_schedule = schedule(...)`).  The slots, in order, are:

```
slot:   0     1         2     3     4       5       6        7
        gen   splitter  incx  decy  clampx  merger  consume  (pad)
```

The test runner starts every test at slot 0, so `api::run_to_thread(channels::splitter_splitter_MON)` stops right after GenStruct has run and before SplitStruct reads its output -- exactly the place to inject a struct.

Starting at slot 0 resets only the *position* in the schedule, not the system's state.  Each thread initializes once, at boot; ports and state variables keep whatever the previous test left in them.  So every test must set every input and state value it depends on, and must not depend on the order the tests run in.

## Activity 4 - Write a Test That Follows One Struct Through the Pipeline

* **Task:** In `tests.rs`, replace the existing `use` lines with the following imports, and add the helpers immediately below them:

```rust
use crate::system_tests::api;
use crate::system_tests::inspect;
use crate::system_tests::inspect::channels;
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

fn merged() -> Option<StructXY> {
  inspect::get_merger_merger_outstruct()
}

fn last_x() -> Option<i32> {
  inspect::get_consume_consume_sv_last_x()
}
```

* **Task:** Inside the `system_tests!` block, after the `smoke` suite, add a new suite with one test:

```rust
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
  }
```

Some things to notice:

- **`api::sstep(1)`** dispatches exactly one slot.  Because the test parked before SplitStruct, each step runs the next thread in the schedule, and the test checks that thread's output right after it ran.  The result of each command is discarded with `let _ =`: a command that fails (for example, a thread that overruns its slot) fails the test by itself.
- **`sys_assert_eq!`, not `assert_eq!`.**  The tests run inside a protection domain on seL4; a panic would take it down and end the whole run.  `sys_assert_eq!` records the failure and returns from the test, and the runner moves on to the next one.
- **The injected struct counts as what GenStruct sent.**  Everything downstream, including the system assertions checked during the run, treats `(37, -12)` as this frame's generated struct.

* **Task:** Run only the new suite, by passing a filter -- a substring of the qualified `suite::test` name:

```
bin/run-tests.cmd pipeline::
```

You should see the new test pass:

```
TEST | BEGIN pipeline::struct_flows_through_each_stage
TEST | PASS  pipeline::struct_flows_through_each_stage
TEST | DONE  matched=1 passed=1 failed=0 init=ok
OK: 1 of 1 tests passed
```

Changing only the filter relinks the test image in seconds; it does not rebuild it.

## Activity 5 - Test the Boundaries

A test need not step slot by slot.  When only the end result matters, run to the end of ConsumeStruct's dispatch and check the results there.

* **Task:** Add this helper next to the others:

```rust
/// Runs to the end of ConsumeStruct's dispatch.
fn run_through_consume() {
  let _ = api::run_to_thread(channels::consume_consume_MON);
  let _ = api::sstep(1);
}
```

* **Task:** Work out, from the component contracts (`sysmlv2/struct-split/SysPropStructSplit_Software.sysml`) and without running anything, what each component produces when GenStruct's struct is at the edges of its range:
  - `(100, 100)`: what does IncX produce?  What does ClampX make of it?  What struct does MergeStruct produce, and what does ConsumeStruct latch in `last_x`?
  - `(-100, -100)`: the same questions.  Does anything saturate?  Does every value stay within ConsumeStruct's `[-200, 200]` compute assumption?

* **Task:** Add two tests to the `pipeline` suite, `x_saturates_at_the_upper_bound` and `lower_bound_passes_through`, that inject those structs, call `run_through_consume()`, and assert what you worked out: IncX's output for the first, and for both, `merged()` and `last_x()`.  Run `bin/run-tests.cmd pipeline::` and check that all three tests pass.

The upper bound is the one case where the x field arrives unchanged -- the reason the `End_To_End_Functional` property you verified earlier states the x result as a case split.

## Activity 6 - See a Failing Test

* **Task:** In `x_saturates_at_the_upper_bound`, deliberately change the expected merged struct to a wrong value (for example, an x of `101`), and run just that test, using a filter that matches only its name:

```
bin/run-tests.cmd saturates
```

You should see something like:

```
TEST | BEGIN pipeline::x_saturates_at_the_upper_bound
TEST | FAIL  src/system_tests/tests.rs:<line> merged() == Some(sxy(101, 99))
TEST | DONE  matched=1 passed=0 failed=1 init=ok
FAILED: 1 of 1 tests failed
```

A failing test prints one `FAIL` line, naming the file, line, and condition of its *first* failure; any further failures in the same test follow as `INFO  also failed: ...` lines.  (`sys_assert_eq!` reports the condition it checked, not the two values it compared.)

* **Task:** Restore the correct expected value and rerun the test.

## Activity 7 - Select and List Tests

* **Task:** List the tests without running any, with and without a filter:

```
bin/run-tests.cmd --list
bin/run-tests.cmd --list pipeline::
```

With a filter, tests it does not select are marked:

```
TEST | LIST  smoke::schedule_advances_by_hyperperiod (not selected)
TEST | LIST  smoke::stepping_one_slot_at_a_time_wraps (not selected)
TEST | LIST  pipeline::struct_flows_through_each_stage
TEST | LIST  pipeline::x_saturates_at_the_upper_bound
TEST | LIST  pipeline::lower_bound_passes_through
TEST | DONE  matched=3 passed=0 failed=0 init=ok list=1
OK: listed 3 test(s) matching 'pipeline::'; none were run
```

* **Task:** Run with a mistyped filter, for example `bin/run-tests.cmd satruates`.  The run **fails**:

```
TEST | DONE  matched=0 passed=0 failed=0 init=ok
FAILED: the filter 'satruates' matched no tests
```

This is deliberate.  A typo in a filter must not turn a test job green with nothing run, so the driver treats "no test matched" as a failure.  For the same reason, a run that ends without a `DONE` line -- a hang, a crash, a controller that never started -- is a failure, not a pass.

* **Task:** Run the whole suite one more time with no filter, and check that all five tests pass.

## Activity 8 - Commit Your Tests

* **Task:** Commit/push your changes with a commit message such as "StructSplit add-SysTest - pipeline tests".

**Conclusion**: You have run the generated smoke tests, read the test API generated for this model, written tests that inject a struct into the running system and check every component's output, and worked out expected results from the contracts.  While your tests ran, every dispatched component's contract was checked, and so was every system assertion at each place the schedule reached -- silently, because nothing broke.  In Part 2 you will make them speak: you will drive the pipeline with random structs, seed a bug and watch both layers of contract checking report it, and write tests that break contracts on purpose.
