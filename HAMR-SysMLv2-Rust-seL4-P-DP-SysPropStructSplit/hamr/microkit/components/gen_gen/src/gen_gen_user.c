#include "gen_gen.h"

// This file will not be overwritten if HAMR codegen is rerun

void gen_gen_initialize(void) {
  printf("%s: gen_gen_initialize invoked\n", microkit_name);
}

void gen_gen_timeTriggered(void) {
  printf("%s: gen_gen_timeTriggered invoked\n", microkit_name);
}

void gen_gen_notify(microkit_channel channel) {
  // this method is called when the monitor does not handle the passed in channel
  switch (channel) {
    default:
      printf("%s: Unexpected channel %d\n", microkit_name, channel);
  }
}
