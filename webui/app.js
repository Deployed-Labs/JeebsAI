// Global state
let currentUser = null;
let currentConversationId = null;
let token = null;
const API_BASE = '/api';

// Initialize app
document.addEventListener('DOMContentLoaded', () => {
    token = localStorage.getItem('token');
    if (token) {
        verifyTokenAndShowChat();
    } else {
        showAuthSection();
    }
});

// Auth tab switching
function switchAuthMode(mode) {
    document.querySelectorAll('.tab-btn').forEach(btn => btn.classList.remove('active'));
    document.querySelectorAll('.auth-form').forEach(form => form.classList.remove('active'));
    
    event.target.classList.add('active');
    document.getElementById(`${mode}-form`).classList.add('active');
    
    // Clear error messages
    document.getElementById(`${mode}-error`).textContent = '';
}

// Handle login
async function handleLogin(e) {
    e.preventDefault();
    const username = document.getElementById('login-username').value;
    const password = document.getElementById('login-password').value;
    const errorDiv = document.getElementById('login-error');
    
    try {
        const response = await fetch(`${API_BASE}/auth/login`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ username, password })
        });
        
        const data = await response.json();
        
        if (!response.ok) {
            errorDiv.textContent = data.message || 'Login failed';
            errorDiv.classList.add('show');
            return;
        }
        
        token = data.token;
        currentUser = { id: data.user_id, username: data.username, is_admin: data.is_admin };
        localStorage.setItem('token', token);
        localStorage.setItem('user', JSON.stringify(currentUser));
        
        showChatSection();
        loadConversations();
    } catch (error) {
        errorDiv.textContent = 'Connection error. Please try again.';
        errorDiv.classList.add('show');
    }
}

// Handle register
async function handleRegister(e) {
    e.preventDefault();
    const username = document.getElementById('register-username').value;
    const email = document.getElementById('register-email').value;
    const password = document.getElementById('register-password').value;
    const errorDiv = document.getElementById('register-error');
    
    try {
        const response = await fetch(`${API_BASE}/auth/register`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ username, email, password })
        });
        
        const data = await response.json();
        
        if (!response.ok) {
            errorDiv.textContent = data.message || 'Registration failed';
            errorDiv.classList.add('show');
            return;
        }
        
        token = data.token;
        currentUser = { id: data.user_id, username: data.username, is_admin: false };
        localStorage.setItem('token', token);
        localStorage.setItem('user', JSON.stringify(currentUser));
        
        showChatSection();
        createNewConversation();
    } catch (error) {
        errorDiv.textContent = 'Connection error. Please try again.';
        errorDiv.classList.add('show');
    }
}

// Verify token and show chat
async function verifyTokenAndShowChat() {
    try {
        const response = await fetch(`${API_BASE}/auth/verify-token`, {
            headers: { 'Authorization': `Bearer ${token}` }
        });
        
        if (!response.ok) {
            logout();
            return;
        }
        
        const userData = localStorage.getItem('user');
        if (userData) {
            currentUser = JSON.parse(userData);
        }
        
        showChatSection();
        loadConversations();
        
        // Check if user is admin
        if (currentUser.is_admin) {
            document.getElementById('admin-section').style.display = 'block';
        }
    } catch (error) {
        logout();
    }
}

// Show/hide sections
function showAuthSection() {
    document.getElementById('auth-section').classList.remove('hidden');
    document.getElementById('chat-section').classList.add('hidden');
}

function showChatSection() {
    document.getElementById('auth-section').classList.add('hidden');
    document.getElementById('chat-section').classList.remove('hidden');
}

// Load conversations
async function loadConversations() {
    try {
        const response = await fetch(`${API_BASE}/chat/conversations`, {
            headers: { 'Authorization': `Bearer ${token}` }
        });
        
        if (!response.ok) {
            if (response.status === 401) logout();
            return;
        }
        
        const conversations = await response.json();
        const list = document.getElementById('conversations-list');
        list.innerHTML = '';
        
        conversations.forEach(conv => {
            const item = document.createElement('div');
            item.className = 'conversation-item';
            if (conv.id === currentConversationId) item.classList.add('active');
            item.textContent = conv.title;
            item.onclick = () => selectConversation(conv.id);
            list.appendChild(item);
        });
    } catch (error) {
        console.error('Error loading conversations:', error);
    }
}

// Create new conversation
async function createNewConversation() {
    try {
        const response = await fetch(`${API_BASE}/chat/conversations`, {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${token}`,
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ title: 'New Chat' })
        });
        
        if (!response.ok) return;
        
        const conversation = await response.json();
        selectConversation(conversation.id);
        loadConversations();
    } catch (error) {
        console.error('Error creating conversation:', error);
    }
}

// Select conversation
async function selectConversation(convId) {
    currentConversationId = convId;
    
    try {
        const response = await fetch(`${API_BASE}/chat/conversations/${convId}`, {
            headers: { 'Authorization': `Bearer ${token}` }
        });
        
        if (!response.ok) return;
        
        const data = await response.json();
        document.getElementById('conv-title').textContent = data.conversation.title;
        
        // Load messages
        const container = document.getElementById('messages-container');
        container.innerHTML = '';
        
        if (data.messages.length === 0) {
            container.innerHTML = '<div class="welcome-message"><p>Start a conversation</p></div>';
        } else {
            data.messages.forEach(msg => {
                addMessageToUI(msg.role, msg.content);
            });
            container.scrollTop = container.scrollHeight;
        }
        
        // Update UI
        loadConversations();
        document.getElementById('message-input').focus();
    } catch (error) {
        console.error('Error loading conversation:', error);
    }
}

// Send message
async function handleSendMessage(e) {
    e.preventDefault();
    
    if (!currentConversationId) {
        await createNewConversation();
        return;
    }
    
    const input = document.getElementById('message-input');
    const content = input.value.trim();
    
    if (!content) return;
    
    input.value = '';
    addMessageToUI('user', content);
    
    try {
        const response = await fetch(`${API_BASE}/chat/conversations/${currentConversationId}/messages`, {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${token}`,
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ content })
        });
        
        if (!response.ok) {
            if (response.status === 401) logout();
            return;
        }
        
        const data = await response.json();
        addMessageToUI('assistant', data.response);
        
        // Scroll to bottom
        const container = document.getElementById('messages-container');
        container.scrollTop = container.scrollHeight;
        
        // Refresh conversation title if it changed
        loadConversations();
    } catch (error) {
        console.error('Error sending message:', error);
        addMessageToUI('assistant', 'Error: Could not send message. Please try again.');
    }
}

// Add message to UI
function addMessageToUI(role, content) {
    const container = document.getElementById('messages-container');
    
    // Remove welcome message if present
    const welcome = container.querySelector('.welcome-message');
    if (welcome) welcome.remove();
    
    const message = document.createElement('div');
    message.className = `message ${role}`;
    message.innerHTML = `<div class="message-content">${escapeHtml(content)}</div>`;
    container.appendChild(message);
}

// Escape HTML
function escapeHtml(text) {
    const map = { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#039;' };
    return text.replace(/[&<>"']/g, m => map[m]);
}

// Admin dashboard
async function showAdminDashboard() {
    if (!currentUser.is_admin) {
        alert('Admin access required');
        return;
    }
    
    // Navigate to the admin dashboard page
    window.location.href = '/admin';
}

async function loadAllUsers() {
    try {
        const response = await fetch(`${API_BASE}/admin/users`, {
            headers: { 'Authorization': `Bearer ${token}` }
        });
        
        if (!response.ok) return;
        
        const users = await response.json();
        const usersDiv = document.getElementById('admin-users');
        usersDiv.innerHTML = '<h3>All Users</h3>';
        
        users.forEach(user => {
            const userEl = document.createElement('div');
            userEl.className = 'user-management-item';
            userEl.innerHTML = `
                <div class="user-info">
                    <div class="user-item-name">${escapeHtml(user.username)}</div>
                    <div class="user-item-email">${escapeHtml(user.email)}</div>
                    ${user.is_admin ? '<span class="badge-admin">ADMIN</span>' : ''}
                </div>
                <div class="user-actions">
                    <button class="btn-small" onclick="toggleUserAdmin(${user.id}, ${!user.is_admin})">
                        ${user.is_admin ? 'Revoke Admin' : 'Make Admin'}
                    </button>
                    <button class="btn-small btn-danger" onclick="resetUserPassword(${user.id})">Reset Pass</button>
                    ${user.id !== currentUser.id ? `<button class="btn-small btn-danger" onclick="deleteUserAdmin(${user.id})">Delete</button>` : ''}
                </div>
            `;
            usersDiv.appendChild(userEl);
        });
    } catch (error) {
        console.error('Error loading users:', error);
    }
}

async function viewAllConversations() {
    try {
        const response = await fetch(`${API_BASE}/admin/conversations`, {
            headers: { 'Authorization': `Bearer ${token}` }
        });
        
        if (!response.ok) return;
        
        const conversations = await response.json();
        const convDiv = document.getElementById('admin-conversations');
        convDiv.innerHTML = '<h3>All Conversations</h3>';
        
        conversations.forEach(conv => {
            const convEl = document.createElement('div');
            convEl.className = 'conv-management-item';
            convEl.innerHTML = `
                <div class="conv-info">
                    <div class="conv-title">${escapeHtml(conv.title)}</div>
                    <div class="conv-user">By: ${escapeHtml(conv.username)}</div>
                    <div class="conv-date">Created: ${new Date(conv.created_at).toLocaleDateString()}</div>
                </div>
                <div class="conv-actions">
                    <button class="btn-small" onclick="viewConversationAdmin(${conv.id})">View</button>
                    <button class="btn-small btn-danger" onclick="deleteConversationAdmin(${conv.id})">Delete</button>
                </div>
            `;
            convDiv.appendChild(convEl);
        });
    } catch (error) {
        console.error('Error loading conversations:', error);
    }
}

async function viewConversationAdmin(convId) {
    try {
        const response = await fetch(`${API_BASE}/admin/conversations/${convId}/messages`, {
            headers: { 'Authorization': `Bearer ${token}` }
        });
        
        if (!response.ok) return;
        
        const messages = await response.json();
        const convDiv = document.getElementById('admin-conversations');
        convDiv.innerHTML = '<h3>Messages in Conversation</h3>';
        
        messages.forEach(msg => {
            const msgEl = document.createElement('div');
            msgEl.className = `message-item message-${msg.role}`;
            msgEl.innerHTML = `
                <div class="msg-header">${msg.role.toUpperCase()}</div>
                <div class="msg-content">${escapeHtml(msg.content)}</div>
                <div class="msg-time">${new Date(msg.created_at).toLocaleString()}</div>
                <button class="btn-tiny btn-danger" onclick="deleteMessageAdmin(${msg.id})">Delete</button>
            `;
            convDiv.appendChild(msgEl);
        });
    } catch (error) {
        console.error('Error loading messages:', error);
    }
}

async function toggleUserAdmin(userId, makeAdmin) {
    if (!confirm(`${makeAdmin ? 'Make' : 'Revoke'} admin status for user ${userId}?`)) return;
    
    try {
        const response = await fetch(`${API_BASE}/admin/users/${userId}/admin`, {
            method: 'PUT',
            headers: {
                'Authorization': `Bearer ${token}`,
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ is_admin: makeAdmin })
        });
        
        if (response.ok) {
            alert(`User ${userId} admin status updated`);
            showAdminDashboard();
        }
    } catch (error) {
        console.error('Error updating user:', error);
        alert('Error updating user');
    }
}

async function resetUserPassword(userId) {
    const newPassword = prompt('Enter new password for user:');
    if (!newPassword) return;
    
    try {
        const response = await fetch(`${API_BASE}/admin/users/${userId}/password`, {
            method: 'PUT',
            headers: {
                'Authorization': `Bearer ${token}`,
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ password: newPassword })
        });
        
        if (response.ok) {
            alert(`Password reset successful. New password: ${newPassword}`);
            showAdminDashboard();
        }
    } catch (error) {
        console.error('Error resetting password:', error);
        alert('Error resetting password');
    }
}

async function deleteUserAdmin(userId) {
    if (!confirm(`Permanently delete user ${userId} and all their data?`)) return;
    
    try {
        const response = await fetch(`${API_BASE}/admin/users/${userId}`, {
            method: 'DELETE',
            headers: { 'Authorization': `Bearer ${token}` }
        });
        
        if (response.ok) {
            alert(`User ${userId} deleted`);
            showAdminDashboard();
        }
    } catch (error) {
        console.error('Error deleting user:', error);
        alert('Error deleting user');
    }
}

async function deleteConversationAdmin(convId) {
    if (!confirm(`Permanently delete conversation ${convId} and all messages?`)) return;
    
    try {
        const response = await fetch(`${API_BASE}/admin/conversations/${convId}`, {
            method: 'DELETE',
            headers: { 'Authorization': `Bearer ${token}` }
        });
        
        if (response.ok) {
            alert(`Conversation ${convId} deleted`);
            viewAllConversations();
        }
    } catch (error) {
        console.error('Error deleting conversation:', error);
        alert('Error deleting conversation');
    }
}

async function deleteMessageAdmin(msgId) {
    if (!confirm(`Delete this message?`)) return;
    
    try {
        const response = await fetch(`${API_BASE}/admin/messages/${msgId}`, {
            method: 'DELETE',
            headers: { 'Authorization': `Bearer ${token}` }
        });
        
        if (response.ok) {
            alert('Message deleted');
            // Reload current view
            const convDiv = document.getElementById('admin-conversations');
            if (convDiv.innerHTML.includes('Messages in Conversation')) {
                // Refresh the message list
                location.reload();
            }
        }
    } catch (error) {
        console.error('Error deleting message:', error);
        alert('Error deleting message');
    }
}

async function cleanupDatabase() {
    if (!confirm('Clean up empty conversations? This cannot be undone.')) return;
    
    try {
        const response = await fetch(`${API_BASE}/admin/cleanup`, {
            method: 'POST',
            headers: { 'Authorization': `Bearer ${token}` }
        });
        
        if (response.ok) {
            const data = await response.json();
            alert(`Cleanup complete. Deleted ${data.empty_conversations_deleted} empty conversations`);
            showAdminDashboard();
        }
    } catch (error) {
        console.error('Error cleaning database:', error);
        alert('Error cleaning database');
    }
}

async function exportAllData() {
    try {
        const response = await fetch(`${API_BASE}/admin/export`, {
            headers: { 'Authorization': `Bearer ${token}` }
        });
        
        if (response.ok) {
            const data = await response.json();
            const dataStr = JSON.stringify(data, null, 2);
            const dataBlob = new Blob([dataStr], { type: 'application/json' });
            const url = URL.createObjectURL(dataBlob);
            const link = document.createElement('a');
            link.href = url;
            link.download = `jeebs-export-${new Date().toISOString()}.json`;
            link.click();
            URL.revokeObjectURL(url);
            alert('Data exported successfully');
        }
    } catch (error) {
        console.error('Error exporting data:', error);
        alert('Error exporting data');
    }
}

function closeAdminDashboard() {
    document.getElementById('admin-modal').classList.add('hidden');
}

function switchAdminTab(tabName) {
    // Hide all tabs
    document.querySelectorAll('.admin-tab-content').forEach(tab => {
        tab.classList.remove('active');
    });
    document.querySelectorAll('.admin-tab-btn').forEach(btn => {
        btn.classList.remove('active');
    });
    
    // Show selected tab
    const tabId = `admin-${tabName}-tab`;
    document.getElementById(tabId).classList.add('active');
    
    // Mark button as active
    event.target.classList.add('active');
    
    // Load data for the tab
    if (tabName === 'users') {
        loadAllUsers();
    } else if (tabName === 'conversations') {
        viewAllConversations();
    }
}

// Logout
function handleLogout() {
    logout();
}

function logout() {
    token = null;
    currentUser = null;
    currentConversationId = null;
    localStorage.removeItem('token');
    localStorage.removeItem('user');
    showAuthSection();
    document.getElementById('login-username').value = '';
    document.getElementById('login-password').value = '';
    document.getElementById('register-username').value = '';
    document.getElementById('register-email').value = '';
    document.getElementById('register-password').value = '';
}

// ============================================================================
// TOOLS PANEL FUNCTIONS
// ============================================================================

function toggleToolsPanel() {
    document.getElementById('tools-panel').classList.toggle('hidden');
}

function openTool(toolName) {
    const panel = document.getElementById('tool-input-panel');
    const title = document.getElementById('tool-title');
    const content = document.getElementById('tool-input-content');
    
    title.textContent = '🔧 ' + toolName.replace('_', ' ').toUpperCase();
    
    let html = '';
    
    if (toolName === 'search') {
        html = `
            <input type="text" id="search-query" placeholder="Search query..." required>
            <button onclick="executeTool_search()">🔍 Search</button>
            <div id="search-results" class="tool-results"></div>
        `;
    } else if (toolName === 'calculator') {
        html = `
            <input type="text" id="calc-expression" placeholder="e.g., 2 + 3 * 4">
            <select id="calc-operation">
                <option value="">Expression</option>
                <option value="add">Add</option>
                <option value="subtract">Subtract</option>
                <option value="multiply">Multiply</option>
                <option value="divide">Divide</option>
                <option value="power">Power</option>
                <option value="sqrt">Square Root</option>
            </select>
            <input type="number" id="calc-a" placeholder="First number">
            <input type="number" id="calc-b" placeholder="Second number">
            <button onclick="executeTool_calculator()">🧮 Calculate</button>
            <div id="calc-result" class="tool-results"></div>
        `;
    } else if (toolName === 'code_analyzer') {
        html = `
            <textarea id="code-input" placeholder="Paste Python code here..."></textarea>
            <select id="code-check">
                <option value="syntax">Syntax Check</option>
                <option value="security">Security Check</option>
                <option value="performance">Performance Check</option>
                <option value="style">Style Check</option>
            </select>
            <button onclick="executeTool_codeAnalyzer()">👨‍💻 Analyze</button>
            <div id="code-result" class="tool-results"></div>
        `;
    } else if (toolName === 'text_stats') {
        html = `
            <textarea id="text-input" placeholder="Paste text here..."></textarea>
            <label><input type="checkbox" id="stats-words" checked> Word Count</label>
            <label><input type="checkbox" id="stats-sentences" checked> Sentences</label>
            <label><input type="checkbox" id="stats-readability" checked> Readability</label>
            <button onclick="executeTool_textStats()">📊 Analyze</button>
            <div id="text-result" class="tool-results"></div>
        `;
    } else if (toolName === 'branch_conv') {
        html = `
            <p style="font-size: 12px; color: #666; margin-bottom: 10px;">Create an alternate conversation path from current point.</p>
            <input type="text" id="branch-title" placeholder="New branch title...">
            <button onclick="executeTool_branchConv()">🌳 Create Branch</button>
            <div id="branch-result" class="tool-results"></div>
        `;
    } else if (toolName === 'analytics') {
        html = `
            <p style="font-size: 12px; color: #666; margin-bottom: 10px;">View conversation analytics and trends.</p>
            <select id="analytics-type">
                <option value="current">Current Conversation</option>
                <option value="user">User Statistics</option>
                <option value="trending">Trending Topics</option>
            </select>
            <button onclick="executeTool_analytics()">📈 View Analytics</button>
            <div id="analytics-result" class="tool-results"></div>
        `;
    }
    
    content.innerHTML = html;
    panel.classList.remove('hidden');
}

function closeTool() {
    document.getElementById('tool-input-panel').classList.add('hidden');
}

async function executeTool_search() {
    const query = document.getElementById('search-query').value;
    if (!query) { alert('Enter a search query'); return; }
    
    try {
        const res = await fetch(`${API_BASE}/tools/search`, {
            method: 'POST',
            headers: { 'Authorization': `Bearer ${token}`, 'Content-Type': 'application/json' },
            body: JSON.stringify({ query, max_results: 5 })
        });
        const data = await res.json();
        
        let html = '<h5 style="margin-bottom: 10px; color: #333;">Search Results:</h5>';
        if (data.results && data.results.length) {
            data.results.forEach((r, i) => {
                html += `
                    <div style="margin-bottom: 12px; padding: 10px; background: white; border-radius: 4px; border-left: 3px solid #667eea;">
                        <strong>${i+1}. ${r.title}</strong><br>
                        <small>${r.snippet}</small><br>
                        <a href="${r.url}" target="_blank" style="font-size: 11px; color: #667eea;">View →</a>
                    </div>
                `;
            });
        } else {
            html += '<p style="color: #999;">No results found.</p>';
        }
        document.getElementById('search-results').innerHTML = html;
    } catch (e) {
        alert('Search failed: ' + e.message);
    }
}

async function executeTool_calculator() {
    const expr = document.getElementById('calc-expression').value;
    const op = document.getElementById('calc-operation').value;
    const a = document.getElementById('calc-a').value;
    const b = document.getElementById('calc-b').value;
    
    try {
        const res = await fetch(`${API_BASE}/tools/calculator`, {
            method: 'POST',
            headers: { 'Authorization': `Bearer ${token}`, 'Content-Type': 'application/json' },
            body: JSON.stringify({ expression: expr, operation: op, a: Number(a) || 0, b: Number(b) || 0 })
        });
        const data = await res.json();
        
        const html = data.error ? 
            `<p style="color: red;"><strong>Error:</strong> ${data.error}</p>` :
            `<p style="font-size: 16px; color: #667eea;"><strong>Result:</strong> ${data.result}</p>`;
        
        document.getElementById('calc-result').innerHTML = html;
    } catch (e) {
        alert('Calculation failed: ' + e.message);
    }
}

async function executeTool_codeAnalyzer() {
    const code = document.getElementById('code-input').value;
    const checkType = document.getElementById('code-check').value;
    
    if (!code) { alert('Enter code to analyze'); return; }
    
    try {
        const res = await fetch(`${API_BASE}/tools/analyze-code`, {
            method: 'POST',
            headers: { 'Authorization': `Bearer ${token}`, 'Content-Type': 'application/json' },
            body: JSON.stringify({ code, check_type: checkType })
        });
        const data = await res.json();
        
        let html = `<h5 style="margin-bottom: 10px; color: #333;">Analysis Results:</h5>`;
        html += `<p><strong>Syntax Valid:</strong> ${data.syntax_valid ? '✓ Yes' : '✗ No'}</p>`;
        
        if (data.issues && data.issues.length) {
            html += `<p style="color: red;"><strong>Issues (${data.issues.length}):</strong></p><ul style="margin-left: 20px;">`;
            data.issues.forEach(issue => html += `<li>${issue}</li>`);
            html += `</ul>`;
        }
        
        if (data.suggestions && data.suggestions.length) {
            html += `<p style="color: #f39c12;"><strong>Suggestions (${data.suggestions.length}):</strong></p><ul style="margin-left: 20px;">`;
            data.suggestions.forEach(sug => html += `<li>${sug}</li>`);
            html += `</ul>`;
        }
        
        document.getElementById('code-result').innerHTML = html;
    } catch (e) {
        alert('Analysis failed: ' + e.message);
    }
}

async function executeTool_textStats() {
    const text = document.getElementById('text-input').value;
    if (!text) { alert('Enter text'); return; }
    
    let include = [];
    if (document.getElementById('stats-words').checked) include.push('words');
    if (document.getElementById('stats-sentences').checked) include.push('sentences');
    if (document.getElementById('stats-readability').checked) include.push('readability');
    
    try {
        const res = await fetch(`${API_BASE}/tools/text-stats`, {
            method: 'POST',
            headers: { 'Authorization': `Bearer ${token}`, 'Content-Type': 'application/json' },
            body: JSON.stringify({ text, include: include.join(',') })
        });
        const data = await res.json();
        
        let html = '<h5 style="margin-bottom: 10px; color: #333;">Text Statistics:</h5>';
        for (const [key, value] of Object.entries(data)) {
            if (key !== 'character_count') {
                html += `<p><strong>${key.replace(/_/g, ' ')}:</strong> ${value}</p>`;
            }
        }
        document.getElementById('text-result').innerHTML = html;
    } catch (e) {
        alert('Analysis failed: ' + e.message);
    }
}

async function executeTool_branchConv() {
    if (!currentConversationId) {
        alert('No conversation selected'); return;
    }
    
    const title = document.getElementById('branch-title').value || `${document.getElementById('conv-title').textContent} (Branch)`;
    
    try {
        const res = await fetch(`${API_BASE}/tools/conversations/${currentConversationId}/branch`, {
            method: 'POST',
            headers: { 'Authorization': `Bearer ${token}`, 'Content-Type': 'application/json' },
            body: JSON.stringify({ from_message_id: -1, new_title: title })
        });
        const data = await res.json();
        
        const html = data.success ?
            `<p style="color: green;"><strong>✓ Branch Created!</strong><br>New conversation: ${data.branch_title}<br>${data.messages_copied} messages copied.</p>` :
            `<p style="color: red;"><strong>Error:</strong> ${data.error}</p>`;
        
        document.getElementById('branch-result').innerHTML = html;
        if (data.success) {
            setTimeout(() => {
                closeTool();
                toggleToolsPanel();
                loadConversations();
            }, 1500);
        }
    } catch (e) {
        alert('Branch failed: ' + e.message);
    }
}

async function executeTool_analytics() {
    const type = document.getElementById('analytics-type').value;
    let endpoint = '';
    
    if (type === 'current') {
        if (!currentConversationId) {
            alert('No conversation selected'); return;
        }
        endpoint = `/analytics/conversation/${currentConversationId}`;
    } else if (type === 'user') {
        endpoint = '/analytics/user';
    } else if (type === 'trending') {
        endpoint = '/analytics/trending?limit=10';
    }
    
    try {
        const res = await fetch(`${API_BASE}/tools${endpoint}`, {
            headers: { 'Authorization': `Bearer ${token}` }
        });
        const data = await res.json();
        
        let html = '<h5 style="margin-bottom: 10px; color: #333;">Analytics:</h5>';
        
        if (type === 'trending' && data.topics) {
            html += '<p><strong>Trending Topics:</strong></p><ul style="margin-left: 20px;">';
            data.topics.forEach(t => html += `<li>${t.topic} (${t.frequency} mentions)</li>`);
            html += '</ul>';
        } else {
            for (const [key, value] of Object.entries(data)) {
                if (!key.startsWith('_') && typeof value !== 'object') {
                    html += `<p><strong>${key.replace(/_/g, ' ')}:</strong> ${value}</p>`;
                }
            }
        }
        
        document.getElementById('analytics-result').innerHTML = html;
    } catch (e) {
        alert('Analytics failed: ' + e.message);
    }
}

// Keyboard shortcuts
document.addEventListener('keydown', (e) => {
    if (e.ctrlKey && e.key === 'n') {
        e.preventDefault();
        createNewConversation();
    }
});
