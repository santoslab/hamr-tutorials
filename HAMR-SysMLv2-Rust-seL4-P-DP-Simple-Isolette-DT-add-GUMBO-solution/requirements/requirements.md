# Isolette Simple Requirements

This file contains a sketch of functional requirements for the Simple Isolette system (these need to be refined and expanded).

### Requirements

REQ_THERM_1 
 - The Heat Control command shall be initially Off

REQ_THERM_2 
- If Current Temperature is less than the Lower Desired Temperature, 
  the Heat Control shall be set to On.
			
REQ_THERM_3 
- If the Current Temperature is greater than the Upper Desired Temperature, 
  the Heat Control shall be set to Off.
			
REQ_THERM_4 
- If the Current Temperature is greater than or equal to the Lower Desired Temperature
  and less than or equal to the Upper Desired Temperature, the value of
  the Heat Control shall not be changed."

REQ_THERM_5
- The Thermostat shall accept Current Temperature values between 95 and 104 inclusive.

REQ_THERM_6
- The Display Temperature output provided by the Thermostat lies within the range of 
  95 and 104 inclusive.

REQ_THERM_7
- The Display Temperature output shall be set to the value of the input Current Temperature.

REQ_TS_1: 
- the Current Temperature provided by the temperature sensor lies within the range of 
  96 and 103 inclusive.

REQ_HS_1
- The heat source is initially in the OFF state

REQ_HS_2
- When commanded to be ON, the heat source shall be active (be in the On state)

REQ_HS_3
- When commanded to be OFF, the heat source shall not be active (be in the Off state)

REQ_OP_1
- The lower desired temperature shall be less than or equal to the upper desired temperature

REQ_OP_2
 - The lower desired temperature lies within the range of 97 to 99 inclusive.

REQ_OP_3
 - The upper desired temperature lies within the range of 99 to 102 inclusive.

REQ_OP_4
- The visual display provided by the Operator Interface shall display the value received on the `display_temp` input port. 

REQ_OP_5
- The Operator Interface shall accept Display Temperature values between 90 and 110.

