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

The following example projects are used to support the tutorials above.




