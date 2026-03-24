#!/usr/bin/env python3
"""
Test script to verify all new tools are properly registered and callable
"""

import sys
sys.path.insert(0, 'd:\\GitHub\\JeebsAI')

from app.tools import TOOLS_REGISTRY, execute_tool, suggest_tools

def test_tools():
    """Test all registered tools"""
    print("=" * 80)
    print("TESTING JEEBSAI TOOLS")
    print("=" * 80)
    
    # List all tools
    print(f"\n📊 Total Tools Registered: {len(TOOLS_REGISTRY)}\n")
    
    # Group tools by category
    categories = {}
    for tool_name, tool in TOOLS_REGISTRY.items():
        # Try to guess category from name
        desc = tool['description'].lower()
        
        if any(word in desc for word in ['web', 'search', 'wikipedia', 'news']):
            cat = 'Research'
        elif any(word in desc for word in ['sentiment', 'analyze', 'extract', 'summary']):
            cat = 'Analysis'
        elif any(word in desc for word in ['convert', 'translate', 'format']):
            cat = 'Conversion'
        elif any(word in desc for word in ['hash', 'encode', 'password', 'crypto']):
            cat = 'Security'
        elif any(word in desc for word in ['calculate', 'math', 'equation', 'solver']):
            cat = 'Math'
        elif any(word in desc for word in ['todo', 'pomodoro', 'task', 'priority', 'effort']):
            cat = 'Productivity'
        elif any(word in desc for word in ['csv', 'data', 'validator', 'parser']):
            cat = 'Data'
        elif any(word in desc for word in ['text', 'summarize', 'keyword', 'outline']):
            cat = 'Text'
        elif any(word in desc for word in ['date', 'time', 'timezone', 'calendar']):
            cat = 'Time'
        else:
            cat = 'Utilities'
        
        if cat not in categories:
            categories[cat] = []
        categories[cat].append(tool_name)
    
    # Print tools by category
    for cat in sorted(categories.keys()):
        print(f"🔧 {cat} ({len(categories[cat])} tools)")
        for tool in sorted(categories[cat]):
            desc = TOOLS_REGISTRY[tool]['description']
            print(f"   ✓ {tool:30} - {desc}")
    
    # Test some tools
    print("\n" + "=" * 80)
    print("RUNNING SAMPLE TOOL TESTS")
    print("=" * 80)
    
    test_cases = [
        ('calculator', {'expression': '10 + 5'}),
        ('create_todo', {'task': 'Test creating a todo', 'priority': 'high'}),
        ('date_calculator', {'start_date': '2024-01-01', 'end_date': '2024-12-31'}),
        ('pomodoro_calculator', {'work_sessions': 4}),
        ('task_priority_score', {'task': 'Test task', 'urgency': 8, 'importance': 9}),
        ('csv_parser', {'csv_data': 'Name,Age\\nAlice,30\\nBob,25', 'has_header': True}),
        ('text_summarizer', {'text': 'This is a sample text. It has multiple sentences. We will summarize it.'}),
        ('keyword_extractor', {'text': 'Machine learning is a subset of artificial intelligence that focuses on data analysis'}),
        ('date_calculator', {'start_date': '2024-01-01', 'end_date': '2024-06-30'}),
        ('time_range_calculator', {'start_time': '09:00', 'end_time': '17:30'}),
    ]
    
    passed = 0
    failed = 0
    
    for tool_name, params in test_cases:
        try:
            result = execute_tool(tool_name, **params)
            if result and 'error' not in result:
                print(f"✅ {tool_name:30} - SUCCESS")
                passed += 1
            else:
                error_msg = result.get('error', 'Unknown error')
                print(f"❌ {tool_name:30} - FAILED: {error_msg}")
                failed += 1
        except Exception as e:
            print(f"❌ {tool_name:30} - EXCEPTION: {str(e)}")
            failed += 1
    
    # Test tool suggestions
    print("\n" + "=" * 80)
    print("TESTING TOOL SUGGESTION ENGINE")
    print("=" * 80)
    
    test_messages = [
        "Can you search the web for information about Python?",
        "Calculate 25 + 30",
        "Convert 5 kilometers to miles",
        "Create a todo item for my meeting",
        "Summarize this long text for me",
        "What are the keywords in this document?",
    ]
    
    for msg in test_messages:
        try:
            suggestions = suggest_tools(msg, max_suggestions=3)
            print(f"\n📝 Message: \"{msg}\"")
            print(f"   💡 Suggested tools ({len(suggestions)}):")
            for i, tool in enumerate(suggestions, 1):
                print(f"      {i}. {tool['name']} (Match score: {tool['score']})")
        except Exception as e:
            print(f"❌ Error suggesting tools: {str(e)}")
    
    # Summary
    print("\n" + "=" * 80)
    print("SUMMARY")
    print("=" * 80)
    print(f"✅ Tools Passed: {passed}/{len(test_cases)}")
    print(f"❌ Tools Failed: {failed}/{len(test_cases)}")
    print(f"📊 Total Tool Count: {len(TOOLS_REGISTRY)}")
    print(f"🎯 Categories: {len(categories)}")
    
    return passed, failed


if __name__ == '__main__':
    passed, failed = test_tools()
    sys.exit(0 if failed == 0 else 1)
