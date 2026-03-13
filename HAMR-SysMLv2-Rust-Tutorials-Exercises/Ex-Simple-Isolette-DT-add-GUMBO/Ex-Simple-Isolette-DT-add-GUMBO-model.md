# HAMR Exercise: Adding a GUMBO Contract Clause (Part 1 - SysMLv2 model) 

**Purpose**:  The purpose of this exercise is to get you familiar with basic aspects of HAMR GUMBO contracts and the testing and verification infrastructure that HAMR automatically generates for them in code.

Starting from the previous Simple Isolette system with the Display Temperature feature, you will go through the steps of...
 - adding a new clause to an existing GUMBO contract in SysMLv2, 
 - running the HAMR type checker to check the well-formedness of your model

Upon completion of Part 1, you will have produced a version of the model which we will use for an exercise on HAMR GUMBOX Rust unit testing in Parts 2 and 3 of this exercise and Verus verification of HAMR-generated Rust components in Part 4 of this exercise.

## Prerequisites and Resources

Before working through this exercise, you should have gone through the following HAMR lectures (or read the equivalent documentation):

* HAMR Overview, HAMR Rust Component Development
* HAMR GUMBO Overview 

## Excercise Overview

This exercise is based on the "Simple Isolette" system concept used in a number of other HAMR tutorials and examples.  We are using a variant of the system for which the Display Temperature feature has been added.  The system has the following HAMR architecture.

![Simple Isolette (before refactoring)](images/simple-isolette-after.png)

In the architecture, note the `display_temp` output port on the `Thermostat` added in the last exercise aimed to satisfy the informal constraint that the output Display Temperature is equal to the input Current Temperature value.

In this exercise, we explicitly represent that intuition in a requirement (REQ_THERM_7) plus add additional requirements (invariant properties) on the Thermostat and Operator Interface Display Temp ports (REQ_THERM_6 and REQ_OP_5).

- **REQ_THERM_6**:
  The Display Temperature output provided by the Thermostat lies within the range of 
  95 and 104 inclusive.

- **REQ_THERM_7**:
  The Display Temperature output shall be set to the value of the input Current Temperature.

- **REQ_OP_5**: 
  The Operator Interface shall accept Display Temperature values between 90 and 110.

You will add clauses to the existing GUMBO contracts for the Thermostat thread and Operator Interface threads to formally specify the requirements above.  

In the subsequent parts of this exercise, you will test and verify that your Rust component implementations for the `Thermostat` and `Operator_Interface` satifies these contracts.

## Setting up the Starting Files for the Project 

The starting project (which includes both SysMLv2 models and Rust implementation) can be found in the folder `HAMR-SysMLv2-Rust-seL4-P-DP-Simple-Isolette-add-DT-solution` in [this repository](https://github.com/santoslab/hamr-tutorials/tree/main/HAMR-SysMLv2-Rust-seL4-P-DP-Simple-Isolette-add-DT-solution) Copy this folder (the *folder* itself, not just the contents) into a personal git repository and rename the folder to `HAMR-SysMLv2-Rust-seL4-P-DP-Simple-Isolette-DT-add-GUMBO-solution`.  One reason that it's important to copy the entire folder is that it contains several `.gitignore` that configure the folder for git use (e.g., by ignoring the very large executable files associated with Rust builds).  Open the `sysmlv2` subfolder in the CodeIVE to work on Part 1 of this exercise. 

## Activity 1 - Commit the Starting Files to a Git Repository

To understand the functionality (and the benefits) of HAMR, it will be useful to see (via file diffs) the updates to models and code that you have to make, versus updates to the code and microkernel specifications that HAMR makes for you.

* **Task:** Commit and push this addition to your repository with a commit message such as "Simple Isolette DT add GUMBO - initial files".

This requirement is independent of the particular value of the current temperature and the associated control flows. 

## Activity 2 - Add Integration Constraint to Thermostat Display Temp Output

REQ_THERM_6 above indicates an invariant property for the Thermostat Display Temperature output: its value always lies within a given range.  This is naturally expressed as an integration constraint on the `Thermostat` `display_temp` output port.

* **Task:** Add the following integration constraint below the existing `assume ASSM_CT_Range` integration constraint.  We can reuse the GUMBO declared functions/constants for the temperature bounds declared previously.

```
  guarantee REQ_THERM_6:
  	    Temp_Lower_Bound() <= display_temp.degrees & display_temp.degrees <= Temp_Upper_Bound();
```

Note that because this is a constraint for an output port, we use the `guarantee` key word when declaring the constraint.  This indicates that it is the responsibility of the Thermostat implementation to satisfy this constraint.

It is good practice to run CodeIVE "HAMR Type Checking" tool after adding contracts because these are not checking activity ("as you type") by the native SysMLv2 editor.

* **Task:** Run the "HAMR Type Checking" tool from the command palette to confirm that the syntax of the contract above is well-formed.  You should see 0 problems reported in the tool problem summary in the lower status bar of the editor.

* **Task:** Seed an error in the contract clause, e.g., mistype the port `display_temp` port name as `disxlay_temp`.  Run "HAMR Type Checking" and see that this name resolution error is detected.  You may also try some additional common forms of errors such as using the "assume" keyword for an output port constraint, or having an operator with a bad syntax such as `<+=`.

* **Task:** Reset the specification to the correct syntax and confirm that "HAMR Type Checking" reports no problems.

## Activity 3 - Add Clause to Thermostat Compute Entry Point Contract 

* **Task:** Add a contract clause to the `Thermostat` thread component to enforce the REQ_THERM_7 requirement above.  

This requirement is independent of the particular value of the current temperature and the associated control laws. Therefore, it can be placed in the general clause section of the `compute` contract (right below the `guarantee lastCmd` clause and above the `compute_cases` section).

Below is an example of how you might write the clause..
```
  guarantee REQ_THERM_7 "The Display Temperature output shall be set 
                         |to the value of the input Current Temperature":
    display_temp == current_temp;
```

Reminder: Run "HAMR Type Checking" to confirm that the syntax of the contract is well-formed (we won't mention this from now on).

## Activity 4 - Add Integration Constraint to Operator Interface Display Temp Input

REQ_OP_5 above indicates an invariant property for the Operator Interface Display Temperature input: it only accepts values within a given range.  This is naturally expressed as an integration constraint on the `Operator_Interface` `display_temp` input port.

* **Task:** Add an appropriate integration constraint on the Operator Interface below the current declaration of the ports.  The Operator Interface contains no GUMBO contracts, so we will need to add an entire GUMBO contract block to achieve this.  You can use something like the following..
```
  //-- B e h a v i o r    C o n s t r a i n t s --
  language "GUMBO" /*{
    integration
      assume REQ_OP_5:
         90 [i32] <= display_temp.degrees & display_temp.degrees <= 110 [i32];
  }*/
```

## Activity 5 - Perform Model-Level Contract Verification

This exercise has been designed to introduce integration constraints on both the sending and receiving side of the `display_temp` communication between the `Thermostat` and `Operator_Interface`.  This enables a learning activity in which we apply HAMR's verification for compatible integration constraints.  The exercise has been designed so that `display_temp` ports on the `Thermostat` and `Operator_Interface` have compatible constraints.  Let's confirm that in the task below.

* **Task:** Use the "HAMR SysMLv2 Logika Checking" tool from the tool command palette to check that all the connections in our model have compatible integration constraints.

Note that when the tool completes (its status can be seen on the lower status bar), no problems are reported.

* **Task:** Seed an integration constraint error and confirm that it is detected by the "HAMR SysMLv2 Logika Checking" tool.  For example, set the upper bound on the acceptable `display_temp` below.  The integration constraint checking should indicate that this is a violation.

```
  assume REQ_OP_5:
         90 [i32] <= display_temp.degrees & display_temp.degrees <= 100 [i32];
```

Click on the problems report indicator.  The problems report should indicate that there is an error associated with the connection below...

```
 connection dt : PortConnection connect thermostat.display_temp to operator_interface.display_temp;
```

* **Task:**  Remove the seeded bug above, re-run the "HAMR SysMLv2 Logika Checking" to confirm that all integration constraints are satisfied.

## Activity 6 - Commit Your Solution

Commit/Push your solution to your git repository with a commit message such as "Simple Isolette DT Add GUMBO - models completed".

## Activity 7 - Run HAMR Code Generation for Microkit

We now want to update the Verus contracts and GUMBOX (executable contracts derived from GUMBO contracts used for testing) in the code.

* **Task:**  Using the CodeIVE command palette, run the "HAMR SysMLv2 CodeGen" 

After a bunch of logging messages, you should see a message saying
```CodeGen Successful```

Scroll back up through the logging messages.  For the thermostat application file, you should see the following message (if needed, if you put the cursor in the terminal window where the logging info is, you can use the Command-F (perhaps CTRL-F on your OS) to activate the search function)...

```
info: Wrote and preserved existing content: /Users/hatcliff/Dev/git-repos/hamr-tutorials-git/HAMR-SysMLv2-Rust-seL4-P-DP-Simple-Isolette-DT-GUMBO-solution/hamr/microkit/crates/thermostat_thermostat/src/component/thermostat_thermostat_app.rs
```

This indicates that HAMR wrote to the file holding your application code for the thermostat, it preserved your code ("existing content"), but updated the contracts (adding a Verus contract clause in the `timeTriggered` method generated from the GUMBO `compute` clause for  `current_temp == display_temp`).

Also see if you can find the following message 
```
Wrote: /Users/hatcliff/Dev/git-repos/hamr-tutorials-git/HAMR-SysMLv2-Rust-seL4-P-DP-Simple-Isolette-DT-GUMBO-solution/hamr/microkit/crates/thermostat_thermostat/src/bridge/thermostat_thermostat_GUMBOX.rs
```

The `thermostat_thermostat_GUMBOX.rs` file holds the GUMBOX contracts generated for the thermostat.  This file contains no developer written code, so HAMR also overwrites it each time code generation is run.  It will contain a new boolean function representing the GUMBO `compute` clause constraint mentioned above.

In the activities below, we will take a look at these files.

## Activity 8 - Look at Code Generation Updates for Thermostat

* **Task**: Open a new window in the CodeIVE, then use the Open Folder option to open the Thermostat code crate (`hamr/microkit/crates/thermostat_thermostat`).  Now open the application code file (`/src/component/thermostat_thermostat_app.rs`).  Scroll down to the time-triggered method.  Toward the top of the `ensures` section, you should see that the Verus contract now includes a Verus clause corresponding to your newly added GUMBO `compute` `REQ_THERM_7` clause:

```
ensures
        // BEGIN MARKER TIME TRIGGERED ENSURES
        // guarantee lastCmd
        //   Set lastCmd to value of output Cmd port
        self.lastCmd == api.heat_control,
        // guarantee REQ_THERM_7
        //   The Display Temperature output shall be set 
        //   to the value of the input Current Temperature
        api.display_temp == api.current_temp,
```

HAMR knows where to insert the contract because of the `BEGIN MARKER TIME TRIGGERED ENSURES`.  Actually, HAMR regenerates the entire content between the each set of markers each time code generation is run.  That's why you should never directly modify anything between the markers.

* **Task**: Now open the infrastructure file (`src/bridge/thermostat_thermostat_api.rs`).  This file contains the APIs that HAMR auto-generates for working with thread component ports.  Look for the following method which (among other things), includes the newly added integration constraint for the `display_temp` output port...

```rust
  pub fn put_display_temp(
      &mut self,
      value: Isolette_Data_Model::Temp)
      requires
        // guarantee REQ_THERM_6
        (crate::component::thermostat_thermostat_app::Temp_Lower_Bound() <= value.degrees) &&
          (value.degrees <= crate::component::thermostat_thermostat_app::Temp_Upper_Bound()),
      ensures
        old(self).current_temp == self.current_temp,
        old(self).desired_temp == self.desired_temp,
        old(self).heat_control == self.heat_control,
        self.display_temp == value,
    {
      self.api.unverified_put_display_temp(value);
      self.display_temp = value;
    }
```

When your application code calls the `api.put_display_temp` method, Verus will see the contract for the method above, and due to the `requires` pre-condition, it will check that the value supplied as an argument to the `put_` method satisfies the outgoing integration constraint for the port.  Since all values that go onto that port must only be put there via the `put` call, this enforces an *invariant* for the port contents -- all values that ever exist in that port during run-time will satisfy the integration constraint.

Note for advanced users: There is a subtlety in the way that HAMR uses Verus for this file (and all other API files):
 - Verus is called (e.g., when you do `make verus`) to verify the contents of the *application* code file.  As Verus runs, Verus will check that, for any method called in the application code (including the api method), the pre-condition holds.  For the api methods, Verus will look at this API file to find the pre-conditions for the API `put_` methods.
 - HAMR doesn't call Verus to verify the contents of this API file (so the body of the method above is not verified by Verus).  That is because the API file in some sense forms the boundary between the Verus-verified application code, and the HAMR-generated infrastructure code.  For example, the method above `self.api.unverified_put_display_temp` is the root of code that interacts with seL4 move the `display_temp` value across the seL4 Microkit protection domain boundary. 
 
For now, HAMR is trusted to produce correct infrastructure code.  The infrastructure code is "lower-level" and contains many features that Verus cannot handle (including the interactions at the seL4 Microkit APIs).  Eventually, we hope to verify the correctness of the infrastructure code using other means.  In the meantime, the infrastructure code is tested to a high degree of confidence as part of HAMR development continuous integration process.

Finally, note that this strategy of forming a "boundary" being the verified code and unverified code (typically code outside of the language subset that the verifier can process) is common in all contract-based verification frameworks like Verus and Logika.  For example, in Logika, the notion of "Slang extension interfaces" are used to form such a boundary.  In the SPARK verification framework for Ada, defining a "SPARK boundary" is a well-established part of the methodology (You can Google for `SPARK Ada verification "SPARK boundary"`).

* **Task**: Now open the test infrastructure file (`src/bridge/thermostat_thermostat_GUMBOX.rs`). Around line 155 or so, you should see the following...

```rust
/** Compute Entrypoint Contract
  *
  * guarantee REQ_THERM_7
  *   The Display Temperature output shall be set 
  *   to the value of the input Current Temperature
  * @param api_current_temp incoming data port
  * @param api_display_temp outgoing data port
  */
pub fn compute_spec_REQ_THERM_7_guarantee(
  api_current_temp: Isolette_Data_Model::Temp,
  api_display_temp: Isolette_Data_Model::Temp) -> bool
{
  api_display_temp == api_current_temp
}
```

This is the "GUMBOX" (executable GUMBO contract) boolean function that can be executed during testing or run-time monitoring to evaluate the newly added `REQ_THERM_7` GUMBO contract on the components run-time state.  From another point of view, this function implements the semantics of the GUMBO contract clause directly in Rust.  This function is a "semantic companion" to the Verus clause view above.  (Note: we would actually like some way to prove that the GUMBOX clause has the same semantics as the Verus clause, but for now one just has to trust the HAMR code generation for that.)

At the bottom of the file, you can see the following method, which forms an executable post-condition derived from the complete GUMBO `compute` contract..

```rust
ub fn compute_CEP_Post(
  In_lastCmd: Isolette_Data_Model::On_Off,
  lastCmd: Isolette_Data_Model::On_Off,
  api_current_temp: Isolette_Data_Model::Temp,
  api_desired_temp: Isolette_Data_Model::Set_Points,
  api_display_temp: Isolette_Data_Model::Temp,
  api_heat_control: Isolette_Data_Model::On_Off) -> bool
{
  // I-Guar-Guard: Integration constraints for thermostat's outgoing ports
  let r0: bool = I_Guar_display_temp(api_display_temp);

  // CEP-Guar: guarantee clauses of thermostat's compute entrypoint
  let r1: bool = compute_CEP_T_Guar(lastCmd, api_current_temp, api_display_temp, api_heat_control);

  // CEP-T-Case: case clauses of thermostat's compute entrypoint
  let r2: bool = compute_CEP_T_Case(In_lastCmd, api_current_temp, api_desired_temp, api_heat_control);

  return r0 && r1 && r2;
}
```

This function is automatically called from GUMBOX testing infrastructure after a GUMBOX unit test is performed on this component to check that the input and output state of the component satisfy the GUMBO contract.  Take some time to follow the call tree for this method.  Note that...
 - `compute_CEP_T_Guar` will eventually call the previous method `compute_spec_REQ_THERM_7_guarantee` that checks that the output state of `display_temp` equals the input state of `current_temp`.
 - `I_Guar_display_temp` will check that the outgoing *GUMBO integration constraint* for `display_temp` is satisfied.
Intuitively, the semantics of this function aggregates the semantics of the Verus `timetriggered` `ensures` along with any integration constraints on output ports.

 Here is the GUMBOX function representing the integration constraint mentioned above (you can find it toward the top of the file)...

 ```rust
/** I-Guar: Integration constraint on thermostat's outgoing data port display_temp
  *
  * guarantee REQ_THERM_6
  */
pub fn I_Guar_display_temp(display_temp: Isolette_Data_Model::Temp) -> bool
{
  (Temp_Lower_Bound() <= display_temp.degrees) &
    (display_temp.degrees <= Temp_Upper_Bound())
}
 ```

**Conclusion**: Hopefully the above code walkthrough gives you some sense of some of the significant work that HAMR does for you behind the scenes to leverage the relatively few lines of GUMBO contracts that you wrote.  In the remaining parts of this exercise, we will working with GUMBOX testing and Verus verification to make use of the HAMR-generated artifacts in the walk-through above.





