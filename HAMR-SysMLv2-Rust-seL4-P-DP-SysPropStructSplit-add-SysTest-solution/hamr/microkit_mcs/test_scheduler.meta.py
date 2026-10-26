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

    # BEGIN META TEMPLATE MARKER
    #######################################
    # TEST SCHEDULER REGIONS
    # Command input, status output, and the published schedule.  The virtual addresses
    # must match TEST_CMD_VADDR / TEST_STATUS_VADDR / TEST_SCHEDULE_VADDR in
    # test_scheduler.scheduler_config.h.  Change both if there is a conflict.
    #######################################
    TEST_CMD_VADDR      = 0x4_002_000
    TEST_CMD_SIZE       = 0x1000  # 4 KB
    TEST_STATUS_VADDR   = 0x4_003_000
    TEST_STATUS_SIZE    = 0x1000  # 4 KB
    TEST_SCHEDULE_VADDR = 0x4_004_000
    TEST_SCHEDULE_SIZE  = 0x1000  # 4 KB

    test_cmd = MemoryRegion(sdf, "test_cmd", TEST_CMD_SIZE)
    sdf.add_mr(test_cmd)
    scheduler.add_map(Map(test_cmd, TEST_CMD_VADDR, perms="r"))

    test_status = MemoryRegion(sdf, "test_status", TEST_STATUS_SIZE)
    sdf.add_mr(test_status)
    scheduler.add_map(Map(test_status, TEST_STATUS_VADDR, perms="rw"))

    test_schedule = MemoryRegion(sdf, "test_schedule", TEST_SCHEDULE_SIZE)
    sdf.add_mr(test_schedule)
    scheduler.add_map(Map(test_schedule, TEST_SCHEDULE_VADDR, perms="rw"))
    #######################################
    # STATE VARIABLE INJECTION
    # One region per GUMBO state variable, carrying a value from the test
    # controller to the owning thread.  Declared here rather than as AADL
    # ports so they stay out of the component test harness and the system
    # verification model (design D16a).
    #######################################
    inj_consume_consume_sv_last_x = MemoryRegion(sdf, "inj_consume_consume_sv_last_x", 0x1000)
    sdf.add_mr(inj_consume_consume_sv_last_x)
    # END META TEMPLATE MARKER

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

    test_controller_process_test_controller_thread_MON = ProtectionDomain(
      name="test_controller_process_test_controller_thread_MON",
      program_image="test_controller_process_test_controller_thread_MON.elf",
      priority=101,
      passive=True)
    scheduler.add_child_pd(test_controller_process_test_controller_thread_MON)

    test_controller_process_test_controller_thread = ProtectionDomain(
      name="test_controller_process_test_controller_thread",
      program_image="test_controller_process_test_controller_thread.elf",
      priority=100,
      passive=True,
      stack_size=0x10_000)
    test_controller_process_test_controller_thread_MON.add_child_pd(test_controller_process_test_controller_thread, child_id=1)


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
    StructSplit_System_Instance_consume_consume_sv_last_x_1_Memory_Region = MemoryRegion(sdf, "StructSplit_System_Instance_consume_consume_sv_last_x_1_Memory_Region", 0x1_000)
    sdf.add_mr(StructSplit_System_Instance_consume_consume_sv_last_x_1_Memory_Region)

    gen_gen.add_map(Map(StructSplit_System_Instance_gen_gen_outstruct_1_Memory_Region, 0x10_000_000, perms="rw", setvar_vaddr="outstruct_queue_1"))
    splitter_splitter.add_map(Map(StructSplit_System_Instance_gen_gen_outstruct_1_Memory_Region, 0x10_000_000, perms="r", setvar_vaddr="instruct_queue_1"))
    test_controller_process_test_controller_thread.add_map(Map(StructSplit_System_Instance_gen_gen_outstruct_1_Memory_Region, 0x20_000_000, perms="rw"))
    splitter_splitter.add_map(Map(StructSplit_System_Instance_splitter_splitter_xfield_1_Memory_Region, 0x10_002_000, perms="rw", setvar_vaddr="xfield_queue_1"))
    incx_incx.add_map(Map(StructSplit_System_Instance_splitter_splitter_xfield_1_Memory_Region, 0x10_000_000, perms="r", setvar_vaddr="inxfield_queue_1"))
    test_controller_process_test_controller_thread.add_map(Map(StructSplit_System_Instance_splitter_splitter_xfield_1_Memory_Region, 0x20_002_000, perms="rw"))
    splitter_splitter.add_map(Map(StructSplit_System_Instance_splitter_splitter_yfield_1_Memory_Region, 0x10_004_000, perms="rw", setvar_vaddr="yfield_queue_1"))
    decy_decy.add_map(Map(StructSplit_System_Instance_splitter_splitter_yfield_1_Memory_Region, 0x10_000_000, perms="r", setvar_vaddr="inyfield_queue_1"))
    test_controller_process_test_controller_thread.add_map(Map(StructSplit_System_Instance_splitter_splitter_yfield_1_Memory_Region, 0x20_004_000, perms="rw"))
    incx_incx.add_map(Map(StructSplit_System_Instance_incx_incx_outxfield_1_Memory_Region, 0x10_002_000, perms="rw", setvar_vaddr="outxfield_queue_1"))
    clampx_clampx.add_map(Map(StructSplit_System_Instance_incx_incx_outxfield_1_Memory_Region, 0x10_000_000, perms="r", setvar_vaddr="inxfield_queue_1"))
    test_controller_process_test_controller_thread.add_map(Map(StructSplit_System_Instance_incx_incx_outxfield_1_Memory_Region, 0x20_006_000, perms="rw"))
    clampx_clampx.add_map(Map(StructSplit_System_Instance_clampx_clampx_outxfield_1_Memory_Region, 0x10_002_000, perms="rw", setvar_vaddr="outxfield_queue_1"))
    merger_merger.add_map(Map(StructSplit_System_Instance_clampx_clampx_outxfield_1_Memory_Region, 0x10_000_000, perms="r", setvar_vaddr="inxfield_queue_1"))
    test_controller_process_test_controller_thread.add_map(Map(StructSplit_System_Instance_clampx_clampx_outxfield_1_Memory_Region, 0x20_008_000, perms="rw"))
    decy_decy.add_map(Map(StructSplit_System_Instance_decy_decy_outyfield_1_Memory_Region, 0x10_002_000, perms="rw", setvar_vaddr="outyfield_queue_1"))
    merger_merger.add_map(Map(StructSplit_System_Instance_decy_decy_outyfield_1_Memory_Region, 0x10_002_000, perms="r", setvar_vaddr="inyfield_queue_1"))
    test_controller_process_test_controller_thread.add_map(Map(StructSplit_System_Instance_decy_decy_outyfield_1_Memory_Region, 0x20_00A_000, perms="rw"))
    merger_merger.add_map(Map(StructSplit_System_Instance_merger_merger_outstruct_1_Memory_Region, 0x10_004_000, perms="rw", setvar_vaddr="outstruct_queue_1"))
    consume_consume.add_map(Map(StructSplit_System_Instance_merger_merger_outstruct_1_Memory_Region, 0x10_000_000, perms="r", setvar_vaddr="instruct_queue_1"))
    test_controller_process_test_controller_thread.add_map(Map(StructSplit_System_Instance_merger_merger_outstruct_1_Memory_Region, 0x20_00C_000, perms="rw"))
    consume_consume.add_map(Map(StructSplit_System_Instance_consume_consume_sv_last_x_1_Memory_Region, 0x10_002_000, perms="rw", setvar_vaddr="sv_last_x_queue_1"))
    test_controller_process_test_controller_thread.add_map(Map(StructSplit_System_Instance_consume_consume_sv_last_x_1_Memory_Region, 0x20_00E_000, perms="rw"))
    test_controller_process_test_controller_thread.add_map(Map(test_cmd, 0x4_002_000, perms="rw"))
    test_controller_process_test_controller_thread.add_map(Map(test_status, 0x4_003_000, perms="r"))
    test_controller_process_test_controller_thread.add_map(Map(test_schedule, 0x4_004_000, perms="r"))
    consume_consume.add_map(Map(inj_consume_consume_sv_last_x, 0x20_800_000, perms="r", setvar_vaddr="inj_sv_last_x_queue"))
    test_controller_process_test_controller_thread.add_map(Map(inj_consume_consume_sv_last_x, 0x20_400_000, perms="rw"))



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
    channel_test_controller_process_test_controller_thread_MON = 9

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
    sdf.add_channel(Channel(a=scheduler, a_id=9, b=test_controller_process_test_controller_thread_MON, b_id=0))
    sdf.add_channel(Channel(a=test_controller_process_test_controller_thread_MON, a_id=1, b=test_controller_process_test_controller_thread, b_id=0))

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

    # BEGIN META TAIL MARKER
    #######################################
    # TEST SELECTION
    # Which system tests to run, from the TESTS make variable.  Substring match
    # against the qualified suite::test name; empty selects all of them.
    #
    # And which contract checks are live for the whole run (TestScheduler-design.md,
    # D23), from GUMBO_CHECKS and SYSVERIF_CHECKS: empty or "on" leaves a layer on,
    # "off" turns it off -- not tracked, and no scheduler park taken for it.  They
    # go into the flags word after the filter: bit 0 GUMBO off, bit 1 SYSVERIF off.
    # LIST_TESTS=1 sets bit 2: list the selected tests without running them (D17).
    # PROPTEST_SEED, 64 hex digits, seeds the property tests; empty keeps the
    # built-in default.
    #######################################
    test_selection = bytearray(292)
    try:
        _filter = tests_filter.encode()
    except UnicodeEncodeError:
        raise SystemExit("TESTS must be valid text (UTF-8)")
    # truncating would change which tests run, or split a character, silently
    if len(_filter) > 255:
        raise SystemExit(f"TESTS must be at most 255 bytes (UTF-8), not {len(_filter)}")
    test_selection[0:len(_filter)] = _filter
    import os
    _flags = 0
    for _name, _bit, _generated in [("GUMBO_CHECKS", 0x1, True),
                                    ("SYSVERIF_CHECKS", 0x2, True)]:
        _value = os.environ.get(_name, "")
        if _value not in ("", "on", "off"):
            raise SystemExit(f"{_name} must be 'on' or 'off', not '{_value}'")
        if _value != "" and not _generated:
            print(f"warning: {_name}={_value} has no effect: this model has no such checks")
        if _value == "off":
            _flags |= _bit
    # LIST_TESTS=1: print the test table and run nothing (bit 2).
    _list = os.environ.get("LIST_TESTS", "")
    if _list not in ("", "0", "1"):
        raise SystemExit(f"LIST_TESTS must be '0' or '1', not '{_list}'")
    if _list == "1":
        _flags |= 0x4
    test_selection[256:260] = _flags.to_bytes(4, "little")
    # PROPTEST_SEED: the property tests' seed (bytes 260-291; all zero keeps the default).
    _seed = os.environ.get("PROPTEST_SEED", "")
    if _seed != "":
        try:
            _seed_bytes = bytes.fromhex(_seed)
        except ValueError:
            _seed_bytes = b""
        if len(_seed_bytes) != 32:
            raise SystemExit(f"PROPTEST_SEED must be 64 hex digits, not '{_seed}'")
        test_selection[260:292] = _seed_bytes
    test_selection_path = output_dir + "/test_selection.data"
    with open(test_selection_path, "wb+") as f:
        f.write(bytes(test_selection))
    update_elf_section(obj_copy, test_controller_process_test_controller_thread.program_image,
                       "test_selection",
                       test_selection_path)
    # END META TAIL MARKER

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
