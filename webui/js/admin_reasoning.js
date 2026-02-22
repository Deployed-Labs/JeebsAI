async function fetchTraces() {
  const res = await fetch('/api/admin/reasoning_traces', { credentials: 'same-origin' });
  if (!res.ok) { alert('Failed to load traces'); return []; }
  return res.json();
}

function render(traces){
  const tbody = document.querySelector('#traces-table tbody'); tbody.innerHTML='';
  for(const t of traces){
    const tr = document.createElement('tr');
    tr.innerHTML = `<td>${t.id}</td><td>${t.timestamp}</td><td>${t.username||''}</td><td>${escapeHtml(t.prompt)}</td><td>${escapeHtml(t.response)}</td>`;
    tbody.appendChild(tr);
  }
}

function escapeHtml(s){ return s? String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;') : '' }

document.getElementById('refresh').addEventListener('click', async ()=>{ const t = await fetchTraces(); render(t); });
document.getElementById('export').addEventListener('click', async ()=>{
  const t = await fetchTraces(); const lines = ['id,timestamp,username,prompt,response'];
  for(const r of t){ lines.push([r.id, csvSafe(r.timestamp), csvSafe(r.username||''), csvSafe(r.prompt), csvSafe(r.response)].join(',')); }
  const blob = new Blob([lines.join('\n')],{type:'text/csv'}); const url = URL.createObjectURL(blob); const a=document.createElement('a'); a.href=url; a.download='reasoning_traces.csv'; document.body.appendChild(a); a.click(); a.remove();
});

function csvSafe(s){ if(s==null) return ''; const str=String(s).replace(/"/g,'""'); if(str.includes(',')||str.includes('\n')) return '"'+str+'"'; return str; }

(async ()=>{ const t = await fetchTraces(); render(t); })();
