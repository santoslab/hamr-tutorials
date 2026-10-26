// This file will not be overwritten if HAMR codegen is rerun

//! System tests for the event-data-port Simple Isolette, run on seL4 under
//! the test scheduler.
//!
//! The schedule is operator_interface, temp_sensor, thermostat, heat_source,
//! then a pad.  Every test starts at slot 0 of a fresh hyperperiod (the
//! runner normalizes the schedule position), so `run_to_thread(thermostat)`
//! parks after the Operator Interface and the Temp Sensor have run and before
//! anything has read their outputs.  The tests set up the Thermostat and the
//! Heat Source there, then let the frame finish.
//!
//! The controller checks every dispatch against the GUMBO component
//! contracts and the `nominal` composition's system assertions, so each test
//! also fails on any violation it did not declare.

use crate::system_tests::api;
use crate::system_tests::harness::run_property;
use crate::system_tests::inspect;
use crate::system_tests::inspect::channels;
use crate::system_tests::observe;
use crate::{system_tests, sys_assert, sys_assert_eq};
use data::Isolette_Data_Model::{On_Off, Set_Points, Temp};

fn temp(degrees: i32) -> Temp {
  Temp { degrees }
}

fn set_points(lower: i32, upper: i32) -> Set_Points {
  Set_Points { lower: temp(lower), upper: temp(upper) }
}

/// What the Thermostat sees at its next dispatch.
struct Scenario {
  /// the sensed temperature (data port)
  current_temp: i32,
  /// whether the Temp Sensor reported a change (event port)
  temp_changed: bool,
  /// a pending set point message, if any (event data port)
  set_point_msg: Option<Set_Points>,
  /// the Thermostat's latched set points (GUMBO state variable)
  latched: Set_Points,
  /// the commanded heat state (GUMBO state variable); the heater starts in
  /// agreement with it
  cmd: On_Off,
}

/// Sets up the Thermostat and the Heat Source for scenario `s`, and parks
/// before the Thermostat in the frame where it happens.
///
/// It takes two frames.  An injected state variable takes effect when its
/// thread next runs, so injecting `lastCmd` and `heater_state` in one frame
/// would leave the two disagreeing between the Thermostat's dispatch and the
/// Heat Source's -- a real Heater_Agreement violation, of the test's own
/// making.  So the first frame puts both threads' state in place with no
/// events pending and the system assertions switched off, and the second
/// applies the scenario's events with the state already in agreement.
fn set_up(s: Scenario) {
  observe::set_sysverif(false);
  let _ = api::run_to_thread(channels::thermostat_thermostat_MON);
  set_inputs(&s, false);
  let _ = api::hstep(1);
  observe::set_sysverif(true);

  let _ = api::run_to_thread(channels::thermostat_thermostat_MON);
  set_inputs(&s, true);
}

/// Sets every input and state variable of the Thermostat and the Heat
/// Source; with `events` false, nothing is pending for either.
fn set_inputs(s: &Scenario, events: bool) {
  inspect::set_thermostat_thermostat(inspect::thermostat_thermostat_PreState {
    current_temp: temp(s.current_temp),
    temp_changed: if events && s.temp_changed { Some(0) } else { None },
    desired_temp: if events { s.set_point_msg } else { None },
    sv_currentSetPoints: s.latched,
    sv_lastCmd: s.cmd,
  });
  inspect::set_heat_source_heat_source(
    inspect::heat_source_heat_source_PreState {
      heat_control: None,
      sv_heater_state: s.cmd,
    });
}

/// The Thermostat's control law: the command it should end up with.
fn expected_cmd(s: &Scenario) -> On_Off {
  let latched = match s.set_point_msg { Some(m) => m, None => s.latched };
  if !(s.temp_changed || s.set_point_msg.is_some()) {
    s.cmd
  } else if s.current_temp > latched.upper.degrees {
    On_Off::Off
  } else if s.current_temp < latched.lower.degrees {
    On_Off::Onn
  } else {
    s.cmd
  }
}

fn last_cmd() -> Option<On_Off> {
  inspect::get_thermostat_thermostat_sv_lastCmd()
}

fn latched_set_points() -> Option<Set_Points> {
  inspect::get_thermostat_thermostat_sv_currentSetPoints()
}

fn heat_cmd_sent() -> Option<On_Off> {
  inspect::get_thermostat_thermostat_heat_control()
}

fn heater_state() -> Option<On_Off> {
  inspect::get_heat_source_heat_source_sv_heater_state()
}

system_tests! {
  suite smoke {
    fn schedule_advances_by_hyperperiod() {
      let before = api::info_state();
      let after = api::hstep(1);
      sys_assert!(after.flags & api::FLAG_BAD_COMMAND == 0);
      sys_assert_eq!(after.hyperperiod_num, before.hyperperiod_num + 1);
    }

    fn stepping_one_slot_at_a_time_wraps() {
      let n = api::schedule().num_timeslices;
      let before = api::info_state();
      let mut i = 0;
      while i < n {
        let _ = api::sstep(1);
        i += 1;
      }
      let after = api::info_state();
      sys_assert_eq!(after.current_timeslice, before.current_timeslice);
      sys_assert_eq!(after.hyperperiod_num, before.hyperperiod_num + 1);
    }
  }

  suite control {
    // A reported temperature below the lower set point: the Thermostat
    // commands heat, sends the command (it changed), and the Heat Source
    // latches it.
    fn too_cold_turns_the_heater_on() {
      set_up(Scenario {
        current_temp: 96, temp_changed: true, set_point_msg: None,
        latched: set_points(98, 101), cmd: On_Off::Off,
      });

      let _ = api::sstep(1); // thermostat
      sys_assert_eq!(last_cmd(), Some(On_Off::Onn));
      sys_assert_eq!(heat_cmd_sent(), Some(On_Off::Onn));

      let _ = api::sstep(1); // heat_source
      sys_assert_eq!(heater_state(), Some(On_Off::Onn));
    }

    // The same cold temperature, but no event: the Temp Sensor reported no
    // change and no set point message is pending, so the Thermostat is not
    // triggered.  Nothing changes and, send-on-change, nothing is sent.
    fn no_event_no_command() {
      set_up(Scenario {
        current_temp: 96, temp_changed: false, set_point_msg: None,
        latched: set_points(98, 101), cmd: On_Off::Off,
      });

      let _ = api::sstep(1); // thermostat
      sys_assert_eq!(last_cmd(), Some(On_Off::Off));
      sys_assert_eq!(heat_cmd_sent(), None);

      let _ = api::sstep(1); // heat_source
      sys_assert_eq!(heater_state(), Some(On_Off::Off));
    }

    // A new set point message also triggers the Thermostat, and is latched
    // first: 100 is in range for the old set points but above the new ones,
    // so the heater goes off.
    fn set_point_message_triggers_and_is_latched() {
      set_up(Scenario {
        current_temp: 100, temp_changed: false,
        set_point_msg: Some(set_points(96, 99)),
        latched: set_points(98, 101), cmd: On_Off::Onn,
      });

      let _ = api::sstep(1); // thermostat
      sys_assert_eq!(latched_set_points(), Some(set_points(96, 99)));
      sys_assert_eq!(heat_cmd_sent(), Some(On_Off::Off));

      let _ = api::sstep(1); // heat_source
      sys_assert_eq!(heater_state(), Some(On_Off::Off));
    }

    // In range, triggered: the command does not change, so no message is
    // sent, and the heater keeps its state.
    fn in_range_keeps_the_heater_as_it_is() {
      set_up(Scenario {
        current_temp: 99, temp_changed: true, set_point_msg: None,
        latched: set_points(98, 101), cmd: On_Off::Onn,
      });
      let _ = api::hstep(1);
      sys_assert_eq!(heat_cmd_sent(), None);
      sys_assert_eq!(heater_state(), Some(On_Off::Onn));
    }
  }

  suite properties {
    // Random scenarios within the Thermostat's assumptions: temperatures the
    // Temp Sensor can report to it, well-formed set points, and any
    // combination of events.  After the frame the heater holds the command
    // the control law selects.  The system assertions (Heater_Agreement,
    // Too_Cold_Heats) are checked on every case as well.
    fn heater_follows_the_control_law() {
      sys_assert!(run_property(32, || any_scenario(), |s| {
        let expected = expected_cmd(&s);
        set_up(s);
        let _ = api::hstep(1);
        proptest::prop_assert_eq!(last_cmd(), Some(expected));
        proptest::prop_assert_eq!(heater_state(), Some(expected));
        Ok(())
      }));
    }
  }

  suite robustness {
    // The Thermostat assumes every set point message is well-formed
    // (lower <= upper).  A malformed one fails its CEP_Pre, so the
    // controller rejects the dispatch: the Thermostat does not run, and its
    // state is untouched.  The message is still pending after the
    // rejection, so the test clears it -- otherwise the Thermostat would be
    // rejected again at every later dispatch.
    fn malformed_set_point_is_rejected() {
      observe::expect(observe::Expect::CepPre(
        observe::Thread::thermostat_thermostat));
      set_up(Scenario {
        current_temp: 96, temp_changed: true,
        set_point_msg: Some(set_points(101, 98)),
        latched: set_points(98, 101), cmd: On_Off::Off,
      });
      let _ = api::sstep(1); // the thermostat's slot passes; it does not run
      sys_assert_eq!(latched_set_points(), Some(set_points(98, 101)));
      sys_assert_eq!(last_cmd(), Some(On_Off::Off));
      inspect::clear_thermostat_thermostat_desired_temp();
    }

    // Loses the heat command in transit: the Thermostat sends On, and the
    // test clears it before the Heat Source runs, as a dropped message
    // would.  Each thread still meets its own contract -- the Heat Source
    // received nothing, so it holds its state -- but the heater no longer
    // agrees with the commanded state, which only the system layer can see.
    // The test takes the violations and checks that they are all system
    // assertions, the first of them right after the Heat Source.  (The
    // frame then ends, so the invariant's END and START assertions break as
    // well; the next test's set_up puts the heater back in agreement.)
    fn lost_command_breaks_heater_agreement() {
      set_up(Scenario {
        current_temp: 96, temp_changed: true, set_point_msg: None,
        latched: set_points(98, 101), cmd: On_Off::Off,
      });
      let _ = api::sstep(1); // thermostat sends On
      sys_assert_eq!(heat_cmd_sent(), Some(On_Off::Onn));
      inspect::clear_heat_source_heat_source_heat_control();
      let _ = api::sstep(1); // heat_source receives nothing
      sys_assert_eq!(heater_state(), Some(On_Off::Off));

      let violations = observe::take();
      let first = violations.iter().next();
      sys_assert!(violations.iter()
        .all(|v| v.kind == observe::Kind::SysAssert));
      sys_assert!(first.map_or(false, |v| v.point == "after hs"));
      sys_assert!(violations.contains(observe::Expect::SysAssert(
        observe::sys::nominal::COMPOSITION,
        observe::sys::nominal::Heater_Agreement)));
    }
  }
}

use proptest::prelude::{Just, Strategy};

/// Random scenarios within the Thermostat's assumptions: temperatures it
/// accepts (95..104), well-formed set points (at most 5 apart), any
/// combination of events, and either commanded state.
fn any_scenario() -> impl Strategy<Value = Scenario> {
  let temps = 95i32..=104;
  let events = (proptest::bool::ANY, proptest::bool::ANY);
  let span = (95i32..=104, 0i32..=5);
  let cmd = proptest::prop_oneof![Just(On_Off::Onn), Just(On_Off::Off)];
  (temps, events, span.clone(), span, cmd).prop_map(
    |(t, (changed, msg), (lo, w), (mlo, mw), cmd)| Scenario {
      current_temp: t,
      temp_changed: changed,
      set_point_msg: if msg { Some(set_points(mlo, mlo + mw)) } else { None },
      latched: set_points(lo, lo + w),
      cmd,
    })
}

/// How a shrunk failing scenario is printed:
/// `temp 102, no change, msg 95..95, latched 95..95, cmd Onn`.
impl core::fmt::Debug for Scenario {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    let change = if self.temp_changed { "change" } else { "no change" };
    write!(f, "temp {}, {}, msg ", self.current_temp, change)?;
    match self.set_point_msg {
      Some(m) => write!(f, "{}..{}", m.lower.degrees, m.upper.degrees)?,
      None => write!(f, "none")?,
    }
    write!(f, ", latched {}..{}, cmd {:?}",
           self.latched.lower.degrees, self.latched.upper.degrees, self.cmd)
  }
}
