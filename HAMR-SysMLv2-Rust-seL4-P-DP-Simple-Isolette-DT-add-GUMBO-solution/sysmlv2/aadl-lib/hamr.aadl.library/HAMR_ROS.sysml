library package HAMR_ROS {

  attribute def Ros_Node_Kind :> AADL::Property, Ros_Node_Kinds;

  enum def Ros_Node_Kinds { 
    ros2; 
    microRos;
  }

  attribute ros2: Ros_Node_Kind = Ros_Node_Kinds::ros2 as Ros_Node_Kind;
  attribute microRos: Ros_Node_Kind = Ros_Node_Kinds::microRos as Ros_Node_Kind;

  attribute def Ros_Topic_Name :> AADL::Property, ScalarValues::String {
    doc AADL_Doc
    /*
     * The ROS 2 topic name that a port publishes to or subscribes on.
     *
     * By default the ROS 2 backend derives topic names from port instance
     * paths, which yields names that only ports generated from the same model
     * can know.  Ros_Topic_Name overrides that default so a generated node
     * can exchange messages with preexisting ROS nodes (e.g., turtlesim, joy
     * drivers) whose topic names are fixed, or can publish/subscribe on a
     * conventional name chosen by the modeler.
     *
     * The value follows ROS name-resolution semantics: an absolute name
     * (leading '/') is used verbatim; a relative name composes with the
     * node's namespace as assigned at launch (e.g., relative
     * "turtle1/cmd_vel" under namespace "uros_demo" resolves to
     * "/uros_demo/turtle1/cmd_vel").
     *
     * May be attached to a port or to a connection (a connection-carried
     * value applies to both endpoint ports).  Two ports joined by a
     * connection whose explicit Ros_Topic_Names differ is an error; distinct
     * unconnected ports sharing a topic name is legal ROS pub/sub
     * (multi-writer/multi-reader) provided their payload types agree.  A port
     * that is intentionally unconnected but carries a Ros_Topic_Name is the
     * normal pattern for binding to an off-model (stock) node at the
     * deployment layer.
     *
     * PROPOSED HAMR extension -- not yet implemented by codegen.  See
     * the Topic Naming section of PlatformProvidedTypes-design.md (repo
     * root) for the full design, including the precedence rules and
     * consistency checks.
     */
  }

  attribute def Ros_Namespace :> AADL::Property, ScalarValues::String {
    doc AADL_Doc
    /*
     * The ROS namespace a node is placed in when launched.
     *
     * A node's namespace prefixes every RELATIVE name the node uses: a
     * relative Ros_Topic_Name, and -- for preexisting (stock) nodes -- the
     * relative topic names hardcoded in their implementation.  Absolute
     * names (leading '/') bypass the namespace entirely.  This composition
     * is the standard ROS mechanism for binding unmodified stock nodes into
     * a system: e.g., launching turtlesim_node with namespace "uros_demo"
     * resolves its built-in relative subscription turtle1/cmd_vel to
     * /uros_demo/turtle1/cmd_vel.
     *
     * This is a general per-node deployment property, applicable to any
     * thread regardless of provenance.  A value on a subcomponent usage
     * overrides one on the thread definition.  For a generated node it is
     * applied in both the generated launch entry and node initialization
     * (the namespace argument of node init, which also namespace-qualifies
     * the node's logger name).  For a Platform_Provided component it is
     * applied in the generated launch entry.
     *
     * PROPOSED HAMR extension -- not yet implemented by codegen.  See the
     * Topic Naming and Platform-Provided Components sections of
     * PlatformProvidedTypes-design.md (repo root).
     */
  }
}