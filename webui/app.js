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
    
    if (!username || !password) {
        errorDiv.textContent = '❌ Please enter username and password';
        errorDiv.classList.add('show');
        return;
    }
    
    // Show loading state
    errorDiv.textContent = '⏳ Logging in...';
    errorDiv.classList.add('show');
    errorDiv.style.color = '#666';
    
    try {
        console.log('Attempting login for user:', username);
        const response = await fetch(`${API_BASE}/auth/login`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ username, password })
        });
        
        console.log('Login response status:', response.status);
        
        const data = await response.json();
        console.log('Login response data:', data);
        
        if (!response.ok) {
            errorDiv.textContent = data.message || 'Login failed';
            errorDiv.style.color = '#e74c3c';
            errorDiv.classList.add('show');
            console.error('Login failed:', data.message);
            return;
        }
        
        if (!data.token || !data.user_id) {
            console.error('Invalid response data:', data);
            errorDiv.textContent = '❌ Invalid server response. Please try again.';
            errorDiv.style.color = '#e74c3c';
            errorDiv.classList.add('show');
            return;
        }
        
        token = data.token;
        currentUser = { 
            id: data.user_id, 
            username: data.username, 
            is_admin: data.is_admin || false 
        };
        localStorage.setItem('token', token);
        localStorage.setItem('user', JSON.stringify(currentUser));
        
        console.log('Login successful for user:', username);
        errorDiv.textContent = '✅ Login successful! Loading...';
        errorDiv.style.color = '#27ae60';
        
        // Show chat section
        showChatSection();
        
        // Load conversations after a short delay to ensure UI is updated
        setTimeout(() => {
            loadConversations();
        }, 100);
    } catch (error) {
        console.error('Login error:', error);
        errorDiv.textContent = `❌ Connection error: ${error.message}`;
        errorDiv.style.color = '#e74c3c';
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
    
    if (!username || !email || !password) {
        errorDiv.textContent = '❌ Please fill in all fields';
        errorDiv.classList.add('show');
        return;
    }
    
    // Show loading state
    errorDiv.textContent = '⏳ Creating account...';
    errorDiv.classList.add('show');
    errorDiv.style.color = '#666';
    
    try {
        console.log('Attempting registration for user:', username);
        const response = await fetch(`${API_BASE}/auth/register`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ username, email, password })
        });
        
        console.log('Register response status:', response.status);
        
        const data = await response.json();
        console.log('Register response data:', data);
        
        if (!response.ok) {
            errorDiv.textContent = data.message || 'Registration failed';
            errorDiv.style.color = '#e74c3c';
            errorDiv.classList.add('show');
            console.error('Registration failed:', data.message);
            return;
        }
        
        if (!data.token || !data.user_id) {
            console.error('Invalid response data:', data);
            errorDiv.textContent = '❌ Invalid server response. Please try again.';
            errorDiv.style.color = '#e74c3c';
            errorDiv.classList.add('show');
            return;
        }
        
        token = data.token;
        currentUser = { id: data.user_id, username: data.username, is_admin: false };
        localStorage.setItem('token', token);
        localStorage.setItem('user', JSON.stringify(currentUser));
        
        console.log('Registration successful for user:', username);
        errorDiv.textContent = '✅ Account created! Loading...';
        errorDiv.style.color = '#27ae60';
        
        showChatSection();
        
        // Create first conversation after a short delay
        setTimeout(() => {
            createNewConversation();
        }, 100);
    } catch (error) {
        console.error('Registration error:', error);
        errorDiv.textContent = `❌ Connection error: ${error.message}`;
        errorDiv.style.color = '#e74c3c';
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
            document.getElementById('admin-link').style.display = 'block';
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
        console.log('Loading conversations with token:', token ? 'valid' : 'missing');
        const response = await fetch(`${API_BASE}/chat/conversations`, {
            headers: { 'Authorization': `Bearer ${token}` }
        });
        
        console.log('Conversations response status:', response.status);
        
        if (!response.ok) {
            console.error('Failed to load conversations:', response.status);
            if (response.status === 401) {
                console.warn('Unauthorized - logging out');
                logout();
            }
            return;
        }
        
        const conversations = await response.json();
        console.log('Loaded conversations:', conversations.length || 0);
        
        const list = document.getElementById('conversations-list');
        if (!list) {
            console.error('conversations-list element not found');
            return;
        }
        
        list.innerHTML = '';
        
        if (!conversations || !Array.isArray(conversations)) {
            console.warn('Invalid conversations response:', conversations);
            list.innerHTML = '<div style="padding: 20px; color: #999; text-align: center;">No conversations</div>';
            return;
        }
        
        conversations.forEach(conv => {
            const item = document.createElement('div');
            item.className = 'conversation-item';
            if (conv.id === currentConversationId) item.classList.add('active');
            item.textContent = conv.title || 'Untitled';
            item.onclick = () => selectConversation(conv.id);
            list.appendChild(item);
        });
        
        console.log('Conversations list rendered successfully');
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
        console.log('Loading conversation:', convId);
        const response = await fetch(`${API_BASE}/chat/conversations/${convId}`, {
            headers: { 'Authorization': `Bearer ${token}` }
        });
        
        console.log('Conversation response status:', response.status);
        
        if (!response.ok) {
            console.error('Failed to load conversation:', response.status);
            return;
        }
        
        const data = await response.json();
        console.log('Loaded conversation data:', data);
        
        const convTitle = document.getElementById('conv-title');
        if (!convTitle) {
            console.error('conv-title element not found');
            return;
        }
        
        convTitle.textContent = data.conversation ? data.conversation.title : 'Conversation';
        
        // Load messages
        const container = document.getElementById('messages-container');
        if (!container) {
            console.error('messages-container element not found');
            return;
        }
        
        container.innerHTML = '';
        
        if (!data.messages || data.messages.length === 0) {
            container.innerHTML = '<div class="welcome-message"><p>Start a conversation</p></div>';
        } else {
            data.messages.forEach(msg => {
                addMessageToUI(msg.role, msg.content);
            });
            container.scrollTop = container.scrollHeight;
        }
        
        // Update UI
        loadConversations();
        const msgInput = document.getElementById('message-input');
        if (msgInput) msgInput.focus();
        
        console.log('Conversation loaded successfully');
    } catch (error) {
        console.error('Error loading conversation:', error);
    }
}

// Send message
async function handleSendMessage(e) {
    e.preventDefault();
    
    if (!token) {
        console.error('No token present - user not authenticated');
        return;
    }
    
    if (!currentConversationId) {
        console.log('No conversation selected, creating new one');
        await createNewConversation();
        return;
    }
    
    const input = document.getElementById('message-input');
    const content = input.value.trim();
    
    if (!content) return;
    
    input.value = '';
    addMessageToUI('user', content);
    
    // Check if message suggests tool usage
    detectAndSuggestTools(content);
    
    // Fetch tool suggestions if user is admin
    if (currentUser && currentUser.is_admin) {
        try {
            const suggestResponse = await fetch(`${API_BASE}/chat/suggest-tools`, {
                method: 'POST',
                headers: {
                    'Authorization': `Bearer ${token}`,
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify({ message: content, max_suggestions: 3 })
            });
            
            if (suggestResponse.ok) {
                const suggestData = await suggestResponse.json();
                if (suggestData.suggestions && suggestData.suggestions.length > 0) {
                    displayToolSuggestions(suggestData.suggestions);
                }
            }
        } catch (err) {
            console.error('Error fetching tool suggestions:', err);
        }
    }
    
    try {
        console.log('Sending message to conversation:', currentConversationId);
        const response = await fetch(`${API_BASE}/chat/conversations/${currentConversationId}/messages`, {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${token}`,
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ content })
        });
        
        console.log('Message response status:', response.status);
        
        if (!response.ok) {
            console.error('Failed to send message:', response.status);
            if (response.status === 401) {
                console.warn('Unauthorized - logging out');
                logout();
            }
            return;
        }
        
        const data = await response.json();
        console.log('Message sent successfully, response:', data.response ? 'received' : 'none');
        
        if (data.response) {
            addMessageToUI('assistant', data.response);
        }
        
        // Scroll to bottom
        const container = document.getElementById('messages-container');
        if (container) {
            container.scrollTop = container.scrollHeight;
        }

        // Refresh conversation list (title may have changed)
        loadConversations();
    } catch (error) {
        console.error('Error sending message:', error);
        addMessageToUI('assistant', `⚠️ Error sending message: ${error.message}`);
    }
}

// Display tool suggestions for admin users
function displayToolSuggestions(suggestions) {
    const container = document.getElementById('messages-container');
    
    // Create suggestions display
    const suggestionsDiv = document.createElement('div');
    suggestionsDiv.className = 'tool-suggestions';
    suggestionsDiv.innerHTML = '<div class="suggestions-header">💡 <strong>Suggested Tools:</strong></div>';
    
    suggestions.forEach(tool => {
        const toolCard = document.createElement('div');
        toolCard.className = 'suggestion-card';
        toolCard.innerHTML = `
            <div class="suggestion-tool">
                <strong>${tool.name}</strong>
                <p class="suggestion-desc">${tool.description}</p>
                <small class="suggestion-keywords">Keywords: ${tool.matched_keywords.join(', ')}</small>
                <button class="btn-suggestion" onclick="navigateToTool('${tool.name}')">Use Tool →</button>
            </div>
        `;
        suggestionsDiv.appendChild(toolCard);
    });
    
    container.appendChild(suggestionsDiv);
    container.scrollTop = container.scrollHeight;
}

// Navigate to tool dashboard or execute tool
function navigateToTool(toolName) {
    // For now, navigate to the tools dashboard with the tool pre-selected
    window.location.href = `/tools?tool=${encodeURIComponent(toolName)}`;
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
    let helpText = '';
    
    if (toolName === 'search') {
        helpText = 'Enter a question or topic to search the web.';
        html = `
            <p style="font-size: 12px; color: #666; margin-bottom: 12px;">${helpText}</p>
            <input type="text" id="search-query" placeholder="e.g., 'How to learn Python?'" required autofocus>
            <button onclick="executeTool_search()">🔍 Search the Web</button>
            <div id="search-results" class="tool-results"></div>
        `;
    } else if (toolName === 'calculator') {
        helpText = 'Enter a math expression (e.g., 2+3*4) or use the quick operations below.';
        html = `
            <p style="font-size: 12px; color: #666; margin-bottom: 12px;">${helpText}</p>
            <input type="text" id="calc-expression" placeholder="e.g., 2 + 3 * 4 or sqrt(16)" autofocus>
            <p style="font-size: 11px; color: #999; margin: 10px 0;">Quick Operations:</p>
            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 8px; margin-bottom: 10px;">
                <div>
                    <label style="font-size: 11px;">Number 1:</label>
                    <input type="number" id="calc-a" placeholder="Num 1">
                </div>
                <div>
                    <label style="font-size: 11px;">Number 2:</label>
                    <input type="number" id="calc-b" placeholder="Num 2">
                </div>
            </div>
            <select id="calc-operation" style="margin-bottom: 10px;">
                <option value="">Select Operation</option>
                <option value="add">➕ Add</option>
                <option value="subtract">➖ Subtract</option>
                <option value="multiply">✕ Multiply</option>
                <option value="divide">÷ Divide</option>
                <option value="power">^ Power</option>
                <option value="sqrt">√ Square Root</option>
            </select>
            <button onclick="executeTool_calculator()">🧮 Calculate</button>
            <div id="calc-result" class="tool-results"></div>
        `;
    } else if (toolName === 'code_analyzer') {
        helpText = 'Paste Python code and get analysis for syntax, performance, security, and code style.';
        html = `
            <p style="font-size: 12px; color: #666; margin-bottom: 12px;">${helpText}</p>
            <textarea id="code-input" placeholder="Paste Python code here..." autofocus></textarea>
            <label style="font-size: 12px; color: #666; margin-top: 8px;">Analysis Type:</label>
            <select id="code-check">
                <option value="syntax">✓ Syntax Check</option>
                <option value="security">🔒 Security Check</option>
                <option value="performance">⚡ Performance</option>
                <option value="style">🎨 Code Style</option>
            </select>
            <button onclick="executeTool_codeAnalyzer()">👨‍💻 Analyze Code</button>
            <div id="code-result" class="tool-results"></div>
        `;
    } else if (toolName === 'text_stats') {
        helpText = 'Analyze text for word count, sentence count, readability metrics, and more.';
        html = `
            <p style="font-size: 12px; color: #666; margin-bottom: 12px;">${helpText}</p>
            <textarea id="text-input" placeholder="Paste text here..." autofocus></textarea>
            <div style="margin: 10px 0;">
                <label style="font-size: 12px; display: block; margin-bottom: 6px;"><input type="checkbox" id="stats-words" checked> 📝 Word Count</label>
                <label style="font-size: 12px; display: block; margin-bottom: 6px;"><input type="checkbox" id="stats-sentences" checked> 📄 Sentences</label>
                <label style="font-size: 12px;"><input type="checkbox" id="stats-readability" checked> 👁️ Readability</label>
            </div>
            <button onclick="executeTool_textStats()">📊 Analyze Text</button>
            <div id="text-result" class="tool-results"></div>
        `;
    } else if (toolName === 'branch_conv') {
        helpText = 'Create an alternate conversation path. Useful for exploring "what if" scenarios.';
        html = `
            <p style="font-size: 12px; color: #666; margin-bottom: 12px;">${helpText}</p>
            <input type="text" id="branch-title" placeholder="Give your branch a name..." autofocus>
            <p style="font-size: 11px; color: #999; margin-top: 10px;">The current conversation will be preserved, and a new branch will be created.</p>
            <button onclick="executeTool_branchConv()">🌳 Create Branch</button>
            <div id="branch-result" class="tool-results"></div>
        `;
    } else if (toolName === 'analytics') {
        helpText = 'View detailed analytics about your conversations, user behavior, and trending topics.';
        html = `
            <p style="font-size: 12px; color: #666; margin-bottom: 12px;">${helpText}</p>
            <label style="font-size: 12px; color: #666; display: block; margin-bottom: 6px;">Choose Analytics Type:</label>
            <select id="analytics-type">
                <option value="current">📊 Current Conversation</option>
                <option value="user">👤 Your Statistics</option>
                <option value="trending">🔥 Trending Topics</option>
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

// ============================================================================
// TOOL HELPER FUNCTIONS - Improve UX and error handling
// ============================================================================

function detectAndSuggestTools(message) {
    // Detect if message suggests tool usage and show a suggestion
    const msg = message.toLowerCase();
    let suggestedTool = null;
    
    // Map keywords to tools
    const keywords = {
        'search': ['search', 'find', 'look up', 'what is', 'who is', 'news', 'information about'],
        'calculator': ['calculate', 'math', 'compute', 'how much', 'equals', '+', '-', '*', '/', 'times'],
        'code_analyzer': ['code', 'python', 'analyze', 'error', 'bug', 'check'],
        'text_stats': ['analyze text', 'word count', 'sentences', 'readability', 'text analysis'],
        'branch_conv': ['branch', 'alternate', 'what if', 'different path'],
        'analytics': ['analytics', 'statistics', 'trending', 'analysis', 'data']
    };
    
    for (const [tool, words] of Object.entries(keywords)) {
        if (words.some(word => msg.includes(word))) {
            suggestedTool = tool;
            break;
        }
    }
    
    // Show tool suggestion if detected
    if (suggestedTool) {
        const toolDisplay = suggestedTool.replace('_', ' ').toUpperCase();
        const container = document.getElementById('messages-container');
        const sugDiv = document.createElement('div');
        sugDiv.className = 'tool-quick-suggestion';
        sugDiv.innerHTML = `
            <div style="padding: 12px; background: #f0f0ff; border-left: 4px solid #667eea; border-radius: 4px; font-size: 12px;">
                <span style="color: #667eea;"><strong>💡 Tip:</strong> Use the <strong>${toolDisplay}</strong> tool for this!</span>
                <button onclick="openTool('${suggestedTool}'); toggleToolsPanel();" style="margin-left: 10px; padding: 4px 10px; background: #667eea; color: white; border: none; border-radius: 3px; cursor: pointer; font-size: 11px; font-weight: bold;">Open Tool</button>
            </div>
        `;
        container.appendChild(sugDiv);
        container.scrollTop = container.scrollHeight;
        
        // Auto-hide after 5 seconds
        setTimeout(() => {
            if (sugDiv.parentElement) {
                sugDiv.style.opacity = '0';
                sugDiv.style.transition = 'opacity 0.3s';
                setTimeout(() => sugDiv.remove(), 300);
            }
        }, 5000);
    }
}

function showToolLoading(resultElementId) {
    document.getElementById(resultElementId).innerHTML = `
        <div style="text-align: center; padding: 20px;">
            <div class="tool-spinner"></div>
            <p style="color: #999; margin-top: 10px; font-size: 13px;">Processing...</p>
        </div>
    `;
}

function displayToolResult(resultElementId, html) {
    document.getElementById(resultElementId).innerHTML = html;
}

function displayToolError(resultElementId, errorMessage) {
    const html = `
        <div style="background: #fee; border-left: 4px solid #f44; padding: 12px; border-radius: 4px;">
            <p style="color: #c33; font-weight: bold;">❌ Error</p>
            <p style="color: #666; font-size: 13px; margin-top: 5px;">${escapeHtml(errorMessage)}</p>
        </div>
    `;
    document.getElementById(resultElementId).innerHTML = html;
}

function createToolResultCard(content, title = '', actions = []) {
    let html = `<div style="background: white; border-radius: 6px; padding: 12px; border-left: 4px solid #667eea;">`;
    
    if (title) {
        html += `<h6 style="color: #333; margin-bottom: 10px; margin-top: 0;">${title}</h6>`;
    }
    
    html += `<div style="color: #333; font-size: 13px; line-height: 1.6;">${content}</div>`;
    
    if (actions.length > 0) {
        html += `<div style="margin-top: 10px; display: flex; gap: 8px; flex-wrap: wrap;">`;
        actions.forEach(action => {
            html += `<button onclick="${action.onclick}" style="padding: 6px 12px; background: #667eea; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 11px; font-weight: 600;">${action.label}</button>`;
        });
        html += `</div>`;
    }
    
    html += `</div>`;
    return html;
}

async function executeTool_search() {
    const query = document.getElementById('search-query').value.trim();
    if (!query) { 
        displayToolError('search-results', 'Please enter a search query');
        return; 
    }
    
    showToolLoading('search-results');
    
    try {
        const res = await fetch(`${API_BASE}/tools/search`, {
            method: 'POST',
            headers: { 'Authorization': `Bearer ${token}`, 'Content-Type': 'application/json' },
            body: JSON.stringify({ query, max_results: 5 })
        });
        const data = await res.json();
        
        if (!res.ok || !data.results) {
            displayToolError('search-results', data.error || 'Search failed');
            return;
        }
        
        if (data.results.length === 0) {
            displayToolResult('search-results', 
                createToolResultCard('No results found. Try different keywords.', '🔍 Search Results'));
            return;
        }
        
        let resultsHtml = '';
        data.results.forEach((r, i) => {
            resultsHtml += `
                <div style="margin-bottom: 12px; padding: 10px; background: #f9f9f9; border-radius: 4px; border-left: 3px solid #667eea;">
                    <strong style="color: #333;">${i+1}. ${escapeHtml(r.title)}</strong><br>
                    <small style="color: #666;">${escapeHtml(r.snippet)}</small><br>
                    <a href="${r.url}" target="_blank" style="font-size: 11px; color: #667eea; text-decoration: none;">🔗 View →</a>
                </div>
            `;
        });
        
        const actions = [
            {
                label: '💬 Send to Chat',
                onclick: `sendSearchResultsToChat('${escapeHtml(query)}')`
            },
            {
                label: '🔄 New Search',
                onclick: `document.getElementById('search-query').value = ''; document.getElementById('search-query').focus();`
            }
        ];
        
        displayToolResult('search-results', 
            createToolResultCard(resultsHtml, `🔍 Search Results for "${escapeHtml(query)}"`, actions));
    } catch (e) {
        displayToolError('search-results', e.message);
    }
}

async function executeTool_calculator() {
    const expr = document.getElementById('calc-expression').value.trim();
    const op = document.getElementById('calc-operation').value;
    const a = document.getElementById('calc-a').value;
    const b = document.getElementById('calc-b').value;
    
    if (!expr && !op) {
        displayToolError('calc-result', 'Enter an expression (e.g., "2+3*4") or select an operation');
        return;
    }
    
    showToolLoading('calc-result');
    
    try {
        const res = await fetch(`${API_BASE}/tools/calculator`, {
            method: 'POST',
            headers: { 'Authorization': `Bearer ${token}`, 'Content-Type': 'application/json' },
            body: JSON.stringify({ 
                expression: expr, 
                operation: op, 
                a: Number(a) || 0, 
                b: Number(b) || 0 
            })
        });
        const data = await res.json();
        
        if (data.error) {
            displayToolError('calc-result', data.error);
            return;
        }
        
        const resultText = data.result ? data.result.toString() : 'No result';
        const actions = [
            {
                label: '📋 Copy',
                onclick: `copyToClipboard('${resultText}')`
            },
            {
                label: '💬 Send to Chat',
                onclick: `sendToChat('Calculation Result: ${resultText}')`
            }
        ];
        
        const content = `<div style="font-size: 24px; color: #667eea; text-align: center; font-weight: bold;">${resultText}</div>`;
        displayToolResult('calc-result', createToolResultCard(content, '🧮 Calculation Result', actions));
    } catch (e) {
        displayToolError('calc-result', e.message);
    }
}

async function executeTool_codeAnalyzer() {
    const code = document.getElementById('code-input').value.trim();
    const checkType = document.getElementById('code-check').value;
    
    if (!code) { 
        displayToolError('code-result', 'Please paste some Python code to analyze');
        return; 
    }
    
    showToolLoading('code-result');
    
    try {
        const res = await fetch(`${API_BASE}/tools/analyze-code`, {
            method: 'POST',
            headers: { 'Authorization': `Bearer ${token}`, 'Content-Type': 'application/json' },
            body: JSON.stringify({ code, check_type: checkType })
        });
        const data = await res.json();
        
        if (!res.ok) {
            displayToolError('code-result', data.error || 'Analysis failed');
            return;
        }
        
        let html = '';
        
        // Syntax status
        const syntaxIcon = data.syntax_valid ? '✅' : '❌';
        html += `<p><strong>${syntaxIcon} Syntax:</strong> ${data.syntax_valid ? 'Valid' : 'Invalid'}</p>`;
        
        // Issues
        if (data.issues && data.issues.length > 0) {
            html += `<p style="color: #d32f2f; font-weight: bold; margin-top: 10px;">🚨 Issues Found (${data.issues.length}):</p>`;
            html += '<ul style="margin-left: 20px; color: #333;">';
            data.issues.forEach(issue => {
                const issueText = typeof issue === 'string' ? issue : `Line ${issue.line}: ${issue.message}`;
                html += `<li style="margin-bottom: 5px;">${escapeHtml(issueText)}</li>`;
            });
            html += '</ul>';
        }
        
        // Suggestions
        if (data.suggestions && data.suggestions.length > 0) {
            html += `<p style="color: #f57c00; font-weight: bold; margin-top: 10px;">💡 Suggestions (${data.suggestions.length}):</p>`;
            html += '<ul style="margin-left: 20px; color: #333;">';
            data.suggestions.forEach(sug => {
                const sugText = typeof sug === 'string' ? sug : `Line ${sug.line}: ${sug.message}`;
                html += `<li style="margin-bottom: 5px;">${escapeHtml(sugText)}</li>`;
            });
            html += '</ul>';
        }
        
        if ((data.issues && data.issues.length === 0) && (!data.suggestions || data.suggestions.length === 0)) {
            html += '<p style="color: #4caf50; margin-top: 10px;">✨ Code looks good!</p>';
        }
        
        const actions = [
            {
                label: '📋 Copy Code',
                onclick: `copyToClipboard(\`${escapeHtml(code).replace(/`/g, '\\`')}\`)`
            }
        ];
        
        displayToolResult('code-result', createToolResultCard(html, '👨‍💻 Code Analysis', actions));
    } catch (e) {
        displayToolError('code-result', e.message);
    }
}

async function executeTool_textStats() {
    const text = document.getElementById('text-input').value.trim();
    if (!text) { 
        displayToolError('text-result', 'Please paste some text to analyze');
        return; 
    }
    
    let include = [];
    if (document.getElementById('stats-words').checked) include.push('words');
    if (document.getElementById('stats-sentences').checked) include.push('sentences');
    if (document.getElementById('stats-readability').checked) include.push('readability');
    
    if (include.length === 0) {
        displayToolError('text-result', 'Select at least one analysis option');
        return;
    }
    
    showToolLoading('text-result');
    
    try {
        const res = await fetch(`${API_BASE}/tools/text-stats`, {
            method: 'POST',
            headers: { 'Authorization': `Bearer ${token}`, 'Content-Type': 'application/json' },
            body: JSON.stringify({ text, include: include.join(',') })
        });
        const data = await res.json();
        
        if (!res.ok) {
            displayToolError('text-result', data.error || 'Analysis failed');
            return;
        }
        
        let html = '';
        // Create a nice grid of stats
        for (const [key, value] of Object.entries(data)) {
            if (key !== 'character_count' && value !== undefined) {
                const label = key.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase());
                const icon = key.includes('word') ? '📝' : key.includes('sentence') ? '📄' : '👁️';
                html += `
                    <div style="background: #f9f9f9; padding: 12px; border-radius: 4px; margin-bottom: 8px;">
                        <span style="font-weight: bold; color: #667eea;">${icon} ${label}:</span>
                        <span style="color: #333; font-size: 16px; margin-left: 8px;">${escapeHtml(String(value))}</span>
                    </div>
                `;
            }
        }
        
        const actions = [
            {
                label: '📋 Copy Text',
                onclick: `copyToClipboard(\`${text.replace(/`/g, '\\`')}\`)`
            },
            {
                label: '💬 Send Stats to Chat',
                onclick: `sendTextStatsToChat('${text.substring(0, 100)}...')`
            }
        ];
        
        displayToolResult('text-result', createToolResultCard(html, '📊 Text Statistics', actions));
    } catch (e) {
        displayToolError('text-result', e.message);
    }
}

async function executeTool_branchConv() {
    if (!currentConversationId) {
        displayToolError('branch-result', 'No conversation selected. Please select a conversation first.');
        return;
    }
    
    const title = document.getElementById('branch-title').value.trim() || 
                  `${document.getElementById('conv-title').textContent} (Branch)`;
    
    showToolLoading('branch-result');
    
    try {
        const res = await fetch(`${API_BASE}/tools/conversations/${currentConversationId}/branch`, {
            method: 'POST',
            headers: { 'Authorization': `Bearer ${token}`, 'Content-Type': 'application/json' },
            body: JSON.stringify({ from_message_id: -1, new_title: title })
        });
        const data = await res.json();
        
        if (!data.success) {
            displayToolError('branch-result', data.error || 'Failed to create branch');
            return;
        }
        
        const content = `
            <p style="color: #4caf50; margin-bottom: 10px;"><strong>✅ Branch Created!</strong></p>
            <p><strong>Branch Title:</strong> ${escapeHtml(data.branch_title)}</p>
            <p><strong>Messages Copied:</strong> ${data.messages_copied}</p>
            <p style="color: #666; font-size: 12px; margin-top: 10px;">Your original conversation remains unchanged. The branch is a new conversation.</p>
        `;
        
        const actions = [
            {
                label: '🔄 Reload Conversations',
                onclick: `loadConversations(); closeTool();`
            }
        ];
        
        displayToolResult('branch-result', createToolResultCard(content, '🌳 Branch Created', actions));
        
        // Reload conversations in the background
        setTimeout(() => {
            loadConversations();
        }, 1000);
    } catch (e) {
        displayToolError('branch-result', e.message);
    }
}

async function executeTool_analytics() {
    const type = document.getElementById('analytics-type').value;
    let endpoint = '';
    
    if (type === 'current') {
        if (!currentConversationId) {
            displayToolError('analytics-result', 'No conversation selected');
            return;
        }
        endpoint = `/analytics/conversation/${currentConversationId}`;
    } else if (type === 'user') {
        endpoint = '/analytics/user';
    } else if (type === 'trending') {
        endpoint = '/analytics/trending?limit=10';
    }
    
    showToolLoading('analytics-result');
    
    try {
        const res = await fetch(`${API_BASE}/tools${endpoint}`, {
            headers: { 'Authorization': `Bearer ${token}` }
        });
        const data = await res.json();
        
        if (!res.ok) {
            displayToolError('analytics-result', data.error || 'Failed to load analytics');
            return;
        }
        
        let html = '';
        
        if (type === 'trending' && data.topics) {
            html = '<div style="display: flex; flex-wrap: wrap; gap: 8px;">';
            data.topics.forEach(t => {
                html += `<span style="background: #667eea; color: white; padding: 6px 12px; border-radius: 20px; font-size: 12px;">
                    <strong>${escapeHtml(t.topic)}</strong> <em style="opacity: 0.8;">×${t.frequency}</em>
                </span>`;
            });
            html += '</div>';
        } else {
            for (const [key, value] of Object.entries(data)) {
                if (!key.startsWith('_') && typeof value !== 'object') {
                    const label = key.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase());
                    html += `
                        <div style="background: #f9f9f9; padding: 10px; margin: 8px 0; border-radius: 4px;">
                            <span style="font-weight: bold; color: #667eea;">${label}:</span>
                            <span style="color: #333; margin-left: 8px;">${escapeHtml(String(value))}</span>
                        </div>
                    `;
                }
            }
        }
        
        const actions = [
            {
                label: '💬 Send to Chat',
                onclick: `sendAnalyticsToChat('${type}')`
            }
        ];
        
        const typeLabel = type === 'current' ? 'Conversation' : type === 'user' ? 'User' : 'Trending Topics';
        displayToolResult('analytics-result', createToolResultCard(html, `📈 ${typeLabel} Analytics`, actions));
    } catch (e) {
        displayToolError('analytics-result', e.message);
    }
}

// Keyboard shortcuts
document.addEventListener('keydown', (e) => {
    if (e.ctrlKey && e.key === 'n') {
        e.preventDefault();
        createNewConversation();
    }
});

// ============================================================================
// TOOL RESULT ACTIONS - Send results to chat, copy, etc.
// ============================================================================

function copyToClipboard(text) {
    navigator.clipboard.writeText(text).then(() => {
        alert('✅ Copied to clipboard!');
    }).catch(() => {
        alert('Failed to copy');
    });
}

function sendToChat(message) {
    const input = document.getElementById('message-input');
    input.value = message;
    input.focus();
    closeTool();
    toggleToolsPanel();
}

function sendSearchResultsToChat(query) {
    const message = `🔍 I searched for: "${query}" and found relevant information.`;
    sendToChat(message);
}

function sendTextStatsToChat(textPreview) {
    const message = `📊 I analyzed the text "${textPreview}" and got detailed statistics.`;
    sendToChat(message);
}

function sendAnalyticsToChat(type) {
    const typeLabel = type === 'current' ? 'conversation' : type === 'user' ? 'user' : 'trending topics';
    const message = `📈 Here are the ${typeLabel} analytics.`;
    sendToChat(message);
}

// ============================================================================
// TEACHING FUNCTIONS - Allow users to teach JeebsAI new knowledge
// ============================================================================

function toggleTeachingPanel() {
    const panel = document.getElementById('teaching-panel');
    const toolsPanel = document.getElementById('tools-panel');
    
    // Hide tools panel if showing
    if (!toolsPanel.classList.contains('hidden')) {
        toolsPanel.classList.add('hidden');
    }
    
    panel.classList.toggle('hidden');
}

async function submitTeaching() {
    const keyText = document.getElementById('teach-key').value.trim();
    const responseText = document.getElementById('teach-response').value.trim();
    const category = document.getElementById('teach-category').value;
    const statusDiv = document.getElementById('teach-status');
    
    if (!keyText || !responseText) {
        statusDiv.innerHTML = '<p style="color: red;">❌ Both fields are required!</p>';
        return;
    }
    
    try {
        const response = await fetch(`${API_BASE}/chat/teach`, {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${token}`,
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({
                key: keyText,
                response: responseText,
                category: category,
                conversation_id: currentConversationId || 0
            })
        });
        
        const data = await response.json();
        
        if (response.ok) {
            statusDiv.innerHTML = `<p style="color: green;">✅ Knowledge saved! JeebsAI will remember this.</p>`;
            
            // Add teaching confirmation to chat
            addMessageToUI('assistant', `📚 Got it! I've learned: "${keyText}" → "${responseText}"`);
            
            // Clear form
            document.getElementById('teach-key').value = '';
            document.getElementById('teach-response').value = '';
            document.getElementById('teach-category').value = 'general';
            
            // Close panel after 2 seconds
            setTimeout(() => {
                toggleTeachingPanel();
                statusDiv.innerHTML = '';
            }, 2000);
        } else {
            statusDiv.innerHTML = `<p style="color: red;">❌ Error: ${data.message}</p>`;
        }
    } catch (error) {
        statusDiv.innerHTML = `<p style="color: red;">❌ Failed to submit: ${error.message}</p>`;
    }
}

async function loadJeebasMem ories() {
    // Load and display all memories JeebsAI has learned
    try {
        const response = await fetch(`${API_BASE}/chat/brain/memories?limit=20`, {
            method: 'GET',
            headers: {
                'Authorization': `Bearer ${token}`
            }
        });
        
        const data = await response.json();
        if (response.ok && data.memories) {
            let html = '<h4 style="margin-bottom: 15px;">📚 JeebsAI\'s Memory Bank</h4>';
            
            if (data.memories.length === 0) {
                html += '<p style="color: #999;">No memories yet. Start teaching JeebsAI!</p>';
            } else {
                html += '<div style="max-height: 400px; overflow-y: auto;">';
                data.memories.forEach((mem, idx) => {
                    const category = mem.category || 'general';
                    html += `
                        <div style="background: #f5f5f5; padding: 10px; margin: 8px 0; border-radius: 4px; border-left: 3px solid #667eea;">
                            <div style="display: flex; justify-content: space-between; align-items: start;">
                                <div style="flex: 1;">
                                    <strong style="color: #667eea;">Q: ${escapeHtml(mem.key_text)}</strong><br>
                                    <p style="margin: 5px 0; color: #333; font-size: 13px;">A: ${escapeHtml(mem.response_text.substring(0, 80))}${mem.response_text.length > 80 ? '...' : ''}</p>
                                    <small style="color: #999;">Category: ${category} | Used ${mem.access_count} times</small>
                                </div>
                                <button 
                                    onclick="forgetMemory(${mem.id})"
                                    style="background: #ff6b6b; color: white; border: none; padding: 4px 8px; border-radius: 3px; cursor: pointer; font-size: 11px;"
                                >
                                    🗑️ Delete
                                </button>
                            </div>
                        </div>
                    `;
                });
                html += '</div>';
            }
            
            const container = document.getElementById('messages-container');
            const memoryDiv = document.createElement('div');
            memoryDiv.className = 'memory-display';
            memoryDiv.innerHTML = html;
            container.appendChild(memoryDiv);
            container.scrollTop = container.scrollHeight;
        }
    } catch (error) {
        console.error('Error loading memories:', error);
    }
}

async function forgetMemory(memoryId) {
    if (!confirm('Are you sure you want to delete this memory?')) return;
    
    try {
        const response = await fetch(`${API_BASE}/chat/brain/forget/${memoryId}`, {
            method: 'DELETE',
            headers: {
                'Authorization': `Bearer ${token}`
            }
        });
        
        if (response.ok) {
            document.querySelector(`[data-memory-id="${memoryId}"]`)?.remove();
            addMessageToUI('assistant', '🗑️ Memory deleted. I\'ve forgotten that.');
        }
    } catch (error) {
        console.error('Error deleting memory:', error);
    }
}

// ============================================================================
// BRAIN INSIGHTS FUNCTIONS - Show what JeebsAI has learned about conversations
// ============================================================================

function toggleBrainInsights() {
    const panel = document.getElementById('brain-insights-panel');
    const toolsPanel = document.getElementById('tools-panel');
    const teachingPanel = document.getElementById('teaching-panel');
    
    // Hide other panels if showing
    if (!toolsPanel.classList.contains('hidden')) {
        toolsPanel.classList.add('hidden');
    }
    if (!teachingPanel.classList.contains('hidden')) {
        teachingPanel.classList.add('hidden');
    }
    
    if (panel.classList.contains('hidden')) {
        panel.classList.remove('hidden');
        loadBrainInsights();
    } else {
        panel.classList.add('hidden');
    }
}

async function loadBrainInsights() {
    if (!currentConversationId) {
        document.getElementById('brain-insights-data').innerHTML = 
            '<p style="color: #999; text-align: center;">No conversation selected.</p>';
        return;
    }
    
    const loadingDiv = document.getElementById('brain-insights-loading');
    const dataDiv = document.getElementById('brain-insights-data');
    
    try {
        loadingDiv.style.display = 'block';
        dataDiv.style.display = 'none';
        
        const response = await fetch(`${API_BASE}/chat/brain/conversation-context/${currentConversationId}`, {
            method: 'GET',
            headers: {
                'Authorization': `Bearer ${token}`
            }
        });
        
        const data = await response.json();
        
        if (response.ok && data.learning_context) {
            const context = data.learning_context;
            let html = '';
            
            // Display style
            html += `
                <div class="brain-insight-stat">
                    <strong>Conversation Style</strong>
                    <div class="brain-insight-value">
                        ${formatConversationStyle(context.style)}
                    </div>
                </div>
            `;
            
            // Display topics
            if (context.topics && context.topics.length > 0) {
                html += `
                    <div class="brain-insight-stat">
                        <strong>Main Topics Discussed</strong>
                        <div class="brain-topics-list">
                            ${context.topics.map(topic => `<span class="brain-topic-tag">${escapeHtml(topic)}</span>`).join('')}
                        </div>
                    </div>
                `;
            }
            
            // Display categories
            if (context.categories && context.categories.length > 0) {
                html += `
                    <div class="brain-insight-stat">
                        <strong>Memory Categories</strong>
                        <div class="brain-categories-list">
                            ${context.categories.map(cat => `<span class="brain-category-badge">${escapeHtml(cat)}</span>`).join('')}
                        </div>
                    </div>
                `;
            }
            
            // Display stats
            html += `<div class="brain-insights-divider"></div>`;
            html += `
                <div class="brain-insight-stat">
                    <strong>Learning Statistics</strong>
                    <div class="brain-insight-value">
                        📚 Memories: <strong>${context.memory_count}</strong><br>
                        ⭐ Avg Priority: <strong>${context.avg_priority}</strong><br>
                        🔄 Avg Access: <strong>${context.avg_access}</strong> times
                    </div>
                </div>
            `;
            
            dataDiv.innerHTML = html;
            loadingDiv.style.display = 'none';
            dataDiv.style.display = 'block';
        } else {
            dataDiv.innerHTML = '<p style="color: #999;">No learning data available yet. Start chatting to build context!</p>';
            loadingDiv.style.display = 'none';
            dataDiv.style.display = 'block';
        }
    } catch (error) {
        console.error('Error loading brain insights:', error);
        dataDiv.innerHTML = `<p style="color: #e74c3c;">Error: ${error.message}</p>`;
        loadingDiv.style.display = 'none';
        dataDiv.style.display = 'block';
    }
}

function formatConversationStyle(style) {
    const styles = {
        'formal_detailed': '📋 Formal & Detailed - Technical discussions with depth',
        'frequently_referenced': '🔄 Frequently Referenced - Important topics discussed multiple times',
        'conversational': '💬 Conversational - Casual, flowing discussion',
        'neutral': '➖ Neutral - Just getting started'
    };
    return styles[style] || style;
}
