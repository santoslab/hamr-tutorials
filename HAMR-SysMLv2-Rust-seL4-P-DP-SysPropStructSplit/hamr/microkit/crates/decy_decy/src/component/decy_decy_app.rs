// This file will not be overwritten if HAMR codegen is rerun

use data::*;
use crate::bridge::decy_decy_api::*;
use vstd::prelude::*;

verus! {

  pub struct decy_decy {
    // PLACEHOLDER MARKER STATE VARS
  }

  impl decy_decy {
    pub fn new() -> Self
    {
      Self {
        // PLACEHOLDER MARKER STATE VAR INIT
      }
    }

    pub fn initialize<API: decy_decy_Put_Api> (
      &mut self,
      api: &mut decy_decy_Application_Api<API>)
      ensures
        // PLACEHOLDER MARKER INITIALIZATION ENSURES
    {
      log_info("initialize entrypoint invoked");
      // Output data ports MUST be initialized.
      api.put_outyfield(0i32);
    }

    pub fn timeTriggered<API: decy_decy_Full_Api> (
      &mut self,
      api: &mut decy_decy_Application_Api<API>)
      requires
        // BEGIN MARKER TIME TRIGGERED REQUIRES
        // assume inyfield_bounded
        //   Underflow guard only: the input lies in a
        //   conservative [-1000, 1000] envelope so the decrement cannot underflow.
        (-1000i32 <= old(api).inyfield) &&
          (old(api).inyfield <= 1000i32),
        // END MARKER TIME TRIGGERED REQUIRES
      ensures
        // BEGIN MARKER TIME TRIGGERED ENSURES
        // guarantee decy
        //   Output is the input decremented by one.
        api.outyfield == api.inyfield - 1i32,
        // END MARKER TIME TRIGGERED ENSURES
    {
      log_info("compute entrypoint invoked");
      // Decrement the input by one.  The inyfield_bounded compute assume
      // ([-1000, 1000]) precludes underflow.
      let v = api.get_inyfield();
      api.put_outyfield(v - 1i32);
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

  // PLACEHOLDER MARKER GUMBO METHODS

}
