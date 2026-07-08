// This file will not be overwritten if HAMR codegen is rerun

use data::*;
use crate::bridge::merger_merger_api::*;
use vstd::prelude::*;

verus! {

  pub struct merger_merger {
    // PLACEHOLDER MARKER STATE VARS
  }

  impl merger_merger {
    pub fn new() -> Self
    {
      Self {
        // PLACEHOLDER MARKER STATE VAR INIT
      }
    }

    pub fn initialize<API: merger_merger_Put_Api> (
      &mut self,
      api: &mut merger_merger_Application_Api<API>)
      ensures
        // PLACEHOLDER MARKER INITIALIZATION ENSURES
    {
      log_info("initialize entrypoint invoked");
      // Output data ports MUST be initialized.  Merge carries no integration
      // constraints, so put_outstruct has no range precondition.
      api.put_outstruct(SysPropStructSplit_Data_Model::StructXY { x: 0i32, y: 0i32 });
    }

    pub fn timeTriggered<API: merger_merger_Full_Api> (
      &mut self,
      api: &mut merger_merger_Application_Api<API>)
      requires
        // PLACEHOLDER MARKER TIME TRIGGERED REQUIRES
      ensures
        // BEGIN MARKER TIME TRIGGERED ENSURES
        // guarantee merge_x
        //   Output struct's x field equals the inxfield input.
        api.outstruct.x == api.inxfield,
        // guarantee merge_y
        //   Output struct's y field equals the inyfield input.
        api.outstruct.y == api.inyfield,
        // END MARKER TIME TRIGGERED ENSURES
    {
      log_info("compute entrypoint invoked");
      // Re-assemble the struct from the two scalar inputs.
      let x = api.get_inxfield();
      let y = api.get_inyfield();
      let out = SysPropStructSplit_Data_Model::StructXY { x, y };
      api.put_outstruct(out);
      log_merged(out);
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
  pub fn log_merged(value: SysPropStructSplit_Data_Model::StructXY)
  {
    log::info!("Merge: reassembled struct (x={0}, y={1})", value.x, value.y);
  }

  // PLACEHOLDER MARKER GUMBO METHODS

}
