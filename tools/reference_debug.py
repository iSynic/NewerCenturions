"""Inspect the owned reference process and apply a recorded presentation workaround."""
import argparse
import ctypes
from ctypes import wintypes
import hashlib
import json
from pathlib import Path
import struct
import reference


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--skip-text-scroll", action="store_true")
    parser.add_argument("--skip-story-display", action="store_true")
    parser.add_argument("--seed", type=lambda value: int(value, 0), help="set a recorded controlled QuickDraw RNG input")
    parser.add_argument("--clock", help="controlled hour:minute input for live time-boundary cases")
    parser.add_argument("--day", type=int)
    parser.add_argument("--position", help="controlled first-map x:y input")
    parser.add_argument("--viewport", help="controlled exploration viewport x:y; requires --position")
    parser.add_argument("--stamina", action="append", default=[], help="controlled warrior:stamina input")
    parser.add_argument("--loaded", help="controlled warrior:loaded-rounds input for the equipped starter firearm")
    parser.add_argument("--condition", action="append", default=[], help="controlled warrior:hex-offset:duration input (0x5e or 0x66)")
    parser.add_argument("--receipt", type=Path)
    args = parser.parse_args()
    if args.seed is not None and not 1 <= args.seed < 2147483647:
        parser.error("controlled RNG seed must be between 1 and 2147483646")
    pid = int(Path("reference/basilisk.pid").read_text().strip())
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.OpenProcess.restype = wintypes.HANDLE
    kernel.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
    kernel.ReadProcessMemory.argtypes = [wintypes.HANDLE, ctypes.c_void_p, ctypes.c_void_p, ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t)]
    kernel.WriteProcessMemory.argtypes = kernel.ReadProcessMemory.argtypes
    kernel.QueryFullProcessImageNameW.argtypes = [wintypes.HANDLE, wintypes.DWORD, wintypes.LPWSTR, ctypes.POINTER(wintypes.DWORD)]
    kernel.CloseHandle.argtypes = [wintypes.HANDLE]
    kernel.CreateToolhelp32Snapshot.argtypes = [wintypes.DWORD,wintypes.DWORD]
    kernel.CreateToolhelp32Snapshot.restype = wintypes.HANDLE
    class Module(ctypes.Structure):
        _fields_ = [("size",wintypes.DWORD),("module_id",wintypes.DWORD),("process_id",wintypes.DWORD),
            ("global_count",wintypes.DWORD),("process_count",wintypes.DWORD),("base",ctypes.c_void_p),
            ("base_size",wintypes.DWORD),("module",wintypes.HMODULE),("name",wintypes.WCHAR*256),
            ("path",wintypes.WCHAR*260)]
    kernel.Module32FirstW.argtypes = [wintypes.HANDLE,ctypes.POINTER(Module)]
    kernel.Module32NextW.argtypes = kernel.Module32FirstW.argtypes
    controlled = args.clock or args.day is not None or args.position or args.stamina or args.condition or args.loaded
    access = 0x1000 | 0x10 | (0x20 | 0x8 if args.skip_text_scroll or args.skip_story_display or args.seed is not None or controlled else 0)
    handle = kernel.OpenProcess(access, False, pid)
    if not handle:
        raise ctypes.WinError(ctypes.get_last_error())
    try:
        image = ctypes.create_unicode_buffer(32768)
        size = wintypes.DWORD(len(image))
        if not kernel.QueryFullProcessImageNameW(handle, 0, image, ctypes.byref(size)):
            raise ctypes.WinError(ctypes.get_last_error())
        expected = Path("reference/bin/BasiliskII.exe").resolve()
        if Path(image.value).resolve() != expected:
            raise RuntimeError("PID does not belong to the isolated reference emulator")
        emulator_hash = hashlib.sha256(expected.read_bytes()).hexdigest()
        if emulator_hash != "f553e60aef1ce1af960abc3dcb9622275700c8e4dc013d6cad41e7341c36519a":
            raise RuntimeError("reference debug RVAs require the recorded Basilisk build")

        def read(address, count):
            buffer = ctypes.create_string_buffer(count)
            actual = ctypes.c_size_t()
            if not kernel.ReadProcessMemory(handle, address, buffer, count, ctypes.byref(actual)) or actual.value != count:
                raise ctypes.WinError(ctypes.get_last_error())
            return buffer.raw

        snapshot = kernel.CreateToolhelp32Snapshot(0x18,pid)
        module = Module()
        module.size = ctypes.sizeof(module)
        module_base = None
        try:
            valid = kernel.Module32FirstW(snapshot,ctypes.byref(module))
            while valid:
                if Path(module.path).resolve() == expected:
                    module_base = module.base
                    break
                valid = kernel.Module32NextW(snapshot,ctypes.byref(module))
        finally:
            kernel.CloseHandle(snapshot)
        if module_base is None or read(module_base,2) != b"MZ":
            raise RuntimeError("reference executable module was not found")
        # Symbol RVAs from the supplied 32-bit Basilisk build.
        ram_host = struct.unpack("<I", read(module_base+0x3969a8, 4))[0]
        a5 = int(reference.read_region(0x904, 4)["hex"], 16)
        code = int(reference.read_region(a5-0x1de2, 4)["hex"], 16)
        registers = struct.unpack("<19I", read(module_base+0x3d7620, 76))
        receipt = dict(pid=pid, a5=a5, code1=code, pc=registers[17]-ram_host,
                       registers=list(registers[:16]), emulator_sha256=emulator_hash)
        if controlled:
            writes=[]
            def field(offset, value, size, label):
                address=ram_host+a5+offset
                replacement=value.to_bytes(size,"big",signed=True)
                before=read(address,size)
                actual=ctypes.c_size_t()
                if not kernel.WriteProcessMemory(handle,address,ctypes.create_string_buffer(replacement),size,ctypes.byref(actual)) or actual.value!=size:
                    raise ctypes.WinError(ctypes.get_last_error())
                if read(address,size)!=replacement:
                    raise RuntimeError("controlled state input readback mismatch")
                writes.append(dict(field=label,a5_offset=hex(offset),before=before.hex(),after=replacement.hex()))
            if args.clock:
                hour,minute=map(int,args.clock.split(":"))
                if not 0<=hour<24 or not 0<=minute<60: raise ValueError("invalid clock input")
                field(0x427ca,hour,2,"hour");field(0x427c8,minute,2,"minute")
            if args.day is not None:
                if not 1<=args.day<=32767: raise ValueError("invalid day input")
                field(0x427d4,args.day,2,"day")
            if args.position:
                x,y=map(int,args.position.split(":"))
                if not 0<=x<90 or not 0<=y<90 or int(reference.read_region(a5+0x42774,4)["hex"],16)!=0:
                    raise ValueError("position input requires valid first-map coordinates")
                origin_x=int.from_bytes(read(ram_host+a5+0x42778,4),"big",signed=True)
                origin_y=int.from_bytes(read(ram_host+a5+0x4277c,4),"big",signed=True)
                if args.viewport:
                    origin_x,origin_y=map(int,args.viewport.split(":"))
                    if not 0<=origin_x<=75 or not 0<=origin_y<=77: raise ValueError("invalid viewport input")
                    field(0x42778,origin_x,4,"viewport x");field(0x4277c,origin_y,4,"viewport y")
                field(0x42764,x-origin_x,4,"relative x");field(0x42768,y-origin_y,4,"relative y")
            for value in args.stamina:
                warrior,hp=map(int,value.split(":"))
                if not 0<=warrior<6: raise ValueError("invalid warrior input")
                maximum=int.from_bytes(read(ram_host+a5+0x48aa0+warrior*3138+0x52,2),"big",signed=True)
                if not -10<=hp<=maximum: raise ValueError("invalid stamina input")
                field(0x48aa0+warrior*3138+0x50,hp,2,f"warrior{warrior} stamina")
            for value in args.condition:
                warrior,offset,duration=value.split(":");warrior=int(warrior);offset=int(offset,0);duration=int(duration)
                if not 0<=warrior<6 or offset not in (0x5e,0x66) or not -1<=duration<=10:
                    raise ValueError("invalid condition input")
                field(0x48aa0+warrior*3138+offset,duration,2,f"warrior{warrior} condition")
            if args.loaded:
                warrior,rounds=map(int,args.loaded.split(":"))
                if not 0<=warrior<6: raise ValueError("invalid warrior input")
                offset=0x48aa0+warrior*3138+0x132
                item=read(ram_host+a5+offset,22)
                if item[:2]!=b"\x00\x01" or item[4:6]!=b"\x00\xc8" or item[9]!=1:
                    raise ValueError("loaded input requires the equipped starter firearm in slot zero")
                if not 0<=rounds<=int.from_bytes(item[6:8],"big",signed=True): raise ValueError("invalid loaded rounds")
                field(offset+2,rounds,2,f"warrior{warrior} loaded rounds")
            receipt["controlled_state_inputs"]=dict(writes=writes,scope="isolated reference-copy RAM input only; gameplay instructions unchanged",reason="repeatable live time and authored supply cases")
        if args.seed is not None:
            quickdraw = int(reference.read_region(a5, 4)["hex"], 16)
            address = ram_host+quickdraw-0x7e
            before = struct.unpack(">I", read(address, 4))[0]
            replacement = struct.pack(">I", args.seed)
            actual = ctypes.c_size_t()
            if not kernel.WriteProcessMemory(handle, address, ctypes.create_string_buffer(replacement), 4, ctypes.byref(actual)) or actual.value != 4:
                raise ctypes.WinError(ctypes.get_last_error())
            if read(address, 4) != replacement:
                raise RuntimeError("controlled RNG input readback mismatch")
            receipt["controlled_input"] = dict(guest_address=quickdraw-0x7e, original_seed=before, seed=args.seed,
                scope="isolated reference-copy QuickDraw RNG state only", reason="repeatable live encounter comparison")
            Path("analysis/native/reference-input.json").write_text(json.dumps(receipt, indent=2)+"\n")
        if args.skip_text_scroll or args.skip_story_display:
            offset = 0x3ebde if args.skip_story_display else 0x3edb0
            original = bytes.fromhex("4e56ffee" if args.skip_story_display else "a9dd")
            replacement = bytes.fromhex("70004e75" if args.skip_story_display else "508f")
            address = ram_host+code+offset
            before = read(address, len(original))
            if before != original:
                raise RuntimeError(f"TEScroll callsite mismatch: {before.hex()}")
            actual = ctypes.c_size_t()
            buffer = ctypes.create_string_buffer(replacement)
            if not kernel.WriteProcessMemory(handle, address, buffer, len(replacement), ctypes.byref(actual)) or actual.value != len(replacement):
                raise ctypes.WinError(ctypes.get_last_error())
            if read(address, len(replacement)) != replacement:
                raise RuntimeError("reference workaround readback mismatch")
            receipt["patch"] = dict(code1_offset=hex(offset), original=before.hex(), replacement=replacement.hex(),
                scope="loaded reference-copy code only", reason="TEScroll and offscreen EraseRect stall while displaying opening TEXT 1000",
                limitation="modified presentation path; subsequent observations are not unmodified reference proof")
            Path("analysis/native/reference-workaround.json").write_text(json.dumps(receipt, indent=2)+"\n")
        if controlled:
            Path("analysis/native/reference-state-input.json").write_text(json.dumps(receipt,indent=2)+"\n")
        if args.receipt:
            args.receipt.parent.mkdir(parents=True,exist_ok=True)
            args.receipt.write_text(json.dumps(receipt,indent=2)+"\n")
        print(json.dumps(receipt, indent=2))
    finally:
        kernel.CloseHandle(handle)


if __name__ == "__main__":
    main()
