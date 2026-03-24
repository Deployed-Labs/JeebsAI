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
    if (!currentUser.is_admin) return;
    
    try {
        const response = await fetch(`${API_BASE}/admin/dashboard`, {
            headers: { 'Authorization': `Bearer ${token}` }
        });
        
        if (!response.ok) return;
        
        const data = await response.json();
        const statsDiv = document.getElementById('admin-stats');
        const usersDiv = document.getElementById('admin-users');
        
        statsDiv.innerHTML = `
            <div class="stat-card">
                <h4>Total Users</h4>
                <div class="stat-value">${data.stats.total_users}</div>
            </div>
            <div class="stat-card">
                <h4>Conversations</h4>
                <div class="stat-value">${data.stats.total_conversations}</div>
            </div>
            <div class="stat-card">
                <h4>Messages</h4>
                <div class="stat-value">${data.stats.total_messages}</div>
            </div>
        `;
        
        usersDiv.innerHTML = '<h3>Recent Users</h3>';
        data.recent_users.forEach(user => {
            const userEl = document.createElement('div');
            userEl.className = 'user-item';
            userEl.innerHTML = `
                <div class="user-item-name">${escapeHtml(user.username)}</div>
                <div class="user-item-email">${escapeHtml(user.email)}</div>
            `;
            usersDiv.appendChild(userEl);
        });
        
        document.getElementById('admin-modal').classList.remove('hidden');
    } catch (error) {
        console.error('Error loading admin dashboard:', error);
    }
}

function closeAdminDashboard() {
    document.getElementById('admin-modal').classList.add('hidden');
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

// Keyboard shortcuts
document.addEventListener('keydown', (e) => {
    if (e.ctrlKey && e.key === 'n') {
        e.preventDefault();
        createNewConversation();
    }
});
