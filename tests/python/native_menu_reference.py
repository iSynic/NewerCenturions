"""Verify the finite original menu fixture against original forks and native contracts."""
import copy
import hashlib
import json
import struct
from collections import Counter
from pathlib import Path


def verify_menu_boundary(data=None):
    ROOT = Path(__file__).resolve().parents[2]
    PATH = ROOT / 'evidence/fixtures/menu-boundary.json'
    if data is None:
        data = json.loads(PATH.read_text())
    resources = json.loads((PATH.parent / 'menu-resources.json').read_text())
    for key, path in {'probe_sha256':'tools/ghidra/VerifyMenuBoundary.java', 'harness_sha256':'tools/ghidra/ActionOracleHarness.java', 'music_boundary_sha256':'tools/ghidra/SubmitMusicBoundary.java'}.items():
        assert hashlib.sha256((ROOT / path).read_bytes()).hexdigest() == data['cache_identity'][key]
    assert data['cache_identity']['ghidra'] == '12.1.2'
    for resource in resources['menus'] + resources['bars']:
        fork = (ROOT / resource['fork_path']).read_bytes()
        assert hashlib.sha256(fork).hexdigest() == resource['fork_sha256']
        assert struct.unpack_from('>II',fork) == (0x00051607,0x00020000)
        entries = [struct.unpack_from('>III',fork,26+n*12) for n in range(struct.unpack_from('>H',fork,24)[0])]
        _, offset, length = next(entry for entry in entries if entry[0] == 2)
        assert offset == resource['resource_fork_entry_offset'] == 82
        assert resource['physical_file_offset'] == offset + resource['fork_offset']
        raw_begin = resource['physical_file_offset']
        if 'menu_resource_ids' in resource:
            count = struct.unpack_from('>H',fork,raw_begin)[0]
            raw = fork[raw_begin:raw_begin+2+count*2]
            assert list(struct.unpack_from('>'+ 'h'*count,raw,2)) == resource['menu_resource_ids']
        else:
            header = struct.unpack_from('>hhhhHI',fork,raw_begin)
            assert header == tuple(resource[k] for k in ['menu_id','width','height','proc_id','reserved','enable_flags'])
            p=raw_begin+14
            n=fork[p];p+=1
            assert fork[p:p+n].hex()==resource['title_hex'] and fork[p:p+n].decode('mac_roman')==resource['title'];p+=n
            items=[]
            while True:
                n=fork[p];p+=1
                if not n:break
                text_bytes=fork[p:p+n];p+=n
                icon,key,mark,style=fork[p:p+4];p+=4;ordinal=len(items)+1
                items.append({'ordinal':ordinal,'text_hex':text_bytes.hex(),'text':text_bytes.decode('mac_roman'),'icon':icon,'key':key,'mark':mark,'style':style,'resource_enabled':bool(header[-1] & (1<<ordinal)),'submenu_id':mark if key==27 else None})
            assert items == resource['items']
            raw=fork[raw_begin:p]
        assert hashlib.sha256(raw).hexdigest() == resource['resource_sha256']
        assert resource['fork_offset'] + len(raw) <= length
    # Exact source Pascal captions pinned by the source/native agreement.
    def text(address):
        return {0x40282a:b'Activate Warrior Editor',0x40280c:b'Activate Warrior Editor DEMO',0x40355e:b'Start Music',0x40351a:b'Stop Music',0x403526:b"Music Stoped, Select 'Start Music' To Continue Playing"}[address].hex()
    def handle(menu_id):
        return 0x900000 + menu_id * 256
    def expected_menu(resource_id):
        r = next(row for row in resources['menus'] if row['resource_id'] == resource_id)
        return {key:copy.deepcopy(r[key]) for key in ['resource_id', 'menu_id', 'title_hex', 'enable_flags']} | {
            'items': [{key:item[key] for key in ['text_hex', 'icon', 'key', 'mark', 'style']} for item in r['items']]}

    assert [c['case'] for c in data['cases']] == ['install_demo','install_registered','draw_bar','speed_1','speed_6','speed_11','playlist_on','playlist_off','music_stop','music_resume_busy','command_key_load']
    service_total = 0
    for case in data['cases']:
        name = case['case']
        before, after = case['before'], case['after']
        expect = copy.deepcopy(before)
        assert len(before) == len(after) == 28
        if name.startswith('install_'):
            expect['menus']['133'] = expected_menu(133)
            pointers = {0x4427be:0xa00000+129*256,0x4427ba:0xa00000+128*256,
                        0x4386fe:handle(133),0x43872a:handle(200),0x438706:handle(128),
                        0x43870a:handle(129),0x43870e:handle(130),0x438712:0,
                        0x43871e:handle(136),0x438722:handle(132),0x438702:handle(137),
                        0x4386a4:handle(146),0x4386a0:handle(145),0x43871a:handle(135),0x438716:handle(134)}
            for address, value in pointers.items():
                expect[format(address,'x')] = value
            for resource_id in [128,129,130,136,132,146,200,137,145,134,135]:
                expect['menus'][str(resource_id)] = expected_menu(resource_id)
            expect['menus']['136']['items'][7]['text_hex'] = text(0x40282a if case['editor_control'] else 0x40280c)
            placeholder = {'text_hex':'2d'*12,'icon':0,'key':0,'mark':0,'style':0}
            expect['menus']['200']['items'].extend(copy.deepcopy(placeholder) for _ in range(19))
            for item in range(4,23):
                expect['menus']['200']['enable_flags'] &= ~(1 << item)
            expect.update(sp=0x6ffcf0,a6=0x6ffdfc)
            counts = Counter(e.get('trap',e.get('service')) for e in case['services'])
            assert counts == {'a933':19,'a93a':19,'a949':11,'a9c0':2,'a93c':2,'a9bf':2,'a935':2,'literal sprintf platform copy':1,'a947':1}
            assert [e['item'] for e in case['services'] if e.get('trap') == 'a93a'] == list(range(4,23))
            assert [e['menu_id'] for e in case['services'] if e.get('trap') == 'a949'] == [133,200,128,129,130,139,136,132,137,146,145]
        elif name == 'draw_bar':
            expect.update(sp=0x6ffdfc,a6=0x6ffdfc)
            assert [e['trap'] for e in case['services']] == ['a93c','a935','a935','a937']
        else:
            menu_id, item = divmod(case['selected_menu_item'],65536)
            expect.update({'438746':menu_id,'438748':item})
            if name == 'command_key_load':
                assert (menu_id,item) == (129,1) and before['447218'] == 256
                expect.update(character_key_byte=108,sp=0x6ffdea,a6=0x6ffdf8)
                assert case['stop'] == '12f0cc'
                assert [e['trap'] for e in case['services']] == ['a858','a858','a93e','a938']
                assert next(e for e in case['services'] if e['trap']=='a93e')['selection'] == (129<<16)|1
            else:
                expect['sp'] += 4
                assert case['stop'] == '910000' and after['a6'] == before['a6']
            if name.startswith('speed_'):
                expect['menus']['134']['items'][before['43956a']-1]['mark'] = 0
                expect['menus']['134']['items'][item-1]['mark'] = 19
                expect.update({'43956a':item,'439568':5*item-5})
                assert [e.get('trap',e.get('service')) for e in case['services']] == ['a938','a944','a944','savepref request only']
                save = case['services'][-1]
                assert save['address'] == '14c01c' and save['return_address'] == 0x127f96
                assert save['state']['43956a'] == item and save['state']['439568'] == 5*item-5
            if name.startswith('playlist_'):
                mask = bytearray.fromhex(before['playlist13']); mask[2] = 0 if mask[2] else 1
                expect['playlist13'] = mask.hex();expect['menus']['145']['items'][11]['mark'] = 19 if mask[2] else 0
                assert before['447218'] == 512
                assert [e['trap'] for e in case['services']] == ['a938','a860','a858','a858','a944']
            if name.startswith('music_'):
                disabled = 0 if before['439594'] else 1;expect['439594'] = disabled
                expect['menus']['145']['items'][0]['text_hex'] = text(0x40355e if disabled else 0x40351a)
                if disabled:
                    expect['menus']['145']['items'][3]['text_hex'] = text(0x403526)
                assert len(case['music_native']) == 1
                music = case['music_native'][0]
                assert music['entry'] == '133d46' and music['argument'] == 0 and music['disabled_word'] == disabled
                assert music['return_sp'] == music['entry_sp'] + 4 and music['return_address'] == 0x127d14
                assert music['hardware_busy'] == 1 and music['d0_return_word'] == 0
                calls = music['services']
                assert [e['service'] for e in calls] == (['SndDoImmediate']*2 if disabled else ['SndChannelStatus'])
                if disabled:
                    assert [e['command8'] for e in calls] == ['0004000000000000','0003000000000000']
                else:
                    assert calls[0]['status24'] == '000000000000000000000000010000000000000000000000'
                for call in calls:
                    assert call['result_word'] == 0
                    assert call['sp_after'] - call['sp_before'] == (8 if disabled else 10)
            else:
                assert case['music_native'] == []
        assert expect == after, (name,[key for key in expect if expect[key]!=after[key]])
        for service in case['services']:
            if 'trap' not in service:
                continue
            pop = {'a9c0':2,'a93c':4,'a949':2,'a9bf':2,'a935':6,'a947':10,'a933':8,'a93a':6,'a939':6,'a944':8,'a938':2,'a937':0,'a860':14,'a858':8,'a93e':2}[service['trap']]
            assert service['sp_after'] - service['sp_before'] == pop
            if service['trap']=='a858':
                assert service['result'] == service['left'] & service['right']
            service_total += 1
    return {'cases':11,'fields':28,'toolbox_replies':service_total,'music_replies':3}


if __name__ == "__main__":
    print("PASS menu boundary:", verify_menu_boundary())
