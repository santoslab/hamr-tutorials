// This file will not be overwritten if HAMR codegen is rerun

use data::*;
use crate::bridge::gen_gen_api::*;
use vstd::prelude::*;

verus! {

  pub struct gen_gen {
    // PLACEHOLDER MARKER STATE VARS
    // Developer-added (non-GUMBO) local state: a counter that drives a
    // deterministic sawtooth in the range [-100, 100].
    pub count: i32,
  }

  impl gen_gen {
    pub fn new() -> Self
    {
      Self {
        // PLACEHOLDER MARKER STATE VAR INIT
        count: 0,
      }
    }

    pub fn initialize<API: gen_gen_Put_Api> (
      &mut self,
      api: &mut gen_gen_Application_Api<API>)
      ensures
        // BEGIN MARKER INITIALIZATION ENSURES
        // guarantee init_outstruct
        //   The output data port must be initialized.
        (api.outstruct.x == 0i32) &&
          (api.outstruct.y == 0i32),
        // END MARKER INITIALIZATION ENSURES
    {
      log_info("initialize entrypoint invoked");
      // Output data ports MUST be initialized.  Zero satisfies both the
      // init_outstruct guarantee and the outstruct_range integration
      // constraint ([-100, 100]).
      api.put_outstruct(SysPropStructSplit_Data_Model::StructXY { x: 0i32, y: 0i32 });
    }

    pub fn timeTriggered<API: gen_gen_Full_Api> (
      &mut self,
      api: &mut gen_gen_Application_Api<API>)
      requires
        // PLACEHOLDER MARKER TIME TRIGGERED REQUIRES
      ensures
        // PLACEHOLDER MARKER TIME TRIGGERED ENSURES
    {
      log_info("compute entrypoint invoked");
      // Advance the counter, keeping it within [-100, 100] by construction so
      // the value satisfies the outstruct_range integration constraint.
      let next = next_in_range(self.count);
      self.count = next;
      let out = SysPropStructSplit_Data_Model::StructXY { x: next, y: next };
      api.put_outstruct(out);
      log_generated(out);
    }

    pub fn notify(
      &mut self,
      channel: microkit_channel)
    {
      // this method is called when the monitor does not handle the passed in channel
      match channel {
        _ => {
          log_warn_channel(channel)
        }
      }
    }
  }

  #[verifier::external_body]
  pub fn log_info(msg: &str)
  {
    log::info!("{0}", msg);
  }

  #[verifier::external_body]
  pub fn log_warn_channel(channel: u32)
  {
    log::warn!("Unexpected channel: {0}", channel);
  }

  #[verifier::external_body]
  pub fn log_generated(value: SysPropStructSplit_Data_Model::StructXY)
  {
    log::info!("Gen: produced struct (x={0}, y={1})", value.x, value.y);
  }

  // Advance the sawtooth counter, staying within [-100, 100].  The branch
  // guard lets Verus discharge both the overflow check on `c + 1` and the
  // postcondition that the result is in range.
  pub fn next_in_range(c: i32) -> (r: i32)
    ensures
      (-100i32 <= r) && (r <= 100i32),
  {
    if (-100i32 <= c) && (c < 100i32) {
      c + 1
    } else {
      -100i32
    }
  }

  // PLACEHOLDER MARKER GUMBO METHODS

}
