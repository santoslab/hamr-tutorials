#include "splitter_splitter.h"

// This file will not be overwritten if HAMR codegen is rerun

void splitter_splitter_initialize(void) {
  printf("%s: splitter_splitter_initialize invoked\n", microkit_name);
}

void splitter_splitter_timeTriggered(void) {
  printf("%s: splitter_splitter_timeTriggered invoked\n", microkit_name);
}

void splitter_splitter_notify(microkit_channel channel) {
  // this method is called when the monitor does not handle the passed in channel
  switch (channel) {
    default:
      printf("%s: Unexpected channel %d\n", microkit_name, channel);
  }
}
