# HAMR Exercise: Adding a GUMBO Contract Clause (Part 2 - GUMBOX Testing) 

**Purpose**:  The purpose of this exercise is to get you familiar with basic aspects of HAMR GUMBO contracts and the testing and verification infrastructure that HAMR automatically generates for them in code.

This exercise picks up where Part 1 left off.  In Part 1, you...
 - added simple GUMBO contracts for the Display Temp feature in the Simple Isolette example, 
 - ran HAMR code generation and observed the HAMR-generated GUMBO eXecutable (GUMBOX) contracts and Verus logical contracts correspond to your added GUMBO contracts.
 
In this exercise, you will work with both manual and automated property-based GUMBOX tests.  These are thread component (unit) tests that use the auto-generated GUMBOX functions as test oracles.  "Manual GUMBOX tests" are manual in the sense that you explicitly construct the test vectors (test inputs) for tests.   These "manual" tests stand in contrast to the "automated" or "property-based" GUMBOX tests in which randomized test vectors are automatically created using the Rust "PropTest" libraries.

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

In this exercise, we will see the effect of the HAMR-generated GUMBOX test oracles for the requirements.  In the final part of this exercise, you will verify using Verus that your Rust component implementations for the `Thermostat` satisfy these contracts.

## Setting up the Starting Files for the Project 

You will start from the files that were completed in Part 1 of this exercise.

## Activity 1 - Running Existing GUMBOX Test and Observing that Oracle is Automatically Updated with Newly Added GUMBO Constraints

One of the nice features of the GUMBOX testing framework is that when you update the model-level GUMBO contracts for a thread, the code-level GUMBOX test oracles for the thread are automatically updated when you run HAMR code generation.  Thus, as we have added the display temperature feature and the associated GUMBO contracts, the original Simple Isolette manual GUMBOX test oracles are updated behind the scenes to account for the new GUMBO clauses.  For example, for the Thermostat component, HAMR auto-generated a new version of the GUMBOX test oracle code in `src/bridge/thermostat_thermostat_GUMBOX.rs` to check that `REQ_THERM_6` and `REQ_THERM_7` hold (i.e., the outgoing `display_temp` value is between 95 and 104 inclusive, and the outgoing `display_temp` value equals the the incoming `current_temp` value).  Below we go through some activities to try to understand how this works.

Open the `thermostat_thermostat` crate in the CodeIVE, and then open the `tests.rs` file for the component.

* **Task:** Run the existing test `test_compute_GUMBOX_manual_REQ_THERM_2` *using the CodeIVE individual test runner annotation*. The test should pass.  Despite the name ending in `THERM_2` this test will now also check for `REQ_THERM_6` and `REQ_THERM_7` because the GUMBOX oracle that it calls has been updated.  

Looking at the test code for `test_compute_GUMBOX_manual_REQ_THERM_2`, we can see that the construction of the test vector for the test is coded as follows...

```rust
 // generate values for the incoming data ports
let current_temp = Temp { degrees: 95 };
let lower_desired_temp = Temp { degrees: 98 };
let upper_desired_temp = Temp { degrees: 100 };
let desired_temp = Set_Points {lower: lower_desired_temp, upper: upper_desired_temp};
```

Note that if we had added a new *input* port to the component, then we would need to add code to construct an *input value* for the added port.   However, the `display_temp` port that we added is an *output port*.  Because of this, we don't actually have to update our test code (but HAMR automatically updated the test oracle when we ran code generation).  When we added the associated GUMBO contract in Part 1 of this exercise,...

```gumbo
 guarantee REQ_THERM_7 "The Display Temperature output shall be set 
                       |to the value of the input Current Temperature":
   display_temp == current_temp;
```
...HAMR code generation automatically updated the test oracle for our tests to include a new check for this requirement.  Specifically, whenever we now call the `cb_apis::testComputeCB` method, it will now include a check that `REQ_THERM_7` (and actually `REQ_THERM_6` as well) hold for our same input test vector and the associated output.  Recall that when we were writing manual HAMR tests without GUMBOX (like you did in the original "Add Display Temp" exercise), we needed to tests and code to explicitly fetch the resulting output `display_temp` value and compare it for equality against the input `current_temp` value.  Now, the auto-generated GUMBOX infrastructure does that for us automatically.

Let's first focus on the compute contract clause associated with `REQ_THERM_7`.

To understand what is happening with the HAMR-generated GUMBO test oracle behind the scenes, navigate through the following calls in the `cb_apis::testComputeCB` call tree (i.e., successively open each of the methods below and observe their implementations)...

- `cb_apis::testComputeCB` - called from your test
- `GUMBOX::compute_CEP_Post` - called to evaluate the GUMBOX post-condition for the component that HAMR auto-generated from the GUMBO contract
- `compute_CEP_T_Guar` - called to check the top-level guarantees (the non-case clauses) of the GUMBO contract
- `compute_spec_REQ_THERM_7_guarantee` - called to check the newly added `REQ_THERM_7` guarantee GUMBO clause

In this last method, you can see the Rust executable version that was auto-generated by HAMR from the `REQ_THERM_7` contract clause...
```rust
pub fn compute_spec_REQ_THERM_7_guarantee(
  api_current_temp: Isolette_Data_Model::Temp,
  api_display_temp: Isolette_Data_Model::Temp) -> bool
 {
   api_display_temp == api_current_temp
 }
```

* **Task:** To see the effect of the check for `REQ_THERM_7` being present in the oracle now, seed the following bug in the application code at the point where the `display_temp` value is placed on the output port...
```rust
  // api.put_display_temp(currentTemp);  // add call to output display_temp
  api.put_display_temp(Temp {degrees: currentTemp.degrees + 1}); // seeded bug for display_temp
```
This will change the application code to make the `display_temp` output value one greater (i.e., not equal to) than the `current_temp` input value on every invocation of the compute entry point.  

Now, rerun the test.  You should see output in the terminal window something like the following (indicating test failure)...
```
failures:
    test::tests::GUMBOX_manual_tests::test_compute_GUMBOX_manual_REQ_THERM_2

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 22 filtered out; finished in 0.03s

error: test failed, to rerun pass `-p thermostat_thermostat --lib`
```

Leave the error in the code -- we will now use it to see the effects on running all the tests at once from the command line.

* **Task:** In a terminal window, at the top-level of the `thermostat_thermostat` crate, run `make test` to run all of the tests in `tests.rs`.  You should get output concluding in something like the below..
```
failures:
    test::tests::GUMBOX_manual_tests::test_compute_GUMBOX_manual_1_a
    test::tests::GUMBOX_manual_tests::test_compute_GUMBOX_manual_REQ_THERM_2
    test::tests::GUMBOX_manual_tests::test_compute_GUMBOX_manual_REQ_THERM_2_container
    test::tests::GUMBOX_tests::prop_testComputeCBwGSV_REQ2thru4
    test::tests::tests::test_compute_display_temp

test result: FAILED. 16 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Take a moment to think about the reasons for this output before reading below.

Here are some key insights...
- all of the GUMBOX tests, whether manual (`GUMBOX_manual_tests`) or property-based (`GUMBOX_tests`) testing, fail because they are calling the same oracle, which now checks that `display_temp == current_temp` after the compute entry point is executed.
- your original test `test_compute_display_temp` -- added when you first added the `display_temp` feature in an earlier exercise -- now fails.  In that test, you manually fetched the `display_temp` output value and compared it for equality against the `current_temp` input value.  This is what the GUMBOX tests accomplish automatically (because we later specified the desired equality property as a GUMBO contract).
- the other tests for the other `THERM` requirements pertaining to the `heat_control` still pass, because they don't check anything about `display_temp` and we haven't modified any of the code that pertains to the `heat_control`.

* **Task:** Restore the Original Code by Commenting Out the Seeded Error.

Now, comment out the seeded error as follows...
```rust
 api.put_display_temp(currentTemp);  // add call to output display_temp
 // api.put_display_temp(Temp {degrees: currentTemp.degrees + 1}); // seeded bug for display_temp
```
Leave the seeded error *as a comment* in the code.  We will use it later for assessing Verus verification.

Re-run `make test`.  This time, there should be no test failures.


## Activity 2 - Observing the Presence of Outgoing Integration Constraint in GUMBOX Oracle

We would like to see the effects of the newly added integration constraint `REQ_THERM_6` on the testing.  But it is difficult to see the effects for the following reason.  The Thermostat declares integration contracts with the same constraint on both the input `current_temp` and output `display_temp`.  

```
//-- == I n t e g r a t i o n     C o n s t r a i n t s	
 integration
 	  assume ASSM_CT_Range:
      Temp_Lower_Bound() <= current_temp.degrees & current_temp.degrees <= Temp_Upper_Bound();
    guarantee REQ_THERM_6:
      Temp_Lower_Bound() <= display_temp.degrees & display_temp.degrees <= Temp_Upper_Bound();
```

Since the intent of the compute logic is that the `display_temp` is set to the value of `current_temp`, there is no way to set the input `current_temp` to violate the integration constraint range on `display_temp` (because that would mean that we would need to violate the input integration constraint for `current_temp`).  But if the input integration is violated (it is part of the component precondition), the GUMBOX test will not even complete -- it will return a value indicating that the pre-condition is violated.  If we try to modify the code as was done earlier with the seeded error to put a bad value out for the `display_temp` this will cause the `compute` clause for `REQ_THERM_7` to be violated as well.  Therefore, as it stands, there is no good way to write a test or seed a bug showing the effects of the `REQ_THERM_6` integration constraint.  This is not a limitation of the framework; it just so happens that the arrangement of constraints for the component makes it difficult to pedagogically illustrate the `REQ_THERM_6` oracle behavior.

What we can do is temporarily modify the integration constraint in model and run HAMR code generation to generate a code-level constraint for which we can more easily observe its behavior.  The temporary modification that we propose doesn't really make sense in the "big picture" of the requirements design for the thermostat, but we can use it to learn about the way that the integration constraint impacts the GUMBOX oracle.

In the previous exercise, we saw that the GUMBOX contract for `REQ_THERM_6` integration constraint is generated as follows in `src/bridge/thermostat_thermostat_GUMBOX.rs`...

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

We'll now temporarily modify the model level integration constraint, which will cause HAMR to generate a modified version of the code above.

* **Task**: Create a new window in the CodeIVE and open the project's `sysmlv2` folder.  Then modify the `display_temp` output integration constraint for the Thermostat thread in `Isolette_Software.sysml` as follows (commenting out the original constraint)..

```
guarantee REQ_THERM_6:
   // Temp_Lower_Bound() <= display_temp.degrees & display_temp.degrees <= Temp_Upper_Bound();
   96 [i32] <= display_temp.degrees & display_temp.degrees <= Temp_Upper_Bound();
```

This will give us a situation where the input input integration constraint on `current_temp` has a lower bound of `95`, but the output integration constraint has a lower bound of `96`.  Thus, we will be able to construct a manual GUMBOX test with an input of `95` for `current_temp` that passes the component precondition but violates the output integration in the post-condition (because `95` is below the declared lower bound of `96`).

Once you have change the integration constraint, open the `Isolette.sysml` folder and run code generation using the existing Microkit code generation options embedded at the top of your file.

Once code generation completes, observe that the `I_Guar_display_temp` implementation in the GUMBOX oracle code looks like this now (the original lower bound has been replaced with `96i32`)...

```
/** I-Guar: Integration constraint on thermostat's outgoing data port display_temp
  *
  * guarantee REQ_THERM_6
  */
pub fn I_Guar_display_temp(display_temp: Isolette_Data_Model::Temp) -> bool
{
  (96i32 <= display_temp.degrees) &
    (display_temp.degrees <= Temp_Upper_Bound())
}
```

* **Task**: In `tests.rs`, run the manual GUMBOX test `test_compute_GUMBOX_manual_REQ_THERM_2` *using the CodeIVE individual test runner annotation*.

You should see output that concludes as follows, indicating a failure of the test...
```
failures:
    test::tests::GUMBOX_manual_tests::test_compute_GUMBOX_manual_REQ_THERM_2

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 20 filtered out; finished in 0.02s
```

Before reading below, try to explain why this happens...

To understand what is happening with the HAMR-generated GUMBO test oracle behind the scenes, navigate through the following calls in the `cb_apis::testComputeCB` call tree (i.e., successively open each of the methods below and observe their implementations)...

- `cb_apis::testComputeCB` - called from your test
- `GUMBOX::compute_CEP_Post` - called to evaluate the GUMBOX post-condition for the component that HAMR auto-generated from the GUMBO contract
- `I_Guar_display_temp` - (the method illustrated) - called to check output integration constraint for REQ_THERM_6.

* **Task**: Now restore the original integration constraint in the Thermostat thread in `Isolette_Software.sysml`, re-run code generation from `Isolette.sysml`, and re-run `test_compute_GUMBOX_manual_REQ_THERM_2` to see that it passes, and run all tests using `make test` to see that they pass.

## Activity 3 - Observing the Presence of Incoming Integration Constraint in GUMBOX Oracle for Operator Interface

Finally, we want to observe the effects of the newly added integration on the Operator Interface `display_temp` input.

```
//-- B e h a v i o r    C o n s t r a i n t s --
  language "GUMBO" /*{
     integration
        assume REQ_OP_5:
           90 [i32] <= display_temp.degrees & display_temp.degrees <= 110 [i32];
  }*/
```

In a new CodeIVE window, open the folder for the `operator_interface_operator_interface` crate.  You can see that HAMR auto-generated a GUMBOX oracle file `src/bridge/operator_interface_operator_interface_GUMBOX.rs` with methods below providing the executable version of the constraint above as part of a pre-condition...

```
/** I-Assm: Integration constraint on operator_interface's incoming data port display_temp
  *
  * assume REQ_OP_5
  */
pub fn I_Assm_display_temp(display_temp: Isolette_Data_Model::Temp) -> bool
{
  (90i32 <= display_temp.degrees) &
    (display_temp.degrees <= 110i32)
}

/** CEP-Pre: Compute Entrypoint Pre-Condition for operator_interface
  *
  * @param api_display_temp incoming data port
  */
pub fn compute_CEP_Pre(api_display_temp: Isolette_Data_Model::Temp) -> bool
{
  // I-Assm-Guard: Integration constraints for operator_interface's incoming ports
  let r0: bool = I_Assm_display_temp(api_display_temp);

  return r0;
}
```

Then open `tests.rs`.  In `tests.rs`, you can see that there are no HAMR-generated examples for writing GUMBOX tests.  That's because we originally (before Part 1 of this exercise) did not have any GUMBO contracts, so HAMR did not generate any GUMBOX infrastructure.  When we added the Operator Interface GUMBO integration constraint, HAMR would have generated GUMBOX example tests for us, but it does not overwrite `tests.rs` because it contains developer-written code, and HAMR currently does not support automated "weaving" of tests as it does for Verus contracts generated from GUMBO.  

However, now that we have seen many manual GUMBOX tests in the Thermostat, we can easily add some appropriate tests using the HAMR-generated helpers like `cb_apis::testComputeCB` in the file `cb_apis.rs` (which abbreviates "contract-based" (testing) apis).

* **Task**:  Below the existing `tests` module, add a new module with a single GUMBOX manual test as follows...

```rust
mod GUMBOX_manual_tests {
  use serial_test::serial;
 
  use crate::test::util::*;
  use data::Isolette_Data_Model::*;

  #[test]
  #[serial]
  fn test_compute_GUMBOX_manual_REQ_OP_5() {
    // generate values for the incoming data ports
    let display_temp = Temp { degrees: 95 };

    let harness_result = 
       cb_apis::testComputeCB(display_temp);

    assert!(matches!(harness_result, cb_apis::HarnessResult::Passed));
  }
}
```

Run the test and see that it passes.  This illustrates an input value of `95` that satisfies the range constraint of the declared integration contract.

Now we want to design a test that illustrates that our input integration constraint is working appropriately as a pre-condition.  To do this, we want to design a test where the `harness_result` has a pre-condition failure for a input, say `78` that doesn't satisfy the pre-condition.  If you navigate through to the definition of `harness_result`, you see that one of its enum values is `RejectedPrecondition` (i.e., the test harness rejects the attempt to run because the given input doesn't satisfy the pre-condition).  

* **Task**:  Add another manual GUMBOX test with the name `test_compute_GUMBOX_manual_failing_REQ_OP_5` that gives an input value of `78` for `display_temp`.  The test should look something like the one below.

```rust
  #[test]
  #[serial]
  fn test_compute_GUMBOX_manual_failing_REQ_OP_5() {
    // generate values for the incoming data ports
    let display_temp = Temp { degrees: 78 };

    let harness_result = 
       cb_apis::testComputeCB(display_temp);

    assert!(matches!(harness_result, cb_apis::HarnessResult::Passed));
  }
```

Run the test and see that it fails.

When we have a test that we expect to fail like this (i.e., test failure indicates that things are working properly), it's best to turn it into a succeeding test so that when we run tests, a result of "all tests passing" communicates the idea that "everything is OK".  There are multiple ways to turn the test above into a succeeding test.  Probably the best way in this case to require that harness result matches `RejectedPrecondition`.

Modify the test as follows (changing the expected harness result), and config

```rust
  #[test]
  #[serial]
  fn test_compute_GUMBOX_manual_failing_REQ_OP_5() {
    // generate values for the incoming data ports
    let display_temp = Temp { degrees: 78 };

    let harness_result = 
       cb_apis::testComputeCB(display_temp);

    assert!(matches!(harness_result, cb_apis::HarnessResult::RejectedPrecondition)); 
  }
```

## Activity 4 - Commit / Push

Although we have done a lot of experimenting with the code, we haven't actually haven't made a lot of changes that are still reflected in the code at this point (only the seeded (commented out) bug in the Thermostat application code and the new tests for Operator Interface).

* **Task**: Commit / Push your changes with a message like "Simple Isolette DT Add GUMBO - GUMBOX updates completed".

