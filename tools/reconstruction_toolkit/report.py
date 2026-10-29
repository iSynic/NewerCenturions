"""Self-contained, offline report viewer; captured text is always escaped."""
import html
import json


def write_report(path, title, report):
    if path.exists():
        raise ValueError(f'refusing to overwrite {path}')
    payload = json.dumps(report, ensure_ascii=True).replace('<', '\\u003c').replace('&', '\\u0026')
    page = '''<!doctype html><html lang="en"><meta charset="utf-8">
<meta name="viewport" content="width=device-width"><title>TITLE</title>
<style>body{font:15px system-ui;margin:2rem auto;max-width:1100px;padding:0 1rem;background:#101820;color:#e9eff4}
h1{font-size:1.8rem}input,select,button{font:inherit;padding:.6rem;background:#20313d;color:inherit;border:1px solid #587180;border-radius:4px}
nav{display:flex;gap:1rem;flex-wrap:wrap;margin:1rem 0}article{border:1px solid #405665;border-radius:6px;margin:.6rem 0;padding:1rem}
pre{white-space:pre-wrap;overflow-wrap:anywhere}summary{cursor:pointer}.muted{color:#acc3d2}.bad{color:#ffb6a8}.good{color:#9ee3b3}</style>
<h1>TITLE</h1><p id="status"></p><p id="scope" class="muted"></p><nav>
<input id="filter" placeholder="Filter paths, fields, addresses" aria-label="Filter report">
<select id="section" aria-label="Section"><option value="">All sections</option></select>
<button id="download">Download JSON</button></nav><main id="rows"></main>
<details><summary>Full receipt and provenance</summary><pre id="receipt"></pre></details>
<script type="application/json" id="data">PAYLOAD</script><script>
const d=JSON.parse(document.getElementById('data').textContent), rows=d.differences||d.rows||[];
const text=v=>typeof v==='string'?v:JSON.stringify(v,null,2);
const status=document.getElementById('status');status.textContent=d.evidence_usable===false?'Native evidence failed or was truncated':d.equal===true?'Recorded comparison matches':d.equal===false?`${d.difference_count} differences`:d.passed===false?'Probe failures captured':`${rows.length} report entries`;
status.className=d.equal===false||d.passed===false||d.evidence_usable===false?'bad':'good';document.getElementById('scope').textContent=d.qualification||'';
document.getElementById('receipt').textContent=JSON.stringify(d,null,2);
const select=document.getElementById('section');[...new Set(rows.map(r=>r.section||r.kind||'entry'))].sort().forEach(s=>{const o=document.createElement('option');o.value=s;o.textContent=s;select.append(o)});
function show(){const q=document.getElementById('filter').value.toLowerCase(),root=document.getElementById('rows');root.replaceChildren();
rows.filter(r=>(!select.value||(r.section||r.kind||'entry')===select.value)&&text(r).toLowerCase().includes(q)).forEach((r,i)=>{
const a=document.createElement('article'),h=document.createElement('strong'),p=document.createElement('pre');h.textContent=r.path||r.title||r.id||`Entry ${i+1}`;p.textContent=text(r);a.append(h,p);root.append(a)})}
document.getElementById('filter').addEventListener('input',show);select.addEventListener('change',show);show();
document.getElementById('download').onclick=()=>{const u=URL.createObjectURL(new Blob([JSON.stringify(d,null,2)],{type:'application/json'})),a=document.createElement('a');a.href=u;a.download='report.json';a.click();URL.revokeObjectURL(u)};
</script></html>'''
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(page.replace('TITLE', html.escape(title)).replace('PAYLOAD', payload), encoding='utf-8')
