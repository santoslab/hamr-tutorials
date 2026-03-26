# HAMR Exercise: Adding a GUMBO Contract Clause (Part 3 - Verus Verification) 

**Purpose**:  The purpose of this exercise is to get you familiar with basic aspects of HAMR GUMBO contracts and the testing and verification infrastructure that HAMR automatically generates for them in code.

This exercise picks up where Part 1 left off.  In Part 1, you...
 - added simple GUMBO contracts for the Display Temp feature in the Simple Isolette example, 
 - ran HAMR code generation and observed the HAMR-generated GUMBO eXecutable (GUMBOX) contracts and Verus logical contracts correspond to your added GUMBO contracts.

In Part 2, you worked with GUMBOX testing.  This exercise (Part 3) doesn't have any dependences on Part 2 (only Part 1).

In this exercise, you will apply Verus to verify code associated with the Display Temp feature of the Simple Isolette.  We'll also make extra effort to understand all parts of the Verus contracts generated from GUMBO (especially integration constraints).  As part of this process, we will do a very detailed walkthrough of the way that Verus and SMT track constraints from contracts and programs.  Don't be discouraged if you find this complicated.  The point is to illustrate a few times how it all works.  After that, you should just trust the tool to manage things.  However, it is important to know a bit of how this works so that you can appropriately debug situations when Verus finds a mismatch between your contracts and code.

As we come to the end of this exercise, you should feel confident applying GUMBO with both GUMBOX testing and Verus verification to your own examples.

## Prerequisites and Resources

Before working through this exercise, you should have gone through the following HAMR lectures (or read the equivalent documentation):

* HAMR Overview, HAMR Rust Component Development
* HAMR GUMBO Overview 
* Part 1 of this exercise

## Excercise Overview

This exercise is based on the "Simple Isolette" system concept used in a number of other HAMR tutorials and examples.  We are using a variant of the system for which the Display Temperature feature has been added, and then, from Part 1 of this assignment, GUMBO contracts were added in the SysMLv2 models for the Display Tempeature feature.  The system has the following HAMR architecture.

![Simple Isolette with Display Temp](images/simple-isolette-after.png)

In the architecture, note the `display_temp` output port on the `Thermostat` added in the last exercise aimed to satisfy the informal constraint that the output Display Temperature is equal to the input Current Temperature value.  Then, in Part 1 of this exercise we added GUMBO contracts to address the following requirements.

- **REQ_THERM_6**:
  The Display Temperature output provided by the Thermostat lies within the range of 
  95 and 104 inclusive.

- **REQ_THERM_7**:
  The Display Temperature output shall be set to the value of the input Current Temperature.

- **REQ_OP_5**: 
  The Operator Interface shall accept Display Temperature values between 90 and 110.

In this exercise, we will see the effect of the HAMR-generated Verus contracts for the requirements. 


## Setting up the Starting Files for the Project 

You will start from the files that were completed in Part 2 of this exercise.

## Activity 1 - Verifying Thermostat Compute Entry Point Code against the Auto-Generated Verus Contract for REQ_THERM_7 (and REQ_THERM_6)

In Part 1 of this exercise, we saw that the GUMBO compute clause for the Thermostat component, written as follows...

```
  guarantee REQ_THERM_7 "The Display Temperature output shall be set 
                         |to the value of the input Current Temperature":
    display_temp == current_temp;
```

...got translated by HAMR to the following Verus `ensures` clause in the `time-triggered` method.

```
ensures
        // BEGIN MARKER TIME TRIGGERED ENSURES
        // ...
        // guarantee REQ_THERM_7
        //   The Display Temperature output shall be set 
        //   to the value of the input Current Temperature
        api.display_temp == api.current_temp,
```

Recall from our documentation and lectures on GUMBO and Verus, HAMR declares code-level ghost variables (e.g., for Verus or Logika verification frameworks, depending on HAMR-generated application code language) to represent the application code's view of the port contents (this is what is termed the "application port state" in the formal HAMR semantics).   For example, in the file `src/bridge/thermostat_thermostat_api.rs`, you can see the ghost variable declarations for each Thermostat port...

```rust
  pub struct thermostat_thermostat_Application_Api<API: thermostat_thermostat_Api> {
    pub api: API,

    pub ghost current_temp: Isolette_Data_Model::Temp,
    pub ghost desired_temp: Isolette_Data_Model::Set_Points,
    pub ghost heat_control: Isolette_Data_Model::On_Off,
    pub ghost display_temp: Isolette_Data_Model::Temp
  }
```  

When you call HAMR-generated `put_` and `get_` methods for accessing component ports, behind the scenes in the `thermostat_thermostat_api.rs`, the HAMR generated infrastructure methods will maintain the ghost variables to reflect the abstract state of the ports.  For example, in the `put_display_temp` method, the method body..
- sends the actual value out to the communication infrastructure via the `unverified_put_display_temp` method
- updates the ghost variable `display_temp` to indicate that `value` is in the output port.

```rust
#[verifier::external_body]
pub fn put_display_temp(
   &mut self,
   value: Isolette_Data_Model::Temp)
 requires
   // guarantee REQ_THERM_6
   (crate::component::thermostat_thermostat_app::Temp_Lower_Bound() <= value.degrees) &&
       (value.degrees <= crate::component::thermostat_thermostat_app::Temp_Upper_Bound()),
 ensures
   old(self).current_temp == self.current_temp, // ghost variables of other port states don't change
   old(self).desired_temp == self.desired_temp, // ghost variables of other port states don't change
   old(self).heat_control == self.heat_control, // ghost variables of other port states don't change
   self.display_temp == value,
 {
   self.api.unverified_put_display_temp(value); // send value out comm infrastructure
   self.display_temp = value; // update ghost variable abstraction to indicate the value of the output port state
 }
```
In this way, the ghost state `api.display_temp` abstracts the status of the much more complicated lower-level communication channel (e.g., implemented as seL4 Microkit memory regions).

Similarly, in the `get_current_temp` method...
``` rust
 pub fn get_current_temp(&mut self) -> (res : Isolette_Data_Model::Temp)
      ensures
        old(self).current_temp == self.current_temp,
        res == self.current_temp,  // link the port ghost state to value returned from infrastructure
        old(self).desired_temp == self.desired_temp,
        old(self).heat_control == self.heat_control,
        old(self).display_temp == self.display_temp,
        // assume ASSM_CT_Range (integration constraint)
        (crate::component::thermostat_thermostat_app::Temp_Lower_Bound() <= res.degrees) &&
          (res.degrees <= crate::component::thermostat_thermostat_app::Temp_Upper_Bound()),
    {
      self.api.unverified_get_current_temp(&Ghost(self.current_temp))
    }
```
...we don't know exactly what value is coming in from the infrastructure as a result of the call to `unverified_get_current_temp` (represented by `res` in the contract), but have a constraint saying that the value of the port's ghost variable `self.current_temp` and `res` are equal.  Moreover, whatever value `res` is returned by the infrastructure, that value must satisfy any integration constraint declared for the port.  Such an assumption is sound because the HAMR model-level integration constraint framework will check that every component sending a value to the `current_temp` port satisfies the same constraint.

Thus, in the post-condition of `timetriggered` method, the equality below, which checks that the value of the `current_temp` ghost variable equals the `display_temp` ghost variable, has the effect of enforcing that, whatever value is present on actual infrastructure input port `current_temp`, it will match the value sent out on the actual infrastructure port `display_temp`.

```
ensures
        // BEGIN MARKER TIME TRIGGERED ENSURES
        // ...
        // guarantee REQ_THERM_7
        //   The Display Temperature output shall be set 
        //   to the value of the input Current Temperature
        api.display_temp == api.current_temp,
```

* **Task**: In the root folder of the Thermostat crate, run `make verus` have Verus try to verify that the code in your `.._app.rs` file (most importantly, the `initialize` and `timetriggered` methods) satisfies the Verus contracts.  You should see something like the below...

```
thermostat_thermostat $ make verus
RUSTC_BOOTSTRAP=1 cargo-verus verify -Z build-std=core,alloc,compiler_builtins -Z build-std-features=compiler-builtins-mem --target aarch64-unknown-none
   Compiling data v0.1.0 (/Users/hatcliff/Dev/git-repos/hamr-tutorials-git/HAMR-SysMLv2-Rust-seL4-P-DP-Simple-Isolette-DT-add-GUMBO-solution/hamr/microkit/crates/data)
verification results:: 6 verified, 0 errors
   Compiling thermostat_thermostat v0.1.0 (/Users/hatcliff/Dev/git-repos/hamr-tutorials-git/HAMR-SysMLv2-Rust-seL4-P-DP-Simple-Isolette-DT-add-GUMBO-solution/hamr/microkit/crates/thermostat_thermostat)
verification results:: 11 verified, 0 errors
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.20s
```

Before reading below, try to explain to yourself why the verification succeeds.  In particular, think about the relationships between the ghost variables representing the application port state, the program variable `currentTemp`, the result of the `get_current_temp` method, and the argument to the `put_display_temp` method.

Here are some key insights, in the application code, toward the top of the `timetriggered` method, we have...
```rust
 let currentTemp: Temp = api.get_current_temp();
```
In the lower-levels of the SMT verification the assignment statement above will establish an equality constraint between the program variable `currentTemp` and the result symbol `res` in the `get_current_temp` method, whose contents we showed above.  The `get_current_temp` contract established an equality constraint between `res` and the ghost variable `api.current_temp`.   This, at the point, the Verus/SMT has the following facts...
```
res == api.current_temp
currentTemp == res
```
`currentTemp` is an immutable variable and thus is never modified in the method.  
Towards the bottom of the `timetriggered` method, we have...
```rust
 api.put_display_temp(currentTemp); 
```
Working from the semantics of the method call, Verus/SMT establishes an equality constraint between the method actual parameter `currentTemp` and its formal parameter `value`.  Then, looking at the contract of `put_display_temp` shown above, we see that the contract `ensures` clause establishes an equality constraint between `value` and the ghost variable `api.display_port`.

Putting this all together, Verus/SMT has established the following facts...
```
[get_current_temp]res == api.current_temp
currentTemp == [get_current_temp]res
currentTemp == [put_display_temp]value
[put_display_temp]value == api.display_temp
```
These fact persist until the end of the `timetriggered` method, and from these Verus can prove (transitivity of equality) that the post-condition (`ensures`) clause holds:
```
   api.display_temp == api.current_temp,
```

Now we will seed an error (the same error that we used for illustration in GUMBOX testing) to show that Verus catches errors related to the `REQ_THERM_7` requirement.

* **Task**: Modify the body Switch out the `put_current_temp` with the seeded error in the comments of your code from the previous exercise.  You should have something like the following...
```
 // api.put_display_temp(currentTemp);  // add call to output display_temp
 api.put_display_temp(Temp {degrees: currentTemp.degrees + 1}); // seeded bug for display_temp
```

Run `make verus` to verify the Thermostat application code.  You should get something like the following as a result...

```
note: function body check: not all errors may have been reported; rerun with a higher value for --multiple-errors to find other potential errors in this function
  --> src/component/thermostat_thermostat_app.rs:83:5
   |
83 | /     pub fn timeTriggered<API: thermostat_thermostat_Full_Api> (
84 | |       &mut self,
85 | |       api: &mut thermostat_thermostat_Application_Api<API>)
   | |___________________________________________________________^

error: postcondition not satisfied
   --> src/component/thermostat_thermostat_app.rs:99:9
    |
 99 |         api.display_temp == api.current_temp,
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ failed this postcondition
...
146 |       self.lastCmd = currentCmd      
    |       ------------------------- at the end of the function body

error: precondition not satisfied
   --> src/component/thermostat_thermostat_app.rs:144:7
    |
144 |         api.put_display_temp(Temp {degrees: currentTemp.degrees + 1}); // seeded bug for dis...
    |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    |
   ::: src/bridge/thermostat_thermostat_api.rs:82:9
    |
 82 | /         (crate::component::thermostat_thermostat_app::Temp_Lower_Bound() <= value.degrees) &&
 83 | |           (value.degrees <= crate::component::thermostat_thermostat_app::Temp_Upper_Bound()),
    | |____________________________________________________________________________________________- failed precondition

verification results:: 10 verified, 1 errors
error: could not compile `thermostat_thermostat` (lib) due to 2 previous errors
make: *** [verus] Error 101
```

You can see that this message indicates that the specific post-condition clause `api.display_temp == api.current_temp,` fails to verify.   Before reading below, try to explain why this happens in terms of the facts/constraints Verus knows.

Based on same type of walkthrough of the code above, by the time it gets to the end of the method, Verus/SMT has established the following facts...
```
[get_current_temp]res == api.current_temp
currentTemp == [get_current_temp]res
Temp{ degrees: currentTemp.degrees + 1 } == [put_display_temp]value // effect of the seeded bug
[put_display_temp]value == api.display_temp
```
From these facts, Verus is able to prove that `api.display_temp == api.current_temp` does not hold.

Looking at the Verus output, you can also see that, with the bug, the `timetriggered` method fails to establish the outgoing integration constraint on the `display_temp` port.  Before reading below, try to explain why this happens in terms of the facts/constraints Verus knows.

From the `get_current_temp` contract, we have...
```
[get_current_temp]res == api.current_temp
(crate::component::thermostat_thermostat_app::Temp_Lower_Bound() <= [get_current_temp]res.degrees) &&
          ([get_current_temp]res.degrees <= crate::component::thermostat_thermostat_app::Temp_Upper_Bound()),
```
From the assignment of the `get_current_temp` to the program variable `currentTemp`, we have..
```
currentTemp == [get_current_temp]res
```
From the method call to `put_current_temp` we have..
```
Temp{ degrees: currentTemp.degrees + 1 } == [put_display_temp]value // effect of the seeded bug
```

However, from the constraints above, we cannot verify that the following pre-condition to `put_display_temp` holds...
```
   (crate::component::thermostat_thermostat_app::Temp_Lower_Bound() <= [put_display_temp]value.degrees) &&
     ([put_display_temp]value.degrees <= crate::component::thermostat_thermostat_app::Temp_Upper_Bound()),
```
because, due to the previous equality constraint between `value` and `currentTemp.degrees + 1`, that would require...
```
   (crate::component::thermostat_thermostat_app::Temp_Lower_Bound() <= currentTemp.degrees + 1) &&
     (currentTemp.degrees + 1 <= crate::component::thermostat_thermostat_app::Temp_Upper_Bound()),
```
and due to the equality constraint between `currentTemp` and `[get_current_temp]res`, that would require
```
   (crate::component::thermostat_thermostat_app::Temp_Lower_Bound() <= [get_current_temp]res.degrees + 1) &&
     ([get_current_temp]res.degrees + 1 <= crate::component::thermostat_thermostat_app::Temp_Upper_Bound()),
```
which cannot be proved from the initial fact..
```
(crate::component::thermostat_thermostat_app::Temp_Lower_Bound() <= [get_current_temp]res.degrees) &&
          ([get_current_temp]res.degrees <= crate::component::thermostat_thermostat_app::Temp_Upper_Bound()),
```          
For example, according to the initial fact from `get_current_temp`, it could be the case that the current temperature is at the upper bound of the allowed range, i.e.,...
```
[get_current_temp]res.degrees == crate::component::thermostat_thermostat_app::Temp_Upper_Bound()
```
..but, due to equality constraints, this would mean..
```
currentTemp.degrees == crate::component::thermostat_thermostat_app::Temp_Upper_Bound()
```
Then, when we executed the seeded bug `api.put_display_temp(Temp {degrees: currentTemp.degrees + 1})`, we would know
```
currentTemp.degrees + 1 > crate::component::thermostat_thermostat_app::Temp_Upper_Bound()
```
..and since `currentTemp.degrees + 1 == [put_display_temp]value.degrees` due to parameter passing to `put_display_temp`, then (substituting "equals for equals")..
```
[put_display_temp]value.degrees > crate::component::thermostat_thermostat_app::Temp_Upper_Bound())
```
..which violates the following conjunct from the `put_display_temp` pre-condition...
```
[put_display_temp]value.degrees <= crate::component::thermostat_thermostat_app::Temp_Upper_Bound()
```

In conclusion, fortunately, you don't have to do this type of reasoning in your head for every contract -- Verus and SMT does it for you!   But you should at least have some intuition of how the reasoning process works for simple examples.  

* **Task**: Replace the seeded bug with the original code for the output value of `display_temp`, and re-run `make verus` to make sure the code has no errors.

## Activity 2 - Understanding the Impact of the Outgoing `display_temp` Integration Constraint in the Thermostat Initialize Entry Point

We have emphasized that GUMBO integration constraints enforce "port invariants" on values flowing through the port...
- if application code is `put`ing a value on the port, it must ensure that the integration constraint holds for the argument of the `put` method
- if application code is `get`ing a value from the the port, it gets to assume that the integration constraint holds for the value it is receiving

These principles also hold for application code in `initialize` methods.  Principles for `get` methods don't apply, because `initialize` entry point code cannot read from ports.  However, any uses of `put` must satisfy any integration contraints on the associated port.  To see this in action, consider the code in the Thermostat `initialize` entry port.

```rust
self.lastCmd = On_Off::Off;

// REQ_THERM_1: The Heat Control shall be initially Off
let currentCmd = On_Off::Off;
api.put_heat_control(currentCmd);

// Add initialization of display temp
api.put_display_temp(Temp { degrees: initial_display_temp_degrees });
```
When Verus is run for on a HAMR component, it will check both `initilize` and compute (`timetriggered` method) entry point code against contracts.   In the code above, the `put_display_temp` call satisfies the stated integration constraint associated with `REQ_THERM_6` because `initial_display_temp_degrees` is declared to have a value of `98`.

```rust
pub const initial_display_temp_degrees: i32 = 98;
```

To see that Verus is checking the call to `put_display_temp` as appropriate (i.e., that its precondition on its argument enforcing the integration constraint for `REQ_THERM_6` holds), let's seed an error and watch Verus catch it.

* **Task**: Change the argument of `put_display_temp` to be a `Temp` value with degrees of `89` (the lower bound of the required range of `REQ_THERM_6` is `95`) as shown below, and run Verus on the code (`make Verus` in the top folder of the `thermostat_thermostat` crate).

```rust
// Add initialization of display temp
// api.put_display_temp(Temp { degrees: initial_display_temp_degrees });
api.put_display_temp(Temp { degrees: 89i32 });  // seeded error
```

The output from Verus should look something like what is shown below...
```
error: precondition not satisfied
  --> src/component/thermostat_thermostat_app.rs:78:7
   |
78 |         api.put_display_temp(Temp { degrees: 89i32 });
   |         ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
  ::: src/bridge/thermostat_thermostat_api.rs:82:9
   |
82 | /         (crate::component::thermostat_thermostat_app::Temp_Lower_Bound() <= value.degrees) &&
83 | |           (value.degrees <= crate::component::thermostat_thermostat_app::Temp_Upper_Bound()),
   | |____________________________________________________________________________________________- failed precondition

verification results:: 10 verified, 1 errors
```

You can see from the output that Verus detected that the value being placed on the `display_temp` port does not satisfy the pre-condition of the `put_display_temp` method, which realizes the integration constraint.

* **Task**: Remove the seeded error, re-run Verus, and confirm that the application code of the Thermostat thread conforms to its contracts.  You should see output from Verus that is something like the following...
```
RUSTC_BOOTSTRAP=1 cargo-verus verify -Z build-std=core,alloc,compiler_builtins -Z build-std-features=compiler-builtins-mem --target aarch64-unknown-none
   Compiling thermostat_thermostat v0.1.0 (/Users/hatcliff/Dev/git-repos/hamr-tutorials-git/HAMR-SysMLv2-Rust-seL4-P-DP-Simple-Isolette-DT-add-GUMBO-solution/hamr/microkit/crates/thermostat_thermostat)
verification results:: 11 verified, 0 errors
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.50s
```


## Activity 3 - Understanding the Impact of the Incoming `display_temp` Integration Constraint in the Operator Interface 

As we've discussed the integration constraints for the Thermostat thread, we've already gained a lot of intuition about how...
- the integration constraint on the `current_temp` input port set up a constraint on the `api.current_temp` ghost variable, and 
- how the effects of that constraint perculated through the `timetriggered` method code to enable the verification of the outgoing integration constraint on `display_temp`.

We now want to explore the effects of the integration constraint on the Operator Interface input `display_temp` port.

* **Task**: In a new CodeIVE window, open the top-level folder for the `operator_interface_operator_interface` crate, and open a terminal window in the top-level folder of the crate so that you can run Verus on it.

Verus will report errors on the existing Operator Interface code that are unrelated to the new contracts that we have added for `display_temp`.  It turns out that these are not actually errors in the code; Verus just needs more information (in the form of additional non-GUMBO-derived contracts) to prove correctness.  We'll address techniques for getting Verus to verify successfully in future lectures.

What we want to do now is "work around" the current Verus-reported errors to understand the impact of the newly added `REQ_OP_5` integration constraint on the input `display_temp` port.

In the SysML models, in Part 1 of this exercise, we added the following integration constraint in the Operator Interface thread component...

```
 //-- B e h a v i o r    C o n s t r a i n t s --
 language "GUMBO" /*{
    integration
      assume REQ_OP_5:
        90 [i32] <= display_temp.degrees & display_temp.degrees <= 110 [i32];
 }*/
```

When you ran HAMR code generation at the end of Part 1, HAMR updated the port API definitions in `src/bridge/operator_interface_operator_interface_api.rs` to include a contract on the `get_display_temp` method that makes the "assume"-ed integration constraint above visible to the application code.

```rust
  pub fn get_display_temp(&mut self) -> (res : Isolette_Data_Model::Temp)
      ensures
        old(self).desired_temp == self.desired_temp,
        old(self).display_temp == self.display_temp,
        res == self.display_temp,
        // assume REQ_OP_5
        (90i32 <= res.degrees) &&
          (res.degrees <= 110i32),
    {
      self.api.unverified_get_display_temp(&Ghost(self.display_temp))
    }
```

Don't be confused by the fact that we are using an "ensures" in this method to implement what is in essence an assumption for the application code in the time-triggered method.  Following the principles of compositional verification, when code in the `timetriggered` method (the "calling content") calls `get_display_temp`, after `get_display_temp` returns, it gets to assume that constraints in the `get_display_temp` post-condition hold.  Specifically, after the return of the call below...

```rust
let display_temp: Temp = api.get_display_temp(); 
```
...the Verus fact set includes the following constraint `(90i32 <= display_temp.degrees) && (display_temp.degrees <= 110i32)`.  The subsequent code in `timetriggered` gets to use the fact in its Verus-supported verification. 

This assumption is sound because the broader HAMR compositional verification framework will not indicate a "completed verification" unless every component that sends a value to the Operator Interface `display_temp` port ensures that the value they are sending satisifies this constraint.  This is typically achieved in two steps..
- the model-level integration constraint checking (HAMR SysMLv2 Logika Check) checks that the integration constraint on the sending port (e.g., the constraint implementing `REQ_THERM_6` on the Thermostat `display_temp` output port) entails integration constraints on receiving ports (e.g., the Operator Interface integration constraint above implementing the `REQ_OP_5` requirement on the input `display_temp` port) -- we saw this checked in Part 1 of this exercise, 
- the application code in the sending component (e.g., the `initialize` and `timetriggered` methods in Thermostat) satisfy the integration constraint on the sending port (e.g., the code satisfies the `REQ_THERM_6` constraint for the `put_display_temp` method as we illustrated in Activity 1 of this exercise above).

We now go through some simple tasks to see the effect of the "assume" integration constraint above: we add some assertions to the application code to illustrate that Verus "knows" the fact after the `get_display_temp` call.

* **Task**: In the `timetriggered` method, right below the call to `get_display_temp` add an assertion as shown below, and then run Verus on the code.

```rust
// -------------- Process display temp ------------------
   let display_temp: Temp = api.get_display_temp(); 

   assert(display_temp.degrees >= 91i32);
```

You should see Verus output something like the following (there will be other unrelated error reports as well)...
```
error: assertion failed
   --> src/component/operator_interface_operator_interface_app.rs:112:14
    |
112 |       assert(display_temp.degrees >= 91i32);
    |              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ assertion failed
```

This indicates that Verus cannot prove that the `display_temp.degrees` is greater than or equal to `91`.  Note that this is because the integration constaint on `display_temp` tells that `display_temp.degrees` will be greater or equal to `90`.  In other words, it is possible to satisfy the integration constraint by having `display_temp.degrees == 90`, which would violate our assertion.

* **Task**: Modify the assertion as below (changing the value from `91` to `90`), and then run Verus on the code.

```rust
// -------------- Process display temp ------------------
   let display_temp: Temp = api.get_display_temp(); 

   assert(display_temp.degrees >= 90i32);
```

You should see that the "assertion failed" is no longer present in the Verus output (other unrelated errors will appear, as noted above). 

This is evidence that Verus "knows about" the integration constraint on the incoming `display_temp` port.  In terms of our earlier discussion in this activity, immediately after the call to `get_display_temp`, the Verus fact set includes `(90i32 <= display_temp.degrees) && (display_temp.degrees <= 110i32)` as a result of the post-condition on `get_display_temp`.

* **Task**: Comment out the assertion in your code, and Commit / Push your changes with a message like "Simple Isolette DT Add GUMBO - Verus verification activities completed".
