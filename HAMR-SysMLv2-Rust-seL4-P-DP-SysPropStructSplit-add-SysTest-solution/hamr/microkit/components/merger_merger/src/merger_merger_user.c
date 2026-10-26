#include "merger_merger.h"

// This file will not be overwritten if HAMR codegen is rerun

void merger_merger_initialize(void) {
  printf("%s: merger_merger_initialize invoked\n", microkit_name);
}

void merger_merger_timeTriggered(void) {
  printf("%s: merger_merger_timeTriggered invoked\n", microkit_name);
}

void merger_merger_notify(microkit_channel channel) {
  // this method is called when the monitor does not handle the passed in channel
  switch (channel) {
    default:
      printf("%s: Unexpected channel %d\n", microkit_name, channel);
  }
}
