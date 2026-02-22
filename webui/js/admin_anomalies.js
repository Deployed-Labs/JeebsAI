async function fetchAnomalies() {
  const res = await fetch('/api/admin/anomalies', { credentials: 'same-origin' });
  if (!res.ok) {
    alert('Failed to load anomalies. Are you an admin?');
    return [];
  }
  return res.json();
}

function render(anoms) {
  const tbody = document.querySelector('#anomalies-table tbody');
  tbody.innerHTML = '';
  for (const a of anoms) {
    const tr = document.createElement('tr');
    tr.innerHTML = `
      <td>${a.id}</td>
      <td>${a.timestamp}</td>
      <td class="level-${a.level}">${a.level}</td>
      <td>${a.category}</td>
      <td>${escapeHtml(a.message)}</td>
      <td>${a.reason || ''}</td>
    `;
    tbody.appendChild(tr);
  }
}

function escapeHtml(s){
  return s ? s.replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;') : '';
}

document.getElementById('refresh').addEventListener('click', async ()=>{
  const anoms = await fetchAnomalies();
  render(anoms);
});

document.getElementById('export').addEventListener('click', ()=>{
  // reuse logs export endpoint by fetching and converting
  fetch('/api/admin/anomalies', { credentials: 'same-origin' }).then(r=>r.json()).then(data=>{
    const lines = ['id,timestamp,level,category,message,reason'];
    for(const r of data){
      const row = [r.id, csvSafe(r.timestamp), csvSafe(r.level), csvSafe(r.category), csvSafe(r.message), csvSafe(r.reason || '')];
      lines.push(row.join(','));
    }
    const blob = new Blob([lines.join('\n')], {type:'text/csv'});
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a'); a.href = url; a.download = 'anomalies.csv'; document.body.appendChild(a); a.click(); a.remove();
  });
});

function csvSafe(s){
  if(s==null) return '';
  const str = String(s).replace(/"/g,'""');
  if(str.includes(',')||str.includes('\n')) return '"'+str+'"';
  return str;
}

// Auto-load on open
(async ()=>{ const a = await fetchAnomalies(); render(a); })();

document.getElementById('scan')?.addEventListener('click', async ()=>{
  if(!confirm('Scan recent logs for anomalies? This may take a few seconds.')) return;
  const res = await fetch('/api/admin/anomalies/scan', { method: 'POST', credentials: 'same-origin' });
  if (!res.ok) { alert('Scan failed'); return; }
  const data = await res.json();
  alert('Scan finished. Flagged: ' + (data.flagged || 0));
  const anoms = await fetchAnomalies(); render(anoms);
});

document.getElementById('scanAsync')?.addEventListener('click', async ()=>{
  if(!confirm('Start background scan of recent logs? This will return immediately.')) return;
  const res = await fetch('/api/admin/anomalies/scan?async=1', { method: 'POST', credentials: 'same-origin' });
  if(!res.ok){ alert('Failed to start background scan'); return; }
  const data = await res.json();
  const jobId = data.job_id;
  alert('Background scan started (job: ' + jobId + '). Polling status...');
  // poll status
  const poll = setInterval(async ()=>{
    const r = await fetch('/api/admin/anomalies/scan/status/' + jobId, { credentials: 'same-origin' });
    if(!r.ok) return;
    const s = await r.json();
    if(s.status && s.status.startsWith('done')){
      clearInterval(poll);
      alert('Background scan completed: ' + s.status);
      const anoms = await fetchAnomalies(); render(anoms);
    }
  }, 3000);
});
