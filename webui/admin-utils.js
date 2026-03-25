// Admin utility functions for JeebsAI

/**
 * Admin data export utilities
 */
const AdminExport = {
    // Export users as CSV
    exportUsersCSV: function(users) {
        let csv = 'ID,Username,Email,Is Admin,Created At\n';
        users.forEach(user => {
            const adminStatus = user.is_admin ? 'Yes' : 'No';
            const line = `${user.id},"${user.username}","${user.email}",${adminStatus},"${user.created_at}"`;
            csv += line + '\n';
        });
        downloadFile(csv, `users-${Date.now()}.csv`, 'text/csv');
    },

    // Export conversations as CSV
    exportConversationsCSV: function(conversations) {
        let csv = 'ID,User ID,Title,Created At,Updated At,Messages Count\n';
        conversations.forEach(conv => {
            const line = `${conv.id},${conv.user_id},"${conv.title}","${conv.created_at}","${conv.updated_at}",${conv.message_count || 0}`;
            csv += line + '\n';
        });
        downloadFile(csv, `conversations-${Date.now()}.csv`, 'text/csv');
    },

    // Export system stats as JSON
    exportStatsJSON: function(stats) {
        const json = JSON.stringify({
            timestamp: new Date().toISOString(),
            ...stats
        }, null, 2);
        downloadFile(json, `system-stats-${Date.now()}.json`, 'application/json');
    },

    // Export brain memories as JSONL (JSON Lines format)
    exportBrainMemoriesJSONL: function(memories) {
        let jsonl = '';
        memories.forEach(mem => {
            jsonl += JSON.stringify(mem) + '\n';
        });
        downloadFile(jsonl, `brain-memories-${Date.now()}.jsonl`, 'application/x-ndjson');
    },

    // Helper function to download files
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

// Shorthand for downloadFile
function downloadFile(content, filename, mimeType) {
    AdminExport.downloadFile(content, filename, mimeType);
}

/**
 * Admin dashboard statistics
 */
const AdminStats = {
    // Calculate user engagement
    calculateEngagement: function(users, conversations, messages) {
        const totalUsers = users.length;
        const activeUsers = conversations.length > 0 ? [...new Set(conversations.map(c => c.user_id))].length : 0;
        const engagementRate = totalUsers > 0 ? ((activeUsers / totalUsers) * 100).toFixed(2) : 0;
        
        const avgMessagesPerConv = conversations.length > 0 
            ? (messages.length / conversations.length).toFixed(2) 
            : 0;
        
        return {
            totalUsers,
            activeUsers,
            engagementRate: `${engagementRate}%`,
            conversationCount: conversations.length,
            messageCount: messages.length,
            avgMessagesPerConv
        };
    },

    // Calculate brain stats
    calculateBrainStats: function(memories) {
        return {
            totalMemories: memories.length,
            avgMemoryLength: memories.length > 0 
                ? Math.round(memories.reduce((sum, m) => sum + (m.response_text?.length || 0), 0) / memories.length)
                : 0,
            memoryDensity: memories.length > 0 ? memories.length.toString() : '0'
        };
    },

    // Generate performance report
    generatePerformanceReport: function(stats) {
        let report = '# System Performance Report\n\n';
        report += `Generated: ${new Date().toLocaleString()}\n\n`;
        
        report += '## Database\n';
        report += `- Size: ${stats.database?.size_mb || 'N/A'} MB\n`;
        report += `- Status: ${stats.database?.exists ? 'OK' : 'Missing'}\n\n`;
        
        report += '## Process\n';
        report += `- CPU: ${stats.process?.cpu_percent || 'N/A'}%\n`;
        report += `- Memory: ${stats.process?.memory_mb || 'N/A'} MB\n`;
        report += `- VMS: ${stats.process?.vms_mb || 'N/A'} MB\n\n`;
        
        return report;
    }
};

/**
 * Admin dashboard utilities
 */
const AdminDashboard = {
    // Format large numbers
    formatNumber: function(num) {
        if (num >= 1000000) {
            return (num / 1000000).toFixed(1) + 'M';
        }
        if (num >= 1000) {
            return (num / 1000).toFixed(1) + 'K';
        }
        return num.toString();
    },

    // Create sparkline-like chart (ASCII)
    createMiniChart: function(values) {
        if (!values || values.length === 0) return '─';
        
        const sparkChars = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
        const min = Math.min(...values);
        const max = Math.max(...values);
        const range = max - min || 1;
        
        return values.map(v => {
            const index = Math.round(((v - min) / range) * (sparkChars.length - 1));
            return sparkChars[index];
        }).join('');
    },

    // Create status indicator
    createStatusIndicator: function(status) {
        const icons = {
            'healthy': '🟢',
            'warning': '🟡',
            'error': '🔴',
            'loading': '🔵'
        };
        return icons[status] || '⚪';
    },

    // Auto-refresh dashboard data
    setupAutoRefresh: function(callback, interval = 30000) {
        // Initial load
        callback();
        
        // Set up interval
        return setInterval(callback, interval);
    }
};

/**
 * Admin security utilities
 */
const AdminSecurity = {
    // Generate strong password
    generatePassword: function(length = 16) {
        const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*';
        let password = '';
        for (let i = 0; i < length; i++) {
            password += chars.charAt(Math.floor(Math.random() * chars.length));
        }
        return password;
    },

    // Validate email
    validateEmail: function(email) {
        return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email);
    },

    // Check password strength
    checkPasswordStrength: function(password) {
        let strength = 0;
        if (password.length >= 8) strength++;
        if (password.length >= 12) strength++;
        if (/[A-Z]/.test(password)) strength++;
        if (/[a-z]/.test(password)) strength++;
        if (/[0-9]/.test(password)) strength++;
        if (/[!@#$%^&*]/.test(password)) strength++;
        
        const levels = ['Very Weak', 'Weak', 'Fair', 'Good', 'Strong', 'Very Strong', 'Excellent'];
        return {
            score: strength,
            level: levels[strength],
            percentage: (strength / 6) * 100
        };
    }
};

// Export for use in admin.html
console.info('✅ Admin utilities loaded. Use AdminExport, AdminStats, AdminDashboard, AdminSecurity');
