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
    gen_gen_MON = ProtectionDomain(
      name="gen_gen_MON",
      program_image="gen_gen_MON.elf",
      priority=150,
      passive=True)
    scheduler.add_child_pd(gen_gen_MON)

    gen_gen = ProtectionDomain(
      name="gen_gen",
      program_image="gen_gen.elf",
      priority=140,
      passive=True)
    gen_gen_MON.add_child_pd(gen_gen, child_id=1)

    splitter_splitter_MON = ProtectionDomain(
      name="splitter_splitter_MON",
      program_image="splitter_splitter_MON.elf",
      priority=150,
      passive=True)
    scheduler.add_child_pd(splitter_splitter_MON)

    splitter_splitter = ProtectionDomain(
      name="splitter_splitter",
      program_image="splitter_splitter.elf",
      priority=140,
      passive=True)
    splitter_splitter_MON.add_child_pd(splitter_splitter, child_id=1)

    incx_incx_MON = ProtectionDomain(
      name="incx_incx_MON",
      program_image="incx_incx_MON.elf",
      priority=150,
      passive=True)
    scheduler.add_child_pd(incx_incx_MON)

    incx_incx = ProtectionDomain(
      name="incx_incx",
      program_image="incx_incx.elf",
      priority=140,
      passive=True)
    incx_incx_MON.add_child_pd(incx_incx, child_id=1)

    clampx_clampx_MON = ProtectionDomain(
      name="clampx_clampx_MON",
      program_image="clampx_clampx_MON.elf",
      priority=150,
      passive=True)
    scheduler.add_child_pd(clampx_clampx_MON)

    clampx_clampx = ProtectionDomain(
      name="clampx_clampx",
      program_image="clampx_clampx.elf",
      priority=140,
      passive=True)
    clampx_clampx_MON.add_child_pd(clampx_clampx, child_id=1)

    decy_decy_MON = ProtectionDomain(
      name="decy_decy_MON",
      program_image="decy_decy_MON.elf",
      priority=150,
      passive=True)
    scheduler.add_child_pd(decy_decy_MON)

    decy_decy = ProtectionDomain(
      name="decy_decy",
      program_image="decy_decy.elf",
      priority=140,
      passive=True)
    decy_decy_MON.add_child_pd(decy_decy, child_id=1)

    merger_merger_MON = ProtectionDomain(
      name="merger_merger_MON",
      program_image="merger_merger_MON.elf",
      priority=150,
      passive=True)
    scheduler.add_child_pd(merger_merger_MON)

    merger_merger = ProtectionDomain(
      name="merger_merger",
      program_image="merger_merger.elf",
      priority=140,
      passive=True)
    merger_merger_MON.add_child_pd(merger_merger, child_id=1)

    consume_consume_MON = ProtectionDomain(
      name="consume_consume_MON",
      program_image="consume_consume_MON.elf",
      priority=150,
      passive=True)
    scheduler.add_child_pd(consume_consume_MON)

    consume_consume = ProtectionDomain(
      name="consume_consume",
      program_image="consume_consume.elf",
      priority=140,
      passive=True)
    consume_consume_MON.add_child_pd(consume_consume, child_id=1)


    #######################################
    # MEMORY REGIONS
    #######################################
    StructSplit_System_Instance_gen_gen_outstruct_1_Memory_Region = MemoryRegion(sdf, "StructSplit_System_Instance_gen_gen_outstruct_1_Memory_Region", 0x1_000)
    sdf.add_mr(StructSplit_System_Instance_gen_gen_outstruct_1_Memory_Region)
    StructSplit_System_Instance_splitter_splitter_xfield_1_Memory_Region = MemoryRegion(sdf, "StructSplit_System_Instance_splitter_splitter_xfield_1_Memory_Region", 0x1_000)
    sdf.add_mr(StructSplit_System_Instance_splitter_splitter_xfield_1_Memory_Region)
    StructSplit_System_Instance_splitter_splitter_yfield_1_Memory_Region = MemoryRegion(sdf, "StructSplit_System_Instance_splitter_splitter_yfield_1_Memory_Region", 0x1_000)
    sdf.add_mr(StructSplit_System_Instance_splitter_splitter_yfield_1_Memory_Region)
    StructSplit_System_Instance_incx_incx_outxfield_1_Memory_Region = MemoryRegion(sdf, "StructSplit_System_Instance_incx_incx_outxfield_1_Memory_Region", 0x1_000)
    sdf.add_mr(StructSplit_System_Instance_incx_incx_outxfield_1_Memory_Region)
    StructSplit_System_Instance_clampx_clampx_outxfield_1_Memory_Region = MemoryRegion(sdf, "StructSplit_System_Instance_clampx_clampx_outxfield_1_Memory_Region", 0x1_000)
    sdf.add_mr(StructSplit_System_Instance_clampx_clampx_outxfield_1_Memory_Region)
    StructSplit_System_Instance_decy_decy_outyfield_1_Memory_Region = MemoryRegion(sdf, "StructSplit_System_Instance_decy_decy_outyfield_1_Memory_Region", 0x1_000)
    sdf.add_mr(StructSplit_System_Instance_decy_decy_outyfield_1_Memory_Region)
    StructSplit_System_Instance_merger_merger_outstruct_1_Memory_Region = MemoryRegion(sdf, "StructSplit_System_Instance_merger_merger_outstruct_1_Memory_Region", 0x1_000)
    sdf.add_mr(StructSplit_System_Instance_merger_merger_outstruct_1_Memory_Region)

    gen_gen.add_map(Map(StructSplit_System_Instance_gen_gen_outstruct_1_Memory_Region, 0x10_000_000, perms="rw", setvar_vaddr="outstruct_queue_1"))
    splitter_splitter.add_map(Map(StructSplit_System_Instance_gen_gen_outstruct_1_Memory_Region, 0x10_000_000, perms="r", setvar_vaddr="instruct_queue_1"))
    splitter_splitter.add_map(Map(StructSplit_System_Instance_splitter_splitter_xfield_1_Memory_Region, 0x10_002_000, perms="rw", setvar_vaddr="xfield_queue_1"))
    incx_incx.add_map(Map(StructSplit_System_Instance_splitter_splitter_xfield_1_Memory_Region, 0x10_000_000, perms="r", setvar_vaddr="inxfield_queue_1"))
    splitter_splitter.add_map(Map(StructSplit_System_Instance_splitter_splitter_yfield_1_Memory_Region, 0x10_004_000, perms="rw", setvar_vaddr="yfield_queue_1"))
    decy_decy.add_map(Map(StructSplit_System_Instance_splitter_splitter_yfield_1_Memory_Region, 0x10_000_000, perms="r", setvar_vaddr="inyfield_queue_1"))
    incx_incx.add_map(Map(StructSplit_System_Instance_incx_incx_outxfield_1_Memory_Region, 0x10_002_000, perms="rw", setvar_vaddr="outxfield_queue_1"))
    clampx_clampx.add_map(Map(StructSplit_System_Instance_incx_incx_outxfield_1_Memory_Region, 0x10_000_000, perms="r", setvar_vaddr="inxfield_queue_1"))
    clampx_clampx.add_map(Map(StructSplit_System_Instance_clampx_clampx_outxfield_1_Memory_Region, 0x10_002_000, perms="rw", setvar_vaddr="outxfield_queue_1"))
    merger_merger.add_map(Map(StructSplit_System_Instance_clampx_clampx_outxfield_1_Memory_Region, 0x10_000_000, perms="r", setvar_vaddr="inxfield_queue_1"))
    decy_decy.add_map(Map(StructSplit_System_Instance_decy_decy_outyfield_1_Memory_Region, 0x10_002_000, perms="rw", setvar_vaddr="outyfield_queue_1"))
    merger_merger.add_map(Map(StructSplit_System_Instance_decy_decy_outyfield_1_Memory_Region, 0x10_002_000, perms="r", setvar_vaddr="inyfield_queue_1"))
    merger_merger.add_map(Map(StructSplit_System_Instance_merger_merger_outstruct_1_Memory_Region, 0x10_004_000, perms="rw", setvar_vaddr="outstruct_queue_1"))
    consume_consume.add_map(Map(StructSplit_System_Instance_merger_merger_outstruct_1_Memory_Region, 0x10_000_000, perms="r", setvar_vaddr="instruct_queue_1"))



    #######################################
    # CHANNELS
    #######################################
    channel_gen_gen_MON = 2
    channel_splitter_splitter_MON = 3
    channel_incx_incx_MON = 4
    channel_clampx_clampx_MON = 6
    channel_decy_decy_MON = 5
    channel_merger_merger_MON = 7
    channel_consume_consume_MON = 8

    sdf.add_channel(Channel(a=scheduler, a_id=2, b=gen_gen_MON, b_id=0))
    sdf.add_channel(Channel(a=gen_gen_MON, a_id=1, b=gen_gen, b_id=0))
    sdf.add_channel(Channel(a=scheduler, a_id=3, b=splitter_splitter_MON, b_id=0))
    sdf.add_channel(Channel(a=splitter_splitter_MON, a_id=1, b=splitter_splitter, b_id=0))
    sdf.add_channel(Channel(a=scheduler, a_id=4, b=incx_incx_MON, b_id=0))
    sdf.add_channel(Channel(a=incx_incx_MON, a_id=1, b=incx_incx, b_id=0))
    sdf.add_channel(Channel(a=scheduler, a_id=6, b=clampx_clampx_MON, b_id=0))
    sdf.add_channel(Channel(a=clampx_clampx_MON, a_id=1, b=clampx_clampx, b_id=0))
    sdf.add_channel(Channel(a=scheduler, a_id=5, b=decy_decy_MON, b_id=0))
    sdf.add_channel(Channel(a=decy_decy_MON, a_id=1, b=decy_decy, b_id=0))
    sdf.add_channel(Channel(a=scheduler, a_id=7, b=merger_merger_MON, b_id=0))
    sdf.add_channel(Channel(a=merger_merger_MON, a_id=1, b=merger_merger, b_id=0))
    sdf.add_channel(Channel(a=scheduler, a_id=8, b=consume_consume_MON, b_id=0))
    sdf.add_channel(Channel(a=consume_consume_MON, a_id=1, b=consume_consume, b_id=0))

    #######################################
    # SCHEDULE
    #######################################
    ts_gen_gen_MON = (channel_gen_gen_MON, 50000000, True)
    ts_splitter_splitter_MON = (channel_splitter_splitter_MON, 50000000, True)
    ts_incx_incx_MON = (channel_incx_incx_MON, 50000000, True)
    ts_decy_decy_MON = (channel_decy_decy_MON, 50000000, True)
    ts_clampx_clampx_MON = (channel_clampx_clampx_MON, 50000000, True)
    ts_merger_merger_MON = (channel_merger_merger_MON, 50000000, True)
    ts_consume_consume_MON = (channel_consume_consume_MON, 50000000, True)
    ts_pad = (0, 650000000, False)

    user_schedule = schedule(
      ts_gen_gen_MON,
      ts_splitter_splitter_MON,
      ts_incx_incx_MON,
      ts_decy_decy_MON,
      ts_clampx_clampx_MON,
      ts_merger_merger_MON,
      ts_consume_consume_MON,
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
