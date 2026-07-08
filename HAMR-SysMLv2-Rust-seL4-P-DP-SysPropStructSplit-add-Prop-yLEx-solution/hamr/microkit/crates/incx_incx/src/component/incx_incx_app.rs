// This file will not be overwritten if HAMR codegen is rerun

use data::*;
use crate::bridge::incx_incx_api::*;
use vstd::prelude::*;

verus! {

  pub struct incx_incx {
    // PLACEHOLDER MARKER STATE VARS
  }

  impl incx_incx {
    pub fn new() -> Self
    {
      Self {
        // PLACEHOLDER MARKER STATE VAR INIT
      }
    }

    pub fn initialize<API: incx_incx_Put_Api> (
      &mut self,
      api: &mut incx_incx_Application_Api<API>)
      ensures
        // PLACEHOLDER MARKER INITIALIZATION ENSURES
    {
      log_info("initialize entrypoint invoked");
      // Output data ports MUST be initialized.
      api.put_outxfield(0i32);
    }

    pub fn timeTriggered<API: incx_incx_Full_Api> (
      &mut self,
      api: &mut incx_incx_Application_Api<API>)
      requires
        // BEGIN MARKER TIME TRIGGERED REQUIRES
        // assume inxfield_bounded
        //   Overflow guard only: the input lies in a
        //   conservative [-1000, 1000] envelope so the increment cannot overflow.
        (-1000i32 <= old(api).inxfield) &&
          (old(api).inxfield <= 1000i32),
        // END MARKER TIME TRIGGERED REQUIRES
      ensures
        // BEGIN MARKER TIME TRIGGERED ENSURES
        // guarantee incx
        //   Output is the input incremented by one.
        api.outxfield == api.inxfield + 1i32,
        // END MARKER TIME TRIGGERED ENSURES
    {
      log_info("compute entrypoint invoked");
      // Increment the input by one.  The inxfield_bounded compute assume
      // ([-1000, 1000]) precludes overflow.
      let v = api.get_inxfield();
      api.put_outxfield(v + 1i32);
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
