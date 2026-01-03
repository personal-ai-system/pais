#!/usr/bin/env python3
"""
hello-world - A PAIS plugin
"""
import json
import sys


def main():
    """Main entry point."""
    if len(sys.argv) < 2:
        print(json.dumps({"error": "No action specified"}))
        sys.exit(1)

    action = sys.argv[1]
    args = sys.argv[2:]

    if action == "greet":
        name = args[0] if args else "World"
        print(json.dumps({"message": f"Hello, {name}!"}))
    elif action == "version":
        print(json.dumps({"version": "0.1.0"}))
    else:
        print(json.dumps({"error": f"Unknown action: {action}"}))
        sys.exit(1)


if __name__ == "__main__":
    main()
