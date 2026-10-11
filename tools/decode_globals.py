"""Recover startup DATA initialization and relocation from CODE 1 instructions."""
import json
from pathlib import Path
import struct

from extract import span


def s32(value):
    value &= 0xffffffff
    return value - 0x100000000 if value & 0x80000000 else value


def initialize(data, above, below):
    image = bytearray(above + below)
    cursor = 4
    def take(count):
        nonlocal cursor
        value = span(data, cursor, count)
        cursor += count
        return value
    regions = []
    for _ in range(3):
        offset = struct.unpack('>i', take(4))[0]
        position = below + offset
        start = position
        def write(value):
            nonlocal position
            span(image, position, len(value))
            image[position:position+len(value)] = value
            position += len(value)
        while True:
            opcode = take(1)[0]
            if opcode & 0x80:
                write(take((opcode & 0x7f) + 1))
            elif opcode & 0x40:
                position += (opcode & 0x3f) + 1
            elif opcode & 0x20:
                write(take(1) * ((opcode & 0x1f) + 2))
            elif opcode & 0x10:
                write(b'\xff' * ((opcode & 0xf) + 1))
            elif opcode == 0:
                break
            elif opcode in (1, 2):
                position += 4
                write(b'\xff' * (3-opcode))
                write(take(opcode+1))
            elif opcode == 3:
                write(b'\xa9\xf0')
                position += 2
                write(take(2))
                position += 1
                write(take(1))
            elif opcode == 4:
                write(b'\xa9\xf0')
                position += 1
                write(take(3))
                position += 1
                write(take(1))
            else:
                raise ValueError(f'unknown initializer opcode {opcode}')
            span(image, position, 0)
        regions.append(dict(offset=offset, initialized_span=position-start))
    return image, cursor, regions


def relocations(data, cursor):
    count = struct.unpack('>I', span(data, cursor, 4))[0]
    cursor += 4
    offsets = []
    offset = 0
    for _ in range(count):
        first = span(data, cursor, 1)[0]
        if first & 0x80:
            delta = (first << 1) & 0xff
            offset += delta - 256 if delta & 128 else delta
            cursor += 1
        elif first & 0x40:
            value = struct.unpack('>H', span(data,cursor,2))[0]
            value = (value << 2) & 0xffff
            value = value - 65536 if value & 32768 else value
            offset += value >> 1
            cursor += 2
        else:
            offset = s32(struct.unpack('>I',span(data,cursor,4))[0] << 2) >> 1
            cursor += 4
        offsets.append(offset)
    return offsets, cursor


def main():
    root = Path('analysis/extracted/resources/106')
    header = (root/'434f4445_0.bin').read_bytes()
    above,below = struct.unpack_from('>II',header)
    data = (root/'44415441_0.bin').read_bytes()
    image,cursor,regions = initialize(data,above,below)
    lists = []
    for _ in range(6):
        offsets,cursor = relocations(data,cursor)
        lists.append(offsets)
    output = Path('analysis/ghidra/startup')
    output.mkdir(parents=True,exist_ok=True)
    (output/'a5-initial.bin').write_bytes(image)
    (output/'relocations.json').write_text(json.dumps(dict(above_a5=above,below_a5=below,regions=regions,
        consumed=cursor,data_size=len(data),lists=lists),indent=2)+'\n')
    code = bytearray((root/'434f4445_1.bin').read_bytes())
    a5_base = 0x400000
    code_base = 0x100000
    for target, bias, offsets in ((image,below,lists[0]),(image,below,lists[1]),
                                  (image,below,lists[2]),(code,0,lists[3]),(code,0,lists[4]),(code,0,lists[5])):
        index = (0 if target is image else 3)
        base = a5_base if offsets is lists[index] else code_base
        for offset in offsets:
            pos = bias+offset
            value = struct.unpack('>I',span(target,pos,4))[0]
            struct.pack_into('>I',target,pos,(value+base)&0xffffffff)
    thunks = []
    for offset in range(0x28,0x400,8):
        opcode,address,segment = struct.unpack('>HIH',span(image,below+offset,8))
        if opcode != 0xa9f0:
            break
        target = segment*0x100000+address
        thunks.append(dict(offset=offset,segment=segment,resource_offset=address,analysis_target=target))
        struct.pack_into('>HIH',image,below+offset,0x4ef9,target,0x4e71)
    (output/'thunks.json').write_text(json.dumps(thunks,indent=2)+'\n')
    (output/'a5-mapped.bin').write_bytes(image)
    (output/'code1-mapped.bin').write_bytes(code)
    print(f'Initialized {len(image)} bytes in three regions; relocation counts {[len(x) for x in lists]}; consumed {cursor}/{len(data)}')
    print('A5+0x20 jump stubs:',image[below+0x20:below+0x90].hex())


if __name__ == '__main__':
    main()
