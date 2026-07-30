/* Opt-in staging area for HAMR AADL extensions that are not yet part of
 * HAMR_AADL.
 *
 * PURPOSE
 *
 * These definitions extend the AADL port defs with the HAMR property
 * vocabulary described in the platform-provided-types design (currently
 * Port_Provenance and Ros_Topic_Name, used to model ROS 2 platform-provided
 * ports and deployment-layer topic naming).  They logically belong in
 * HAMR_AADL, but live here so that models can adopt them independently of
 * the AADL libraries as a whole.
 *
 * WHY SEPARATE
 *
 * A port whose def is a *subtype* of an AADL port def is only recognized by
 * recent versions of the HAMR SysML frontend.  Older versions match the port
 * classifier by exact name (AADL::DataPort, AADL::EventDataPort,
 * AADL::EventPort) and report "Unexpected port type" for anything else, so a
 * model typed against these defs fails to instantiate.  Keeping them in their
 * own file lets users of older Sireum releases simply omit it: nothing in
 * HAMR_AADL or HAMR refers to this package, so its absence is not an error.
 *
 * LIFECYCLE
 *
 * This package is temporary.  Once the AADL libraries ship inside sireum.jar
 * -- so that the library and frontend versions can no longer drift apart --
 * these definitions move into HAMR_AADL and this package is deleted.
 *
 * IMPORT ORDER IS SIGNIFICANT
 *
 * The port defs here deliberately shadow the same-named defs in AADL, which
 * models normally reach transitively through HAMR (HAMR publicly imports both
 * HAMR_AADL::* and AADL::*).  Import this package BEFORE HAMR:
 *
 *   private import HAMR_AADL_Preview::*;
 *   private import HAMR::*;
 *
 * With the imports in the opposite order, unqualified DataPort /
 * EventDataPort / EventPort resolve to the AADL defs instead, silently
 * dropping the attributes below -- the model still instantiates, but without
 * Port_Provenance or Ros_Topic_Name. */

package HAMR_AADL_Preview {

  // Mixed into each port def below; not usable on its own.  Declaring the
  // attributes once here keeps the three port defs in sync.
  abstract port def Port {
    attribute Port_Provenance: HAMR::Port_Provenance;

    attribute Ros_Topic_Name: HAMR_ROS::Ros_Topic_Name;
  }

  // Each specializes its AADL counterpart, so a usage typed against one of
  // these remains an ordinary AADL port for every other purpose.
  port def DataPort :> AADL::DataPort, Port {}
  port def EventDataPort :> AADL::EventDataPort, Port {}
  port def EventPort :> AADL::EventPort, Port {}
}
