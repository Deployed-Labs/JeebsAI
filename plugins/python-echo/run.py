#!/usr/bin/env python3
import sys
import json

def main():
    try:
        # Read input from stdin
        input_data = sys.stdin.read()
        if not input_data:
            return
            
        data = json.loads(input_data)
        user_input = data.get("input", "")
        
        # Echo response
        response = {"response": f"Echo: {user_input}"}
        print(json.dumps(response))
        
    except Exception as e:
        print(json.dumps({"error": str(e)}))
        sys.exit(1)

if __name__ == "__main__":
    main()