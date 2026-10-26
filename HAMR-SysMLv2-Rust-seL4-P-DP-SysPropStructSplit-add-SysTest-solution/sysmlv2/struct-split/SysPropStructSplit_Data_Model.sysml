/*============================================================
    SysPropStructSplit   D a t a   M o d e l

    Defines the data types used for inter-component communication
    in the struct-split pipeline.

    StructXY is a simple record with two Integer_32 fields that
    is generated, split into its components, transformed, and
    re-assembled as it flows through the pipeline.
  ============================================================*/

package SysPropStructSplit_Data_Model {
    private import HAMR::*;

    part def StructXY :> Data {
        part x : Base_Types::Integer_32;
        part y : Base_Types::Integer_32;
    }
}
