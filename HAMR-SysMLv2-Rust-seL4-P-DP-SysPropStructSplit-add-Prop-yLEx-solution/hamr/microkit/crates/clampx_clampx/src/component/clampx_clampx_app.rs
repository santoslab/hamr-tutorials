// This file will not be overwritten if HAMR codegen is rerun

use data::*;
use crate::bridge::clampx_clampx_api::*;
use vstd::prelude::*;

verus! {

  pub struct clampx_clampx {
    // PLACEHOLDER MARKER STATE VARS
  }

  impl clampx_clampx {
    pub fn new() -> Self
    {
      Self {
        // PLACEHOLDER MARKER STATE VAR INIT
      }
    }

    pub fn initialize<API: clampx_clampx_Put_Api> (
      &mut self,
      api: &mut clampx_clampx_Application_Api<API>)
      ensures
        // PLACEHOLDER MARKER INITIALIZATION ENSURES
    {
      log_info("initialize entrypoint invoked");
      // Output data ports MUST be initialized.
      api.put_outxfield(0i32);
    }

    pub fn timeTriggered<API: clampx_clampx_Full_Api> (
      &mut self,
      api: &mut clampx_clampx_Application_Api<API>)
      requires
        // PLACEHOLDER MARKER TIME TRIGGERED REQUIRES
      ensures
        // BEGIN MARKER TIME TRIGGERED ENSURES
        // case In_Range
        //   values already in [-100, 100] pass through unchanged
        ((-100i32 <= old(api).inxfield) &&
          (old(api).inxfield <= 100i32)) ==>
          (api.outxfield == api.inxfield),
        // case Above
        //   values above 100 saturate to 100
        (old(api).inxfield > 100i32) ==>
          (api.outxfield == 100i32),
        // case Below
        //   values below -100 saturate to -100
        (old(api).inxfield < -100i32) ==>
          (api.outxfield == -100i32),
        // END MARKER TIME TRIGGERED ENSURES
    {
      log_info("compute entrypoint invoked");
      // Saturating clamp to [-100, 100]: pass in-range values through
      // unchanged, saturate out-of-range values to the nearest bound.
      let v = api.get_inxfield();
      let clamped: i32;
      if v > 100i32 {
        clamped = 100i32;
      } else if v < -100i32 {
        clamped = -100i32;
      } else {
        clamped = v;
      }
      api.put_outxfield(clamped);
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
