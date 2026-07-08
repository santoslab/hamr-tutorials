#include "decy_decy.h"

// This file will not be overwritten if HAMR codegen is rerun

void decy_decy_initialize(void) {
  printf("%s: decy_decy_initialize invoked\n", microkit_name);
}

void decy_decy_timeTriggered(void) {
  printf("%s: decy_decy_timeTriggered invoked\n", microkit_name);
}

void decy_decy_notify(microkit_channel channel) {
  // this method is called when the monitor does not handle the passed in channel
  switch (channel) {
    default:
      printf("%s: Unexpected channel %d\n", microkit_name, channel);
  }
}
