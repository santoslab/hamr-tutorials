// This file will not be overwritten if HAMR codegen is rerun

use data::*;
use crate::bridge::consume_consume_api::*;
use vstd::prelude::*;

verus! {

  pub struct consume_consume {
    // BEGIN MARKER STATE VARS
    pub last_x: i32,
    // END MARKER STATE VARS
  }

  impl consume_consume {
    pub fn new() -> Self
    {
      Self {
        // BEGIN MARKER STATE VAR INIT
        last_x: 0,
        // END MARKER STATE VAR INIT
      }
    }

    pub fn initialize<API: consume_consume_Put_Api> (
      &mut self,
      api: &mut consume_consume_Application_Api<API>)
      ensures
        // BEGIN MARKER INITIALIZATION ENSURES
        // guarantee init_last_x
        //   The state variable starts at zero.
        self.last_x == 0i32,
        // END MARKER INITIALIZATION ENSURES
    {
      log_info("initialize entrypoint invoked");
      // Establish the GUMBO initialize guarantee on the state variable.
      self.last_x = 0;
    }

    pub fn timeTriggered<API: consume_consume_Full_Api> (
      &mut self,
      api: &mut consume_consume_Application_Api<API>)
      requires
        // BEGIN MARKER TIME TRIGGERED REQUIRES
        // assume instruct_range
        //   Incoming struct fields lie in [-200, 200].
        (((-200i32 <= old(api).instruct.x) &&
          (old(api).instruct.x <= 200i32)) &&
          (-200i32 <= old(api).instruct.y)) &&
          (old(api).instruct.y <= 200i32),
        // END MARKER TIME TRIGGERED REQUIRES
      ensures
        // BEGIN MARKER TIME TRIGGERED ENSURES
        // guarantee track_x
        //   last_x records the x field of the most recently consumed struct.
        self.last_x == api.instruct.x,
        // END MARKER TIME TRIGGERED ENSURES
    {
      log_info("compute entrypoint invoked");
      // Sink component: read the incoming struct, latch its x field, and log it.
      // No output ports.
      let s = api.get_instruct();
      self.last_x = s.x;
      log_consumed(s);
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
  pub fn log_consumed(value: SysPropStructSplit_Data_Model::StructXY)
  {
    log::info!("Consume: received struct (x={0}, y={1})", value.x, value.y);
  }

  // PLACEHOLDER MARKER GUMBO METHODS

}
