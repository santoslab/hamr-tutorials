# HAMR Exercise: Adding a Display Temperature (Part 2 - Rust code) 

**Purpose**:  The purpose of this exercise is to get you familiar with basic aspects of HAMR-generated code artifacts, in particular HAMR-generated APIs for port communication as well as basic manual unit testing of HAMR components.

Starting from an updated model (with ports and connections for `display_temp`) produced in Part 1 of this exercise, you will go through the steps of 
 - configurating the HAMR code generator in CodeIVE
 - running the HAMR code generator in CodeIVE
 - opening the Rust implementation of the HAMR generated thread component
 - navigating the different files at the code level
 - making changes to the code to support the display temp function
 - compiling your component code
 - adding some simple tests for your component code

## Prerequisites and Resources

Before working through this exercise, you should have gone through the following HAMR lectures (or read the equivalent documentation):

* ...all prerequisites for Part 1 of this exercise
* HAMR Rust project file organization
* HAMR Rust components - component development


## Excercise Overview

This exercise continues with the "Simple Isolette" system concept used in a number of other HAMR tutorials and examples.  At the completion of Part 1, we had refactored our initial architecture to add the `display_temp` feature, as illustrated in the diagram below.

![Simple Isolette (before refactoring)](images/simple-isolette-after.png)

In this exercise, you will...
- Update the Thermostat thread implementation to send the received `current_temp` value out the `display_temp` port (and add appropriate tests)
- Update the Operator Interface to get the temperature value to display from the `display_temp` port and to update a simulated user interface with this new information (and add appropriate tests).

## Activity 1 - Running HAMR Code Generation using a Previous Configuration

* **Task**: Use the CodeIVE command "HAMR SysML CodeGen" from the command palette to run code generation, and select `Microkit` for the target platform.  

As long as you have not altered the initial starting files beyond adding the ports and the connections, the code generation action will use the code generation configuration embedded in the comment at the top of the file.
```
//@ HAMR: --platform Microkit --output-dir ../../hamr
```
Based on the configuration above, the HAMR-generated project will be in `hamr` folder (as configured by the `output-dir` option above, and then in the `microkit` folder (the default name chosen for code generation when targetting the seL4 microkit platform).

As the code generation runs, it will produce status information along with information about generated files in the CodeIVE Terminal pane.  Recall that successful code generation is indicated by the message...
```
info: Code generation successful!
```
..appearing at the end of code generation.

## Activity 2 - Commit the Results of Code Generation to Git and Observe the Results 

* **Task:** Commit and push to git the changes to your project that have been made from running HAMR code generation.

Take some time to browse the changes.  Note that from 7 lines of model changes, a significant amount of code has been generated or updated to reflect the addition of the `display_temp` ports and pathways withing the seL4 microkernel.  You want be able to understand all of the changes at this point.

## Activity 3 - Updating the Functionality of the Thermostat

In this exercise, we will be triggering the build of the component application code by running tests (and we will add some new tests along the way).   

* **Task:** In the CodeIVE for the Thermostat crate (we will refer to this as the "code project"), navigate to the `src/test/test.rs` file, and scroll down until you see the `test_compute_REQ_THERM_2` test (or use the outline view to navigate to the test).  Read the documentation for the test and see if you can understand its purpose.  Click on the "Run Test" annotation to run the test.  This will trigger a build of the crate and the test will be executed (it should pass).

Now, open the file `src/component/thermostat_thermostat_app.rs` and we will update the application code.

Recall that HAMR component application code is organized into "entrypoints":
* `initialize` method - called during system initialization 
* `timetriggered` method - the name of the Compute Entry Point for periodic methods.  Called at the beginning of the component's period.

We will need to update both of these methods.

* **Task:** Update the body of the `initialize` entry point.  Every HAMR component should initialize its output data ports.  So we need to put some type of appropriate initial value on the `temp_display` port.  One issue here is that we don't know yet what the initial current temperature is -- recall that HAMR/AADL Initialize entry points are to allowed to read from input ports; they only initialize local state and set initial values for output ports.   Eventually, when we complete the formal contracts for the component, we will need all values (even initialization values) to be in the appropriate range, so we should try to pick an initial value that lies with the expected temperature range.

One final note, with respect to safety, you would be right in thinking that we really don't want to pick an arbitray value that could lead to the heater being turned on if is the case that the air temperature in the Isolette is already warm (more generally, we would like for the thermostat's understanding of the environment to match the actual environment state).   Real systems (and the full Isolette system -- not the simple version that we are dealing with here) typically have an initialization mode in which the system runs through several cycles until actual sensor values can flow through the system completely and the control aspects of the system can stabilize and reflect the true state of the environment.  We will set aside such concerns for now and chose an arbitrary value to use for initialization that is in the expected temperature range.

Let's decide to set the initial display temperature value to `98` degrees.  It's best if we introduce a constant for the initial value.  If you look in the `temp_sensor_temp_sensor` crate in the application code file `src/component/temp_sensor_temp_sensor_app.rs` you can see some constants declared...

```rust
verus! {

//-------------------------------------------
//  Constants
//-------------------------------------------

// --- temp bounds ---
//  ..Upper and lowers bounds for sensed temperature
//  We will see later that this can also be accomplished using model-level GUMBO constants
pub const sensed_temp_lower_bound: i32 = 96;
pub const sensed_temp_upper_bound: i32 = 103;
```

In our Thermostat application code, in a similar position right after the opening of the Verus macro block, add a public constant for for initial display temperature as follows, with some comments, as follows.

```rust
verus! {

//-------------------------------------------
//  Constants
//-------------------------------------------

// --- initial temperature value for display_temp ---
pub const initial_display_temp_degrees: i32 = 98;
```

Now, we will update the body of the `initialize` method.  The existing code for the method is..

```rust
{
  log_info("initialize entrypoint invoked");

  self.lastCmd = On_Off::Off;
  // REQ_THERM_1: The Heat Control shall be initially Off
  let currentCmd = On_Off::Off;
  api.put_heat_control(currentCmd) 
}
```

The last line of code initializes the `heat_control` output port.  We need a similar initialization for the new `display_temp` port.  In doing this, we also need to take the `i32` value representing the temperature degrees and wrap that in a `Temp` struct (automatically generated by HAMR from a corresponding definition in HAMR SysMLv2 `Isolette_Data_Model` package).  Add the following line of code at the end of the method.

```rust
{
  log_info("initialize entrypoint invoked");

  self.lastCmd = On_Off::Off;
  // REQ_THERM_1: The Heat Control shall be initially Off
  let currentCmd = On_Off::Off;
  api.put_heat_control(currentCmd) 

  // Add initialization of display temp
  api.put_display_temp(Temp { degrees: initial_display_temp_degrees });
}
```

* **Task:**  In `src/test/tests.rs`, add a new test for the `initialize` entry point to verify that, after running the `initialize` entry point, the value on the `display_temp` port is `98`.  We haven't yet covered the details of HAMR's testing framework in our lectures, so to make it easy, the fully written test is given to you below.   Add this test to your code right after the existing `test_initialization_REQ_THERM_1` method.

```rust
  #[test]
  #[serial]
  fn test_initialization_display_temp() {
    // invoke initialize entry point
    crate::thermostat_thermostat_initialize();

    // use auto-generated test APIs to retrieve values of 
    // output port
    let display_temp: Temp = test_apis::get_display_temp(); 

    // ..compare outputs to expected results..
    assert!(display_temp.degrees == 98);
  }
```

Once the above test is in the code, a VSCode "Run test" annotation button should appear right above the method declaration.  Press this button to run the test (which will also trigger the compilation of your code).  If you have done everything correctly, the test should pass.

BTW, the VSCode extension for Rust is a bit quirky with `#[test]` and `#[serial]` annotations: sometimes the editor will flag them with "red squiggles" for no good reason.  Generally, you can ignore these extraneous error reportings. These will usually go away when saving the file or re-running the tests.  


* **Task:**  Back in `src/component/thermostat_thermostat_app.rs`, update the Compute entry point (`timetriggered` method) to send the current temperature out the `temp_display` port.

Here is the body of the existing method (with a few annotations added)...
```rust
{
  log_info("compute entrypoint invoked");

  // -------------- Get values of input ports ------------------
  let desired_temp: Set_Points = api.get_desired_temp(); 
  let currentTemp: Temp = api.get_current_temp();  // current temperature read

  //================ compute / control logic ===========================

  // current command defaults to value of last command (REQ-THERM-4)
  let mut currentCmd: On_Off = self.lastCmd;

  if (currentTemp.degrees > desired_temp.upper.degrees) {
     // REQ-THERM-3
     currentCmd = On_Off::Off;
  } else if (currentTemp.degrees < desired_temp.lower.degrees) {
     assert(api.current_temp.degrees < api.desired_temp.lower.degrees);
     // REQ-THERM-2
     //currentCmd = On_Off::Off; // seeded bug/error
     currentCmd = On_Off::Onn;
  }

  // -------------- Set values of output ports ------------------
  api.put_heat_control(currentCmd);
  self.lastCmd = currentCmd      
}
```
Notice that we read the current temperature from the associated input port towards the top of the method and store it in the local variable `currentTemp`.  All we need to do is send that value out the `display_temp` output port.  We can place the call to the associated `put_display_temp` API anywhere following the definition of `currentTemp` because behind the scenes, HAMR waits until after the entry point completes before propagating the output port values to any connected components.   However, for readability (to emphasize the read/compute/write pattern), we typically place the `put_` methods to set values on output ports at the end of the entry point method.

Add, the call to `put_display_temp` with the other `put_` call as shown below...

```rust
  // -------------- Set values of output ports ------------------
  api.put_heat_control(currentCmd);
  api.put_display_temp(currentTemp);  // add call to output display_temp
  self.lastCmd = currentCmd      
}
```

* **Task:**  In `src/test/tests.rs`, add a new test for the `compute` entry point to verify that, after running the application code in the `timetriggered` method, the value on the `display_temp` matches the value that came into the component on the `current_temp` port.  Technically, we should try to demonstrate that "for all possible current temperature input values ct, the value of the `display_temp` port is equal to ct".  It's infeasible to do this with testing (because the number of possible values is too great), although we can actually prove it with formal verification using Verus later on.  For now, we will just pick a single representative value and test using that.  Later, we will see that with HAMR's randomizing property-based testing, we can automatically generate an arbitrary number of values to test this.

Right below where you created the test for the `initialize` entry point, you can see the outline of a HAMR manual unit test for the Thermostat component (illustrated with hints about what you might do to test the heat command values).  Below is that test code outline. 

```rust
  // auto-generated example test for `compute` entry point
  #[test]
  #[serial]
  fn test_compute() {
    // initialize entry point should always be called to initialize 
    // output ports and local state
    crate::thermostat_thermostat_initialize();

    // implement input of "test vector"
    // ..set values for input data ports
    test_apis::put_current_temp(Isolette_Data_Model::Temp::default());
    test_apis::put_desired_temp(Isolette_Data_Model::Set_Points::default());
    // ..[optional] set values for GUMBO-declared local state variables
    test_apis::put_lastCmd(Isolette_Data_Model::On_Off::default());

    // invoke initialize entry point
    crate::thermostat_thermostat_timeTriggered();

    // use auto-generated test APIs to retrieve values of 
    // output ports and GUMBO-defined local state as illustrated below

    // let heat_control: On_Off = test_apis::get_heat_control();
    // let post_lastCmd : On_Off = test_apis::get_lastCmd();

    // ..compare outputs to expected results..
    // assert!(..)   
  }
```
A couple of notes about this code before moving on:
- we always start a `compute` test with calling the `initialize` entry point to make sure that output ports and local state are initialized.
- the 'get'/'put' modalities for input/output ports are reversed from what we see in the application code.  For instance, for output port like `display_temp`, in the application code we `put_` the value on the output port using `put_display_temp`, but from the testing perspective after the compute entry point has executed we `get_` the output value using `get_display_temp`.

Now on to designing the test...  What we need to do is decide what "test vector" (set of input values) we need to test our desired property.  Note that we are only going to test that the output display temp value matches the input current temp value, so we really don't care about the `desired_temp` port input and value of the `lastCmd` state variable.  However, we need to at least input some value for `desired_temp` because HAMR data ports must always have a value on them (a run-time error results if they don't).

If you look at the first test written for `REQ_THERM_2`, you see that we typically document in a readable way the input test vector and the expect output relevant to the property.
```
     /*
       Inputs:
         current_temp:  95f 
         lower_desired_temp: 98f
         upper_desired_temp: *irrelevant to requirement* (use 100f)

       Expected Outputs:
         heat_control: On
         last_cmd (post): On
    */
```

For testing the display temp, we can re-orient this to 

```
     /*
       Inputs:
         current_temp:  95f 
         lower_desired_temp: *irrelevant to requirement* (use 98f)
         upper_desired_temp: *irrelevant to requirement* (use 100f)

       Expected Outputs:
         display_temp: 95f
    */
```

Thus, you can insert the documentation above and the following test code below which implements the desired test right below your previously written test for the `initialize` entry point.

```rust
  #[test]
  #[serial]
  fn test_compute_display_temp() {
    // [InvokeEntryPoint]: invoke the entry point test method
    crate::thermostat_thermostat_initialize();

    // generate values for the incoming data ports
    let current_temp = Temp { degrees: 95 };
    let lower_desired_temp = Temp { degrees: 98 };
    let upper_desired_temp = Temp { degrees: 100 };
    let desired_temp = Set_Points {lower: lower_desired_temp, upper: upper_desired_temp};
 
    // [PutInPorts]: put values on the input ports
    test_apis::put_current_temp(current_temp);
    test_apis::put_desired_temp(desired_temp);

    // [InvokeEntryPoint]: Invoke the entry point
    crate::thermostat_thermostat_timeTriggered();

    // get relevant values from output ports
    let api_display_temp = test_apis::get_display_temp();

    // assert the desired property: the output display temp value matches the input current temp
    assert!(api_display_temp == current_temp);
  }
```

Run the test to confirm that your implementation is correct.  If the test passes, you are all done with implementing the new display temperature functionality in the Thermostat component!


## Activity 4 - Updating the Functionality of the Operator Interface

In preparation for this activity, in the CodeIVE use the `File / Open Folder` option to open the `operator_interface_operator_interface` crate.  This will replace your Thermostat crate files with the Operator Interface files.  If you want to keep your Thermostat files open for some reason, use the `File / New Window` option to create a new empty VSCodium window before opening the Operator Interface files.

Our objective in this activity is to add some code (with associated tests) that gets the temperature value to display from the component input port and posts it to a simulated display.  In a small embedded device, we might encode the temperature value and some way and use a GPIO mechanism to write it out to some type of LCD.  In a full seL4-based system, we might send the value to a virtual machine in another protection domain that is hosting a full operating system with connections to a conventional monitor display.

We will simulate both the state update and output by...
 - storing the most recent received temperature degree value in a local variable, 
 - writing a log message with the degree value formatted as a string.

Overall, the needed implementation is simple, and almost half of our work is associated with just declaring and initializing the local variable to hold the degree value.

Right after the Verus block opening, you can see the struct used to represent the state of the component.  The component has no model-level declared GUMBO state variables (HAMR inserts a marker which will be explained below).  The developer has manually added some fields to support a simulation that generates set points.

```rust
  //-------------------------------------------
  //  Application State (as a struct)
  //
  //  The application state includes a non-GUMBO state variable 
  //  that supports the "simulation" of the operator interface.
  //
  //  There is no GUMBO declared application state for this component.
  //-------------------------------------------
  pub struct operator_interface_operator_interface {
    // PLACEHOLDER MARKER STATE VARS

    // The variables below are used for operator input (set points) simulation.
    // This also illustrates non-GUMBO declared state variable for a component
    pub lower_desired_temp: i32, 
    pub upper_desired_temp: i32,
    pub lower_desired_temp_trajectory: i32,
    pub upper_desired_temp_trajectory: i32,
    pub activations_until_update: i32,
  }
```
Specifically, five fields such as `lower_desired_temp`, etc. are declared and used to evolve simulated set point values.

The struct begins the following comment.
```rust
    // PLACEHOLDER MARKER STATE VARS
```
Since there are no model-level GUMBO state variables declared, none are generated into the component code.  However, HAMR does insert a special marker to indicate the spot where it will automatically weave declarations for GUMBO declared state variables if they are added in the model in the future.  *DO NOT DELETE SUCH MARKERS*.   If they are removed, HAMR won't know how to automatically update your code, and you will need to manually merge generated declarations and contracts auto-derived from GUMBO specifications.

**Task**: At the bottom of the struct above, add a struct field to hold the latest display temp value as shown below.

```rust
 // Simulate temperature display on operator interface screen
 pub temp_on_display: i32,
```

The Rust Analyzer may have generated an error 
```
error[E0063]: missing field `temp_on_display` in initializer of `operator_interface_operator_interface_app::operator_interface_operator_interface`
```
This indicates that we need to initialize the field in the constructor for the struct.  Add an initialization value in the constructor as shown below...

```rust
    pub fn new() -> Self
    {
      Self {
        // PLACEHOLDER MARKER STATE VAR INIT

        // initialization of non-GUMBO declared state variable
        // ...simulated operator input
        lower_desired_temp: 98, 
        upper_desired_temp: 101,
        lower_desired_temp_trajectory: -1, // value with either be +1 or -1
        upper_desired_temp_trajectory: 1, // value with either be +1 or -1
        activations_until_send: 5, // activations of compute entry point until send

        // ...simulated temperature display output
        temp_on_display: 98,
      }
    }
```
If you wish, you can declare a constant to name the `98` similar to what we did in the Thermostat component.


**Task**:  Initialize the state (again) in the `initialize` entry point method.   

You may wonder why we need to initialize the field again.  The reason is that a design decision was made within HAMR to following AADL standard and have a distinct `initialize` entry point code that is activated by the HAMR underlying scheduling framework.  In the `initialize` entry point we need to follow the abstract semantics of AADL and carry out initialization of local state and output data ports (with optional sends on event and event data ports).  The GUMBO contracts for the `initialize` are also aligned with this intent and expose outcomes of the `initialize` step at the model level for system reasoning.   The execution of the Rust constructor code in `new()` is not under the direct control of the HAMR scheduling framework.  Moreover, it is difficult to link the implicit propagation of output port values (implemented by HAMR behind the scenes) to the `new()`.  Therefore, the HAMR Rust component developer should just accept the fact that there will be the following two notions of initialization...
 - a Rust language-level initialization of state
 - a HAMR-managed `initialize` method whose execution is linked to the HAMR scheduling approach and whose behavior is supported by HAMR testing and contract verification.

Unfortunately, there will likely give rise to some redundant initialization code.  Conceptually, the HAMR `initialize` actions will likely "overwrite" any Rust language initialization.  In fact, `initialize` actions should be complete and canonical initializaton steps because they are the actions tracked by HAMR's verification framework.

In the `initialize` entry point code, after the other `self` field initializations, add the following code to initialize the `temp_on_display` field.
```rust
// initialize temperature on simulated display
self.temp_on_display = 98;
```

**Task**: Modifying the `timetriggered` method to update the simulated display.  

For our simple simulation, we want to update the stored display temperature in `temp_on_display` and then log the value.  Logging is not conceptually difficult given that we have some logger helper functions.  The primary issue here is getting things to work in terms of what the Verus verifier can handle and what functionality is available for the minimal libraries designed for code running on the "bare metal" (no underling OS) in seL4.  

Let's first discuss the Verus issue.  Even though we are not applying Verus in this exercise, we follow a typical approach in HAMR Rust components and work under the assumption that we eventually want to verify the code with Verus.  HAMR doesn't force you to do this -- for example, if you are fairly certain that you are never going to verify with Verus (you may be making a decision to work with testing only), you can remove the Verus macro block wrapping the code.  Also, if you are just desperate to get something working, you can remove the Verus macro to get any type of code that will work for you, and then clean things up for Verus later.  Note that in these cases, you may also need to modify the auto-generated HAMR `make` files to control what Verus is invoked on.  In general, if you are a novice developer with HAMR, we don't recommend doing any of these things.

In any case, the main point here is that, as with most any contract-based verification tool, Verus will try to verify the body of the method of the `timetriggered` method while relying on contracts of any method called in the body (like the logging functions).  Then, Verus will attempt to verify that the called method bodies conform to their contracts.

The problem here is that we can not cleanly specify the full semantics of the logging function in a contract because logging involves I/O to the underlying operating system, etc.  This is a general issue when trying to use contract-based verification.  So what almost all such frameworks do is to enable developers to do a "controlled exit" from the contract based verification framework by labelling some methods (e.g., library methods like the logging functions) as "external".   For these methods, the contract verification will not try to verify that the implementation of the external functions conforms to their contracts.  Instead, it just trusts that the contracts are correct (leaving the verification to testing or manual inspection).  

You can see at the bottom of the Operator Interface application code that the logging functions are marked 
```rust
#[verifier::external_body]
```
indicating that Verus should not try to verify the bodies of these functions -- we'll just trust that they are correct.

A second issue is that we have limited library functions available in general, due to the fact we writing code that is designed to run without a full OS on seL4.  Specifically, the logging functions like `log::info!` that are used in HAMR-generated logger helper functions like `pub fn log_info` are a part of seL4's minimal logging library for Rust.  If you look through all the comonent application code files for the Simple Isolette, you will see that a few of them add additional custom logger helper functions beyond what HAMR auto-generates, such as the `log_set_point_simulation` (labelled `#[verifier::external_body]`).  These enable the developer to send structured data that Verus understands to a "external" function outside of what Verus processes so that the more complicated string manipulations that Verus might have trouble with can be performed there in method bodies that Verus doesn't process.   The entire "but the body isn't verified" issue isn't so important here because the low-level logging code actions don't impact our verification objectives.

With all of that explanation behind us, at the top of the `timetriggered` method, right after the initial `log_info` message, add the following...

```rust
// -------------- Process display temp ------------------
let display_temp: Temp = api.get_display_temp(); 

// simulate output to operator screen
self.temp_on_display = display_temp.degrees;
log_temp_on_display(self.temp_on_display);
```

The code above will...
- get the `Temp` struct value off the input port
- extract the `degrees` value and assign it to the local state variable simulating the actual operator interface display
- call a dedicated logging method for the temperature display, that we will define below.

Now, towards the bottom of the file in the logging method section, e.g., right after the `log_set_point_simulation`, add the following logging function.

```rust
#[verifier::external_body]
pub fn log_temp_on_display(display_temp: i32)
{
   log::info!("Temp on Display: {}", display_temp);
}
```

This completes the updating of the Operator Interface application code.

**Task**: Fix existing Tests

At this point, if you try running the existing tests for the Operator Interface Compute Entry point (timetriggered), they will fail.  Looking at the stack trace in the terminal window indicates the root of the problem is...
```
thread 'test::tests::tests::test_compute' panicked at src/bridge/extern_c_api.rs:59:46:
Not expecting None
```
This is a bit cryptic for novice HAMR users, but its fairly common scenario when working with HAMR components.  Let's first explain things at a higher level related to HAMR concepts.   The issue is that HAMR components alway expect data ports to have values.  The above error arises because our test tried to execute the HAMR component but no value was on the `display_temp` input data port.   The tests worked before our model/code updates because there was no input port for this component.  At a lower Rust language level, HAMR's port storage uses an option type.  So an empty port is represented as "None".  Thus, "Not expecting None" means "I wasn't expecting the data port to be empty".

We want to fix the existing tests so that they pass.  The `test_initialization` already works because the `initialize` entry point, by its very nature, doesn't read input ports.

In the `test_compute` method, add the following code after the call to `crate::operator_interface_operator_interface_initialize();`.  This will ensure that the `display_temp` input data port has a value before the `timetriggered` method is called. 

```rust
 // Put arbitrary in range value for display temp in input port 
 let display_temp = Temp { degrees: 98 };

 // [PutInPorts]: put values on the input ports
 test_apis::put_display_temp(display_temp);
```
Execute the test, and confirm that the test succeeds.

In the `test_compute_REQ_OP_INTERFACE_repeated`, after the `initialize` entry point is invoked,  the time triggered method is invoked repeatedly to illustrate the evolution of the simulated set point. When we try to fix this test, it may be surprising that we don't need to put a value on `display_temp` for each invocation of `timetriggered`.  That's because data ports are not queued ports; once they have a value, that value persists in future calls until it is overwritten by a newly arriving data item.  So we only need to make one call (after initialization) to put a value on `display_temp`.

Accordingly, add the same code as above directly below the `crate::operator_interface_operator_interface_initialize();` call.  Run the test to confirm it succeeds.  In the terminal output, you can see that the display temp logging message
```
[2026-02-07T15:16:32Z INFO  operator_interface_operator_interface::component::operator_interface_operator_interface_app] Display Temp: 98
```
repeatedly appears (for each `timetriggered` invocation), indicating that, once given a value, the input data port maintains its value.

**Note**:  If we desired, we could test that the `display_temp` input value gets appropriately stored in the local state.  This takes a fair amount of addition work to do, so we will omit in this introductory exercise.  HAMR includes auto-generated methods that make it easy to access GUMBO-declared state.  However, since the `temp_on_display` field that we added is not GUMBO declared (it's just for internal simulation, not for reasoning about the correctness of the component), HAMR doesn't know about it (and therefore, it can't autogenerate a helper method).   To access `temp_on_display` we would need to use some `unsafe` Rust code.  An example of how this can be done is illustrated for the tests in the Heat Source component where we test that the appropriate heat control command is stored in the local (non-GUMBO declared) `heater_state`. 

## Activity 5 - Commit Completed Code and Tests

**Task:** Commit the completed code and tests.  

View the git logs to gain an overall understand of the code and tests that you had to update manually compared to what HAMR code generation did automatically for you.






