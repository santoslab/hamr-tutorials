# HAMR Tutorials

This repository provides tutorials and small examples for the HAMR high-assurance model-driven development environment ([HAMR web site](https://hamr.sireum.org)).

Please see the [HAMR documentation tutorial guide](https://hamr.sireum.org/hamr-doc/reading-order/) for how to use these files.

## Contents

### HAMR SysMLv2 with Rust Components Tutorials

- Tutorial: Modifying an Existing HAMR SysMLv2/Rust system (working with the Simple Isolette example)
  - Part 1: [Modifying an existing HAMR SysMLv2 model](./HAMR-SysMLv2-Rust-Tutorials-Exercises/Ex-Simple-Isolette-add-DT/01-Ex-Simple-Isolette-add-Display-Temp-model.md) (adding a Display Temperature feature to the Simple Isolette model)
  - Part 2: [Re-running Code Generation and Modifying existing HAMR Rust Components and Tests](./HAMR-SysMLv2-Rust-Tutorials-Exercises/Ex-Simple-Isolette-add-DT/02-Ex-Simple-Isolette-add-Display-Temp-code.md) (adding code and tests for the Display Temperature feature)

- Tutorial: Building a simple HAMR SysMLv2/Rust system from scratch (building the Simple Netword Guard example)
  - Part 1: [Building a HAMR SysMLv2 model from scratch](./HAMR-SysMLv2-Rust-Tutorials-Exercises/Ex-Simple-Network-Guard/Ex-Simple-Network-Guard-model.md) 
  - Part 2: [Generating Code, Implementing Rust component application logic, writing manual unit tests](./HAMR-SysMLv2-Rust-Tutorials-Exercises/Ex-Simple-Network-Guard/Ex-Simple-Network-Guard-code.md) (adding code and tests for the Display Temperature feature)

- Tutorial: Working with GUMBO Contracts in the Simple Isolette Example (writing contracts, testing using GUMBOX executable contracts, formally verifying Rust component application code to contracts using Verus)
  - Part 1: [Adding GUMBO contracts to models](./HAMR-SysMLv2-Rust-Tutorials-Exercises/Ex-Simple-Isolette-DT-add-GUMBO/01-Ex-Simple-Isolette-DT-add-GUMBO-model.md) (adding GUMBO contracts to SysMLv2 models)
  - Part 2: [Utilizing executable contracts (GUMBOX contracts) for manual unit tests](./HAMR-SysMLv2-Rust-Tutorials-Exercises/Ex-Simple-Isolette-DT-add-GUMBO/02-Ex-Simple-Isolette-DT-add-GUMBO-GUMBOX.md) (writing manual GUMBOX tests)

- Tutorial: Specifying and Verifying System-Level Properties in the Struct Split Example (writing GUMBO system specifications, generating verification conditions, verifying them with Verus, and diagnosing verification failures)
  - Part 1: [Specifying a system-level property](./HAMR-SysMLv2-Rust-Tutorials-Exercises/Ex-SysPropStructSplit-add-Prop-yLEx/01-Ex-SysPropStructSplit-add-Prop-yLEx-property.md) (stating an end-to-end property as a GUMBO spec function and `property` block, type checking, generating and reading the verification conditions)
  - Part 2: [Verifying, diagnosing, and repairing the property](./HAMR-SysMLv2-Rust-Tutorials-Exercises/Ex-SysPropStructSplit-add-Prop-yLEx/02-Ex-SysPropStructSplit-add-Prop-yLEx-verify.md) (running Verus, reading a failing verification condition back to a missing carry assertion, and contrasting the two reasons a VC can fail)

  



### HAMR SysMLv2 with Rust Components Examples

The following example projects support the tutorials above and also serve as stand-alone reference examples.
The folder naming indicates the tasking/communication style: `P` = periodic thread components, `DP` = data ports only, `EDP` = event data ports (event-style communication).
All of these examples use Rust component implementations deployable on seL4 via the Microkit framework.

- [HAMR-SysMLv2-Rust-seL4-P-DP-Example](./HAMR-SysMLv2-Rust-seL4-P-DP-Example) -- the Simple Isolette: the primary running example for the tutorials.  Four periodic components (temperature sensor, operator interface, thermostat, heat source) communicating solely via data ports.  Illustrates GUMBO contracts on the thermostat (integration constraints, initialize guarantees, compute cases over a state variable), Rust component implementations, and manual unit / GUMBOX / property-based tests.

- [HAMR-SysMLv2-Rust-seL4-P-DP-Simple-Isolette-add-DT-solution](./HAMR-SysMLv2-Rust-seL4-P-DP-Simple-Isolette-add-DT-solution) -- solution project for the "Modifying an Existing HAMR SysMLv2/Rust system" tutorial: the Simple Isolette extended with the Display Temperature feature (model changes, re-running code generation, and the associated code and tests).

- [HAMR-SysMLv2-Rust-seL4-P-DP-Simple-Isolette-DT-add-GUMBO-solution](./HAMR-SysMLv2-Rust-seL4-P-DP-Simple-Isolette-DT-add-GUMBO-solution) -- solution project for the "Working with GUMBO Contracts" tutorial: the Display Temperature variant of the Simple Isolette with GUMBO contracts added to the models and GUMBOX-based tests for the affected components.

- [HAMR-SysMLv2-Rust-seL4-P-DP-SysPropStructSplit](./HAMR-SysMLv2-Rust-seL4-P-DP-SysPropStructSplit) -- the Struct Split example for GUMBO **system-level properties**: a fork/join pipeline of seven components whose top-level system carries a GUMBO `composition` block (component/port/state aliases, an abstract schedule `schema` with independent branches, and place-assertion `property` blocks with inheritance).  HAMR generates a `sys_proof_nominal` crate whose verification conditions are discharged by Verus.

- [HAMR-SysMLv2-Rust-seL4-P-DP-SysPropStructSplit-add-Prop-yLEx-solution](./HAMR-SysMLv2-Rust-seL4-P-DP-SysPropStructSplit-add-Prop-yLEx-solution) -- solution project for the "Specifying and Verifying System-Level Properties" tutorial: the Struct Split example with an additional end-to-end system property specified, generated, and verified.

- [HAMR-SysMLv2-Rust-seL4-P-EDP-Example](./HAMR-SysMLv2-Rust-seL4-P-EDP-Example) -- an event/event-data-port variant of the Simple Isolette example (periodic components communicating via data ports, event data ports with GUMBO `HasEvent`/`MustSend`/`NoSend` contracts, latched GUMBO state variables, and a send-on-change output policy). Worked end-to-end: GUMBO component contracts, Rust component implementations, manual unit tests, manual GUMBOX tests, property-based GUMBOX tests, and Verus verification of all component crates.

- [HAMR-SysMLv2-Rust-seL4-P-EDP-Prod-Cons-Example](./HAMR-SysMLv2-Rust-seL4-P-EDP-Prod-Cons-Example) -- a minimal producer/consumer example introducing event data ports: two periodic threads with different periods connected by an event data port, GUMBO integration constraints on the message payload, and a GUMBO state variable on the consumer (illustrating `Option`-valued port APIs and state-variable-aware GUMBOX testing).

- [HAMR-SysMLv2-Rust-seL4-P-EDP-SNG-Example](./HAMR-SysMLv2-Rust-seL4-P-EDP-SNG-Example) -- the Simple Network Guard: a four-stage message pipeline (test sender, gate, filter, test receiver) over event data ports.  Illustrates GUMBO integration constraints and compute contracts using `HasEvent`/`NoSend` guarantees and `compute_cases` to specify message dropping, pass-through, and payload clamping, with the corresponding Rust implementations and test suites.




