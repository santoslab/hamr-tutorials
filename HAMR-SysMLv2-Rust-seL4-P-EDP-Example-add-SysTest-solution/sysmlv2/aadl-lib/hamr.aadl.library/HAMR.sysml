library package HAMR {

  public import HAMR_Time_Units::*;
  public import HAMR_Microkit::*;
  public import HAMR_ROS::*;
  public import HAMR_AADL::*;
  public import AADL::*;
  public import Base_Types::*;

  attribute def Array_Size_Kind :> AADL::Property, Array_Size_Kinds {
    doc AADL_Doc 
    /* 
     * Specifies the kind of array size semantics for an array data type.
     * 
     * Fixed: The array always has exactly the number of elements specified in
     *   the Data_Model::Dimension property. Both memory allocation and logical
     *   content size are fixed.
     *
     * Bounded: The array may contain fewer elements than specified by
     *   Data_Model::Dimension, but never more. The full size may be allocated in
     *   memory, but only part of it is considered logically valid.
     *
     * Unbounded: The array does not have a statically known size limit.
     *   If Data_Model::Dimension is provided then all dimensions must be equal to 0. 
     *   The array may grow arbitrarily, and size checks must be enforced at runtime.
     */
  }

  enum def Array_Size_Kinds {
    enum Fixed;
    enum Bound;
    enum Unbounded;
  }

  //alias Fixed for Array_Size_Kinds::Fixed;
  //alias Bound for Array_Size_Kinds::Bound;
  //alias Unbounded for Array_Size_Kinds::Unbounded;

  attribute Fixed: Array_Size_Kind = Array_Size_Kinds::Fixed as Array_Size_Kind;
  attribute Bound: Array_Size_Kind = Array_Size_Kinds::Bound as Array_Size_Kind;
  attribute Unbounded: Array_Size_Kind = Array_Size_Kinds::Unbounded as Array_Size_Kind;

  attribute def Type_Provenance :> AADL::Property, Provenances {
    doc AADL_Doc
    /*
     * Who supplies the concrete (on-the-wire) realization of a data type.
     *
     * Model (default): codegen emits the type (today's behavior).
     *
     * Platform_Provided: the type already exists on the target platform
     *   (e.g., the ROS 2 message type sensor_msgs/msg/Joy, denoted by the
     *   structural convention SysML package name = ROS package name, part
     *   def name = message name).  Codegen emits no type definition; the
     *   native type flows through ports/topics and generated APIs are typed
     *   against it.  Any fields declared on the data component are optional
     *   MIRRORS -- specification-level projections used by GUMBO contracts,
     *   monitors, and memory sizing -- not layout claims.
     *
     * Infrastructure: not applicable to types (lint error).
     *
     * PROPOSED HAMR extension -- not yet implemented by codegen.  See
     * PlatformProvidedTypes-design.md (repo root).
     */
  }

  attribute def Component_Provenance :> AADL::Property, Provenances {
    doc AADL_Doc
    /*
     * Who supplies the executable realization of a component (thread).
     *
     * Model (default): codegen generates the component's code.
     *
     * Platform_Provided: the component is a preexisting executable (e.g.,
     *   the stock ROS 2 node turtlesim/turtlesim_node, denoted by the
     *   structural convention SysML package name = ROS package name, part
     *   def name = executable name).  Codegen emits no code -- only
     *   deployment artifacts: a launch entry (subcomponent usage name as
     *   the node name, composed with any Ros_Namespace) and a package
     *   dependency so the deployment tooling pulls the stock node.  Such
     *   components are contracts-only by construction: their GUMBO port
     *   contracts serve as trusted assumptions for system-level
     *   composition -- checkable at runtime by monitors, but not by
     *   verification or testing, since there is no code.  The mechanism is
     *   not ROS-specific.
     *
     * Infrastructure: not applicable to components (lint error).
     *
     * PROPOSED HAMR extension -- not yet implemented by codegen.  See
     * PlatformProvidedTypes-design.md (repo root).
     */
  }

  attribute def Port_Provenance :> AADL::Property, Provenances {
    doc AADL_Doc
    /*
     * Who supplies the communication realization of a port.
     *
     * Model (default): codegen emits the port's publisher/subscription and
     *   application API (today's behavior).
     *
     * Infrastructure: the port's traffic is produced as a side effect by
     *   platform infrastructure rather than by application code -- e.g., a
     *   ROS 2 node's /rosout publisher, created and driven by rcl logging.
     *   Codegen emits no data-plane code for the port (no publisher, no
     *   put_ API); the port's topic name is pinned by the platform; and the
     *   port's presence may trigger enablement (e.g., generated init calls
     *   rcl_logging_configure in a micro-ROS node).
     *
     * Infrastructure-realized ports are RESERVED BY NAME: rosout is a
     *   built-in port of every ROS node, so a model port named rosout
     *   denotes that built-in and its declaration must match it (an
     *   EventDataPort typed rcl_interfaces::Log; /rosout if a topic name is
     *   given).  Port_Provenance is therefore OPTIONAL on reserved ports --
     *   documentation rather than trigger -- and when provided it must be
     *   Infrastructure (any other value is a validation error).  Ports with
     *   other names carrying rcl_interfaces::Log are ordinary ports with no
     *   restrictions (e.g., an app-managed log relay publisher).
     *
     * Platform_Provided: reserved; the ports of a Platform_Provided
     *   component are implicitly platform-realized and need no marking.
     *
     * PROPOSED HAMR extension -- not yet implemented by codegen.  See
     * PlatformProvidedTypes-design.md (repo root).
     */
  }

  enum def Provenances {
    enum Model;
    enum Platform_Provided;
    enum Infrastructure;
  }

  attribute def Native_Name :> AADL::Property, ScalarValues::String {
    doc AADL_Doc
    /*
     * Escape hatch for the structural naming convention of
     * Platform_Provided elements.
     *
     * By default the native name of a platform-provided element is derived
     * from model structure: for data types, SysML package name = platform
     * package and part def name = message name (sensor_msgs::Joy ->
     * sensor_msgs/msg/Joy, with the msg sub-namespace implied); for
     * components, part def name = executable name (turtlesim::turtlesim_node
     * -> package turtlesim, executable turtlesim_node).  Native_Name
     * overrides that derivation when the convention cannot spell the real
     * name -- e.g., ROS 2 services or actions (which live under srv/action
     * rather than the implied msg), native names that are not valid SysML
     * identifiers, or a deliberate rename on the model side.
     *
     * The expected format depends on what the property is attached to:
     *   - data component:  "pkg/msg/Type"  (three segments, e.g.
     *     "example_interfaces/srv/AddTwoInts" for a service type)
     *   - thread (component): "pkg/executable"  (two segments)
     *
     * Optional; meaningful only on elements whose provenance is
     * Platform_Provided.  Codegen consumes it wherever it would otherwise
     * derive the native name (includes, payload types, package
     * dependencies, launch entries).
     *
     * PROPOSED HAMR extension -- not yet implemented by codegen.  See
     * PlatformProvidedTypes-design.md (repo root).
     */
  }


  attribute def Implementation_Language :> AADL::Property, Implementation_Languages;
  
  enum def Implementation_Languages :> AADL::Property {
    enum Slang;
    enum C;
    enum Rust;
  }

  // alias Slang for Implementation_Languages::Slang;
  // alias C for Implementation_Languages::C;
  // alias Rust for Implementation_Languages::Rust;

  attribute Slang: Implementation_Language = Implementation_Languages::Slang as Implementation_Language;
  attribute C: Implementation_Language = Implementation_Languages::C as Implementation_Language;
  attribute Rust: Implementation_Language = Implementation_Languages::Rust as Implementation_Language;
}
