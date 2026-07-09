# Isolette Simple (Event / Event Data Port Variant) Requirements

This file contains a sketch of functional requirements for the event/event-data-port
(EDP) variant of the Simple Isolette system (these need to be refined and expanded).

This variant preserves the control-law semantics of the original all-DataPort
Simple Isolette while moving to event-driven communication:

- the temperature sensor announces changes of the sensed value on a `temp_changed`
  event data port carrying the new value (the sampled `current_temp` data port is
  retained),
- the operator's set points arrive at the thermostat as event data messages
  (sent only when the set points change), so the thermostat latches the most
  recently received set points,
- the thermostat's heat control command is an event data message emitted only
  when the commanded state changes (send-on-change), so the heat source latches
  the most recently received command.

### Requirements

REQ_THERM_1
 - The commanded heat state shall be initially Off.
   (Realized jointly: the thermostat's latched command state starts Off and it emits
   no command message during initialization; the heat source independently starts
   in the Off state -- see REQ_HS_1.)

REQ_THERM_TRIGGER
 - The thermostat's control logic shall run only when a triggering event is present:
   a temperature-change event or a new set point message.  Without a trigger, the
   commanded heat state shall not be changed.

REQ_THERM_LATCH
 - The thermostat shall latch the most recently received set point message; until
   the first message arrives, the default desired range of [98, 101] shall be used.

REQ_THERM_2
- If triggered and the Current Temperature is less than the Lower Desired Temperature
  (of the latched set points), the commanded heat state shall be set to On.

REQ_THERM_3
- If triggered and the Current Temperature is greater than the Upper Desired Temperature
  (of the latched set points), the commanded heat state shall be set to Off.

REQ_THERM_4
- If triggered and the Current Temperature is greater than or equal to the Lower Desired
  Temperature and less than or equal to the Upper Desired Temperature (of the latched
  set points), the commanded heat state shall not be changed.

REQ_THERM_SOC (send-on-change)
- The thermostat shall emit a heat control message exactly when the commanded heat
  state changes, and the message shall carry the new commanded state.

REQ_TS_1:
- the Current Temperature provided by the temperature sensor lies within the range of
  96 and 103 inclusive.

REQ_TS_2:
- the temperature sensor shall raise a `temp_changed` event, carrying the new value,
  exactly when the sensed temperature value differs from the previously reported value.

REQ_HS_1
- The heat source is initially in the OFF state.

REQ_HS_2
- When commanded to be ON, the heat source shall be active (be in the On state).

REQ_HS_3
- When commanded to be OFF, the heat source shall not be active (be in the Off state).

REQ_HS_4
- When no command message is received, the heat source shall remain in its current state.

REQ_OP_1
- The lower desired temperature shall be less than or equal to the upper desired
  temperature in every emitted set point message.

REQ_OP_2
 - The lower desired temperature lies within the range of 97 to 99 inclusive.

REQ_OP_3
 - The upper desired temperature lies within the range of 99 to 102 inclusive.

REQ_OP_4
 - The operator interface shall emit a set point message only when the (simulated)
   operator changes the set points.
