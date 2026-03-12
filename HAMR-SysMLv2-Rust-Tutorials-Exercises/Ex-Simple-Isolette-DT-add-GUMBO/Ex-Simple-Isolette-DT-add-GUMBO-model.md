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




