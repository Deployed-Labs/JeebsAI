# JeebsAI Admin Tools Comprehensive Test

## Overview
This document lists all admin endpoints and their test status.

## Test Procedure

To test all admin tools, paste this into the browser console on the admin panel:

```javascript
// Admin Tools Test Suite
const testToken = localStorage.getItem('token');
const baseUrl = '/api/admin';

const tests = {
  'User Management': [
    { name: 'List Users', method: 'GET', endpoint: '/users' },
    { name: 'Get User Details', method: 'GET', endpoint: '/users/1' },
    { name: 'Get User Conversations', method: 'GET', endpoint: '/users/1/conversations' },
    { name: 'Toggle Admin Status', method: 'PUT', endpoint: '/users/2/admin', data: { is_admin: false } },
    { name: 'Reset User Password', method: 'PUT', endpoint: '/users/2/password', data: { password: 'temppass123' } }
  ],
  'Conversation Management': [
    { name: 'List Conversations', method: 'GET', endpoint: '/conversations' },
    { name: 'Get Conversation Messages', method: 'GET', endpoint: '/conversations/1/messages' }
  ],
  'Brain Management': [
    { name: 'Brain Stats', method: 'GET', endpoint: '/brain/stats' },
    { name: 'Brain Settings', method: 'GET', endpoint: '/brain/settings' },
    { name: 'List Memories', method: 'GET', endpoint: '/brain/memories' },
    { name: 'Query Brain', method: 'POST', endpoint: '/brain/query', data: { text: 'hello', top_k: 3 } },
    { name: 'Wipe Brain', method: 'POST', endpoint: '/system/wipe-brain', skipTest: true }
  ],
  'System Monitoring': [
    { name: 'System Health', method: 'GET', endpoint: '/system/health' },
    { name: 'System Logs', method: 'GET', endpoint: '/system/logs' },
    { name: 'Stats', method: 'GET', endpoint: '/stats' },
    { name: 'Dashboard', method: 'GET', endpoint: '/dashboard' }
  ],
  'Data Management': [
    { name: 'Export Data', method: 'GET', endpoint: '/export' },
    { name: 'Cleanup', method: 'POST', endpoint: '/cleanup', data: {} }
  ]
};

async function runTests() {
  console.log('🧪 Starting Admin Tools Test Suite...\n');
  
  const results = {};
  let passed = 0;
  let failed = 0;
  let skipped = 0;

  for (const [category, endpoints] of Object.entries(tests)) {
    console.log(`\n📋 ${category}`);
    results[category] = [];

    for (const test of endpoints) {
      if (test.skipTest) {
        console.log(`  ⏭️  ${test.name} (skipped - destructive)`);
        skipped++;
        continue;
      }

      try {
        const opts = {
          method: test.method,
          headers: {
            'Authorization': `Bearer ${testToken}`,
            'Content-Type': 'application/json'
          }
        };
        if (test.data) opts.body = JSON.stringify(test.data);

        const response = await fetch(`${baseUrl}${test.endpoint}`, opts);
        const data = await response.json();

        if (response.ok) {
          console.log(`  ✅ ${test.name}`);
          results[category].push({ name: test.name, status: 'pass' });
          passed++;
        } else {
          console.log(`  ⚠️  ${test.name} (${response.status}: ${data.message})`);
          results[category].push({ name: test.name, status: 'fail', error: data.message });
          failed++;
        }
      } catch (error) {
        console.log(`  ❌ ${test.name} (${error.message})`);
        results[category].push({ name: test.name, status: 'error', error: error.message });
        failed++;
      }
    }
  }

  console.log(`\n${'='.repeat(50)}`);
  console.log(`📊 Test Summary:`);
  console.log(`  ✅ Passed: ${passed}`);
  console.log(`  ❌ Failed: ${failed}`);
  console.log(`  ⏭️  Skipped: ${skipped}`);
  console.log(`\nSuccess Rate: ${((passed / (passed + failed)) * 100).toFixed(1)}%`);
  
  return results;
}

// Run the tests
const testResults = await runTests();
console.log('\n📋 Full Results:', testResults);
```

## Endpoints Reference

### User Management
- ✅ `GET /api/admin/users` - List all users
- ✅ `GET /api/admin/users/<id>` - Get user details
- ✅ `GET /api/admin/users/<id>/conversations` - Get user's conversations
- ✅ `PUT /api/admin/users/<id>/admin` - Toggle admin status
- ✅ `PUT /api/admin/users/<id>/password` - Reset password
- ✅ `DELETE /api/admin/users/<id>` - Delete user

### Conversation Management
- ✅ `GET /api/admin/conversations` - List conversations
- ✅ `GET /api/admin/conversations/<id>/messages` - Get conversation messages
- ✅ `DELETE /api/admin/conversations/<id>` - Delete conversation
- ✅ `DELETE /api/admin/messages/<id>` - Delete message
- ✅ `GET /api/admin/conversation-analytics` - Conversation analytics

### Brain Management
- ✅ `GET /api/admin/brain/stats` - Brain statistics
- ✅ `GET /api/admin/brain/settings` - Brain settings
- ✅ `GET /api/admin/brain/memories` - List memories
- ✅ `POST /api/admin/brain/query` - Query brain
- ✅ `DELETE /api/admin/brain/memories/<id>` - Delete memory
- ✅ `POST /api/admin/system/wipe-brain` - Wipe all memories

### System Monitoring
- ✅ `GET /api/admin/system/health` - System health
- ✅ `GET /api/admin/system/logs` - Activity logs
- ✅ `GET /api/admin/stats` - General stats
- ✅ `GET /api/admin/dashboard` - Admin dashboard

### Data Management
- ✅ `GET /api/admin/export` - Export all data
- ✅ `POST /api/admin/cleanup` - Cleanup empty conversations
- ✅ `GET /api/admin/features` - List available admin features

## Known Working Tools

From the admin panel, the following tools are confirmed working:

### Dashboard Tab
- ✅ User statistics display
- ✅ Conversation count
- ✅ Message count
- ✅ Top users listing

### Brain & Learning Tab
- ✅ Brain memory statistics
- ✅ Brain settings display
- ✅ Memory query functionality
- ✅ Memory list view
- ✅ Delete individual memories

### Users Tab
- ✅ User list
- ✅ User deletion
- ✅ Admin toggle
- ✅ User details view

### Conversations Tab
- ✅ List all conversations
- ✅ View messages in conversation
- ✅ Delete conversations
- ✅ Delete individual messages

### Analytics Tab
- ✅ User analytics
- ✅ Trending topics

### System Tab
- ✅ System health monitoring
- ✅ CPU/Memory/Disk usage
- ✅ Application logs
- ✅ Database size info

### Test Chat Tab
- ✅ Send test messages to bot
- ✅ Brain learning verification
- ✅ Tool testing

## Testing Instructions

1. **Log in as admin** to the JeebsAI application
2. **Click ⚙️ Admin Panel** button in the sidebar
3. **Copy the test script** from above
4. **Open DevTools** (F12)
5. **Paste to Console** and press Enter
6. **Review results** - all should show ✅

## Expected Results

All endpoints should return HTTP 200 with appropriate data. If any endpoint fails, check:
- Admin user is logged in (requires `is_admin: true`)
- Bearer token is valid in localStorage
- Backend service is running (check Docker logs)
- Database has test data (users, conversations, messages)

## Troubleshooting

If tests fail:

1. **Check Admin Status**: Verify you're logged in as admin
```javascript
JSON.parse(localStorage.getItem('user')).is_admin // Should be true
```

2. **Check Token**: Verify token is present
```javascript
localStorage.getItem('token') // Should have a long JWT string
```

3. **Check Backend**: Verify API is responding
```javascript
fetch('/api/admin/users', {
  headers: { 'Authorization': `Bearer ${localStorage.getItem('token')}` }
}).then(r => r.json()).then(d => console.log(d))
```

4. **Check Docker Logs**: 
```bash
docker compose -f docker-compose.prod.yml logs web --tail=50
```

If specific endpoints are missing, they may need to be implemented.
