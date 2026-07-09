# BR-01: Bugs in HAMR-SysMLv2-Rust-seL4-P-DP-Example (original Simple Isolette)

- **Date found:** 2026-07-08
- **Repository / component:** hamr-tutorials, `HAMR-SysMLv2-Rust-seL4-P-DP-Example`
- **Environment:** Sireum v4.20260630.386eac9f, Verus 0.2026.01.23.1650a05, rustc 1.91.0, macOS (Darwin 24.6.0)

Four issues. Issue 1 is a genuine functional bug in the component application code.
Issues 2-4 are Verus verification failures in the developer-authored `*_app.rs` files:
only `thermostat_thermostat` passes `make verus` today; the other three crates fail,
so the top-level `make verus` fails. All issues are in developer-editable files, not
in HAMR-generated infrastructure -- i.e., these are example-maintenance issues rather
than codegen bugs.

---

## Issue 1 -- OperatorInterface simulation clobbers the upper set point; emits ill-formed set points after 24 dispatches (functional bug)

**File:** `hamr/microkit/crates/operator_interface_operator_interface/src/component/operator_interface_operator_interface_app.rs`, line 126

```rust
119        //   ..update upper_desired_temp simulation according to trajectory
120        self.upper_desired_temp = self.upper_desired_temp + self.upper_desired_temp_trajectory;
121
122        //   ..update the simulation trajectory when temp reaches bounds
123        if self.upper_desired_temp >= 102 {
124          self.upper_desired_temp_trajectory = -1
125        } else if self.upper_desired_temp <= 99 {
126          self.upper_desired_temp = 1          //  <-- BUG
127        };
```

**Root cause:** line 126 assigns `1` to the *set point* instead of the *trajectory*.
It should read `self.upper_desired_temp_trajectory = 1` (compare the correct
lower-bound handling at line 116).

**Concrete trace** (set points update every 6th dispatch; `upper` starts at 101,
trajectory +1):

| Update # (dispatch) | upper after update | note |
|---|---|---|
| 1 (disp. 6)  | 102 | trajectory reverses to -1 (correct) |
| 2 (disp. 12) | 101 | |
| 3 (disp. 18) | 100 | |
| 4 (disp. 24) | 99 -> **clobbered to 1** | bug fires: `upper <= 99` branch |
| 5+ | stuck at 1 | each update computes 1-1=0, then resets to 1 |

**Consequences:**

- From dispatch 24 on, the emitted `desired_temp` is `{lower: 97..99, upper: 1}` --
  violating **REQ_OP_1** (lower <= upper) and **REQ_OP_3** (99 <= upper <= 102) in
  `requirements/requirements.md`.
- It also violates the Thermostat's GUMBO compute assume **`ASSM_LDT_LE_UDT`**
  (`desired_temp.lower.degrees <= desired_temp.upper.degrees`) at runtime -- the
  thermostat is dispatched outside its verified contract every cycle thereafter.
- Functionally: since `current_temp` in [96, 103] is always greater than `upper = 1`,
  the REQ_THERM_3 branch fires forever -- **the heater is permanently commanded Off
  from ~24 seconds after boot**, regardless of temperature.

**Why the existing test misses it:** `test_compute_REQ_OP_INTERFACE_repeated`
(in `src/test/tests.rs`) runs 20 iterations = only 3 set-point updates (iterations
6, 12, 18). The bug first fires on the 4th update, at iteration 24 -- four iterations
past the test horizon.

**Reproduction:** change the test loop from `0..20` to `1..=30` (with a diagnostic
message). Actual output:

```
thread '...test_compute_REQ_OP_INTERFACE_repeated' panicked at src/test/tests.rs:66:8:
iteration 24: set points [98, 1] violate REQ_OP_1/2/3
```

**Suggested fix:** line 126 -> `self.upper_desired_temp_trajectory = 1`. Recommend
also extending the test loop to >= 30 iterations (covering at least one full
oscillation of both set points) as the regression test.

---

## Issue 2 -- `temp_sensor_temp_sensor` fails `make verus` (2 verification errors)

**File:** `hamr/microkit/crates/temp_sensor_temp_sensor/src/component/temp_sensor_temp_sensor_app.rs`

```
error: possible arithmetic underflow/overflow
   --> src/component/temp_sensor_temp_sensor_app.rs:116:26
    |
116 |       self.latest_temp = self.latest_temp + self.temp_trajectory;

error: precondition not satisfied
   --> src/component/temp_sensor_temp_sensor_app.rs:128:7
    |
128 |         api.put_current_temp(Temp { degrees: self.latest_temp });
    |
   ::: src/bridge/temp_sensor_temp_sensor_api.rs:38:9
    |
 38 | /         (96i32 <= value.degrees) &&
 39 | |           (value.degrees <= 103i32),
    | |___________________________________- failed precondition

verification results:: 10 verified, 1 errors
```

**Root cause:** the `temp_range` integration guarantee in `Devices.sysml` is woven
as a `requires (96 <= degrees <= 103)` on `put_current_temp`, but the simulation
state fields (`latest_temp`, `temp_trajectory`) are unconstrained `i32` from Verus's
perspective -- the simulation maintains the range at runtime, but nothing
communicates that invariant to the verifier, and the unguarded `+/-1` update also
produces an overflow VC.

**Suggested fix:** write the simulation update with explicit range guards and clamp
the value before the put (guard the increment with `latest_temp < 103` / decrement
with `latest_temp > 96`, and clamp the reported value into [96, 103]). This is
behavior-preserving (the sim never actually leaves the range) and makes both VCs
discharge. A working pattern is in the EDP variant's
`HAMR-SysMLv2-Rust-seL4-P-EDP-Example/.../temp_sensor_temp_sensor_app.rs`
(`clamp_to_sensed_range` helper).

---

## Issue 3 -- `operator_interface_operator_interface` fails `make verus` (2 verification errors)

**File:** `hamr/microkit/crates/operator_interface_operator_interface/src/component/operator_interface_operator_interface_app.rs`

```
error: possible arithmetic underflow/overflow
   --> src/component/operator_interface_operator_interface_app.rs:110:35
    |
110 |         self.lower_desired_temp = self.lower_desired_temp + self.lower_desired_temp_trajectory;

error: possible arithmetic underflow/overflow
   --> src/component/operator_interface_operator_interface_app.rs:120:35
    |
120 |         self.upper_desired_temp = self.upper_desired_temp + self.upper_desired_temp_trajectory;

verification results:: 6 verified, 1 errors
```

**Root cause:** same class as Issue 2 -- unguarded `i32 + i32` on unconstrained
state fields. (Note: fixing Issue 1 alone does not fix this; the overflow VCs are
about the verifier's view of the fields, not the runtime trajectory values.)

**Suggested fix:** clamp the previous value into the simulation range before the
`+/-1` step (see the EDP variant's `clamp` helper), or add explicit range guards
around the additions.

---

## Issue 4 -- `heat_source_heat_source` fails to compile under Verus

**File:** `hamr/microkit/crates/heat_source_heat_source/src/component/heat_source_heat_source_app.rs`

```
error: The verifier does not yet support the following Rust feature: &mut types, except in special cases
  --> src/component/heat_source_heat_source_app.rs:89:35
   |
89 |       log_heat_source_simulation(&self);
   |                                   ^^^^
```

**Root cause:** two problems compound at line 89 / line 123:

```rust
89       log_heat_source_simulation(&self);      // &self where self: &mut Self -> &&mut Self
...
123  pub fn log_heat_source_simulation(state: &heat_source_heat_source)   // NOT #[verifier::external_body]
124  {
125    log::info!("Heater State: {:?}", state.heater_state);
126  }
```

Unlike every other logging helper in the example, `log_heat_source_simulation` is
**missing the `#[verifier::external_body]` annotation** (see lines 111 and 117 for
the correctly annotated siblings), so Verus tries to verify a call passing a
`&&mut` reference and rejects it. This contradicts the example's own documented
convention (component-implementation-guide: "Logging functions must be marked
`#[verifier::external_body]`").

**Suggested fix:** annotate the function with `#[verifier::external_body]` and pass
the enum value rather than the component reference -- e.g.
`fn log_heat_source_simulation(state: On_Off)` called as
`log_heat_source_simulation(self.heater_state)` (this is what the EDP variant does;
`On_Off` is `Copy`).
