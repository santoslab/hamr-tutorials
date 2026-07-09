# BR-02: Microkit/Rust codegen halts with "Need to handle event ports" when a component has both integration constraints and a pure EventPort

- **Date found:** 2026-07-08
- **Component:** HAMR Microkit codegen, GUMBO Rust plugin (Sireum repo)
- **File:** `hamr/codegen/shared/src/main/scala/org/sireum/hamr/codegen/microkit/plugins/gumbo/GumboRustPlugin.scala`, line 593 (`handleIntegrationConstraints`)
- **Environment:** Sireum v4.20260630.386eac9f, macOS (Darwin 24.6.0)
- **Severity:** blocks code generation for any model that combines pure (payload-less) event ports with GUMBO integration constraints in the same thread component

## Summary

`sireum hamr sysml codegen --platform Microkit` crashes with
`java.lang.Error: Need to handle event ports` whenever a thread component has
**both** (a) a GUMBO `integration` section and (b) a pure `EventPort` -- **even
when no integration constraint mentions the event port** (and none could: a pure
event port has no payload to constrain).

Pure event ports otherwise work end-to-end in the Microkit/Rust backend (see
"What already works" below), so this halt is the only blocker for using them
alongside contracted data ports.

## Root cause (source analysis)

In `handleIntegrationConstraints`, the loop iterates over **all** ports of the
thread and destructures the port kind **before** checking whether the port
actually carries an integration constraint:

```scala
for (p <- thread.getPorts()) {
  val (aadlType, isEvent, isData): (AadlType, B, B) = p match {
    case i: AadlEventDataPort => (i.aadlType, T, T)
    case i: AadlDataPort => (i.aadlType, F, T)
    case i: AadlEventPort => halt("Need to handle event ports")   // <-- line 593
    case x => halt("Unexpected port type: $x")
  }

  subclauseInfo.gclSymbolTable.integrationMap.get(p) match {      // <-- lookup happens too late
    case Some(spec) => ...
```

Because the `halt` fires inside the port-kind match, any `AadlEventPort` owned by
the component kills the run, regardless of the `integrationMap` contents. The
method is only invoked for components with integration constraints, which is why
components with pure event ports and *compute-only* GUMBO (or no GUMBO) generate
fine.

## Minimal reproducer

Two periodic Rust threads connected by a DataPort and a pure EventPort. The
**only** GUMBO in the model is an integration guarantee on the sender's **data**
port:

```sysml
//@ HAMR: --platform Microkit --output-dir ../../hamr

package Repro {
  private import HAMR::*;

  part def Repro_System :> System {
    part snd: Snd_Process;
    part rcv: Rcv_Process;
    part proc: Main_Processor;
    connection c1 : PortConnection connect snd.out_val to rcv.in_val;
    connection c2 : PortConnection connect snd.evt to rcv.evt;
    allocation a1: Deployment_Properties::Actual_Processor_Binding allocate snd to proc;
    allocation a2: Deployment_Properties::Actual_Processor_Binding allocate rcv to proc;
  }

  part def Snd_Process :> Process {
    port out_val : DataPort { out :>> type : Base_Types::Integer_32; }
    out port evt : EventPort;
    attribute :>> Domain = Domain_Snd;
    part snd: Snd;
    connection c1 : PortConnection connect snd.out_val to out_val;
    connection c2 : PortConnection connect snd.evt to evt;
  }

  part def Snd :> Thread {
    port out_val : DataPort { out :>> type : Base_Types::Integer_32; }
    out port evt : EventPort;    // pure event port, mentioned by NO constraint
    attribute :>> Period = 1000 [ms];
    attribute :>> Dispatch_Protocol = Supported_Dispatch_Protocols::Periodic;
    attribute :>> Implementation_Language = Implementation_Languages::Rust;

    language "GUMBO" /*{
      integration
        guarantee out_val_nonneg "constraint on the DATA port only":
          0 [i32] <= out_val;
    }*/
  }

  part def Rcv_Process :> Process {
    port in_val : DataPort { in :>> type : Base_Types::Integer_32; }
    in port evt : EventPort;
    attribute :>> Domain = Domain_Rcv;
    part rcv: Rcv;
    connection c1 : PortConnection connect in_val to rcv.in_val;
    connection c2 : PortConnection connect evt to rcv.evt;
  }

  part def Rcv :> Thread {
    port in_val : DataPort { in :>> type : Base_Types::Integer_32; }
    in port evt : EventPort;
    attribute :>> Period = 1000 [ms];
    attribute :>> Dispatch_Protocol = Supported_Dispatch_Protocols::Periodic;
    attribute :>> Implementation_Language = Implementation_Languages::Rust;
  }

  part def Main_Processor :> Processor {
    attribute :>> Frame_Period = 2000 [ms];
    attribute :>> Clock_Period = 10 [ms];
  }

  attribute Domain_Snd: CASE_Scheduling::Domain = 2;
  attribute Domain_Rcv: CASE_Scheduling::Domain = 3;
}
```

**Steps** (project layout: `sysmlv2/aadl-lib` copied from any HAMR example,
`sysmlv2/repro/Repro.sysml` as above; run from the model directory):

```bash
sireum hamr sysml tipe --sourcepath ../aadl-lib:. Repro.sysml
# => Well-formed!

sireum hamr sysml codegen --sourcepath ../aadl-lib:. --platform Microkit \
  --output-dir ../../hamr --workspace-root-dir ../.. --no-proyek-ive Repro.sysml
```

**Actual result:**

```
Exception in thread "main" java.lang.Error: Need to handle event ports
	at org.sireum.helper$.halt(helper.scala:42)
	...
	at org.sireum.hamr.codegen.microkit.plugins.gumbo.GumboRustPlugin.$anonfun$handleIntegrationConstraints$1(GumboRustPlugin.scala:593)
	at org.sireum.hamr.codegen.microkit.plugins.gumbo.GumboRustPlugin.$anonfun$handleIntegrationConstraints$1$adapted(GumboRustPlugin.scala:589)
	at org.sireum.IS.foreach(IS.scala:274)
	at org.sireum.hamr.codegen.microkit.plugins.gumbo.GumboRustPlugin.handleIntegrationConstraints(GumboRustPlugin.scala:589)
	at org.sireum.hamr.codegen.microkit.plugins.gumbo.GumboRustPlugin.$anonfun$handle$6(GumboRustPlugin.scala:355)
	at org.sireum.hamr.codegen.microkit.plugins.gumbo.GumboRustPlugin.handle(GumboRustPlugin.scala:177)
	at org.sireum.hamr.codegen.microkit.plugins.gumbo.DefaultGumboRustPlugin.handle(GumboRustPlugin.scala:897)
```

**Control case:** commenting out the `language "GUMBO"` block on `Snd` (the only
change) makes codegen succeed and emit all three crates (`data`, `snd_snd`,
`rcv_rcv`). The crash triggers on either side -- it also reproduces with an
integration **assume** on a receiver that owns an **in** event port (observed on a
4-component Isolette variant where both the sensor with a `guarantee` and the
controller with `assume`s owned event ports).

## What already works (why this is worth fixing rather than documenting away)

In a component **without** integration constraints, pure event ports are fully
functional in the Microkit/Rust backend -- verified end-to-end on this same
version:

- Generated API: ghost field `evt: Option<u8>`; receiver `get_evt() -> bool` with
  `ensures res == self.evt.is_some()`; sender no-arg `put_evt()` with
  `ensures self.evt == Some(0u8)`.
- GUMBO compute contracts referencing the port via `HasEvent(evt)` weave correctly
  into Verus ensures (`api.evt.is_some() ==> ...`) and GUMBOX oracles.
- `make verus` passes and GUMBOX/PropTest harnesses
  (`test_apis::put_evt(Option<u8>)`, `testComputeCB`, macros) drive the port
  correctly.

So the halt at line 593 is the single blocker preventing the natural combination
"contracted data ports + pure event trigger port" -- which forced the tutorial
example `HAMR-SysMLv2-Rust-seL4-P-EDP-Example` to model its `temp_changed` trigger
as an `EventDataPort` carrying a payload instead of the intended pure `EventPort`.

## Suggested fix

Skip pure event ports in the integration-constraint loop -- they carry no payload,
so no integration constraint can apply to them:

```scala
for (p <- thread.getPorts() if !p.isInstanceOf[AadlEventPort]) {
  ...
}
```

or equivalently, move the `integrationMap.get(p)` lookup ahead of the port-kind
destructuring and only `halt` if an `AadlEventPort` unexpectedly appears **in**
the integration map (which would indicate a front-end bug rather than a valid
model).

## Secondary issue (same feature area, separate minor bug)

For a component with a pure `EventPort` input, the generated **`src/test/tests.rs`
skeleton references generator functions that codegen does not emit**: the PropTest
macro stanzas use `generators::u8_strategy_default()` (for the event-port
strategy) and `generators::i32_strategy_default()` (for an `Integer_32` GUMBO
state variable), but the generated `src/test/util/generators.rs` contains only
`option_strategy_default`/`option_strategy_bias`. Result: the crate fails
`cargo test` out of the box with `E0425` (cannot find function), including rustc's
unhelpful suggestion to use `option_strategy_default` in place of
`u8_strategy_default`. Workaround: edit `tests.rs` (it's a developer-editable
file) to use proptest's `any::<u8>()` / `any::<i32>()`. Fix: either emit
primitive-type `_strategy_default()` helpers into `generators.rs`, or template the
skeleton with `any::<T>()` for primitive-typed inputs.
