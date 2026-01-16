#!/usr/bin/env python3
"""
Launch the Financial Data Pipeline Web GUI

Usage:
    python launch_gui.py
    
Or directly:
    streamlit run app.py
"""

import os
import sys
from pathlib import Path

def main():
    """Launch Streamlit app"""
    
    # Check if streamlit is installed
    try:
        import streamlit
    except ImportError:
        print("ERROR: Streamlit not installed!")
        print("\nInstall with:")
        print("  pip install streamlit")
        sys.exit(1)

    # Check if database exists
    db_path = Path("data/finance.db")
    if not db_path.exists():
        print("WARNING: Database not found!")
        print("\nInitialize with:")
        print("  python scripts/init_db.py")
        print("\nContinuing anyway (database will be created)...")

    # Launch streamlit
    print("Launching Financial Data Pipeline GUI...")
    print("Opening browser at http://localhost:8501")
    print("\nPress Ctrl+C to stop\n")
    
    os.system("streamlit run app.py")

if __name__ == "__main__":
    main()
