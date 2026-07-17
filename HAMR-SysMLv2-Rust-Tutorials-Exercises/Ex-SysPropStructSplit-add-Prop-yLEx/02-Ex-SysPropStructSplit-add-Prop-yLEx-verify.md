# HAMR Exercise: Adding a System-Level Property (Part 2 - Verifying, Diagnosing, and Repairing)

**Purpose**: In Part 1 you specified the order-preservation property `Y_Stays_Strictly_Below_X` and generated its verification conditions. In this part you will...
 - run Verus on the generated VCs and watch one of them fail,
 - read the failing VC back to the missing piece of the proof outline and repair the property,
 - seed a bug that makes the property *false* and contrast the two reasons a VC can fail,
 - confirm the completed system proof.

The central skill this part teaches: **when a system-property VC fails, the failure names a schedule place, and your job is to decide whether the assertion chain is missing a fact at that place -- or whether the claim itself is wrong.**

## Prerequisites and Resources

Part 1 of this exercise, completed: your working copy `HAMR-SysMLv2-Rust-seL4-P-DP-SysPropStructSplit-add-Prop-yLEx` should contain the `yStaysBelowX` function, the first version of the `Y_Stays_Strictly_Below_X` property (inheriting `Pipeline_Functional`, binding only `before consume`), and the freshly generated `y_stays_strictly_below_x` VC module.

Verus (`cargo-verus`) must be on your `PATH`. Neither `MICROKIT_SDK` nor `MICROKIT_BOARD` is needed for any activity in this exercise.

One workflow rule to internalize now: the proof crate is *generated*. After **every** model edit in this part, the loop is

```
edit SysPropStructSplit.sysml  -->  HAMR Type Checking  -->  HAMR CodeGen  -->  make <target>
```

Running `make` without re-running codegen verifies the *stale* VCs of your previous model.

## Activity 1 - Run Verus on the New Property

The generated Makefile has a target per concrete property, so you can verify just yours without re-verifying the other four.

* **Task:** In a terminal, from `hamr/microkit/crates/sys_nominal_proof`, run:

```
make y_stays_strictly_below_x
```

You should see something like the following...

```
note: verifying module y_stays_strictly_below_x::vc_init

note: verifying module y_stays_strictly_below_x::vc_non_disabling

note: verifying module y_stays_strictly_below_x::vc_post_pre

note: verifying module y_stays_strictly_below_x::vc_sequential

error: postcondition not satisfied
   --> src/y_stays_strictly_below_x/vc_sequential.rs:161:5
    |
153 | pub proof fn vc_next_assert_task_merge_stage(pre: SystemState, post: SystemState)
    | --------------------------------------------------------------------------------- at the end of the function body
...
161 |     sys_assert_y_stays_strictly_below_x_after_merge_stage(post),
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ failed this postcondition

verification results:: 26 verified, 1 errors (partial verification with `--verify-*`)
```

The property does **not** verify. 26 of the 27 VCs discharge -- the entire chain up to and including the branch join is fine -- but the merger's Next-Assert VC fails. Your specification is well-formed, the property is (as you argued informally in Part 1) true of the system, and yet the proof does not go through. The next activity explains why.

## Activity 2 - Diagnose the Failure

A failing Next-Assert VC means: *the assertion at the out-place is not derivable from the assertion at the in-place plus the component's contract and write frame*. So open the VC and take stock of exactly what the prover was given.

* **Task:** Open `src/y_stays_strictly_below_x/vc_sequential.rs` and re-read `vc_next_assert_task_merge_stage` (VC[14]), then open `src/y_stays_strictly_below_x/assertions.rs` and expand its premise, the join assertion `sys_assert_y_stays_strictly_below_x_post_join_1`. Listing everything the prover knows about the pre-state:

1. `inRange100(x2)` -- from `Pipeline_Ranges`
2. `-101 <= y1 <= 99` -- from `Pipeline_Ranges`
3. `(gen_out.x < 100) ==> (x2 == gen_out.x + 1)` -- from `Pipeline_Functional`
4. `(gen_out.x == 100) ==> (x2 == 100)` -- from `Pipeline_Functional`
5. `y1 == gen_out.y - 1` -- from `Pipeline_Functional`

plus MergeStruct's contract (`merged.x == x2`, `merged.y == y1`) and its write frame (everything else, including `gen_out`, unchanged). From this it must prove `(gen_out.y <= gen_out.x) ==> (merged.y < merged.x)`.

* **Task:** Before reading on, try to build a *counter-model*: an assignment of values to `gen_out`, `x2`, `y1` that satisfies all five premises and the antecedent, but violates the conclusion.

Here is one:

```
gen_out.x = 200      gen_out.y = 100      x2 = -100      y1 = 99
```

Check it: premise 1 holds (`-100` is in range); premise 2 holds (`99 <= 99`); premises 3 and 4 hold **vacuously** -- their antecedents `gen_out.x < 100` and `gen_out.x == 100` are both false; premise 5 holds (`99 == 100 - 1`); the property's antecedent holds (`100 <= 200`). But `merged = (-100, 99)`, and `99 < -100` is false. The SMT solver finds exactly this kind of model, so the VC cannot be discharged.

Of course, `gen_out.x = 200` **can never happen in the real system** -- GenStruct's integration guarantee bounds it to `[-100, 100]`. But look back at the five premises: *nothing at the join place says so*. This is the modularity of the system proof: each VC sees only the assertion bound at its own place. The bound on `gen_out.x` was asserted at `after gen`, and it is even still derivable at `after incx` (from `x1 == gen_out.x + 1` and `x1 <= 101`) -- but at `after clampx` the inherited chain keeps only `inRange100(x2)` and the two *conditional* case facts, and a conditional fact whose condition cannot be established is useless. The bound has been **dropped**, and without it, ClampX's cases say nothing about how `x2` relates to `gen_out.x`.

Why didn't the existing `End_To_End_Functional` property hit this problem? Look at its conclusion, `mergedReflectsGen`: it is *itself* written as a case split on `gen_out.x` (`(g.x < 100) implies ...`, `(g.x == 100) implies ...`), so it never needs to know that one of the cases actually applies. Your conclusion `m.y < m.x` is unconditional -- it needs `gen_out.x <= 100` to know the clamp cases cover the situation. The inherited plumbing was designed for a conclusion shaped differently from yours.

## Activity 3 - Repair the Property: Add the Missing Carries

The repair is to carry the missing fact to where it is needed. It is needed at the join (the merger's in-place), and to *establish* it at the join it must also be asserted at `after clampx` (the join VC derives the join assertion only from the two branch-end assertions -- see the optional task below). At `after clampx` it is provable, because the place before it (`after incx`) carries `x1 == gen_out.x + 1` and `x1 <= 101`.

* **Task:** Replace your property with the version below (the `before consume` binding is unchanged; two carry bindings are added):

```
        property Y_Stays_Strictly_Below_X :> Pipeline_Functional
          "if the generated struct has y <= x, the consumed struct has y < x:
          |merged.y == gen.y - 1 < gen.y <= gen.x <= x2 == merged.x, where the
          |last step needs ClampX's cases plus the carried bound gen_out.x <= 100" {
          after clampx "carry the generator's x upper bound past the clamp; derivable
            |here from x1 == gen_out.x + 1 and x1 <= 101, but not carried by the
            |inherited chain -- without it ClampX's case implications are vacuous
            |downstream and nothing relates x2 back to gen_out.x":
            gen_out.x <= 100 [i32];
          at branches_joined "make the bound available to the merger":
            gen_out.x <= 100 [i32];
          before consume "the end-to-end conclusion at the consumer's dispatch":
            yStaysBelowX(merged, gen_out);
        }
```

Note what this demonstrates about property inheritance: a child property can bind places its parents already bind, and the assertions are **conjoined** there. You are strengthening the inherited chain at two places without touching the parents (or the other properties that inherit them).

* **Task:** Run HAMR Type Checking, then HAMR CodeGen, then `make y_stays_strictly_below_x` again. This time:

```
verification results:: 27 verified, 0 errors (partial verification with `--verify-*`)
```

* **Task (optional, recommended):** To see for yourself that *both* carries are needed, temporarily delete the `after clampx` binding (keeping `at branches_joined`), regenerate, and re-verify. The failure **moves upstream** to `vc_next_assert_skip_t6` (VC[12]) -- the control-point VC that derives the join assertion from `after_clampx` and `after_decy`, neither of which now bounds `gen_out.x`. Restore the binding, regenerate, and confirm 27/0 again. This is place-locality in action: an assertion is only as available as the place you bound it to.

* **Task:** Commit/push with a message such as "StructSplit add-Prop-yLEx - property verified (added gen_out.x bound carries)".

## Activity 4 - Read the Completed Proof

With the property verifying, the VC module is now a machine-checked rendering of your Part 1 informal argument. Three VCs carry the weight; read them in order.

* **Task:** Open `src/y_stays_strictly_below_x/vc_sequential.rs` and find `vc_next_assert_task_clampx` (VC[8]) -- the step that *establishes* your new carry:

```rust
/** VC[8]: Next-Assert (task) -- after_incx + frames + CLAMPX postcondition |- after_clampx */
pub proof fn vc_next_assert_task_clampx(pre: SystemState, post: SystemState)
  requires
    sys_assert_y_stays_strictly_below_x_after_incx(pre),
    clampx_local_write_frame(pre, post),
    clampx_global_write_frame(pre, post),
    clampx::compute_case_In_Range(pre.x1, post.x2),
    clampx::compute_case_Above(pre.x1, post.x2),
    clampx::compute_case_Below(pre.x1, post.x2),
  ensures
    sys_assert_y_stays_strictly_below_x_after_clampx(post),
{}
```

The `after_incx` assertion (inherited) supplies `x1 == gen_out.x + 1` and `x1 <= 101`, so `gen_out.x <= 100` follows by arithmetic; ClampX's write frame guarantees `gen_out` is untouched by the firing, so the fact holds in the post-state. Also open `sys_assert_y_stays_strictly_below_x_after_clampx` in `assertions.rs` and observe the three-way conjunction: `Pipeline_Ranges`' `inRange100(x2)`, `Pipeline_Functional`'s case implications, and your `(st.gen_out.x <= 100i32)` -- three property layers, one place, flattened.

* **Task:** Find `vc_next_assert_skip_t6` (VC[12]), the branch join. No component fires at a control point, so the state is unchanged and the VC simply conjoins the two branch-end assertions into the join assertion -- which now includes your carried bound.

* **Task:** Re-read `vc_next_assert_task_merge_stage` (VC[14]) and convince yourself the counter-model from Activity 2 is now excluded: with `gen_out.x <= 100` in the premises, one of the two clamp cases *must* apply, giving `x2 >= gen_out.x` in both. The full chain the SMT solver assembles is exactly your informal argument:

```
merged.y == y1 == gen_out.y - 1  <  gen_out.y  <=  gen_out.x  <=  x2 == merged.x
```

## Activity 5 - Seed a Bug: When the Property Itself Is Wrong

In Activity 2 the VC failed because the *proof outline* was incomplete -- the property was true, but a fact was missing at a place. A VC can also fail because the *claim is false*. These two situations produce the **same kind of error message**, and learning to tell them apart is essential. Let's manufacture the second kind.

Suppose you had reasoned: "IncX adds one to `x` and DecY subtracts one from `y`, so the gap between `y` and `x` grows by *two* -- the consumed struct should satisfy `m.y + 1 < m.x`."

* **Task:** Change the conclusion of the `yStaysBelowX` function to claim this stronger gap:

```
        def yStaysBelowX(m: SysPropStructSplit_Data_Model::StructXY,
                         g: SysPropStructSplit_Data_Model::StructXY): Base_Types::Boolean :=
          (g.y <= g.x) implies (m.y + 1 [i32] < m.x);
```

* **Task:** Run Type Checking (well-formed -- the type checker has no opinion about truth), CodeGen, and `make y_stays_strictly_below_x`. You get the *same* failure as in Activity 1:

```
error: postcondition not satisfied
   --> src/y_stays_strictly_below_x/vc_sequential.rs:161:5
...
161 |     sys_assert_y_stays_strictly_below_x_after_merge_stage(post),
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ failed this postcondition

verification results:: 26 verified, 1 errors (partial verification with `--verify-*`)
```

Same VC, same message -- but this time no amount of extra carrying can fix it, because the claim is false of the system.

* **Task:** Find the run of the system that violates the claim. Work the pipeline by hand starting from `gen_out = (x: 100, y: 100)`:

```
antecedent:  100 <= 100         (holds)
x branch:    x1 = 101  -->  clamped: x2 = 100
y branch:    y1 = 99
merged:      (x: 100, y: 99)
claim:       99 + 1 < 100  ==  100 < 100    (FALSE)
```

The strengthened claim holds everywhere *except* the saturation corner: for `gen.x < 100` the gap really does grow by two. The corner is exactly where ClampX cancels IncX's increment -- the strict `<` you proved in Activity 3 is worth precisely the one unit that DecY contributes, and not a unit more. (This is also why the exercise property's conclusion could not have been strengthened, and why the informal-argument step "`x <= x2`" -- not `x < x2` -- mattered in Part 1.)

The practical lesson for diagnosing any failing system-property VC:

1. **First try to refute the assertion at the failing place** with a concrete run of the system (as above). If you find one, the property (or an intermediate carry) over-claims -- weaken it.
2. **If you cannot refute it**, the claim is likely true but under-supported: hunt for the missing fact by listing the failing VC's premises (as in Activity 2) and asking which step of your informal argument they cannot justify. Add the missing carry at the place that needs it (and at the places needed to establish it).

* **Task:** Restore the original conclusion `(g.y <= g.x) implies (m.y < m.x)`, regenerate, and confirm `make y_stays_strictly_below_x` reports 27 verified, 0 errors.

## Activity 6 - Verify the Whole System Proof

* **Task:** From `hamr/microkit/crates/sys_nominal_proof`, run `make` (all properties plus the shared Integration and Commutativity VCs):

```
verification results:: 138 verified, 0 errors
```

That is the baseline 111 from Part 1 Activity 2, plus the 27 VCs of your new property (1 Init-State + 17 Sequential + 1 Post-Pre + 8 Non-Disabling). Note that the other four properties were untouched by everything you did -- each property's VCs see only its own assertions, so properties can be added (and debugged) without entangling existing proofs.

Remember what this proof does and does not establish: the system property holds *relative to* each component satisfying its own GUMBO contract -- obligations discharged separately by the per-component Verus runs. If you have the time (the full run takes a while), run `make verus` from `hamr/microkit/` to discharge the component-level proofs together with the system proof.

* **Task:** Commit/push your completed exercise with a message such as "StructSplit add-Prop-yLEx - completed".

**Conclusion**: Across the two parts of this exercise you exercised the entire system-property workflow: informal argument, GUMBO specification, type checking, VC generation, reading the generated proof structure, and Verus verification -- including the two failure modes you will meet in practice. The completed solution is available in the `HAMR-SysMLv2-Rust-seL4-P-DP-SysPropStructSplit-add-Prop-yLEx-solution` folder of the tutorials repository; comparing your property text against it is a good final check.
