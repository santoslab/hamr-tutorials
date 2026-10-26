# Copyright 2025, UNSW
# SPDX-License-Identifier: BSD-2-Clause
import argparse
import struct
from random import randint
from dataclasses import dataclass
from typing import List, Tuple, Optional
from sdfgen import SystemDescription, Sddf, DeviceTree, LionsOs
from importlib.metadata import version

# This file will not be overwritten if HAMR codegen is rerun

assert int(version('sdfgen').split(".")[1]) >= 30, f"Requires sdfgen >= 0.30, found {version('sdfgen')}"

from sdfgen_helper import *

ProtectionDomain = SystemDescription.ProtectionDomain
MemoryRegion = SystemDescription.MemoryRegion
Map = SystemDescription.Map
Channel = SystemDescription.Channel
IrqConventional = SystemDescription.IrqConventional
VirtualMachine = SystemDescription.VirtualMachine

@dataclass
class Board:
    name: str
    arch: SystemDescription.Arch
    paddr_top: int
    serial: str
    timer: str
    ethernet: str
    i2c: Optional[str]


BOARDS: List[Board] = [
    Board(
        name="qemu_virt_aarch64",
        arch=SystemDescription.Arch.AARCH64,
        paddr_top=0x6_0000_000,
        serial="pl011@9000000",
        timer="timer",
        ethernet="virtio_mmio@a003e00",
        i2c=None,
    ),
]

def schedule(*entries):
    """
    entries: sequence of (channel, timeslice_ns)
    """
    part_ch, part_timeslices, is_user_partition = zip(*entries)
    return UserSchedule(list(part_timeslices), list(part_ch), list(is_user_partition))

def generate(sdf_path: str, output_dir: str, dtb: DeviceTree):
    timer_node = dtb.node(board.timer)
    assert timer_node is not None

    timer_driver = ProtectionDomain("timer_driver", "timer_driver.elf", priority=201)
    timer_system = Sddf.Timer(sdf, timer_node, timer_driver)

    scheduler = ProtectionDomain("scheduler", "scheduler.elf", priority=200)


    # BEGIN META MARKER

    #######################################
    # PARTITION PROTECTION DOMAINS
    #######################################
    temp_sensor_temp_sensor_MON = ProtectionDomain(
      name="temp_sensor_temp_sensor_MON",
      program_image="temp_sensor_temp_sensor_MON.elf",
      priority=150,
      passive=True)
    scheduler.add_child_pd(temp_sensor_temp_sensor_MON)

    temp_sensor_temp_sensor = ProtectionDomain(
      name="temp_sensor_temp_sensor",
      program_image="temp_sensor_temp_sensor.elf",
      priority=140,
      passive=True)
    temp_sensor_temp_sensor_MON.add_child_pd(temp_sensor_temp_sensor, child_id=1)

    operator_interface_operator_interface_MON = ProtectionDomain(
      name="operator_interface_operator_interface_MON",
      program_image="operator_interface_operator_interface_MON.elf",
      priority=150,
      passive=True)
    scheduler.add_child_pd(operator_interface_operator_interface_MON)

    operator_interface_operator_interface = ProtectionDomain(
      name="operator_interface_operator_interface",
      program_image="operator_interface_operator_interface.elf",
      priority=140,
      passive=True)
    operator_interface_operator_interface_MON.add_child_pd(operator_interface_operator_interface, child_id=1)

    thermostat_thermostat_MON = ProtectionDomain(
      name="thermostat_thermostat_MON",
      program_image="thermostat_thermostat_MON.elf",
      priority=150,
      passive=True)
    scheduler.add_child_pd(thermostat_thermostat_MON)

    thermostat_thermostat = ProtectionDomain(
      name="thermostat_thermostat",
      program_image="thermostat_thermostat.elf",
      priority=140,
      passive=True)
    thermostat_thermostat_MON.add_child_pd(thermostat_thermostat, child_id=1)

    heat_source_heat_source_MON = ProtectionDomain(
      name="heat_source_heat_source_MON",
      program_image="heat_source_heat_source_MON.elf",
      priority=150,
      passive=True)
    scheduler.add_child_pd(heat_source_heat_source_MON)

    heat_source_heat_source = ProtectionDomain(
      name="heat_source_heat_source",
      program_image="heat_source_heat_source.elf",
      priority=140,
      passive=True)
    heat_source_heat_source_MON.add_child_pd(heat_source_heat_source, child_id=1)


    #######################################
    # MEMORY REGIONS
    #######################################
    Isolette_System_Instance_temp_sensor_temp_sensor_current_temp_1_Memory_Region = MemoryRegion(sdf, "Isolette_System_Instance_temp_sensor_temp_sensor_current_temp_1_Memory_Region", 0x1_000)
    sdf.add_mr(Isolette_System_Instance_temp_sensor_temp_sensor_current_temp_1_Memory_Region)
    Isolette_System_Instance_temp_sensor_temp_sensor_temp_changed_1_Memory_Region = MemoryRegion(sdf, "Isolette_System_Instance_temp_sensor_temp_sensor_temp_changed_1_Memory_Region", 0x1_000)
    sdf.add_mr(Isolette_System_Instance_temp_sensor_temp_sensor_temp_changed_1_Memory_Region)
    Isolette_System_Instance_operator_interface_operator_interface_desired_temp_1_Memory_Region = MemoryRegion(sdf, "Isolette_System_Instance_operator_interface_operator_interface_desired_temp_1_Memory_Region", 0x1_000)
    sdf.add_mr(Isolette_System_Instance_operator_interface_operator_interface_desired_temp_1_Memory_Region)
    Isolette_System_Instance_thermostat_thermostat_heat_control_1_Memory_Region = MemoryRegion(sdf, "Isolette_System_Instance_thermostat_thermostat_heat_control_1_Memory_Region", 0x1_000)
    sdf.add_mr(Isolette_System_Instance_thermostat_thermostat_heat_control_1_Memory_Region)

    temp_sensor_temp_sensor.add_map(Map(Isolette_System_Instance_temp_sensor_temp_sensor_current_temp_1_Memory_Region, 0x10_000_000, perms="rw", setvar_vaddr="current_temp_queue_1"))
    thermostat_thermostat.add_map(Map(Isolette_System_Instance_temp_sensor_temp_sensor_current_temp_1_Memory_Region, 0x10_000_000, perms="r", setvar_vaddr="current_temp_queue_1"))
    temp_sensor_temp_sensor.add_map(Map(Isolette_System_Instance_temp_sensor_temp_sensor_temp_changed_1_Memory_Region, 0x10_002_000, perms="rw", setvar_vaddr="temp_changed_queue_1"))
    thermostat_thermostat.add_map(Map(Isolette_System_Instance_temp_sensor_temp_sensor_temp_changed_1_Memory_Region, 0x10_002_000, perms="r", setvar_vaddr="temp_changed_queue_1"))
    operator_interface_operator_interface.add_map(Map(Isolette_System_Instance_operator_interface_operator_interface_desired_temp_1_Memory_Region, 0x10_000_000, perms="rw", setvar_vaddr="desired_temp_queue_1"))
    thermostat_thermostat.add_map(Map(Isolette_System_Instance_operator_interface_operator_interface_desired_temp_1_Memory_Region, 0x10_004_000, perms="r", setvar_vaddr="desired_temp_queue_1"))
    thermostat_thermostat.add_map(Map(Isolette_System_Instance_thermostat_thermostat_heat_control_1_Memory_Region, 0x10_006_000, perms="rw", setvar_vaddr="heat_control_queue_1"))
    heat_source_heat_source.add_map(Map(Isolette_System_Instance_thermostat_thermostat_heat_control_1_Memory_Region, 0x10_000_000, perms="r", setvar_vaddr="heat_control_queue_1"))



    #######################################
    # CHANNELS
    #######################################
    channel_temp_sensor_temp_sensor_MON = 3
    channel_operator_interface_operator_interface_MON = 2
    channel_thermostat_thermostat_MON = 4
    channel_heat_source_heat_source_MON = 5

    sdf.add_channel(Channel(a=scheduler, a_id=3, b=temp_sensor_temp_sensor_MON, b_id=0))
    sdf.add_channel(Channel(a=temp_sensor_temp_sensor_MON, a_id=1, b=temp_sensor_temp_sensor, b_id=0))
    sdf.add_channel(Channel(a=scheduler, a_id=2, b=operator_interface_operator_interface_MON, b_id=0))
    sdf.add_channel(Channel(a=operator_interface_operator_interface_MON, a_id=1, b=operator_interface_operator_interface, b_id=0))
    sdf.add_channel(Channel(a=scheduler, a_id=4, b=thermostat_thermostat_MON, b_id=0))
    sdf.add_channel(Channel(a=thermostat_thermostat_MON, a_id=1, b=thermostat_thermostat, b_id=0))
    sdf.add_channel(Channel(a=scheduler, a_id=5, b=heat_source_heat_source_MON, b_id=0))
    sdf.add_channel(Channel(a=heat_source_heat_source_MON, a_id=1, b=heat_source_heat_source, b_id=0))

    #######################################
    # SCHEDULE
    #######################################
    ts_operator_interface_operator_interface_MON = (channel_operator_interface_operator_interface_MON, 50000000, True)
    ts_temp_sensor_temp_sensor_MON = (channel_temp_sensor_temp_sensor_MON, 50000000, True)
    ts_thermostat_thermostat_MON = (channel_thermostat_thermostat_MON, 50000000, True)
    ts_heat_source_heat_source_MON = (channel_heat_source_heat_source_MON, 50000000, True)
    ts_pad = (0, 800000000, False)

    user_schedule = schedule(
      ts_operator_interface_operator_interface_MON,
      ts_temp_sensor_temp_sensor_MON,
      ts_thermostat_thermostat_MON,
      ts_heat_source_heat_source_MON,
      ts_pad
    )

    # END META MARKER

    sdf.add_pd(timer_driver)
    sdf.add_pd(scheduler)
    timer_system.add_client(scheduler)

    assert timer_system.connect()
    assert timer_system.serialise_config(output_dir)

    data_path = output_dir + "/schedule_config.data"
    with open(data_path, "wb+") as f:
        f.write(user_schedule.serialise())
    update_elf_section(obj_copy, scheduler.program_image,
                       user_schedule.section_name,
                       data_path)

    with open(f"{output_dir}/{sdf_path}", "w+") as f:
        f.write(sdf.render())


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument("--dtb", required=True)
    parser.add_argument("--sddf", required=True)
    parser.add_argument("--board", required=True, choices=[b.name for b in BOARDS])
    parser.add_argument("--output", required=True)
    parser.add_argument("--sdf", required=True)
    parser.add_argument("--objcopy", required=True)
    # Substring filter selecting which system tests to run; empty means all of
    # them.  Only the test scheduler variant consumes it.
    parser.add_argument("--tests", required=False, default="")

    args = parser.parse_args()

    # Import the config structs module from the build directory
    sys.path.append(args.output)
    from config_structs import *

    board = next(filter(lambda b: b.name == args.board, BOARDS))

    sdf = SystemDescription(board.arch, board.paddr_top)
    sddf = Sddf(args.sddf)

    global obj_copy
    obj_copy = args.objcopy

    global tests_filter
    tests_filter = args.tests

    with open(args.dtb, "rb") as f:
        dtb = DeviceTree(f.read())

    generate(args.sdf, args.output, dtb)
