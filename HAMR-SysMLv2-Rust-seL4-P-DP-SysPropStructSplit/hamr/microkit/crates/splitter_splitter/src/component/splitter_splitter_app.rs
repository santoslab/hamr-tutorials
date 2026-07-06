// This file will not be overwritten if HAMR codegen is rerun

use data::*;
use crate::bridge::splitter_splitter_api::*;
use vstd::prelude::*;

verus! {

  pub struct splitter_splitter {
    // PLACEHOLDER MARKER STATE VARS
  }

  impl splitter_splitter {
    pub fn new() -> Self
    {
      Self {
        // PLACEHOLDER MARKER STATE VAR INIT
      }
    }

    pub fn initialize<API: splitter_splitter_Put_Api> (
      &mut self,
      api: &mut splitter_splitter_Application_Api<API>)
      ensures
        // PLACEHOLDER MARKER INITIALIZATION ENSURES
    {
      log_info("initialize entrypoint invoked");
      // Output data ports MUST be initialized.  Zero is within the
      // [-100, 100] guarantee range of both output ports.
      api.put_xfield(0i32);
      api.put_yfield(0i32);
    }

    pub fn timeTriggered<API: splitter_splitter_Full_Api> (
      &mut self,
      api: &mut splitter_splitter_Application_Api<API>)
      requires
        // PLACEHOLDER MARKER TIME TRIGGERED REQUIRES
      ensures
        // BEGIN MARKER TIME TRIGGERED ENSURES
        // guarantee split_x
        //   xfield equals the x field of the incoming struct.
        api.xfield == api.instruct.x,
        // guarantee split_y
        //   yfield equals the y field of the incoming struct.
        api.yfield == api.instruct.y,
        // END MARKER TIME TRIGGERED ENSURES
    {
      log_info("compute entrypoint invoked");
      // Read the incoming struct and forward each field onto its own port.
      // The instruct_range assumption ([-100, 100]) discharges the
      // xfield_range / yfield_range preconditions on the put_ calls.
      let s = api.get_instruct();
      api.put_xfield(s.x);
      api.put_yfield(s.y);
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
