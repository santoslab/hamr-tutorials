// #Sireum
package ProdConsExample

import org.sireum._
import art.Art
import art.scheduling.legacy.Legacy
import art.scheduling.roundrobin.RoundRobin
import art.scheduling.static.Schedule.{DSchedule, DScheduleSpec}
import art.scheduling.static._

// Do not edit this file as it will be overwritten if HAMR codegen is rerun

@datatype class ProcessorTimingProperties(val clockPeriod: Option[Z],
                                          val framePeriod: Option[Z],
                                          val maxDomain: Option[Z],
                                          val slotTime: Option[Z])

@datatype class ThreadTimingProperties(val domain: Option[Z],
                                       val computeExecutionTime: Option[(Z, Z)])

object Schedulers {

  val threadNickNames: Map[String, Art.BridgeId] = Map(
    ISZ(
      Arch.ProdConsSystem_i_Instance_prod_producerThr.name ~> Arch.ProdConsSystem_i_Instance_prod_producerThr.id,
      Arch.ProdConsSystem_i_Instance_cons_consumerThr.name ~> Arch.ProdConsSystem_i_Instance_cons_consumerThr.id)
  )

  val revThreadNickNames: Map[Art.BridgeId, String] = Map.empty[Art.BridgeId, String] ++ (for (e <- threadNickNames.entries) yield e._2 ~> e._1)


  val ProdConsSystem_i_Instance_prod_producerThr_timingProperties: ThreadTimingProperties = ThreadTimingProperties(
    computeExecutionTime = None(),
    domain = None())

  val ProdConsSystem_i_Instance_cons_consumerThr_timingProperties: ThreadTimingProperties = ThreadTimingProperties(
    computeExecutionTime = None(),
    domain = None())


  /**********************************************************************
   * Round Robin Scheduler
   *********************************************************************/

  // roundRobinSchedule represents the component dispatch order
  val roundRobinSchedule: ISZ[Art.BridgeId] = {
    // convert IS[Art.BridgeId, art.Bridge] to an IS[Z, Art.BridgeId] to allow bridges to be dispatched
    // multiple times during a hyper-period
    var ret: ISZ[Art.BridgeId] = ISZ()
    for (e <- Arch.ad.components) {
      ret = ret :+ e.id
    }
    ret
  }

  def getRoundRobinScheduler(schedule: Option[ISZ[Art.BridgeId]]): RoundRobin = {
    if (roundRobinSchedule.isEmpty) {} // line needed for transpiler; do not remove
    schedule match {
      case Some(s) => return RoundRobin(s)
      case _ => return RoundRobin(ScheduleProviderI.getRoundRobinOrder())
    }
  }

  /**********************************************************************
   * Static Scheduler
   *********************************************************************/

  val framePeriod: Z = 1000
  val numComponents: Z = Arch.ad.components.size
  val maxExecutionTime: Z = numComponents / framePeriod

  // defaultStaticSchedule represents the component dispatch order
  val defaultStaticSchedule: DScheduleSpec = DScheduleSpec(0, 0, DSchedule(ISZ(
    Schedule.Slot(0, maxExecutionTime),
    Schedule.Slot(1, maxExecutionTime)
  )))

  val defaultDomainToBridgeIdMap: ISZ[Art.BridgeId] = ISZ(
    /* domain 0 */ Arch.ProdConsSystem_i_Instance_prod_producerThr.id,
    /* domain 1 */ Arch.ProdConsSystem_i_Instance_cons_consumerThr.id
  )

  def getStaticSchedulerH(userProvided: MOption[(DScheduleSpec, ISZ[Art.BridgeId], Map[String, Art.BridgeId], CommandProvider)]): StaticScheduler = {
    if (defaultStaticSchedule.schedule.slots.isEmpty && defaultDomainToBridgeIdMap.isEmpty && threadNickNames.isEmpty) {} // line needed for transpiler; do not remove
    userProvided match {
      case MSome((schedule_, domainToBridgeIdMap_, threadNickNames_, commandProvider)) =>
        return getStaticScheduler(schedule_, domainToBridgeIdMap_, threadNickNames_, commandProvider)
      case _ =>
        return getStaticScheduler(
          ScheduleProviderI.getStaticSchedule(),
          // TODO: get the following from extension so they can be customized via C
          defaultDomainToBridgeIdMap,
          threadNickNames,
          DefaultCommandProvider())
    }
  }

  def getStaticScheduler(schedule: DScheduleSpec,
                         domainToBridgeIdMap: ISZ[Art.BridgeId],
                         threadNickNames: Map[String, Art.BridgeId],
                         commandProvider: CommandProvider): StaticScheduler = {
    return StaticScheduler(schedule, Arch.ad.components, domainToBridgeIdMap, threadNickNames,
      if (commandProvider.isInstanceOf[InfoCommandProvider])
        commandProvider.asInstanceOf[InfoCommandProvider].init(
          threadNickNames,
          schedule.schedule.slots.size,
          domainToBridgeIdMap
        )
      else commandProvider)
  }


  /**********************************************************************
   * Legacy Scheduler
   *********************************************************************/

  def getLegacyScheduler(): Legacy = {
    return Legacy(Arch.ad.components)
  }
}

// the purpose of this extension is to allow users to provide custom schedules
// at the C level after transpiling
@ext(name = "ScheduleProvider") object ScheduleProviderI {
  def getRoundRobinOrder(): ISZ[Art.BridgeId] = $

  def getStaticSchedule(): DScheduleSpec = $
}