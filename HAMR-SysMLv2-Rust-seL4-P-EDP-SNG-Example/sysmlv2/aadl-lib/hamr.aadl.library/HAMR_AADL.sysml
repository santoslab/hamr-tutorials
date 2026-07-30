package HAMR_AADL {
  private import AADL::*;

  part def System :> AADL::System {
    attribute Scheduling: HAMR_Microkit::Scheduling;
  }

  part def Processor :> AADL::Processor {
    attribute Frame_Period: Timing_Properties::Frame_Period;
    attribute Clock_Period: Timing_Properties::Clock_Period;
    attribute Max_Domain: CASE_Scheduling::Max_Domain;
  }

  part def VirtualProcessor :> AADL::VirtualProcessor {
    attribute Max_Domain: CASE_Scheduling::Max_Domain;
  }

  part def Process :> AADL::Process {
    attribute Domain: CASE_Scheduling::Domain;

    attribute Passive: HAMR_Microkit::Passive;
    attribute SMC: HAMR_Microkit::SMC;
  }

  part def Thread :> AADL::Thread {
    attribute Implementation_Language: HAMR::Implementation_Language;

    attribute Component_Provenance: HAMR::Component_Provenance;
    attribute Native_Name: HAMR::Native_Name;

    attribute Ros_Node_Kind: HAMR_ROS::Ros_Node_Kind;
    attribute Ros_Namespace: HAMR_ROS::Ros_Namespace;

    attribute Passive: HAMR_Microkit::Passive;
    attribute SMC: HAMR_Microkit::SMC;
  }


  part def Data :> AADL::Data {
    attribute Type_Provenance: HAMR::Type_Provenance;
    attribute Native_Name: HAMR::Native_Name;
  }

  part def Struct :> Data;

  part def Array :> Data {
    part Base_Type : Data;

    attribute Dimensions: ScalarValues::Positive;

    attribute Array_Size_Kind: HAMR::Array_Size_Kinds;
  }
}
