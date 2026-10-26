/*============================================================
    SysPropStructSplit   P r o p e r t i e s

    Scheduling domain assignments for the pipeline.  Domains start
    at 2 (0 and 1 are reserved by Microkit) and are ordered to
    match the dataflow so a sample propagates end-to-end within a
    single major frame.
  ============================================================*/

package SysPropStructSplit_Properties {

  private import HAMR::*;

  attribute Domain_Gen     : CASE_Scheduling::Domain = 2;
  attribute Domain_Split   : CASE_Scheduling::Domain = 3;
  attribute Domain_IncX    : CASE_Scheduling::Domain = 4;
  attribute Domain_DecY    : CASE_Scheduling::Domain = 5;
  attribute Domain_ClampX  : CASE_Scheduling::Domain = 6;
  attribute Domain_Merge   : CASE_Scheduling::Domain = 7;
  attribute Domain_Consume : CASE_Scheduling::Domain = 8;
}
