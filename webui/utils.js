// Utility functions for JeebsAI

/**
 * Keyboard shortcuts handler
 */
const KeyboardShortcuts = {
    // Register keyboard shortcuts
    register: function() {
        document.addEventListener('keydown', (e) => {
            // Cmd/Ctrl + K: Focus message input
            if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
                e.preventDefault();
                const input = document.getElementById('message-input');
                if (input) input.focus();
            }

            // Cmd/Ctrl + Shift + N: New conversation
            if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.key === 'n') {
                e.preventDefault();
                if (typeof createNewConversation === 'function') {
                    createNewConversation();
                }
            }

            // Escape: Clear message input (when not in any modal)
            if (e.key === 'Escape' && !document.querySelector('.modal:not(.hidden)')) {
                const input = document.getElementById('message-input');
                if (input && input.value) {
                    input.value = '';
                    input.focus();
                }
            }

            // Cmd/Ctrl + L: Clear conversation
            if ((e.ctrlKey || e.metaKey) && e.key === 'l') {
                e.preventDefault();
                const container = document.getElementById('messages-container');
                if (container && confirm('Clear current conversation?')) {
                    container.innerHTML = '<div class="welcome-message"><h2>New Conversation</h2></div>';
                }
            }
        });
    }
};

/**
 * Local storage manager with auto-save
 */
const AutoSave = {
    DRAFT_KEY: 'jeebs_draft_message',
    SETTINGS_KEY: 'jeebs_settings',

    // Save draft message
    saveDraft: function(text) {
        if (text.trim()) {
            localStorage.setItem(this.DRAFT_KEY, text);
        }
    },

    // Load draft message
    loadDraft: function() {
        return localStorage.getItem(this.DRAFT_KEY) || '';
    },

    // Clear draft
    clearDraft: function() {
        localStorage.removeItem(this.DRAFT_KEY);
    },

    // Save user settings
    saveSettings: function(settings) {
        localStorage.setItem(this.SETTINGS_KEY, JSON.stringify(settings));
    },

    // Load settings
    loadSettings: function() {
        try {
            return JSON.parse(localStorage.getItem(this.SETTINGS_KEY)) || {};
        } catch {
            return {};
        }
    }
};

/**
 * Message formatter & utilities
 */
const MessageUtils = {
    // Escape HTML to prevent XSS
    escapeHtml: function(text) {
        const map = {
            '&': '&amp;',
            '<': '&lt;',
            '>': '&gt;',
            '"': '&quot;',
            "'": '&#039;'
        };
        return text.replace(/[&<>"']/g, m => map[m]);
    },

    // Format timestamp
    formatTime: function(isoString) {
        const date = new Date(isoString);
        const now = new Date();
        const diffMs = now - date;
        const diffMins = Math.floor(diffMs / 60000);
        const diffHours = Math.floor(diffMs / 3600000);
        const diffDays = Math.floor(diffMs / 86400000);

        if (diffMins < 1) return 'Just now';
        if (diffMins < 60) return `${diffMins}m ago`;
        if (diffHours < 24) return `${diffHours}h ago`;
        if (diffDays < 7) return `${diffDays}d ago`;
        
        return date.toLocaleDateString();
    },

    // Format file size
    formatSize: function(bytes) {
        if (bytes === 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
    },

    // Markdown-like formatting (basic)
    formatMarkdown: function(text) {
        return text
            .replace(/\*\*(.*?)\*\*/g, '<strong>$1</strong>')
            .replace(/\*(.*?)\*/g, '<em>$1</em>')
            .replace(/`(.*?)`/g, '<code>$1</code>')
            .replace(/\n/g, '<br>');
    }
};

/**
 * Error handling utilities
 */
const ErrorHandler = {
    show: function(message, duration = 5000) {
        // Create error toast
        const toast = document.createElement('div');
        toast.className = 'error-toast';
        toast.textContent = message;
        toast.style.cssText = `
            position: fixed;
            bottom: 20px;
            right: 20px;
            background: #ff4757;
            color: white;
            padding: 15px 20px;
            border-radius: 8px;
            box-shadow: 0 4px 12px rgba(0,0,0,0.3);
            z-index: 10000;
            max-width: 400px;
            animation: slideIn 0.3s ease;
        `;

        document.body.appendChild(toast);

        setTimeout(() => {
            toast.style.animation = 'slideOut 0.3s ease';
            setTimeout(() => toast.remove(), 300);
        }, duration);
    },

    showSuccess: function(message, duration = 3000) {
        const toast = document.createElement('div');
        toast.className = 'success-toast';
        toast.textContent = message;
        toast.style.cssText = `
            position: fixed;
            bottom: 20px;
            right: 20px;
            background: #2ed573;
            color: white;
            padding: 15px 20px;
            border-radius: 8px;
            box-shadow: 0 4px 12px rgba(0,0,0,0.3);
            z-index: 10000;
            max-width: 400px;
            animation: slideIn 0.3s ease;
        `;

        document.body.appendChild(toast);

        setTimeout(() => {
            toast.style.animation = 'slideOut 0.3s ease';
            setTimeout(() => toast.remove(), 300);
        }, duration);
    }
};

/**
 * Conversation search & filter
 */
const ConversationSearch = {
    filter: function(conversations, searchTerm) {
        if (!searchTerm.trim()) return conversations;
        
        const term = searchTerm.toLowerCase();
        return conversations.filter(conv => 
            conv.title.toLowerCase().includes(term)
        );
    },

    // Search in message content (requires backend support)
    searchMessages: async function(conversationId, searchTerm) {
        try {
            const response = await fetch(
                `/api/chat/${conversationId}/search?q=${encodeURIComponent(searchTerm)}`,
                {
                    headers: { 'Authorization': `Bearer ${localStorage.getItem('token')}` }
                }
            );
            if (!response.ok) throw new Error('Search failed');
            return await response.json();
        } catch (error) {
            console.error('Search error:', error);
            return { results: [] };
        }
    }
};

/**
 * Export utilities
 */
const ExportUtils = {
    // Export conversation as JSON
    exportConversationJSON: function(conversationData) {
        const json = JSON.stringify(conversationData, null, 2);
        this.downloadFile(json, `conversation-${Date.now()}.json`, 'application/json');
    },

    // Export conversation as Markdown
    exportConversationMarkdown: function(messages, title = 'Conversation') {
        let markdown = `# ${title}\n\n`;
        markdown += `*Exported on ${new Date().toLocaleString()}*\n\n`;

        messages.forEach(msg => {
            const role = msg.role === 'user' ? '👤 You' : '🤖 AI';
            markdown += `## ${role}\n\n${msg.content}\n\n---\n\n`;
        });

        this.downloadFile(markdown, `conversation-${Date.now()}.md`, 'text/markdown');
    },

    // Export conversation as CSV
    exportConversationCSV: function(messages) {
        let csv = 'Role,Content,Timestamp\n';
        messages.forEach(msg => {
            const time = new Date(msg.created_at).toISOString();
            const content = `"${msg.content.replace(/"/g, '""')}"`;
            csv += `${msg.role},${content},${time}\n`;
        });

        this.downloadFile(csv, `conversation-${Date.now()}.csv`, 'text/csv');
    },

    // Helper to download file
    downloadFile: function(content, filename, mimeType) {
        const blob = new Blob([content], { type: mimeType });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = filename;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    }
};

/**
 * Initialize all utilities on page load
 */
document.addEventListener('DOMContentLoaded', () => {
    KeyboardShortcuts.register();
    
    // Restore draft if exists
    const draft = AutoSave.loadDraft();
    const input = document.getElementById('message-input');
    if (input && draft) {
        input.value = draft;
    }

    // Auto-save drafts
    if (input) {
        input.addEventListener('input', (e) => {
            AutoSave.saveDraft(e.target.value);
        });
    }

    // Add keyboard shortcuts hint
    console.info('💡 Keyboard Shortcuts:\nCtrl/Cmd+K: Focus message\nCtrl/Cmd+Shift+N: New chat\nEsc: Clear input\nCtrl/Cmd+L: Clear chat');
});

// Add CSS animations for toasts
const style = document.createElement('style');
style.textContent = `
    @keyframes slideIn {
        from {
            transform: translateX(400px);
            opacity: 0;
        }
        to {
            transform: translateX(0);
            opacity: 1;
        }
    }

    @keyframes slideOut {
        from {
            transform: translateX(0);
            opacity: 1;
        }
        to {
            transform: translateX(400px);
            opacity: 0;
        }
    }
`;
document.head.appendChild(style);
