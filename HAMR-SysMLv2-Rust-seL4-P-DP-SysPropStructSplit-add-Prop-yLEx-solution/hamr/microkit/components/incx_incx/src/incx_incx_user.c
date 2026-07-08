#include "incx_incx.h"

// This file will not be overwritten if HAMR codegen is rerun

void incx_incx_initialize(void) {
  printf("%s: incx_incx_initialize invoked\n", microkit_name);
}

void incx_incx_timeTriggered(void) {
  printf("%s: incx_incx_timeTriggered invoked\n", microkit_name);
}

void incx_incx_notify(microkit_channel channel) {
  // this method is called when the monitor does not handle the passed in channel
  switch (channel) {
    default:
      printf("%s: Unexpected channel %d\n", microkit_name, channel);
  }
}
