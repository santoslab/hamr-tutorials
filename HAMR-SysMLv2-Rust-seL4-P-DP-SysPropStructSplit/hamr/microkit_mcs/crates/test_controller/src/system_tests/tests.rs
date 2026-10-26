// This file will not be overwritten if HAMR codegen is rerun

//! System tests.  This file is preserved across regeneration.
//!
//! Tests must be **order-independent** and establish their own preconditions: the
//! runner normalizes the schedule position between tests, but component state is
//! whatever the previous test left behind.  Set every input you depend on.
//!
//! Assertions are `sys_assert!` / `sys_assert_eq!`, not `assert!`: a panic would
//! take the protection domain down and end the run.
//!
//! Select a subset at image build time with `make CONFIG=test_scheduler.mk TESTS=<filter>`;
//! the filter is a substring match against the qualified `suite::test` name.
//!
//! `api::` steps the schedule; `inspect::` reads and writes this model's ports and
//! GUMBO state variables (see `inspect.rs` for the generated accessors), and
//! `inspect::channels::` names each thread for `api::run_to_thread(..)`.  The
//! inject / step / observe shape is:
//!
//! ```ignore
//! // Park immediately before the consumer.  Stepping past the producer's own slot
//! // first would overwrite the injected value -- the queue holds one element.
//! let _ = api::run_to_thread(inspect::channels::some_thread_MON);
//! inspect::put_some_thread_someInput(value);
//! let _ = api::sstep(1);
//! sys_assert_eq!(inspect::get_some_thread_someOutput(), Some(expected));
//! ```
//!
//! To split the tests across files, replace the `system_tests!` block below with
//!
//! ```ignore
//! system_test_files! {
//!   mod smoke_tests;      // system_tests/tests/smoke_tests.rs
//!   mod nominal_tests;    // system_tests/tests/nominal_tests.rs
//! }
//! ```
//!
//! where each file has its own `system_tests!` block and the same `use` lines as
//! this one.  Keep the `_tests.rs` suffix so the model's clean script preserves them.
//!
//! For random inputs, `harness::run_property` runs a proptest strategy against the
//! system on target and shrinks a failure to a minimal input:
//!
//! ```ignore
//! use proptest::prelude::Strategy;
//! sys_assert!(crate::system_tests::harness::run_property(64, || 0i32..100, |v| {
//!   // inject v, step, then check with proptest::prop_assert_eq!(..)
//!   Ok(())
//! }));
//! ```

use crate::system_tests::api;
#[allow(unused_imports)]
use crate::system_tests::inspect;
use crate::{system_tests, sys_assert, sys_assert_eq};

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
}
