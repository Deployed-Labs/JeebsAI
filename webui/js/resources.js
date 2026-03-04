// Trainer Resources Proposal Actions


async function fetchProposals() {
    const res = await fetch('/api/brain/proposals?status=pending', { credentials: 'same-origin' });
    if (!res.ok) throw new Error('Failed to load proposals');
    return await res.json();
}

async function approveProposalAPI(proposalId) {
    const res = await fetch(`/api/brain/proposals/${proposalId}/approve`, {
        method: 'POST',
        credentials: 'same-origin'
    });
    const data = await res.json();
    if (!res.ok) throw new Error(data.error || 'Failed to approve proposal');
    return data;
}

async function denyProposalAPI(proposalId, reason) {
    const res = await fetch(`/api/brain/proposals/${proposalId}/deny`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'same-origin',
        body: JSON.stringify(reason ? { reason } : {})
    });
    const data = await res.json();
    if (!res.ok) throw new Error(data.error || 'Failed to deny proposal');
    return data;
}

async function renderProposals() {
    const container = document.getElementById('proposals-list');
    if (!container) return;
    container.innerHTML = '<div class="muted">Loading proposals...</div>';
    try {
        const data = await fetchProposals();
        if (!data.proposals || data.proposals.length === 0) {
            container.innerHTML = '<div class="muted">No proposals found.</div>';
            return;
        }
        container.innerHTML = '';
        data.forEach(p => {
            const card = document.createElement('div');
            card.className = 'card';
            card.innerHTML = `
                <div style="display:flex;justify-content:space-between;align-items:center;">
                    <div>
                        <strong>${p.title || '(untitled)'}</strong>
                        <div class="text-muted" style="font-size:0.9em">${p.proposer_id || 'system'} &mdash; ${p.created_at ? new Date(p.created_at).toLocaleString() : ''}</div>
                        <div style="margin:6px 0">${p.description || ''}</div>
                    </div>
                    <div style="display:flex;gap:8px;">
                        <button class="btn btn-success" onclick="acceptProposal('${p.id}')">Accept</button>
                        <button class="btn btn-danger" onclick="denyProposal('${p.id}')">Deny</button>
                    </div>
                </div>
            `;
            container.appendChild(card);
        });
    } catch (e) {
        container.innerHTML = '<div class="muted">Failed to load proposals.</div>';
    }
}


async function acceptProposal(id) {
    if (!confirm('Accept this proposal?')) return;
    try {
        await approveProposalAPI(id);
        renderProposals();
    } catch (e) {
        alert('Failed to accept: ' + e.message);
    }
}

async function denyProposal(id) {
    if (!confirm('Deny this proposal?')) return;
    let reason = prompt('Optional: Enter a reason for denial (leave blank for none):');
    try {
        await denyProposalAPI(id, reason);
        renderProposals();
    } catch (e) {
        alert('Failed to deny: ' + e.message);
    }
}

