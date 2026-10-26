#include "consume_consume.h"

// This file will not be overwritten if HAMR codegen is rerun

void consume_consume_initialize(void) {
  printf("%s: consume_consume_initialize invoked\n", microkit_name);
}

void consume_consume_timeTriggered(void) {
  printf("%s: consume_consume_timeTriggered invoked\n", microkit_name);
}

void consume_consume_notify(microkit_channel channel) {
  // this method is called when the monitor does not handle the passed in channel
  switch (channel) {
    default:
      printf("%s: Unexpected channel %d\n", microkit_name, channel);
  }
}
