#include "clampx_clampx.h"

// This file will not be overwritten if HAMR codegen is rerun

void clampx_clampx_initialize(void) {
  printf("%s: clampx_clampx_initialize invoked\n", microkit_name);
}

void clampx_clampx_timeTriggered(void) {
  printf("%s: clampx_clampx_timeTriggered invoked\n", microkit_name);
}

void clampx_clampx_notify(microkit_channel channel) {
  // this method is called when the monitor does not handle the passed in channel
  switch (channel) {
    default:
      printf("%s: Unexpected channel %d\n", microkit_name, channel);
  }
}
